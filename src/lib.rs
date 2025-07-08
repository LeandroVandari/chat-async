use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::{Result, anyhow};
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket},
    select,
    sync::mpsc,
};
use tracing::info;

pub mod get_my_ip;

pub const SERVER_PORT: u16 = 4983;

pub const MULTICAST_IP: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 123);

pub const MULTICAST_ADDRESS: SocketAddrV4 = SocketAddrV4::new(MULTICAST_IP, SERVER_PORT);


#[tracing::instrument(name = "Enter Multicast")]
pub async fn connect_to_multicast(address: SocketAddrV4) -> Result<UdpSocket> {
    if !address.ip().is_multicast() {
        return Err(anyhow!("Address must be multicast"));
    }
    info!(
        "Joining multicast on {address}",
    );
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, address.port())).await?;

    socket.join_multicast_v4(*address.ip(), std::net::Ipv4Addr::UNSPECIFIED)?;

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

#[tracing::instrument(name = "New Multicast Members", skip(tx, multicast))]
pub async fn handle_new_multicast_members(
    tx: mpsc::Sender<TcpStream>,
    multicast: UdpSocket,
    my_ip: Ipv4Addr
) -> Result<()> {
    let mut buf = [0; 9];
    loop {
        select! {
            msg = multicast.recv_from(&mut buf) => {
                match msg {
                    Ok((len, peer)) => {
                        if len == 0 {
                            info!("Received length 0 from multicast.");
                            break;
                        }
                        if len == 4
                            && let Ok(port) = parse_hi(&buf[..len])
                        {
                            let addr = SocketAddrV4::new(match peer.ip() {
                                std::net::IpAddr::V6(_) => {
                                    info!("Received unimplemented IPV6 address.");
                                    return Err(anyhow!("IPV6 not implemented."));
                                },
                                std::net::IpAddr::V4(addr) => addr
                            }, port);

                           if addr.ip() == &my_ip{
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

pub fn parse_hi(bytes: &[u8]) -> Result<u16> {
    if &bytes[0..2] != "HI".as_bytes() {
        return Err(anyhow!("Not a multicast hi."));
    }
    let port = u16::from_be_bytes(bytes[2..4].try_into()?);

    Ok(port)
}

#[cfg(test)]
mod tests {
    use crate::{connect_to_multicast, MULTICAST_ADDRESS, MULTICAST_IP};

    #[test]
    fn address_is_multicast() {
        assert!(MULTICAST_IP.is_multicast());
    }

    #[tokio::test]
    async fn connect_to_multicast_test() {
        assert!(connect_to_multicast(MULTICAST_ADDRESS).await.is_ok())
    }
}
