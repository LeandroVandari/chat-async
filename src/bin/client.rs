use std::net::Ipv4Addr;

use anyhow::Result;
use chat_async::{MULTICAST_ADDRESS, SERVER_PORT};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<()> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, SERVER_PORT)).await?;
    socket.join_multicast_v4(MULTICAST_ADDRESS, std::net::Ipv4Addr::UNSPECIFIED)?;
    
    // Connecting the socket with the code below makes the client stop receiving messages from the multicast.
    // socket.connect((MULTICAST_ADDRESS, SERVER_PORT)).await?;


    let mut buf = vec![0; 1024];
    loop {
        let n = socket.recv(&mut buf).await?;
        if n != 0 {
            println!(
                "Received {}",
                buf[0..n]
                    .iter()
                    .map(|b| char::from_u32(*b as u32).unwrap())
                    .collect::<String>()
            );
        } else {
            dbg!(n);
        }
    }

    Ok(())
}
