use crate::rf_explorer::{Error, Result, RfExplorer, SerialPortReader};
use crate::spectrum_analyzer::{
    CalcMode, Config, DspMode, ParseSweepError, RadioModule, Setup, Sweep, TrackingStatus,
};
use crate::Model;
use num_enum::IntoPrimitive;
use serialport::ClearBuffer;
use std::{
    convert::TryFrom, fmt::Debug, io::BufRead, ops::RangeInclusive, time::Duration, time::Instant,
};
use uom::si::{
    f64::Frequency,
    frequency::{kilohertz, megahertz},
};

pub struct SpectrumAnalyzer {
    reader: SerialPortReader,
    setup: Setup,
    config: Config,
    message_buf: Vec<u8>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive)]
#[repr(u8)]
pub enum InputStage {
    Bypass = b'0',
    Attenuator30dB = b'1',
    Lna25dB = b'2',
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive)]
#[repr(u8)]
pub enum WifiBand {
    TwoPointFourGhz = 1,
    FiveGhz,
}

impl SpectrumAnalyzer {
    const MIN_MAX_AMP_RANGE_DBM: RangeInclusive<i16> = -120..=35;
    const DEFAULT_NEXT_SWEEP_TIMEOUT: Duration = Duration::from_secs(2);
    const DEFAULT_REQUEST_CONFIG_TIMEOUT: Duration = Duration::from_secs(2);

    /// Returns the model of the active RF Explorer radio module.
    pub fn active_model(&self) -> Model {
        match self.config.active_radio_module() {
            RadioModule::Main => self.setup.main_model(),
            RadioModule::Expansion => self
                .setup
                .expansion_model()
                .unwrap_or(self.setup.main_model()),
        }
    }

    /// Returns the model of the inactive RF Explorer radio module.
    pub fn inactive_model(&self) -> Option<Model> {
        match self.config.active_radio_module() {
            RadioModule::Main => self.setup.expansion_model(),
            RadioModule::Expansion => Some(self.setup.main_model()),
        }
    }

    /// Returns the next sweep measured by the spectrum analyzer.
    pub fn get_sweep(&mut self) -> Result<Sweep> {
        self.get_sweep_with_timeout(SpectrumAnalyzer::DEFAULT_NEXT_SWEEP_TIMEOUT)
    }

    /// Returns the next sweep measured by the spectrum analyzer.
    pub fn get_sweep_with_timeout(&mut self, timeout: Duration) -> Result<Sweep> {
        // Before reading the next sweep, we should clear the serial port's input buffer
        // This will prevent us from reading a stale sweep
        self.reader.get_ref().clear(ClearBuffer::Input)?;

        self.message_buf.clear();
        let start_time = Instant::now();

        while start_time.elapsed() <= timeout {
            self.reader.read_until(b'\n', &mut self.message_buf)?;

            // It's possible that the byte '\n' could be used to represent an amplitude (-5 dBm)
            // This would result in an invalid sweep with fewer amplitudes than indicated by the length field
            // If parsing the bytes fails with ParseSweepError::TooFewAmplitudes, do not clear the message buffer
            // This will give us another chance to find the real end of the sweep because read_until() appends to the message buffer
            if let Some(rfe_message) = self.message_buf.get(0..self.message_buf.len() - 2) {
                match Sweep::try_from(rfe_message) {
                    Ok(sweep) => return Ok(sweep),
                    Err(ParseSweepError::TooFewAmplitudes { .. }) => continue,
                    Err(_) => (),
                }
            }

            // The line we read was not a sweep, so clear the message buffer before reading the next line
            self.message_buf.clear();
        }

        Err(Error::ResponseTimedOut(timeout))
    }

    pub fn set_start_stop(
        &mut self,
        start_freq: Frequency,
        stop_freq: Frequency,
    ) -> Result<Config> {
        self.set_config(
            start_freq,
            stop_freq,
            self.config.min_amp_dbm(),
            self.config.max_amp_dbm(),
        )
    }

    pub fn set_center_span(&mut self, center_freq: Frequency, span: Frequency) -> Result<Config> {
        self.set_start_stop(center_freq - span / 2., center_freq + span / 2.)
    }

    /// Sets the minimum and maximum amplitudes displayed on the RF Explorer's screen.
    pub fn set_min_max_amps(&mut self, min_amp_dbm: i16, max_amp_dbm: i16) -> Result<Config> {
        self.set_config(
            self.config.start_freq(),
            self.config.stop_freq(),
            min_amp_dbm,
            max_amp_dbm,
        )
    }

    pub fn switch_module_main(&mut self) -> Result<()> {
        self.write_command(&[b'C', b'M', 0])?;
        self.config = self.wait_for_response(Self::DEFAULT_REQUEST_CONFIG_TIMEOUT)?;
        Ok(())
    }

    pub fn switch_module_expansion(&mut self) -> Result<()> {
        self.write_command(&[b'C', b'M', 1])?;
        self.config = self.wait_for_response(Self::DEFAULT_REQUEST_CONFIG_TIMEOUT)?;
        Ok(())
    }

    pub fn start_wifi_analyzer(&mut self, wifi_band: WifiBand) -> Result<()> {
        self.write_command(&[b'C', b'W', wifi_band.into()])
    }

    pub fn stop_wifi_analyzer(&mut self) -> Result<()> {
        self.write_command(&[b'C', b'W', 0])
    }

    pub fn set_calc_mode(&mut self, calc_mode: CalcMode) -> Result<()> {
        self.write_command(&[b'C', b'+', calc_mode.into()])
    }

    pub fn request_tracking(
        &mut self,
        start_freq: Frequency,
        freq_step: Frequency,
    ) -> Result<TrackingStatus> {
        self.reader.get_ref().clear(ClearBuffer::Input)?;
        let command = format!(
            "C3-K:{:07.0},{:07.0}",
            start_freq.get::<kilohertz>(),
            freq_step.get::<kilohertz>()
        );
        self.write_command(command.as_bytes())?;

        self.wait_for_response(Duration::from_secs(3))
    }

    pub fn tracking_step(&mut self, step: u16) -> Result<()> {
        let step_bytes = step.to_be_bytes();
        self.write_command(&[b'k', step_bytes[0], step_bytes[1]])
    }

    pub fn set_dsp(&mut self, dsp_mode: DspMode) -> Result<DspMode> {
        self.reader.get_ref().clear(ClearBuffer::Input)?;
        self.write_command(&[b'C', b'p', dsp_mode.into()])?;

        self.wait_for_response(Duration::from_secs(1))
    }

    pub fn set_offset_db(&mut self, offset_db: i8) -> Result<()> {
        self.write_command(&[b'C', b'O', offset_db as u8])
    }

    pub fn set_input_stage(&mut self, input_stage: InputStage) -> Result<()> {
        self.write_command(&[b'a', input_stage.into()])
    }

    pub fn set_sweep_points(&mut self, sweep_points: u16) -> Result<()> {
        if sweep_points <= 4096 {
            self.write_command(&[b'C', b'J', ((sweep_points / 16) - 1) as u8])
        } else {
            let sweep_points_bytes = sweep_points.to_be_bytes();
            self.write_command(&[b'C', b'j', sweep_points_bytes[0], sweep_points_bytes[1]])
        }
    }

    fn set_config(
        &mut self,
        start_freq: Frequency,
        stop_freq: Frequency,
        min_amp_dbm: i16,
        max_amp_dbm: i16,
    ) -> Result<Config> {
        self.validate_start_stop(start_freq, stop_freq)?;
        self.validate_min_max_amps(min_amp_dbm, max_amp_dbm)?;

        let command = format!(
            "C2-F:{:07.0},{:07.0},{:04},{:04}",
            start_freq.get::<kilohertz>(),
            stop_freq.get::<kilohertz>(),
            max_amp_dbm,
            min_amp_dbm
        );
        // Before asking the RF Explorer to change its config, we should clear the serial port's input buffer
        // This will allow us to read the RF Explorer's response without having to read a bunch of unrelated data first
        self.reader.get_ref().clear(ClearBuffer::Input)?;
        self.write_command(command.as_bytes())?;

        self.config = self.wait_for_response(SpectrumAnalyzer::DEFAULT_REQUEST_CONFIG_TIMEOUT)?;
        Ok(self.config)
    }

    fn validate_start_stop(&self, start_freq: Frequency, stop_freq: Frequency) -> Result<()> {
        if start_freq >= stop_freq {
            return Err(Error::InvalidInput(
                "The start frequency must be less than the stop frequency".to_string(),
            ));
        }

        let active_model = self.active_model();

        let min_max_freq = active_model.min_freq()..=active_model.max_freq();
        if !min_max_freq.contains(&start_freq) {
            return Err(Error::InvalidInput(format!(
                "The start frequency {} MHz is not within the RF Explorer's frequency range of {}-{} MHz",
                start_freq.get::<megahertz>(),
                min_max_freq.start().get::<megahertz>(),
                min_max_freq.end().get::<megahertz>()
            )));
        } else if !min_max_freq.contains(&stop_freq) {
            return Err(Error::InvalidInput(format!(
                "The stop frequency {} MHz is not within the RF Explorer's frequency range of {}-{} MHz",
                stop_freq.get::<megahertz>(),
                min_max_freq.start().get::<megahertz>(),
                min_max_freq.end().get::<megahertz>()
            )));
        }

        let min_max_span = active_model.min_span()..=active_model.max_span();
        if !min_max_span.contains(&(stop_freq - start_freq)) {
            return Err(Error::InvalidInput(format!(
                "The span {} MHz is not within the RF Explorer's span range of {}-{} MHz",
                (stop_freq - start_freq).get::<megahertz>(),
                min_max_span.start().get::<megahertz>(),
                min_max_span.end().get::<megahertz>()
            )));
        }

        Ok(())
    }

    fn validate_min_max_amps(&self, min_amp_dbm: i16, max_amp_dbm: i16) -> Result<()> {
        // The bottom amplitude must be less than the top amplitude
        if min_amp_dbm >= max_amp_dbm {
            return Err(Error::InvalidInput(
                "The minimum amplitude must be less than the maximum amplitude".to_string(),
            ));
        }

        // The top and bottom amplitude must be within the RF Explorer's min and max amplitude range
        if !SpectrumAnalyzer::MIN_MAX_AMP_RANGE_DBM.contains(&min_amp_dbm) {
            return Err(Error::InvalidInput(format!(
                "The amplitude {} dBm is not within the RF Explorer's amplitude range of {}-{} dBm",
                min_amp_dbm,
                SpectrumAnalyzer::MIN_MAX_AMP_RANGE_DBM.start(),
                SpectrumAnalyzer::MIN_MAX_AMP_RANGE_DBM.end()
            )));
        } else if !SpectrumAnalyzer::MIN_MAX_AMP_RANGE_DBM.contains(&max_amp_dbm) {
            return Err(Error::InvalidInput(format!(
                "The amplitude {} dBm is not within the RF Explorer's amplitude range of {}-{} dBm",
                max_amp_dbm,
                SpectrumAnalyzer::MIN_MAX_AMP_RANGE_DBM.start(),
                SpectrumAnalyzer::MIN_MAX_AMP_RANGE_DBM.end()
            )));
        }

        Ok(())
    }
}

impl_rf_explorer!(SpectrumAnalyzer, Setup, Config);

impl Debug for SpectrumAnalyzer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpectrumAnalyzer")
            .field("setup", &self.setup)
            .field("config", &self.config)
            .finish()
    }
}
