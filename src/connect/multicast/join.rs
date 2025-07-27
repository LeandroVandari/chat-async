use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::{Result, anyhow};
use tokio::net::UdpSocket;
use tracing::info;

#[tracing::instrument(name = "Enter Multicast")]
pub(crate) async fn connect_to_multicast(address: SocketAddrV4) -> Result<UdpSocket> {
    if !address.ip().is_multicast() {
        return Err(anyhow!("Address must be multicast"));
    }
    info!("Joining multicast");
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, address.port())).await?;

    socket.join_multicast_v4(*address.ip(), std::net::Ipv4Addr::UNSPECIFIED)?;

    Ok(socket)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn connect_to_multicast_test() {
        assert!(
            super::connect_to_multicast(crate::MULTICAST_ADDRESS)
                .await
                .is_ok()
        )
    }
}
