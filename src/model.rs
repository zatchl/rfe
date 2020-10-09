use crate::messages::ParseMessageError;
use num_enum::TryFromPrimitive;
use std::{convert::TryFrom, str, str::FromStr};

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum Model {
    FourThreeThreeM = 0,
    EightSixEightM = 1,
    NineOneFiveM = 2,
    WSubOneG = 3,
    TwoPointFourG = 4,
    WSubThreeG = 5,
    SixG = 6,
    WSub1GPlus = 10,
    ProAudio = 11,
    TwoPointFourGPlus = 12,
    FourGPlus = 13,
    SixGPlus = 14,
}

impl FromStr for Model {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(u8::from_str(s)?).map_err(|_| ParseMessageError::InvalidData)
    }
}
