#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(feature = "std")]
mod std_alloc {
    pub(crate) use std::borrow::ToOwned;
    pub(crate) use std::string::String;
    pub(crate) use std::vec::Vec;
}

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(not(feature = "std"))]
mod std_alloc {
    pub(crate) use alloc::borrow::ToOwned;
    pub(crate) use alloc::string::String;
    pub(crate) use alloc::vec::Vec;
}

pub mod challenges;

use anyhow::{anyhow, Result};
use core::fmt::{self, Display};
use serde_derive::{Deserialize, Serialize};
use std_alloc::{String, ToOwned, Vec};

#[derive(Debug, Deserialize, Serialize)]
pub enum Challenge {
    CPUChallenge(Vec<u8>),
    NetworkChallenge(Vec<u8>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Response {
    CPUChallengeResponse(Vec<u8>),
    NetworkChallengeResponse(Vec<u8>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Data {
    /// Indicates general log/information we want to be displayed at client side.
    Info(String),
    /// Indicates that an error occurred on other side of the connection
    Error(String),
    /// Same as `Info` but is used to convey results of the measurements
    Result(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    /// Challenge message sent by server to client
    Challenge(Challenge),
    /// Response for the `Challenge` message
    Response(Response),
    /// General data
    Data(Data),
    /// Invalid Message; usually indicates a bug
    Unknown,
}

impl Message {
    #[inline]
    pub fn encode(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self).map_err(|e| anyhow!("Error encoding a Message: {:?}", e))
    }

    #[inline]
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        rmp_serde::from_read(bytes).map_err(|e| anyhow!("Error decoding a Message: {:?}", e))
    }
}

impl Display for Message {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Message::Challenge(_) => "Challenge".to_owned(),
                Message::Response(_) => "Response".to_owned(),
                Message::Data(_) => "Data".to_owned(),
                Message::Unknown => "Unknown".to_owned(),
            }
        )
    }
}
