use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::{Result, anyhow};
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket},
    select,
    sync::mpsc,
};
use tracing::info;

pub const SERVER_PORT: u16 = 4983;

pub const MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 123);

#[tracing::instrument(name = "Enter Multicast")]
pub async fn connect_to_multicast() -> Result<UdpSocket> {
    info!(
        "Joining multicast on {}",
        SocketAddrV4::new(MULTICAST_ADDRESS, SERVER_PORT)
    );
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, SERVER_PORT)).await?;

    socket.join_multicast_v4(MULTICAST_ADDRESS, std::net::Ipv4Addr::UNSPECIFIED)?;

    Ok(socket)
}

#[tracing::instrument(name = "TCP Connections", skip(tx, listener), fields(listener_port = %listener.local_addr().map(|addr| addr.port())?))]
pub async fn handle_tcp_connections(
    tx: mpsc::Sender<TcpStream>,
    listener: TcpListener,
) -> Result<()> {
    loop {
        select! {
            res = listener.accept() => {match res {
                Ok((stream, addr)) => {
                    info!("Connected to {addr}!");
                    if tx.send(stream).await.is_err() {
                        info!("Received error from mpsc channel. Will close TCP receiving task.");
                        break;
            }
                }
                Err(e) => {
                    info!("Received error ({e}) when accepting TCP connection. Will close TCP receivivg task.");
                }
            }}
            () = tx.closed() => {
                info!("MPSC channel was closed. Will close TCP receiving task.");
                break;
            }
        }
    }

    anyhow::Ok(())
}

#[tracing::instrument(name = "New Multicast Members", skip(tx, local_ip, multicast))]
pub async fn handle_new_multicast_members(
    tx: mpsc::Sender<TcpStream>,
    multicast: UdpSocket,
    local_ip: Ipv4Addr,
) -> Result<()> {
    let mut buf = [0; 9];

    loop {
        select! {
            conn_len = multicast.recv(&mut buf) => {
                match conn_len {
                    Ok(len) => {
                        if len == 0 {
                            info!("Received length 0 from multicast.");
                            break;
                        }
                        if len == 8
                            && let Ok(addr) = parse_hi(&buf[..len])
                        {
                            if addr.ip() == &local_ip {
                                continue;
                            }
                            info!("Received HI from {addr}",);
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                match TcpStream::connect(addr).await {
                                    Ok(stream) => tx.send(stream).await?,
                                    Err(e) => info!("Error connecting to {addr}: {e}")
                                }
                                anyhow::Ok(())
                            });
                        }
                    }
                    Err(e) => {
                        info!("Error receiving multicast msg ({e}). Closing multicast members task.");
                        break;
                    }
                }
            }
            () = tx.closed() => {
                info!("MPSC channel was closed. Will close multicast members task.");
                break;
            }

        }
    }

    anyhow::Ok(())
}

pub fn parse_hi(bytes: &[u8]) -> Result<SocketAddrV4> {
    if &bytes[0..2] != "HI".as_bytes() {
        return Err(anyhow!("Not a multicast hi."));
    }

    let ip = Ipv4Addr::from(<_ as TryInto<[u8; 4]>>::try_into(&bytes[2..6])?);
    let port = u16::from_be_bytes(bytes[6..8].try_into()?);

    Ok(SocketAddrV4::new(ip, port))
}

#[cfg(test)]
mod tests {
    use crate::{MULTICAST_ADDRESS, connect_to_multicast};

    #[test]
    fn address_is_multicast() {
        assert!(MULTICAST_ADDRESS.is_multicast());
    }

    #[tokio::test]
    async fn connect_to_multicast_test() {
        assert!(connect_to_multicast().await.is_ok())
    }
}
