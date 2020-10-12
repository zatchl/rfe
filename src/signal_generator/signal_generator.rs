use crate::rf_explorer::{Result, RfExplorer, SerialPortReader};
use crate::spectrum_analyzer::{Config, Setup};
use num_enum::IntoPrimitive;
use std::fmt::Debug;

pub struct SignalGenerator {
    reader: SerialPortReader,
    setup: Setup,
    config: Config,
    message_buf: Vec<u8>,
}

#[derive(Copy, Clone, Debug, IntoPrimitive)]
#[repr(u8)]
pub enum Attenuation {
    On = b'0',
    Off = b'1',
}

#[derive(Copy, Clone, Debug, IntoPrimitive)]
#[repr(u8)]
pub enum PowerLevel {
    Lowest = b'0',
    Low = b'1',
    High = b'2',
    Highest = b'3',
}

impl SignalGenerator {
    pub fn enable_cw(
        &mut self,
        cw_freq_khz: f64,
        attenuation: Attenuation,
        power_level: PowerLevel,
    ) -> Result<()> {
        let command = format!(
            "C3-F:{:07.0},{},{}",
            cw_freq_khz,
            u8::from(attenuation),
            u8::from(power_level)
        );
        self.write_command(command.as_bytes())
    }

    pub fn enable_cw_exp(&mut self, cw_freq_khz: f64, power_dbm: f64) {
        todo!()
    }

    pub fn start_freq_sweep(
        &mut self,
        start_freq_khz: f64,
        attenuation: Attenuation,
        power_level: PowerLevel,
        sweep_steps: u16,
        freq_step_khz: f64,
        step_delay_ms: u32,
    ) -> Result<()> {
        let command = format!(
            "C3-F:{:07.0},{},{},{:04},{:07.0},{:05}",
            start_freq_khz,
            u8::from(attenuation),
            u8::from(power_level),
            sweep_steps,
            freq_step_khz,
            step_delay_ms
        );
        self.write_command(command.as_bytes())
    }

    pub fn start_freq_sweep_exp(
        &mut self,
        start_freq_khz: f64,
        power_dbm: f64,
        sweep_steps: u16,
        freq_step_khz: f64,
        step_delay_ms: u32,
    ) {
        todo!()
    }

    pub fn start_tracking(
        &mut self,
        start_freq_khz: f64,
        attenuation: Attenuation,
        power_level: PowerLevel,
        sweep_steps: u16,
        freq_step_khz: f64,
    ) -> Result<()> {
        let command = format!(
            "C3-T:{:07.0},{},{},{:04},{:07.0}",
            start_freq_khz,
            u8::from(attenuation),
            u8::from(power_level),
            sweep_steps,
            freq_step_khz
        );
        self.write_command(command.as_bytes())
    }

    pub fn start_tracking_exp(
        &mut self,
        start_freq_khz: f64,
        power_dbm: f64,
        sweep_steps: u16,
        freq_step_khz: f64,
    ) {
        todo!()
    }

    pub fn start_amp_sweep(
        &mut self,
        cw_freq_khz: f64,
        start_attenuation: Attenuation,
        start_power_level: PowerLevel,
        end_attenuation: Attenuation,
        end_power: PowerLevel,
        step_delay_ms: u32,
    ) -> Result<()> {
        let command = format!(
            "C3-A:{:07.0},{},{},{},{},{:05}",
            cw_freq_khz,
            u8::from(start_attenuation),
            u8::from(start_power_level),
            u8::from(end_attenuation),
            u8::from(end_power),
            step_delay_ms
        );
        self.write_command(command.as_bytes())
    }

    pub fn start_amp_sweep_exp(
        &mut self,
        cw_freq_khz: f64,
        start_power_dbm: f64,
        step_power_db: f64,
        stop_power_dbm: f64,
        step_delay_ms: u32,
    ) {
        todo!()
    }

    pub fn enable_rf_power(&mut self) -> Result<()> {
        self.write_command(b"CP1")
    }

    pub fn disable_rf_power(&mut self) -> Result<()> {
        self.write_command(b"CP0")
    }

    pub fn set_tracking_steps(&mut self, tracking_steps: u16) -> Result<()> {
        let step_bytes = tracking_steps.to_be_bytes();
        self.write_command(&[b'k', step_bytes[0], step_bytes[1]])
    }
}

impl_rf_explorer!(SignalGenerator, Setup, Config);

impl Debug for SignalGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalGenerator")
            .field("setup", &self.setup)
            .field("config", &self.config)
            .finish()
    }
}
