mod command;
mod config;
mod config_amp_sweep;
mod config_cw;
mod config_freq_sweep;
mod parsers;
mod setup_info;
mod signal_generator;
mod temperature;

pub(crate) use command::Command;
pub use config::Config;
pub use config_amp_sweep::ConfigAmpSweep;
pub use config_cw::ConfigCw;
pub use config_freq_sweep::ConfigFreqSweep;
pub use setup_info::SetupInfo;
pub use signal_generator::SignalGenerator;
pub use temperature::Temperature;

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Attenuation {
    On = 0,
    Off,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PowerLevel {
    Lowest = 0,
    Low,
    High,
    Highest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RfPower {
    On = 0,
    Off,
}
