use crate::{
    rf_explorer::{RfExplorer, SerialPortReader, WriteCommandResult},
    signal_generator::{Attenuation, Config, PowerLevel, Setup},
};
use std::fmt::Debug;
use uom::si::{f64::Frequency, frequency::kilohertz};

pub struct SignalGenerator {
    reader: SerialPortReader,
    setup: Setup,
    config: Config,
}

impl SignalGenerator {
    pub fn start_cw(
        &mut self,
        cw_freq: Frequency,
        attenuation: Attenuation,
        power_level: PowerLevel,
    ) -> WriteCommandResult<()> {
        let command = format!(
            "C3-F:{:07.0},{},{}",
            cw_freq.get::<kilohertz>(),
            u8::from(attenuation),
            u8::from(power_level)
        );
        self.write_command(command.as_bytes())
    }

    pub fn start_cw_exp(&mut self, cw_freq: Frequency, power_dbm: f64) -> WriteCommandResult<()> {
        let command = format!(
            "C5-F:{:07.0},{:+05.1}",
            cw_freq.get::<kilohertz>(),
            power_dbm
        );
        self.write_command(command.as_bytes())
    }

    pub fn start_freq_sweep(
        &mut self,
        start_freq: Frequency,
        attenuation: Attenuation,
        power_level: PowerLevel,
        sweep_steps: u16,
        freq_step: Frequency,
        step_delay_ms: u32,
    ) -> WriteCommandResult<()> {
        let command = format!(
            "C3-F:{:07.0},{},{},{:04},{:07.0},{:05}",
            start_freq.get::<kilohertz>(),
            u8::from(attenuation),
            u8::from(power_level),
            sweep_steps,
            freq_step.get::<kilohertz>(),
            step_delay_ms
        );
        self.write_command(command.as_bytes())
    }

    pub fn start_freq_sweep_exp(
        &mut self,
        start_freq: Frequency,
        power_dbm: f64,
        sweep_steps: u16,
        freq_step: Frequency,
        step_delay_ms: u32,
    ) -> WriteCommandResult<()> {
        let command = format!(
            "C5-F:{:07.0},{:+05.1},{:04},{:07.0},{:05}",
            start_freq.get::<kilohertz>(),
            power_dbm,
            sweep_steps,
            freq_step.get::<kilohertz>(),
            step_delay_ms
        );
        self.write_command(command.as_bytes())
    }

    pub fn start_tracking(
        &mut self,
        start_freq: Frequency,
        attenuation: Attenuation,
        power_level: PowerLevel,
        sweep_steps: u16,
        freq_step: Frequency,
    ) -> WriteCommandResult<()> {
        let command = format!(
            "C3-T:{:07.0},{},{},{:04},{:07.0}",
            start_freq.get::<kilohertz>(),
            u8::from(attenuation),
            u8::from(power_level),
            sweep_steps,
            freq_step.get::<kilohertz>()
        );
        self.write_command(command.as_bytes())
    }

    pub fn start_tracking_exp(
        &mut self,
        start_freq: Frequency,
        power_dbm: f64,
        sweep_steps: u16,
        freq_step: Frequency,
    ) -> WriteCommandResult<()> {
        let command = format!(
            "C5-T:{:07.0},{:+05.1},{:04},{:07.0}",
            start_freq.get::<kilohertz>(),
            power_dbm,
            sweep_steps,
            freq_step.get::<kilohertz>()
        );
        self.write_command(command.as_bytes())
    }

    pub fn start_amp_sweep(
        &mut self,
        cw_freq: Frequency,
        start_attenuation: Attenuation,
        start_power_level: PowerLevel,
        end_attenuation: Attenuation,
        end_power: PowerLevel,
        step_delay_ms: u32,
    ) -> WriteCommandResult<()> {
        let command = format!(
            "C3-A:{:07.0},{},{},{},{},{:05}",
            cw_freq.get::<kilohertz>(),
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
        cw_freq: Frequency,
        start_power_dbm: f64,
        step_power_db: f64,
        stop_power_dbm: f64,
        step_delay_ms: u32,
    ) -> WriteCommandResult<()> {
        let command = format!(
            "C5-A:{:07.0},{:+05.1},{:+05.1},{:05.1},{:05}",
            cw_freq.get::<kilohertz>(),
            start_power_dbm,
            step_power_db,
            stop_power_dbm,
            step_delay_ms
        );
        self.write_command(command.as_bytes())
    }

    pub fn enable_rf_power(&mut self) -> WriteCommandResult<()> {
        self.write_command(b"CP1")
    }

    pub fn disable_rf_power(&mut self) -> WriteCommandResult<()> {
        self.write_command(b"CP0")
    }

    pub fn set_tracking_steps(&mut self, tracking_steps: u16) -> WriteCommandResult<()> {
        let step_bytes = tracking_steps.to_be_bytes();
        self.write_command(&[b'k', step_bytes[0], step_bytes[1]])
    }
}

impl RfExplorer for SignalGenerator {
    type Setup = super::Setup;
    type Config = super::Config;

    fn new(reader: SerialPortReader, setup: Self::Setup, config: Self::Config) -> Self {
        SignalGenerator {
            reader,
            setup,
            config,
        }
    }

    fn reader(&mut self) -> &mut SerialPortReader {
        &mut self.reader
    }

    fn setup(&self) -> Self::Setup {
        self.setup.clone()
    }
}

impl Debug for SignalGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalGenerator")
            .field("setup", &self.setup)
            .field("config", &self.config)
            .finish()
    }
}
