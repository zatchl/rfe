use crate::messages::ParseMessageError;
use num_enum::TryFromPrimitive;
use std::{convert::TryFrom, str, str::FromStr};

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum Model {
    Rfe433 = 0,
    Rfe868 = 1,
    Rfe915 = 2,
    RfeWSub1G = 3,
    Rfe2400 = 4,
    RfeWSub3G = 5,
    Rfe6G = 6,
    RfeWSub1GPlus = 10,
    RfeAudioPro = 11,
    Rfe2400Plus = 12,
    Rfe4GPlus = 13,
    Rfe6GPlus = 14,
}

impl FromStr for Model {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(u8::from_str(s)?).map_err(|_| ParseMessageError::InvalidData)
    }
}
