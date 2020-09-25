mod config;
mod rf_explorer;
mod serial_number;
mod setup;
mod sweep;

pub use config::{ParseConfigError, RfExplorerConfig, RfExplorerMode};
pub use rf_explorer::RfExplorer;
pub use serial_number::{ParseSerialNumberError, RfExplorerSerialNumber};
pub use setup::{ParseSetupError, RfExplorerModel, RfExplorerSetup};
pub use sweep::{ParseSweepError, RfExplorerSweep};

use num_enum::TryFromPrimitive;
use serialport::{self, Error};
use std::convert::TryFrom;

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
