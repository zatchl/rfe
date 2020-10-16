use std::convert::TryFrom;
use thiserror::Error;

pub trait Message: for<'a> TryFrom<&'a [u8]> {
    const PREFIX: &'static [u8];
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseMessageError {
    #[error("")]
    MissingField,

    #[error("")]
    InvalidData,

    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    ParseStringError(#[from] std::convert::Infallible),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
}
