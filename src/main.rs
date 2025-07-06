use std::net::{self, SocketAddrV4};

use anyhow::{Result, anyhow};
use bytes::{Bytes, BytesMut};
use chat_async::{MULTICAST_ADDRESS, SERVER_PORT, connect_to_multicast, parse_hi};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, info_span};

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
    let listener_addr = SocketAddrV4::new(
        local_ip,
        listener.local_addr()?.port(),
    );
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
    let handle_new_multicast_members = tokio::spawn(async move {
        let span = info_span!("New multicast member");

        let mut buf = BytesMut::with_capacity(8);
        while let Ok(len) = multicast.recv_buf(&mut buf).await {
            if len == 0 {
                info!(parent: &span, "Received length 0 from multicast.");
                break;
            }
            if len == 8
                && let Ok(addr) = parse_hi(&buf[0..len])
            {
                if addr.ip() == &local_ip {
                    continue;
                }
                info!(parent: &span, "Received message: {:?}",&buf[..len]);
                let tx = tx.clone();
                tokio::spawn(async move {
                    tx.send(TcpStream::connect(addr).await?).await?;
                    anyhow::Ok(())
                });
            }
        }

        anyhow::Ok(())
    });

    let handle_new_connections = tokio::spawn(async move {
        loop {
            while let Ok((_stream, addr)) = listener.accept().await {
                info!("Connected to {addr}!");
            }
        }
    });

    let (first, second) = tokio::join!(handle_new_multicast_members, handle_new_connections);

    first??;
    second?;

    Ok(())
}
