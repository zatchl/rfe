use crate::rf_explorer::{Error, Result, RfExplorer, SerialPortReader};
use crate::spectrum_analyzer::{
    CalcMode, Config, DspMode, ParseSweepError, Setup, Sweep, TrackingStatus,
};
use num_enum::IntoPrimitive;
use serialport::ClearBuffer;
use std::{
    convert::TryFrom, fmt::Debug, io::BufRead, ops::RangeInclusive, time::Duration, time::Instant,
};

pub struct SpectrumAnalyzer {
    reader: SerialPortReader,
    setup: Setup,
    config: Config,
    message_buf: Vec<u8>,
}

#[derive(Debug, Copy, Clone, IntoPrimitive)]
#[repr(u8)]
pub enum InputStage {
    Bypass = b'0',
    Attenuator30dB = b'1',
    Lna25dB = b'2',
}

#[derive(Debug, Copy, Clone, IntoPrimitive)]
#[repr(u8)]
pub enum WifiBand {
    TwoPointFourGhz = 1,
    FiveGhz,
}

impl SpectrumAnalyzer {
    const MIN_MAX_AMP_RANGE_DBM: RangeInclusive<i16> = -120..=35;
    const DEFAULT_NEXT_SWEEP_TIMEOUT: Duration = Duration::from_secs(2);
    const DEFAULT_REQUEST_CONFIG_TIMEOUT: Duration = Duration::from_secs(2);

    pub fn next_sweep(&mut self) -> Result<Sweep> {
        self.next_sweep_with_timeout(SpectrumAnalyzer::DEFAULT_NEXT_SWEEP_TIMEOUT)
    }

    pub fn next_sweep_with_timeout(&mut self, timeout: Duration) -> Result<Sweep> {
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

    pub fn set_config(
        &mut self,
        start_freq_khz: f64,
        end_freq_khz: f64,
        amp_bottom_dbm: i16,
        amp_top_dbm: i16,
    ) -> Result<Config> {
        self.validate_freq_range(start_freq_khz..=end_freq_khz)?;
        self.validate_amp_range(amp_bottom_dbm..=amp_top_dbm)?;

        let command = format!(
            "C2-F:{:07.0},{:07.0},{:04},{:04}",
            start_freq_khz, end_freq_khz, amp_top_dbm, amp_bottom_dbm
        );
        // Before asking the RF Explorer to change its config, we should clear the serial port's input buffer
        // This will allow us to read the RF Explorer's response without having to read a bunch of unrelated data first
        self.reader.get_ref().clear(ClearBuffer::Input)?;
        self.write_command(command.as_bytes())?;

        self.wait_for_response(SpectrumAnalyzer::DEFAULT_REQUEST_CONFIG_TIMEOUT)
    }

    pub fn set_freq_range(&mut self, start_freq_khz: f64, end_freq_khz: f64) -> Result<Config> {
        self.set_config(
            start_freq_khz,
            end_freq_khz,
            self.config.amp_bottom_dbm(),
            self.config.amp_top_dbm(),
        )
    }

    pub fn set_center_span(&mut self, center_freq_khz: f64, span_khz: f64) -> Result<Config> {
        self.set_freq_range(
            center_freq_khz - span_khz / 2f64,
            center_freq_khz + span_khz / 2f64,
        )
    }

    pub fn switch_module_main(&mut self) -> Result<()> {
        self.write_command(&[b'C', b'M', 0])
    }

    pub fn switch_module_expansion(&mut self) -> Result<()> {
        self.write_command(&[b'C', b'M', 1])
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
        start_freq_khz: f64,
        freq_step_khz: f64,
    ) -> Result<TrackingStatus> {
        self.reader.get_ref().clear(ClearBuffer::Input)?;
        let command = format!("C3-K:{:07.0},{:07.0}", start_freq_khz, freq_step_khz);
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

    fn validate_freq_range(&self, freq_range_khz: RangeInclusive<f64>) -> Result<()> {
        if freq_range_khz.start() >= freq_range_khz.end() {
            return Err(Error::InvalidInput(
                "The start frequency must be less than the end frequency".to_string(),
            ));
        }

        let min_max_freq_range_khz = self.config.min_freq_khz()..=self.config.max_freq_khz();
        if !min_max_freq_range_khz.contains(freq_range_khz.start()) {
            return Err(Error::InvalidInput(format!(
                "The start frequency {} kHz is not within the RF Explorer's frequency range of {}-{} kHz",
                freq_range_khz.start(),
                min_max_freq_range_khz.start(),
                min_max_freq_range_khz.end()
            )));
        } else if !min_max_freq_range_khz.contains(freq_range_khz.end()) {
            return Err(Error::InvalidInput(format!(
                "The end frequency {} kHz is not within the RF Explorer's frequency range of {}-{} kHz",
                freq_range_khz.end(),
                min_max_freq_range_khz.start(),
                min_max_freq_range_khz.end()
            )));
        }

        if freq_range_khz.end() - freq_range_khz.start() > self.config.max_span_khz() {
            return Err(Error::InvalidInput(format!(
                "The span {} kHz must be less than or equal to the RF Explorer's max span {} kHz",
                freq_range_khz.end() - freq_range_khz.start(),
                self.config.max_span_khz()
            )));
        }

        Ok(())
    }

    fn validate_amp_range(&self, amp_range_dbm: RangeInclusive<i16>) -> Result<()> {
        // The bottom amplitude must be less than the top amplitude
        if amp_range_dbm.start() >= amp_range_dbm.end() {
            return Err(Error::InvalidInput("".to_string()));
        }

        // The top and bottom amplitude must be within the RF Explorer's min and max amplitude range
        if !SpectrumAnalyzer::MIN_MAX_AMP_RANGE_DBM.contains(amp_range_dbm.start()) {
            return Err(Error::InvalidInput("".to_string()));
        } else if !SpectrumAnalyzer::MIN_MAX_AMP_RANGE_DBM.contains(amp_range_dbm.end()) {
            return Err(Error::InvalidInput("".to_string()));
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
