use std::net::{self, SocketAddrV4};

use anyhow::{Result, anyhow};
use bytes::Bytes;
use chat_async::{
    MULTICAST_ADDRESS, SERVER_PORT, connect_to_multicast, handle_new_multicast_members,
    handle_tcp_connections,
};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let multicast = connect_to_multicast().await?;

    let listener = TcpListener::bind("0.0.0.0:0").await?;

    let local_ip = match local_ip_address::local_ip()? {
        net::IpAddr::V4(addr) => addr,
        net::IpAddr::V6(_) => return Err(anyhow!("Incorrect type of address: need V4")),
    };
    let listener_addr = SocketAddrV4::new(local_ip, listener.local_addr()?.port());
    info!("Sending listener address ({listener_addr}) to multicast.");
    multicast
        .send_to(
            &"HI"
                .as_bytes()
                .iter()
                .copied()
                .chain(listener_addr.ip().octets())
                .chain(listener_addr.port().to_be_bytes())
                .collect::<Bytes>(),
            (MULTICAST_ADDRESS, SERVER_PORT),
        )
        .await?;

    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let tx1 = tx.clone();
    let handle_new_multicast_members =
        tokio::spawn(handle_new_multicast_members(tx1, multicast, local_ip));

    let handle_new_connections = tokio::spawn(handle_tcp_connections(tx, listener));

    drop(rx);
    let (first, second) = tokio::join!(handle_new_multicast_members, handle_new_connections);

    first??;
    second??;

    Ok(())
}
