mod messages;
mod model;
#[macro_use]
mod rf_explorer;
mod signal_generator;
mod spectrum_analyzer;

pub use messages::RfeMessage;
pub use model::Model;
pub use rf_explorer::{Error, Result, RfExplorer};
pub use signal_generator::SignalGenerator;
pub use spectrum_analyzer::SpectrumAnalyzer;

use serialport::{self, Error as SerialPortError};
use std::convert::TryFrom;

pub fn signal_generators() -> std::result::Result<Vec<SignalGenerator>, SerialPortError> {
    Ok(serialport::available_ports()?
        .iter()
        .filter_map(|serial_port_info| SignalGenerator::try_from(serial_port_info).ok())
        .collect())
}

pub fn spectrum_analyzers() -> std::result::Result<Vec<SpectrumAnalyzer>, SerialPortError> {
    Ok(serialport::available_ports()?
        .iter()
        .filter_map(|serial_port_info| SpectrumAnalyzer::try_from(serial_port_info).ok())
        .collect())
}
