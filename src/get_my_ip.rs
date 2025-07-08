//! Figures out which IP this computer has.
//! 
//! This is utterly stupid. It is the most idiotic idea I've come up with. I do not know how it came to my mind
//! and which of the circles of hell is responsible for its creation.

use std::{net::{IpAddr, Ipv4Addr, SocketAddrV4}, time::UNIX_EPOCH};

use anyhow::{anyhow, Result};

use crate::connect_to_multicast;

/// 
/// Figure out this computer's IP.
/// 
/// Do not use this function. It is stupid.
/// It works by connecting to a multicast in the network, sending a message with an identifier, and reading which IP sent that message.
/// 
/// # Why not [`UdpSocket::local_addr`](tokio::net::UdpSocket::local_addr)?
/// It just returns the address we [`bind`](tokio::net::UdpSocket::bind)ed to. So, if that was `0.0.0.0`, that's what we get back.
/// 
/// # Why not `local_ip_address`?
/// It requires a lot of dependencies, and doesn't actually figure out through which IP multicast packets are sent, in some cases.
/// In a Chromebook with a Linux development environment, for example, it returns the internal Linux IP, and not the one that communicates with the
/// outer network.
// TODO: Make this not hang if something goes wrong in the multicast (add timeout)
#[tracing::instrument(name="Get IP")]
pub async fn get_my_ip() -> Result<Ipv4Addr> {
    let address = SocketAddrV4::new(Ipv4Addr::new(224, 0, 0, 125), 28324);
    let multicast = connect_to_multicast(address).await?;

    let identifier = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_be_bytes();
    let mut buf = [0;16];

    multicast.send_to(&identifier, address).await?;

    while let Ok((size, addr)) = multicast.recv_from(&mut buf).await {
        if buf[..size] == identifier[..] {
            match addr.ip() {
                IpAddr::V4(addr) => {return Ok(addr);}
                IpAddr::V6(_) => {return Err(anyhow!("IPV6 is unsupported"));}
            }
        }
    }

    Err(anyhow!("Local IP not found"))


}