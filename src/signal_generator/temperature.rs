use crate::rf_explorer::Message;
use nom::{
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::{all_consuming, map_res, opt},
    number::complete::u8 as nom_u8,
    IResult,
};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::ops::RangeInclusive;

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Temperature {
    MinusTenToZero = b'0',
    ZeroToTen = b'1',
    TenToTwenty = b'2',
    TwentyToThirty = b'3',
    ThirtyToForty = b'4',
    FortyToFifty = b'5',
    FiftyToSixty = b'6',
}

impl Temperature {
    const PREFIX: &'static [u8] = b"#T:";

    pub fn range(&self) -> RangeInclusive<i8> {
        match self {
            Temperature::MinusTenToZero => -10..=0,
            Temperature::ZeroToTen => 0..=10,
            Temperature::TenToTwenty => 10..=20,
            Temperature::TwentyToThirty => 20..=30,
            Temperature::ThirtyToForty => 30..=40,
            Temperature::FortyToFifty => 40..=50,
            Temperature::FiftyToSixty => 50..=60,
        }
    }
}

impl Message for Temperature {
    fn from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        // Parse the prefix of the message
        let (bytes, _) = tag(Temperature::PREFIX)(bytes)?;

        // Parse the temperature
        let (bytes, temperature) = map_res(nom_u8, Temperature::try_from)(bytes)?;

        // Consume any \r or \r\n line endings and make sure there aren't any bytes left
        let (bytes, _) = all_consuming(opt(line_ending))(bytes)?;

        Ok((bytes, temperature))
    }
}
