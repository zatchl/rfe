use crate::Model;
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
    const PREFIX: &'static str = "#C3-M:";

    fn new(main_model: Model, exp_model: Option<Model>, fw_version: String) -> Self {
        Setup {
            main_model,
            exp_model,
            fw_version,
        }
    }
}
