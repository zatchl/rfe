use crate::rf_explorer::ParseMessageError;
use num_enum::TryFromPrimitive;
use rfe_message::RfeMessage;
use std::{convert::TryFrom, str::FromStr};

#[derive(Debug, Copy, Clone, RfeMessage)]
#[prefix = "#T:"]
pub struct Temperature {
    temperature_range: TemperatureRange,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
enum TemperatureRange {
    MinusTenToZero = b'0',
    ZeroToTen = b'1',
    TenToTwenty = b'2',
    TwentyToThirty = b'3',
    ThirtyToForty = b'4',
    FortyToFifty = b'5',
    FiftyToSixty = b'6',
}

impl FromStr for TemperatureRange {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(u8::from_str(s)?).map_err(|_| ParseMessageError::InvalidData)
    }
}
