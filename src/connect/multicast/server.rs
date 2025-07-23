use std::net::SocketAddrV4;

use anyhow::Result;
use tokio::sync::mpsc::Sender;

use crate::connect::multicast::communicator::{Communicator, SocketCommunicator};

use super::message::Message;

/// Handles communication with the multicast and mantains an updated list with all of the open servers.
#[derive(Debug)]
pub struct MulticastServer<M: Message, C: Communicator = SocketCommunicator> {
    communicator: C,
    buf: [u8; 4096],
    sender: Sender<M>,
}

impl<M: Message, C: Communicator> MulticastServer<M, C> {
    #[tracing::instrument(name = "MulticastServer::send", skip(self))]
    pub async fn send(&mut self, msg: M) -> Result<()> {
        let encoded = Self::encode(&mut self.buf, msg)?;
        self.communicator.communicate(encoded).await?;

        Ok(())
    }

    #[tracing::instrument]
    fn encode(buf: &mut [u8], msg: M) -> Result<&[u8]> {
        let len = bincode::encode_into_slice(msg, buf, bincode::config::standard())?;

        Ok(&buf[..len])
    }
}

impl<M: Message, C: Communicator> MulticastServer<M, C> {
    /// Joins the specified multicast and starts a background tas
    #[tracing::instrument(skip(msg_sender))]
    pub async fn join(address: SocketAddrV4, msg_sender: Sender<M>) -> Result<Self> {
        let buf = [0; 4096];

        let mut server = Self {
            communicator: C::try_from_socket_addr(address).await?,
            buf,
            sender: msg_sender,
        };

        server.send(M::join()).await?;

        Ok(server)
    }
}
