use anyhow::Result;
use tokio::{net::TcpStream, sync::mpsc::Receiver};
use tracing::info;

pub mod get_my_ip;
pub mod multicast;

pub async fn manage_tcp_streams(mut rx: Receiver<TcpStream>) -> Result<()> {
    while let Some(stream) = rx.recv().await {
        info!("Established TCP connection to {}", stream.peer_addr()?.ip());
    }

    anyhow::Ok(())
}
