use crate::Model;
use rfe_message::RfeMessage;

#[derive(Debug, Clone, RfeMessage)]
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
        assert_eq!(setup.main_model(), Model::RfeWSub1G);
        assert_eq!(setup.expansion_model(), None);
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_24g_setup() {
        let setup = Setup::try_from(b"#C2-M:004,255,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::Rfe2400);
        assert_eq!(setup.expansion_model(), None);
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_ism_combo_setup() {
        let setup = Setup::try_from(b"#C2-M:003,004,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::RfeWSub1G);
        assert_eq!(setup.expansion_model(), Some(Model::Rfe2400));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_3g_combo_setup() {
        let setup = Setup::try_from(b"#C2-M:003,005,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::RfeWSub1G);
        assert_eq!(setup.expansion_model(), Some(Model::RfeWSub3G));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_6g_combo_setup() {
        let setup = Setup::try_from(b"#C2-M:006,005,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::Rfe6G);
        assert_eq!(setup.expansion_model(), Some(Model::RfeWSub3G));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_wsub1g_plus_setup() {
        let setup = Setup::try_from(b"#C2-M:010,255,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::RfeWSub1GPlus);
        assert_eq!(setup.expansion_model(), None);
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_ism_combo_plus_setup() {
        let setup = Setup::try_from(b"#C2-M:010,012,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::RfeWSub1GPlus);
        assert_eq!(setup.expansion_model(), Some(Model::Rfe2400Plus));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_4g_combo_plus_setup() {
        let setup = Setup::try_from(b"#C2-M:010,013,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::RfeWSub1GPlus);
        assert_eq!(setup.expansion_model(), Some(Model::Rfe4GPlus));
        assert_eq!(setup.firmware_version(), "XX.XXXX".to_string());
    }

    #[test]
    fn accept_6g_combo_plus_setup() {
        let setup = Setup::try_from(b"#C2-M:010,014,XX.XXXX".as_ref()).unwrap();
        assert_eq!(setup.main_model(), Model::RfeWSub1GPlus);
        assert_eq!(setup.expansion_model(), Some(Model::Rfe6GPlus));
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
