use std::{io, net::SocketAddrV4};

use anyhow::Result;
use tokio::net::UdpSocket;

use super::Communicator;

#[derive(Debug)]
pub struct SocketCommunicator {
    socket: UdpSocket,
    address: SocketAddrV4,
}

impl super::AsyncTryFromSocketAddr for SocketCommunicator {
    async fn try_from_socket_addr(addr: SocketAddrV4) -> Result<Self> {
        let socket = super::super::join::connect_to_multicast(addr).await?;
        Ok(Self {
            socket,
            address: addr,
        })
    }
}

impl Communicator for SocketCommunicator {
    async fn communicate(&mut self, bytes: &[u8]) -> Result<usize, io::Error> {
        self.socket.send_to(bytes, self.address).await
    }
}
