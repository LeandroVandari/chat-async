use anyhow::Result;
use chat_async::{
    MULTICAST_ADDRESS, connect_to_multicast, get_my_ip::get_my_ip, handle_new_multicast_members,
    handle_incoming_connections,
};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let multicast = connect_to_multicast(MULTICAST_ADDRESS).await?;
    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();

    let my_ip = get_my_ip().await?;

    info!("Sending listener port ({port}) to multicast.");
    multicast
        .send_to(
            &[[b'H', b'I'], port.to_be_bytes()].concat(),
            MULTICAST_ADDRESS,
        )
        .await?;

    let (tx, mut rx) = tokio::sync::mpsc::channel(8);
    let tx1 = tx.clone();
    let handle_new_multicast_members =
        tokio::spawn(handle_new_multicast_members(tx1, multicast, my_ip));

    let handle_incoming_connections = tokio::spawn(handle_incoming_connections(tx, listener));

    let handle_tcp_streams = tokio::spawn(async move {
        while let Some(stream) = rx.recv().await {
            info!("Established TCP connection to {}", stream.peer_addr()?.ip());
        }

        anyhow::Ok(())
    });

    let (first, second, third) = tokio::join!(handle_new_multicast_members, handle_incoming_connections, handle_tcp_streams);

    first??;
    second??;
    third??;

    Ok(())
}
