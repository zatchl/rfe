use crate::rf_explorer::Message;
use nom::{
    bytes::complete::{tag, take_while_m_n},
    character::complete::line_ending,
    character::is_alphanumeric,
    combinator::{all_consuming, map, map_res, opt},
    sequence::preceded,
    IResult,
};
use std::str;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SerialNumber {
    serial_number: String,
}

impl SerialNumber {
    const PREFIX: &'static [u8] = b"#Sn";

    pub fn as_str(&self) -> &str {
        &self.serial_number
    }
}

impl Message for SerialNumber {
    fn from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (bytes, serial_number) = preceded(
            tag(SerialNumber::PREFIX),
            map(
                map_res(take_while_m_n(16, 16, is_alphanumeric), str::from_utf8),
                str::to_string,
            ),
        )(bytes)?;

        // Consume any \r or \r\n line endings and make sure there aren't any bytes left
        let (bytes, _) = all_consuming(opt(line_ending))(bytes)?;

        Ok((bytes, SerialNumber { serial_number }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_with_invalid_prefix() {
        assert!(SerialNumber::from_bytes(b"$Sn0SME38SI2X7NGR48".as_ref()).is_err());
    }

    #[test]
    fn accept_valid_serial_number() {
        assert!(SerialNumber::from_bytes(b"#Sn0SME38SI2X7NGR48".as_ref()).is_ok())
    }
}
