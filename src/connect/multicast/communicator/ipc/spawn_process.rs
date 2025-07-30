use std::{
    net::SocketAddrV4,
    sync::{Arc, atomic::AtomicUsize},
    time::Duration,
};

use interprocess::{
    bound_util::RefTokioAsyncRead,
    local_socket::{
        self, ListenerOptions,
        traits::tokio::{Listener, Stream},
    },
};
use procspawn::JoinHandle;
use tokio::{
    io::AsyncReadExt,
    select,
    sync::mpsc::{self, Sender},
};
use tracing::{info, warn};

use crate::connect::multicast::{communicator::error, join::connect_to_multicast};

use super::IpcCommunicator;

static IPC_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

impl IpcCommunicator {
    pub(crate) fn spawn_communicator_process(
        multicast_addr: SocketAddrV4,
    ) -> JoinHandle<Result<(), error::CommunicatorProcessError>> {
        procspawn::spawn(multicast_addr, |multicast_addr: SocketAddrV4| {
            tracing::subscriber::set_global_default(
                tracing_subscriber::FmtSubscriber::builder()
                    .with_writer(Arc::new(std::fs::File::create("/tmp/log.txt").unwrap()))
                    .finish(),
            )
            .unwrap();
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(Self::communicator_function(multicast_addr))
                .inspect_err(|e| tracing::error!("Error on communicator function: {e}"))
        })
    }

    #[tracing::instrument(name = "Multicast Communicator")]
    async fn communicator_function(
        multicast_addr: SocketAddrV4,
    ) -> Result<(), error::CommunicatorProcessError> {
        let listener = ListenerOptions::new()
            .name(Self::local_socket_name(multicast_addr))
            .create_tokio()?;
        info!(
            "Opened IPC listener on {:?}. PID: {}",
            Self::local_socket_name(multicast_addr),
            std::process::id()
        );

        let multicast_connection = connect_to_multicast(multicast_addr)
            .await
            .inspect_err(|e| tracing::error!("Error connecting to multicast: {e}"))
            .unwrap();

        let (t_receiver, mut r_receiver) = mpsc::channel(8);
        let (t_sender, mut r_sender) = mpsc::channel(8);

        tokio::spawn(Self::accept_ipc_connections(listener, t_sender, t_receiver));

        tokio::spawn(async move {
            // This has to be a BytesMut. For some reason, a simple Vec<u8> *does not work*
            let mut buf = bytes::BytesMut::with_capacity(4096);
            let mut receivers = Vec::new();
            let mut remove = Vec::new();
            loop {
                if let Ok(rec) = r_receiver.try_recv() {
                    receivers.push(rec);
                }

                for (i, rec) in receivers.iter_mut().enumerate() {
                    match rec.as_tokio_async_read().read_buf(&mut buf).await {
                        Ok(0) => {
                            info!("IPC connection closed. Removing from list.");
                            IPC_CONNECTIONS.fetch_sub(1, std::sync::atomic::Ordering::Release);
                            remove.push(i);
                        }
                        Ok(_) => {
                            info!("Received message from IPC connection: {:?}", &buf[..])
                        }
                        Err(e) => tracing::error!("Error reading from IPC connection: {e}"),
                    }
                }

                for i in remove.drain(..) {
                    receivers.remove(i);
                }
                // Needed because otherwise this may spin-loop and block the checker for no tasks, in case it's scheduled for the same thread.
                if receivers.is_empty() {
                    tokio::task::yield_now().await;
                }
            }
        });

        let mut interval = tokio::time::interval(Duration::from_secs(10));
        interval.tick().await;
        let check_end = tokio::spawn(async move {
            loop {
                if IPC_CONNECTIONS.load(std::sync::atomic::Ordering::Acquire) == 0 {
                    interval.tick().await;
                    if IPC_CONNECTIONS.load(std::sync::atomic::Ordering::Acquire) == 0 {
                        info!("10 seconds without any connections. Quitting multicast process...");
                        break;
                    }
                }
            }
            tokio::task::yield_now().await;
        });

        check_end.await.unwrap();

        <std::result::Result<(), error::CommunicatorProcessError>>::Ok(())
    }

    async fn accept_ipc_connections(
        listener: local_socket::tokio::Listener,
        t_sender: Sender<local_socket::tokio::SendHalf>,
        t_receiver: Sender<local_socket::tokio::RecvHalf>,
    ) {
        loop {
            let conn = match listener.accept().await {
                Ok(c) => c,
                Err(e) => {
                    warn!("Error accepting IPC connection: {e}");
                    continue;
                }
            };
            info!("New IPC connection");

            IPC_CONNECTIONS.fetch_add(1, std::sync::atomic::Ordering::Release);
            let (r_end, s_end) = conn.split();
            t_receiver.send(r_end).await.unwrap();
            t_sender.send(s_end).await.unwrap();
        }
    }
}
