use anyhow::Result;
use interprocess::local_socket::{
    GenericNamespaced, Stream as IpcStream, ToNsName, traits::Stream,
};

use std::{fmt::Debug, io, net::SocketAddrV4};
use tracing::info;

use crate::connect::multicast::AsyncTryFromSocketAddr;

use super::Communicator;

mod spawn_process;

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

impl Communicator for IpcCommunicator {
    async fn communicate(&self, bytes: &[u8]) -> Result<usize, io::Error> {
        info!("Communicating: {bytes:?}");
        Ok(0)
    }
}
