use super::AsyncTryFromSocketAddr;
use anyhow::Result;
use std::{fmt::Debug, io};

mod ipc;
mod socket;

pub use ipc::IpcCommunicator;
pub use socket::SocketCommunicator;

pub trait Communicator: AsyncTryFromSocketAddr + Debug {
    #[allow(async_fn_in_trait)]
    async fn communicate(&mut self, bytes: &[u8]) -> Result<usize, io::Error>;
}

mod error {
    use std::fmt::Display;

    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    #[derive(Debug, Serialize, Deserialize, Error)]
    pub struct CommunicatorProcessError(String);

    impl From<std::io::Error> for CommunicatorProcessError {
        fn from(value: std::io::Error) -> Self {
            Self(value.to_string())
        }
    }

    impl From<tokio::sync::mpsc::error::TryRecvError> for CommunicatorProcessError {
        fn from(value: tokio::sync::mpsc::error::TryRecvError) -> Self {
            Self(value.to_string())
        }
    }

    impl Display for CommunicatorProcessError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "CommunicatorProcessError: {}", self.0)
        }
    }
}
