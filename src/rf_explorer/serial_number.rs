use rfe_message::Message;

#[derive(Debug, Clone, Eq, PartialEq, Message)]
#[prefix = "#Sn"]
pub struct SerialNumber {
    serial_number: String,
}

impl SerialNumber {
    pub fn as_str(&self) -> &str {
        &self.serial_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn reject_with_invalid_prefix() {
        assert!(SerialNumber::try_from(b"$Sn0SME38SI2X7NGR48".as_ref()).is_err());
    }

    #[test]
    fn accept_valid_serial_number() {
        assert!(SerialNumber::try_from(b"#Sn0SME38SI2X7NGR48".as_ref()).is_ok())
    }
}
