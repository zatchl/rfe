mod config;
mod config_amp_sweep;
mod config_cw;
mod config_freq_sweep;
mod setup;
mod signal_generator;
mod temperature;

pub use config::Config;
pub use setup::Setup;
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
