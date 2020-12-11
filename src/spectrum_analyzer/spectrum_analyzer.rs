use crate::{
    rf_explorer::{
        Error, Model, ReadMessageResult, RfExplorer, RfeResult, SerialPortReader,
        WriteCommandResult,
    },
    spectrum_analyzer::{
        CalcMode, Config, DspMode, InputStage, RadioModule, SetupInfo, Sweep, TrackingStatus,
        WifiBand,
    },
};
use serialport::ClearBuffer;
use std::{fmt::Debug, ops::RangeInclusive, time::Duration};
use uom::si::{
    f64::Frequency,
    frequency::{kilohertz, megahertz},
};

pub struct SpectrumAnalyzer {
    reader: SerialPortReader,
    setup_info: SetupInfo,
    config: Config,
}

impl SpectrumAnalyzer {
    const MIN_MAX_AMP_RANGE_DBM: RangeInclusive<i16> = -120..=35;
    const DEFAULT_NEXT_SWEEP_TIMEOUT: Duration = Duration::from_secs(2);
    const DEFAULT_REQUEST_CONFIG_TIMEOUT: Duration = Duration::from_secs(2);

    /// Returns the model of the active RF Explorer radio module.
    pub fn active_model(&self) -> Model {
        match self.config.active_radio_module() {
            RadioModule::Main => self.setup_info.main_model(),
            RadioModule::Expansion => self
                .setup_info
                .expansion_model()
                .unwrap_or(self.setup_info.main_model()),
        }
    }

    /// Returns the model of the inactive RF Explorer radio module.
    pub fn inactive_model(&self) -> Option<Model> {
        match self.config.active_radio_module() {
            RadioModule::Main => self.setup_info.expansion_model(),
            RadioModule::Expansion => Some(self.setup_info.main_model()),
        }
    }

    /// Returns the next sweep measured by the spectrum analyzer.
    pub fn get_sweep(&mut self) -> ReadMessageResult<Sweep> {
        self.get_sweep_with_timeout(SpectrumAnalyzer::DEFAULT_NEXT_SWEEP_TIMEOUT)
    }

    /// Returns the next sweep measured by the spectrum analyzer.
    pub fn get_sweep_with_timeout(&mut self, timeout: Duration) -> ReadMessageResult<Sweep> {
        // Before reading the next sweep, we should clear the serial port's input buffer
        // This will prevent us from reading a stale sweep
        self.reader.get_ref().clear(ClearBuffer::Input)?;

        Ok(self.read_message(timeout)?)
    }

    pub fn set_start_stop(
        &mut self,
        start_freq: Frequency,
        stop_freq: Frequency,
    ) -> RfeResult<Config> {
        self.set_config(
            start_freq,
            stop_freq,
            self.config.min_amp_dbm(),
            self.config.max_amp_dbm(),
        )
    }

    pub fn set_center_span(
        &mut self,
        center_freq: Frequency,
        span: Frequency,
    ) -> RfeResult<Config> {
        self.set_start_stop(center_freq - span / 2., center_freq + span / 2.)
    }

    /// Sets the minimum and maximum amplitudes displayed on the RF Explorer's screen.
    pub fn set_min_max_amps(&mut self, min_amp_dbm: i16, max_amp_dbm: i16) -> RfeResult<Config> {
        self.set_config(
            self.config.start_freq(),
            self.config.stop_freq(),
            min_amp_dbm,
            max_amp_dbm,
        )
    }

    pub fn switch_module_main(&mut self) -> RfeResult<()> {
        self.write_command(&[b'C', b'M', 0])?;
        self.config = self.read_message(Self::DEFAULT_REQUEST_CONFIG_TIMEOUT)?;
        Ok(())
    }

    pub fn switch_module_expansion(&mut self) -> RfeResult<()> {
        self.write_command(&[b'C', b'M', 1])?;
        self.config = self.read_message(Self::DEFAULT_REQUEST_CONFIG_TIMEOUT)?;
        Ok(())
    }

    pub fn start_wifi_analyzer(&mut self, wifi_band: WifiBand) -> WriteCommandResult<()> {
        self.write_command(&[b'C', b'W', wifi_band.into()])
    }

    pub fn stop_wifi_analyzer(&mut self) -> WriteCommandResult<()> {
        self.write_command(&[b'C', b'W', 0])
    }

    pub fn set_calc_mode(&mut self, calc_mode: CalcMode) -> WriteCommandResult<()> {
        self.write_command(&[b'C', b'+', calc_mode.into()])
    }

    pub fn request_tracking(
        &mut self,
        start_freq: Frequency,
        freq_step: Frequency,
    ) -> RfeResult<TrackingStatus> {
        self.reader.get_ref().clear(ClearBuffer::Input)?;
        let command = format!(
            "C3-K:{:07.0},{:07.0}",
            start_freq.get::<kilohertz>(),
            freq_step.get::<kilohertz>()
        );
        self.write_command(command.as_bytes())?;

        Ok(self.read_message(Duration::from_secs(3))?)
    }

    pub fn tracking_step(&mut self, step: u16) -> WriteCommandResult<()> {
        let step_bytes = step.to_be_bytes();
        self.write_command(&[b'k', step_bytes[0], step_bytes[1]])
    }

    pub fn set_dsp(&mut self, dsp_mode: DspMode) -> RfeResult<DspMode> {
        self.reader.get_ref().clear(ClearBuffer::Input)?;
        self.write_command(&[b'C', b'p', dsp_mode.into()])?;

        Ok(self.read_message(Duration::from_secs(1))?)
    }

    pub fn set_offset_db(&mut self, offset_db: i8) -> WriteCommandResult<()> {
        self.write_command(&[b'C', b'O', offset_db as u8])
    }

    pub fn set_input_stage(&mut self, input_stage: InputStage) -> WriteCommandResult<()> {
        self.write_command(&[b'a', input_stage.into()])
    }

    pub fn set_sweep_points(&mut self, sweep_points: u16) -> WriteCommandResult<()> {
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
    ) -> RfeResult<Config> {
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

        self.config = self.read_message(SpectrumAnalyzer::DEFAULT_REQUEST_CONFIG_TIMEOUT)?;
        Ok(self.config)
    }

    fn validate_start_stop(&self, start_freq: Frequency, stop_freq: Frequency) -> RfeResult<()> {
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

    fn validate_min_max_amps(&self, min_amp_dbm: i16, max_amp_dbm: i16) -> RfeResult<()> {
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

impl RfExplorer for SpectrumAnalyzer {
    fn new(reader: SerialPortReader, setup_info: Self::SetupInfo, config: Self::Config) -> Self {
        SpectrumAnalyzer {
            reader,
            setup_info,
            config,
        }
    }

    fn reader(&mut self) -> &mut SerialPortReader {
        &mut self.reader
    }

    fn setup_info(&self) -> Self::SetupInfo {
        self.setup_info.clone()
    }

    type SetupInfo = crate::spectrum_analyzer::SetupInfo;

    type Config = crate::spectrum_analyzer::Config;
}

impl Debug for SpectrumAnalyzer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpectrumAnalyzer")
            .field("setup_info", &self.setup_info)
            .field("config", &self.config)
            .finish()
    }
}
