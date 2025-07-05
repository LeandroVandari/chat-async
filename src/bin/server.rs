use std::net::Ipv4Addr;

use anyhow::Result;
use chat_async::{MULTICAST_ADDRESS, SERVER_PORT};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<()> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, SERVER_PORT)).await?;

    socket.join_multicast_v4(MULTICAST_ADDRESS, std::net::Ipv4Addr::UNSPECIFIED)?;
    socket.connect((MULTICAST_ADDRESS, SERVER_PORT)).await?;
    loop {
        socket.send("hi".as_bytes()).await?;
    }

    Ok(())
}
