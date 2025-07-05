use std::net::Ipv4Addr;

pub const SERVER_PORT: u16 = 4983;

pub const MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 123);

#[cfg(test)]
mod tests {
    use crate::MULTICAST_ADDRESS;

    #[test]
    fn address_is_multicast() {
        assert!(MULTICAST_ADDRESS.is_multicast());
    }
}
