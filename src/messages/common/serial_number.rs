use rfe_message::RfeMessage;

#[derive(Debug, Clone, Eq, PartialEq, RfeMessage)]
#[prefix = "#Sn"]
pub struct SerialNumberMessage {
    serial_number: String,
}

impl SerialNumberMessage {
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn reject_with_invalid_prefix() {
        assert!(SerialNumberMessage::try_from(b"$Sn0SME38SI2X7NGR48".as_ref()).is_err());
    }

    #[test]
    fn accept_valid_serial_number() {
        assert!(SerialNumberMessage::try_from(b"#Sn0SME38SI2X7NGR48".as_ref()).is_ok())
    }
}
