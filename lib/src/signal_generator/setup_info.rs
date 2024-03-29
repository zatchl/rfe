use super::Model;
use crate::common::MessageParseError;
use crate::rf_explorer::SetupInfo;

impl SetupInfo<Model> {
    pub(crate) const PREFIX: &'static [u8] = b"#C3-M:";
}

impl<'a> TryFrom<&'a [u8]> for SetupInfo<Model> {
    type Error = MessageParseError<'a>;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        SetupInfo::try_from_with_prefix(bytes, Self::PREFIX)
    }
}

#[cfg(test)]
mod tests {
    use crate::rf_explorer::SetupInfo;
    use crate::signal_generator::Model;

    #[test]
    fn accept_rfe_gen_setup() {
        let setup = SetupInfo::<Model>::try_from(b"#C3-M:060,255,01.15\r\n".as_ref()).unwrap();
        assert_eq!(setup.main_radio_model, Some(Model::Rfe6Gen));
        assert_eq!(setup.expansion_radio_model, None);
        assert_eq!(setup.firmware_version, "01.15");
    }

    #[test]
    fn accept_rfe_gen_combo_setup() {
        let setup = SetupInfo::<Model>::try_from(b"#C3-M:060,061,01.15\r\n".as_ref()).unwrap();
        assert_eq!(setup.main_radio_model, Some(Model::Rfe6Gen));
        assert_eq!(setup.expansion_radio_model, Some(Model::Rfe6GenExpansion));
        assert_eq!(setup.firmware_version, "01.15");
    }
}
