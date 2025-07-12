use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    net::SocketAddrV4,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use tokio::net::UdpSocket;

pub mod join;

#[derive(Debug, Serialize, Deserialize, Decode, Encode)]
pub enum MulticastMessage {
    Join,
    NewServer { port: u16 },
    CloseServer { port: u16 },
}

/// Handles communication with the multicast and mantains an updated list with all of the open servers.
#[derive(Debug)]
pub struct MulticastServer<MessageListener: Debug, const BUF_SIZE: usize = 4096> {
    address: SocketAddrV4,
    socket: UdpSocket,
    messages: Arc<Mutex<Vec<MulticastMessage>>>,
    buf: [u8; BUF_SIZE],
    message_listener: MessageListener,
}

impl<MessageListener: Debug, const BUF_SIZE: usize> MulticastServer<MessageListener, BUF_SIZE> {
    #[tracing::instrument]
    pub async fn send<T: Encode + Debug>(&mut self, msg: T) -> Result<()> {
        let encoded = Self::encode(&mut self.buf, msg)?;
        self.socket.send_to(encoded, self.address).await?;

        Ok(())
    }

    #[tracing::instrument]
    fn encode<T: Encode + Debug>(buf: &mut [u8], msg: T) -> Result<&[u8]> {
        let len = bincode::encode_into_slice(msg, buf, bincode::config::standard())?;

        Ok(&buf[..len])
    }
}

impl<const BUF_SIZE: usize> MulticastServer<(), BUF_SIZE> {
    /// Joins the specified multicast and starts a background tas
    #[tracing::instrument]
    pub async fn join(address: SocketAddrV4) -> Result<MulticastServer<(), BUF_SIZE>> {
        let buf = [0; BUF_SIZE];
        let socket = join::connect_to_multicast(address).await?;

        let mut server = MulticastServer::<(), BUF_SIZE> {
            address,
            socket,
            messages: Arc::new(Mutex::new(Vec::new())),
            buf,
            message_listener: (),
        };

        server.send(MulticastMessage::Join).await?;

        Ok(server)
    }
}
