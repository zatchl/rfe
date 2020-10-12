use crate::Model;
use rfe_message::Message;

#[derive(Debug, Clone, Message)]
#[prefix = "#C2-M:"]
pub struct Setup {
    main_model: Model,
    #[optional]
    expansion_model: Option<Model>,
    firmware_version: String,
}

impl Setup {
    pub fn main_model(&self) -> Model {
        self.main_model
    }

    pub fn expansion_model(&self) -> Option<Model> {
        self.expansion_model
    }

    pub fn firmware_version(&self) -> &str {
        &self.firmware_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Model;
    use std::convert::TryFrom;

    #[test]
    fn accept_wsub1g_setup() {
        let setup = Setup::try_from(b"#C2-M:003,255,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::WSubOneG);
        assert_eq!(setup.expansion_model(), None);
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_24g_setup() {
        let setup = Setup::try_from(b"#C2-M:004,255,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::TwoPointFourG);
        assert_eq!(setup.expansion_model(), None);
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_ism_combo_setup() {
        let setup = Setup::try_from(b"#C2-M:003,004,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::WSubOneG);
        assert_eq!(setup.expansion_model(), Some(Model::TwoPointFourG));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_3g_combo_setup() {
        let setup = Setup::try_from(b"#C2-M:003,005,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::WSubOneG);
        assert_eq!(setup.expansion_model(), Some(Model::WSubThreeG));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_6g_combo_setup() {
        let setup = Setup::try_from(b"#C2-M:006,005,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::SixG);
        assert_eq!(setup.expansion_model(), Some(Model::WSubThreeG));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_wsub1g_plus_setup() {
        let setup = Setup::try_from(b"#C2-M:010,255,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::WSub1GPlus);
        assert_eq!(setup.expansion_model(), None);
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_ism_combo_plus_setup() {
        let setup = Setup::try_from(b"#C2-M:010,012,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::WSub1GPlus);
        assert_eq!(setup.expansion_model(), Some(Model::TwoPointFourGPlus));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_4g_combo_plus_setup() {
        let setup = Setup::try_from(b"#C2-M:010,013,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::WSub1GPlus);
        assert_eq!(setup.expansion_model(), Some(Model::FourGPlus));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_6g_combo_plus_setup() {
        let setup = Setup::try_from(b"#C2-M:010,014,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::WSub1GPlus);
        assert_eq!(setup.expansion_model(), Some(Model::SixGPlus));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn reject_setup_without_main_model() {
        assert!(Setup::try_from(b"#C2-M:255,005,01.12B26".as_ref()).is_err());
    }

    #[test]
    fn accept_setup_without_expansion_model() {
        assert!(Setup::try_from(b"#C2-M:006,255,01.12B26".as_ref()).is_ok());
    }

    #[test]
    fn reject_setup_without_firmware_version() {
        assert!(Setup::try_from(b"#C2-M:006,005".as_ref()).is_err());
    }

    #[test]
    fn reject_setup_with_incorrect_prefix() {
        assert!(Setup::try_from(b"$C2-M:006,005,01.12B26".as_ref()).is_err());
    }
}
