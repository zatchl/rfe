use crate::Model;
use rfe_message::Message;

#[derive(Debug, Clone, Message)]
#[prefix = "#C3-M:"]
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
