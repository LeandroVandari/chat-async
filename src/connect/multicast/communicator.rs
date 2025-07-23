use super::AsyncTryFromSocketAddr;
use anyhow::Result;
use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, Stream as IpcStream, ToNsName,
    traits::{Stream, tokio::Listener},
};
use procspawn::JoinHandle;
use std::{fmt::Debug, io, net::SocketAddrV4, sync::Arc, time::Duration};
use tokio::{net::UdpSocket, select, sync::mpsc};
use tracing::{info, warn};

pub trait Communicator: AsyncTryFromSocketAddr + Debug {
    #[allow(async_fn_in_trait)]
    async fn communicate(&self, bytes: &[u8]) -> Result<usize, io::Error>;
}

#[derive(Debug)]
pub struct SocketCommunicator {
    socket: UdpSocket,
    address: SocketAddrV4,
}

impl AsyncTryFromSocketAddr for SocketCommunicator {
    async fn try_from_socket_addr(addr: SocketAddrV4) -> Result<Self> {
        let socket = super::join::connect_to_multicast(addr).await?;
        Ok(Self {
            socket,
            address: addr,
        })
    }
}

impl Communicator for SocketCommunicator {
    async fn communicate(&self, bytes: &[u8]) -> Result<usize, io::Error> {
        self.socket.send_to(bytes, self.address).await
    }
}

#[derive(Debug)]
pub struct IpcCommunicator {
    conn: IpcStream,
}

impl AsyncTryFromSocketAddr for IpcCommunicator {
    #[tracing::instrument]
    fn try_from_socket_addr(_addr: SocketAddrV4) -> impl Future<Output = Result<Self>> {
        async {
            let conn = match IpcStream::connect(
                IpcCommunicator::LOCAL_SOCKET_NAME.to_ns_name::<GenericNamespaced>()?,
            ) {
                Ok(conn) => conn,
                Err(e) => {
                    info!(
                        "Error connecting to IPC Stream: {e}. Spawning new multicast communicator process."
                    );
                    let p = IpcCommunicator::spawn_communicator_process();
                    info!("Communicator process spawned with ID: {:?}", p.pid());

                    loop {
                        if let Ok(conn) = IpcStream::connect(
                            IpcCommunicator::LOCAL_SOCKET_NAME.to_ns_name::<GenericNamespaced>()?,
                        ) {
                            break conn;
                        }
                    }
                }
            };

            Ok(Self { conn })
        }
    }
}

impl IpcCommunicator {
    const LOCAL_SOCKET_NAME: &'static str = "multicast_communicator.sock";

    fn spawn_communicator_process() -> JoinHandle<Result<(), error::CommunicatorProcessError>> {
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

            rt.block_on(communicator_function())
        })
    }
}

impl Communicator for IpcCommunicator {
    async fn communicate(&self, bytes: &[u8]) -> Result<usize, io::Error> {
            info!("Communicating: {bytes:?}");
            Ok(0)
    }
}

mod error {
    use std::fmt::Display;

    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    #[derive(Debug, Serialize, Deserialize, Error)]
    pub struct CommunicatorProcessError(String);

    impl From<std::io::Error> for CommunicatorProcessError {
        fn from(value: std::io::Error) -> Self {
            Self(value.to_string())
        }
    }

    impl From<tokio::sync::mpsc::error::TryRecvError> for CommunicatorProcessError {
        fn from(value: tokio::sync::mpsc::error::TryRecvError) -> Self {
            Self(value.to_string())
        }
    }

    impl Display for CommunicatorProcessError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "CommunicatorProcessError: {}", self.0)
        }
    }
}
