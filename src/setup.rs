use crate::RfExplorerModel;
use std::{convert::TryFrom, str, str::FromStr};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RfExplorerSetup {
    main_model: RfExplorerModel,
    expansion_model: Option<RfExplorerModel>,
    firmware_version: String,
}

#[derive(Error, Debug)]
pub enum ParseSetupError {
    #[error(transparent)]
    InvalidRfExplorerModel(#[from] <RfExplorerModel as TryFrom<u8>>::Error),

    #[error("Invalid RfExplorerSetup: expected bytes to start with #C2-M:")]
    InvalidFormat,

    #[error("A required field is missing from the bytes")]
    MissingField,

    #[error(transparent)]
    InvalidInt(#[from] std::num::ParseIntError),

    #[error(transparent)]
    InvalidUtf8(#[from] std::str::Utf8Error),

    #[error(transparent)]
    InvalidString(#[from] std::string::ParseError),
}

fn parse_field<T>(field: Option<&[u8]>) -> Result<T, ParseSetupError>
where
    T: FromStr,
    ParseSetupError: From<T::Err>,
{
    Ok(T::from_str(
        str::from_utf8(field.ok_or_else(|| ParseSetupError::MissingField)?)?.trim(),
    )?)
}

impl RfExplorerSetup {
    pub fn main_model(&self) -> RfExplorerModel {
        self.main_model
    }

    pub fn expansion_model(&self) -> Option<RfExplorerModel> {
        self.expansion_model
    }

    pub fn firmware_version(&self) -> &str {
        self.firmware_version.as_str()
    }
}

impl TryFrom<&[u8]> for RfExplorerSetup {
    type Error = ParseSetupError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.starts_with(b"#C2-M:") {
            let mut fields = value
                .get(6..)
                .ok_or_else(|| ParseSetupError::MissingField)?
                .split(|&byte| byte == b',');

            Ok(RfExplorerSetup {
                main_model: RfExplorerModel::try_from(parse_field::<u8>(fields.next())?)?,
                expansion_model: RfExplorerModel::try_from(parse_field::<u8>(fields.next())?).ok(),
                firmware_version: parse_field::<String>(fields.next())?,
            })
        } else {
            Err(ParseSetupError::InvalidFormat)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_wsub1g_setup() {
        let setup = RfExplorerSetup::try_from(b"#C2-M:003,255,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), RfExplorerModel::RfeWSub1G);
        assert_eq!(setup.expansion_model(), None);
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_24g_setup() {
        let setup = RfExplorerSetup::try_from(b"#C2-M:004,255,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), RfExplorerModel::Rfe2400);
        assert_eq!(setup.expansion_model(), None);
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_ism_combo_setup() {
        let setup = RfExplorerSetup::try_from(b"#C2-M:003,004,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), RfExplorerModel::RfeWSub1G);
        assert_eq!(setup.expansion_model(), Some(RfExplorerModel::Rfe2400));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_3g_combo_setup() {
        let setup = RfExplorerSetup::try_from(b"#C2-M:003,005,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), RfExplorerModel::RfeWSub1G);
        assert_eq!(setup.expansion_model(), Some(RfExplorerModel::RfeWSub3G));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_6g_combo_setup() {
        let setup = RfExplorerSetup::try_from(b"#C2-M:006,005,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), RfExplorerModel::Rfe6G);
        assert_eq!(setup.expansion_model(), Some(RfExplorerModel::RfeWSub3G));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_wsub1g_plus_setup() {
        let setup = RfExplorerSetup::try_from(b"#C2-M:010,255,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), RfExplorerModel::RfeWSub1GPlus);
        assert_eq!(setup.expansion_model(), None);
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_ism_combo_plus_setup() {
        let setup = RfExplorerSetup::try_from(b"#C2-M:010,012,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), RfExplorerModel::RfeWSub1GPlus);
        assert_eq!(setup.expansion_model(), Some(RfExplorerModel::Rfe2400Plus));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_4g_combo_plus_setup() {
        let setup = RfExplorerSetup::try_from(b"#C2-M:010,013,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), RfExplorerModel::RfeWSub1GPlus);
        assert_eq!(setup.expansion_model(), Some(RfExplorerModel::Rfe4GPlus));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_6g_combo_plus_setup() {
        let setup = RfExplorerSetup::try_from(b"#C2-M:010,014,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), RfExplorerModel::RfeWSub1GPlus);
        assert_eq!(setup.expansion_model(), Some(RfExplorerModel::Rfe6GPlus));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn reject_setup_without_main_model() {
        assert!(RfExplorerSetup::try_from(b"#C2-M:255,005,01.12B26".as_ref()).is_err());
    }

    #[test]
    fn accept_setup_without_expansion_model() {
        assert!(RfExplorerSetup::try_from(b"#C2-M:006,255,01.12B26".as_ref()).is_ok());
    }

    #[test]
    fn reject_setup_without_firmware_version() {
        assert!(RfExplorerSetup::try_from(b"#C2-M:006,005".as_ref()).is_err());
    }

    #[test]
    fn reject_setup_with_incorrect_prefix() {
        assert!(RfExplorerSetup::try_from(b"$C2-M:006,005,01.12B26".as_ref()).is_err());
    }
}
