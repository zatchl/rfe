mod command;
mod config;
mod config_amp_sweep;
mod config_cw;
mod config_freq_sweep;
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
    On = b'0',
    Off = b'1',
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PowerLevel {
    Lowest = b'0',
    Low = b'1',
    High = b'2',
    Highest = b'3',
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RfPower {
    On = b'0',
    Off = b'1',
}
