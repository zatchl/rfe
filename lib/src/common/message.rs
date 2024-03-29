use std::fmt::Debug;

use nom::{error::Error, Err};
use thiserror::Error;

use super::ConnectionResult;

pub trait MessageContainer: Default + Debug + Send + Sync {
    type Message: for<'a> TryFrom<&'a [u8], Error = MessageParseError<'a>> + Debug;
    fn cache_message(&self, message: Self::Message);
    fn wait_for_device_info(&self) -> ConnectionResult<()>;
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum MessageParseError<'a> {
    #[error("Attempted to parse an incomplete message")]
    Incomplete,

    #[error("Attempted to parse a truncated message")]
    Truncated { remainder: Option<&'a [u8]> },

    #[error("Attempted to parse an invalid message")]
    Invalid,

    #[error("Attempted to parse an unknown message type")]
    UnknownMessageType,
}

impl<'a> From<Err<Error<&[u8]>>> for MessageParseError<'a> {
    fn from(error: Err<Error<&[u8]>>) -> Self {
        match error {
            Err::Incomplete(_) => MessageParseError::Incomplete,
            _ => MessageParseError::Invalid,
        }
    }
}
