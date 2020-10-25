use crate::rf_explorer::ParseMessageError;
use num_enum::TryFromPrimitive;
use std::{convert::TryFrom, fmt::Display, str, str::FromStr};

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum Model {
    Rfe433M = 0,
    Rfe868M = 1,
    Rfe915M = 2,
    RfeWSub1G = 3,
    Rfe24G = 4,
    RfeWSub3G = 5,
    Rfe6G = 6,
    RfeWSub1GPlus = 10,
    RfeProAudio = 11,
    Rfe24GPlus = 12,
    Rfe4GPlus = 13,
    Rfe6GPlus = 14,
}

impl Model {
    pub const fn is_plus_model(&self) -> bool {
        match self {
            Model::RfeWSub1GPlus | Model::Rfe24GPlus | Model::Rfe4GPlus | Model::Rfe6GPlus => true,
            _ => false,
        }
    }

    pub const fn has_wifi_analyzer(&self) -> bool {
        match self {
            Model::Rfe24G
            | Model::RfeWSub3G
            | Model::Rfe6G
            | Model::RfeProAudio
            | Model::Rfe24GPlus
            | Model::Rfe4GPlus
            | Model::Rfe6GPlus => true,
            _ => false,
        }
    }

    pub const fn min_freq_hz(&self) -> f64 {
        match self {
            Model::Rfe433M => 430_000_000.,
            Model::Rfe868M => 860_000_000.,
            Model::Rfe915M => 910_000_000.,
            Model::RfeWSub1G => 240_000_000.,
            Model::RfeWSub1GPlus => 50_000.,
            Model::Rfe24G | Model::Rfe24GPlus => 2_350_000_000.,
            Model::RfeWSub3G | Model::RfeProAudio => 15_000_000.,
            Model::Rfe6G => 4_850_000_000.,
            Model::Rfe4GPlus | Model::Rfe6GPlus => 240_000_000.,
        }
    }

    pub const fn max_freq_hz(&self) -> f64 {
        match self {
            Model::Rfe433M => 440_000_000.,
            Model::Rfe868M => 870_000_000.,
            Model::Rfe915M => 920_000_000.,
            Model::RfeWSub1G | Model::RfeWSub1GPlus => 960_000_000.,
            Model::Rfe24G | Model::Rfe24GPlus => 2_550_000_000.,
            Model::RfeWSub3G | Model::RfeProAudio => 2_700_000_000.,
            Model::Rfe4GPlus => 4_000_000_000.,
            Model::Rfe6G | Model::Rfe6GPlus => 6_100_000_000.,
        }
    }

    pub const fn min_span_hz(&self) -> f64 {
        match self {
            Model::Rfe433M
            | Model::Rfe868M
            | Model::Rfe915M
            | Model::RfeWSub1G
            | Model::Rfe24G
            | Model::RfeWSub3G
            | Model::RfeProAudio => 112_000.,
            Model::RfeWSub1GPlus => 100_000.,
            Model::Rfe24GPlus | Model::Rfe4GPlus | Model::Rfe6G | Model::Rfe6GPlus => 2_000_000.,
        }
    }

    pub const fn max_span_hz(&self) -> f64 {
        match self {
            Model::Rfe433M | Model::Rfe868M | Model::Rfe915M => 10_000_000.,
            Model::RfeWSub1G | Model::Rfe24G => 100_000_000.,
            Model::Rfe24GPlus => 85_000_000.,
            Model::RfeWSub3G | Model::RfeProAudio | Model::Rfe6G => 600_000_000.,
            Model::RfeWSub1GPlus | Model::Rfe4GPlus | Model::Rfe6GPlus => 960_000_000.,
        }
    }
}

impl FromStr for Model {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(u8::from_str(s)?).map_err(|_| ParseMessageError::InvalidData)
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Model::Rfe433M => write!(f, "433M"),
            Model::Rfe868M => write!(f, "868M"),
            Model::Rfe915M => write!(f, "915M"),
            Model::RfeWSub1G => write!(f, "WSUB1G"),
            Model::Rfe24G => write!(f, "2.4G"),
            Model::RfeWSub3G => write!(f, "WSUB3G"),
            Model::Rfe6G => write!(f, "6G"),
            Model::RfeWSub1GPlus => write!(f, "WSUB1G+"),
            Model::RfeProAudio => write!(f, "Pro Audio"),
            Model::Rfe24GPlus => write!(f, "2.4G+"),
            Model::Rfe4GPlus => write!(f, "4G+"),
            Model::Rfe6GPlus => write!(f, "6G+"),
        }
    }
}
