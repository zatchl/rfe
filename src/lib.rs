pub mod devices;
pub mod messages;
mod model;

pub use devices::{ConnectionError, Error, Result, RfExplorer, SignalGenerator, SpectrumAnalyzer};
pub use model::Model;

use serialport;
use std::convert::TryFrom;

pub fn signal_generators() -> Vec<SignalGenerator> {
    serialport::available_ports()
        .unwrap_or_default()
        .iter()
        .filter_map(|serial_port_info| SignalGenerator::try_from(serial_port_info).ok())
        .collect()
}

pub fn spectrum_analyzers() -> Vec<SpectrumAnalyzer> {
    serialport::available_ports()
        .unwrap_or_default()
        .iter()
        .filter_map(|serial_port_info| SpectrumAnalyzer::try_from(serial_port_info).ok())
        .collect()
}
