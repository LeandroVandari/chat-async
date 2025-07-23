use std::fmt::Debug;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Decode, Encode)]
pub enum MulticastMessage {
    Join,
    NewServer { port: u16 },
    CloseServer { port: u16 },
}

pub trait Message: Debug + Encode {
    fn join() -> Self;
}

impl Message for () {
    fn join() -> Self {
        
    }
}

impl Message for MulticastMessage {
    fn join() -> Self {
        Self::Join
    }
}
