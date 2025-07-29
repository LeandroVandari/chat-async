use std::{net::SocketAddrV4, sync::Arc, time::Duration};

use interprocess::local_socket::{
 ListenerOptions, traits::tokio::Listener,
};
use procspawn::JoinHandle;
use tokio::{select, sync::mpsc};
use tracing::{info, warn};

use crate::connect::multicast::{communicator::error, join::connect_to_multicast};

use super::IpcCommunicator;

impl IpcCommunicator {

    pub(crate) fn spawn_communicator_process(multicast_addr: SocketAddrV4)
    -> JoinHandle<Result<(), error::CommunicatorProcessError>> {
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


            rt.block_on(Self::communicator_function(multicast_addr)).inspect_err(|e| tracing::error!("Error on communicator function: {e}")) 
        })
    }

    #[tracing::instrument(name = "Multicast Communicator")]
    async fn communicator_function(multicast_addr: SocketAddrV4) -> Result<(), error::CommunicatorProcessError> {
        let listener = ListenerOptions::new()
            .name(Self::local_socket_name(multicast_addr))
            .create_tokio()?;
        info!(
            "Opened IPC listener on {:?}. PID: {}",
            Self::local_socket_name(multicast_addr),
            std::process::id()
        );

        let multicast_connection = connect_to_multicast(multicast_addr).await.unwrap();
        let (tx, mut rx) = mpsc::channel(8);
        let _accept_ipc_connections = tokio::spawn(async move {
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

            connections.pop();
        }


        <std::result::Result<(), error::CommunicatorProcessError>>::Ok(())
    }
}