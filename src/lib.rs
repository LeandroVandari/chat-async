use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::{Ok, Result, anyhow};
use tokio::net::UdpSocket;
use tracing::info;

pub const SERVER_PORT: u16 = 4983;

pub const MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 123);

#[tracing::instrument(name = "Enter multicast")]
pub async fn connect_to_multicast() -> Result<UdpSocket> {
    info!(
        "Joining multicast on {}",
        SocketAddrV4::new(MULTICAST_ADDRESS, SERVER_PORT)
    );
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, SERVER_PORT)).await?;

    socket.join_multicast_v4(MULTICAST_ADDRESS, std::net::Ipv4Addr::UNSPECIFIED)?;

    Ok(socket)
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
