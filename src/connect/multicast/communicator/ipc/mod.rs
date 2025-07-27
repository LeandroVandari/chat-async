use anyhow::Result;
use interprocess::local_socket::{
    traits::Stream, GenericNamespaced, Name, Stream as IpcStream, ToNsName
};
use tokio::join;

use std::{fmt::Debug, io, net::SocketAddrV4};
use tracing::info;

use crate::connect::multicast:: AsyncTryFromSocketAddr;

use super::Communicator;

mod spawn_process;

#[derive(Debug)]
pub struct IpcCommunicator {
    ipc_conn: IpcStream,
    multicast_addr: SocketAddrV4
}

impl IpcCommunicator {
    
    fn local_socket_name(multicast_addr: SocketAddrV4) -> Name<'static> {
        format!("multicast_communicator:{multicast_addr}.sock").to_ns_name::<GenericNamespaced>().unwrap()
    }

    fn connect_to_ipc_stream(multicast_addr: SocketAddrV4) -> Result<IpcStream> {

        let ipc_conn = match IpcStream::connect(
            Self::local_socket_name(multicast_addr)
        ) {
            Ok(ipc_conn) => ipc_conn,
            Err(e) => {
                info!(
                    "Error connecting to IPC Stream: {e}. Spawning new multicast communicator process."
                );
                let p = IpcCommunicator::spawn_communicator_process(multicast_addr);
                info!("Communicator process spawned with ID: {:?}", p.pid());

                loop {
                    if let Ok(ipc_conn) = IpcStream::connect(
                       Self::local_socket_name(multicast_addr)
                    ) {
                        break ipc_conn;
                    }
                }
            }
        };
        info!("Successfully connected to IPC Stream...");

        Ok(ipc_conn )
    }
}

impl AsyncTryFromSocketAddr for IpcCommunicator {
    #[tracing::instrument]
    fn try_from_socket_addr(addr: SocketAddrV4) -> impl Future<Output = Result<Self>> {
        async {
            let ipc_connect = tokio::spawn(async move {
                Self::connect_to_ipc_stream(addr)
            });


            let (ipc_connect,) = join!(ipc_connect);
            Ok(Self {ipc_conn: ipc_connect??, multicast_addr: addr})
        }
    }
}

impl Communicator for IpcCommunicator {
    async fn communicate(&self, bytes: &[u8]) -> Result<usize, io::Error> {
        info!("Communicating: {bytes:?}");
        Ok(0)
    }
}
