mod command;
mod config;
mod dsp_mode;
mod parsers;
mod setup_info;
mod spectrum_analyzer;
mod sweep;
mod tracking_status;

pub(crate) use command::Command;
pub use config::Config;
pub use dsp_mode::DspMode;
pub use setup_info::SetupInfo;
pub use spectrum_analyzer::SpectrumAnalyzer;
pub use sweep::Sweep;
pub use tracking_status::TrackingStatus;

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive)]
#[repr(u8)]
pub enum InputStage {
    Bypass = b'0',
    Attenuator30dB = b'1',
    Lna25dB = b'2',
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive)]
#[repr(u8)]
pub enum WifiBand {
    TwoPointFourGhz = 1,
    FiveGhz,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum RadioModule {
    Main = 0,
    Expansion,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum Mode {
    SpectrumAnalyzer = 0,
    RfGenerator = 1,
    WifiAnalyzer = 2,
    AnalyzerTracking = 5,
    RfSniffer = 6,
    CwTransmitter = 60,
    SweepFrequency = 61,
    SweepAmplitude = 62,
    GeneratorTracking = 63,
    Unknown = 255,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::SpectrumAnalyzer
    }
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, IntoPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum CalcMode {
    Normal = 0,
    Max,
    Avg,
    Overwrite,
    MaxHold,
}

impl Default for RadioModule {
    fn default() -> Self {
        RadioModule::Main
    }
}
