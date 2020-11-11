pub mod rf_explorer;
pub mod signal_generator;
pub mod spectrum_analyzer;

pub use rf_explorer::{Error, Message, Model, RfExplorer};
pub use signal_generator::SignalGenerator;
pub use spectrum_analyzer::SpectrumAnalyzer;

pub use uom;

use serialport;

/// Returns every RF Explorer signal generator connected to the machine.
pub fn signal_generators() -> Vec<SignalGenerator> {
    serialport::available_ports()
        .unwrap_or_default()
        .iter()
        .filter_map(|port_info| SignalGenerator::connect(port_info).ok())
        .collect()
}

/// Returns every RF Explorer spectrum analyzer connected to the machine.
pub fn spectrum_analyzers() -> Vec<SpectrumAnalyzer> {
    serialport::available_ports()
        .unwrap_or_default()
        .iter()
        .filter_map(|port_info| SpectrumAnalyzer::connect(port_info).ok())
        .collect()
}
