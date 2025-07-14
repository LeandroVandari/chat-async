use anyhow::Result;
use chat_async::{
    MULTICAST_ADDRESS,
    connect::{
        get_my_ip::get_my_ip,
        multicast::{
            communicator::IpcCommunicator, message::MulticastMessage, server::MulticastServer,
        },
    },
};
use tokio::{net::TcpListener, sync::mpsc};
use tracing::info;

// Initialize the Runtime manually because `procspawn::init()` must be the first thing called, otherwise a runtime
// already exists but we can't get a handle to it
fn main() -> Result<()> {
    procspawn::init();
    let body = async {
        let subscriber = tracing_subscriber::FmtSubscriber::new();
        tracing::subscriber::set_global_default(subscriber)?;

        /*
           let multicast_server = MulticastServer::join(MULTICAST_ADDRESS).await;

            let available_chat_servers = multicast_server.get_available_servers().await;

            // selecionar servidor

        */

        let (tx, _rx) = mpsc::channel::<MulticastMessage>(8);

        let _multicast_server =
            <MulticastServer<_, IpcCommunicator>>::join(MULTICAST_ADDRESS, tx).await?;
        let listener = TcpListener::bind("0.0.0.0:0").await?;
        let port = listener.local_addr()?.port();
        let _my_ip = get_my_ip().await?;

        info!("Sending listener port ({port}) to multicast.");

        /*  multicast_server
        .send(
            &[[b'H', b'I'], port.to_be_bytes()].concat(),
        )
        .await?; */

        /* let (tx, rx) = tokio::sync::mpsc::channel(8);
        let tx1 = tx.clone();
        let handle_new_multicast_members =
            tokio::spawn(handle_new_multicast_members(tx1, multicast, my_ip));

        let handle_incoming_connections = tokio::spawn(handle_incoming_connections(tx, listener));

        let manage_tcp_streams = tokio::spawn(manage_tcp_streams(rx));

        let (first, second, third) = tokio::join!(handle_new_multicast_members, handle_incoming_connections, manage_tcp_streams);

        first??;
        second??;
        third??; */

        Ok(())
    };

    return tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(body);
}
