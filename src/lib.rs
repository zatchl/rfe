mod config;
mod rf_explorer;
mod serial_number;
mod setup;
mod sweep;

pub use config::{ParseConfigError, RfExplorerConfig};
pub use rf_explorer::RfExplorer;
pub use serial_number::{ParseSerialNumberError, RfExplorerSerialNumber};
pub use setup::{ParseSetupError, RfExplorerSetup};
pub use sweep::{ParseSweepError, RfExplorerSweep};

use num_enum::TryFromPrimitive;
use serialport::{self, Error};
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum RfExplorerModel {
    Rfe433 = 0,
    Rfe868 = 1,
    Rfe915 = 2,
    RfeWSub1G = 3,
    Rfe2400 = 4,
    RfeWSub3G = 5,
    Rfe6G = 6,
    RfeWSub1GPlus = 10,
    RfeAudioPro = 11,
    Rfe2400Plus = 12,
    Rfe4GPlus = 13,
    Rfe6GPlus = 14,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum RfExplorerMode {
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

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum RfExplorerCalcMode {
    Normal = 0,
    Max,
    Avg,
    Overwrite,
    MaxHold,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum RfExplorerWifiMode {
    Disable = 0,
    TwoPointFourGhz,
    FiveGhz,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum RfExplorerDspMode {
    Auto = b'0',
    Filter = b'1',
    Fast = b'2',
}

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum RfExplorerInputStage {
    Bypass = b'0',
    Attenuator30dB = b'1',
    Lna25dB = b'2',
}

pub fn available_rf_explorers() -> Result<Vec<RfExplorer>, Error> {
    Ok(serialport::available_ports()?
        .iter()
        .filter_map(|serial_port_info| RfExplorer::try_from(serial_port_info).ok())
        .collect())
}
