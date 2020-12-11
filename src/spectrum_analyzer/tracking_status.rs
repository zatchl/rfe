use crate::rf_explorer::{Message, ParseFromBytes};
use nom::{
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::{all_consuming, map_res, opt},
    number::complete::u8 as nom_u8,
    IResult,
};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum TrackingStatus {
    Disabled = 0,
    Enabled,
}

impl Message for TrackingStatus {
    const PREFIX: &'static [u8] = b"#K";
}

impl ParseFromBytes for TrackingStatus {
    fn parse_from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        // Parse the prefix of the message
        let (bytes, _) = tag(TrackingStatus::PREFIX)(bytes)?;

        // Parse the tracking status
        let (bytes, tracking_status) = map_res(nom_u8, TrackingStatus::try_from)(bytes)?;

        // Consume any \r or \r\n line endings and make sure there aren't any bytes left
        let (bytes, _) = all_consuming(opt(line_ending))(bytes)?;

        Ok((bytes, tracking_status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_valid_tracking_status_message() {
        let bytes = [b'#', b'K', 0];
        let tracking_status = TrackingStatus::parse_from_bytes(bytes.as_ref()).unwrap().1;
        assert_eq!(tracking_status, TrackingStatus::Disabled);
    }
}
