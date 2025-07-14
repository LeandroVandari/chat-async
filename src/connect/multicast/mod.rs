use anyhow::Result;
use std::net::SocketAddrV4;

pub mod communicator;
pub mod join;
pub mod message;
pub mod server;

pub trait AsyncTryFromSocketAddr
where
    Self: Sized,
{
    #[allow(async_fn_in_trait)]
    async fn try_from_socket_addr(addr: SocketAddrV4) -> Result<Self>;
}
