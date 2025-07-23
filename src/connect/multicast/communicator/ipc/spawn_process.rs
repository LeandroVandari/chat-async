use std::{sync::Arc, time::Duration};

use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, ToNsName, traits::tokio::Listener,
};
use procspawn::JoinHandle;
use tokio::{select, sync::mpsc};
use tracing::{info, warn};

use crate::connect::multicast::communicator::error;

use super::IpcCommunicator;

impl IpcCommunicator {
    pub(super) const LOCAL_SOCKET_NAME: &'static str = "multicast_communicator.sock";

    pub(crate) fn spawn_communicator_process()
    -> JoinHandle<Result<(), error::CommunicatorProcessError>> {
        procspawn::spawn((), |_| {
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

            rt.block_on(Self::communicator_function())
        })
    }

    #[tracing::instrument(name = "Multicast Communicator")]
    async fn communicator_function() -> Result<(), error::CommunicatorProcessError> {
        let listener = ListenerOptions::new()
            .name(IpcCommunicator::LOCAL_SOCKET_NAME.to_ns_name::<GenericNamespaced>()?)
            .create_tokio()?;
        info!(
            "Opened IPC listener on {}",
            IpcCommunicator::LOCAL_SOCKET_NAME
        );

        let (tx, mut rx) = mpsc::channel(8);
        let _accept_connections = tokio::spawn(async move {
            loop {
                let conn = match listener.accept().await {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Error accepting IPC connection: {e}");
                        continue;
                    }
                };
                info!("New IPC connection");
                tx.send(conn).await.unwrap()
            }
        });
        let mut connections = Vec::new();
        loop {
            select! {
                Some(conn) = rx.recv() => {
                    connections.push(conn);
                }

                _ = tokio::time::sleep(Duration::from_secs(10)), if connections.is_empty() => {
                    info!("10 seconds without any connections. Quitting multicast process...");
                    break;
                }
            }
        }

        <std::result::Result<(), error::CommunicatorProcessError>>::Ok(())
    }
}
