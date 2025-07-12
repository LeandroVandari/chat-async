use anyhow::Result;
use chat_async::{
    MULTICAST_ADDRESS,
    connect::{get_my_ip::get_my_ip, manage_tcp_streams, multicast::MulticastServer},
    handle_incoming_connections, handle_new_multicast_members,
};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    /*
       let multicast_server = MulticastServer::join(MULTICAST_ADDRESS).await;

        let available_chat_servers = multicast_server.get_available_servers().await;

        // selecionar servidor

    */

    let mut multicast_server = MulticastServer::join(MULTICAST_ADDRESS).await?;
    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();
    let my_ip = get_my_ip().await?;

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
}
