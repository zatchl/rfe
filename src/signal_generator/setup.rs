use crate::rf_explorer::{Message, Model, ParseFromBytes};
use nom::IResult;
use std::str;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Setup {
    main_model: Model,
    exp_model: Option<Model>,
    fw_version: String,
}

impl Setup {
    pub fn main_model(&self) -> Model {
        self.main_model
    }

    pub fn expansion_model(&self) -> Option<Model> {
        self.exp_model
    }

    pub fn firmware_version(&self) -> &str {
        &self.fw_version
    }
}

impl crate::rf_explorer::Setup for Setup {
    fn new(main_model: Model, exp_model: Option<Model>, fw_version: String) -> Self {
        Setup {
            main_model,
            exp_model,
            fw_version,
        }
    }
}

impl Message for Setup {
    const PREFIX: &'static [u8] = b"#C3-M:";
}

impl ParseFromBytes for Setup {
    fn parse_from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        crate::rf_explorer::Setup::parse_from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Model;

    #[test]
    fn accept_rfe_gen_setup() {
        let setup = Setup::parse_from_bytes(b"#C3-M:060,255,01.15\r\n".as_ref())
            .unwrap()
            .1;
        assert_eq!(setup.main_model(), Model::RfeGen);
        assert_eq!(setup.expansion_model(), None);
        assert_eq!(setup.firmware_version(), "01.15");
    }
}
