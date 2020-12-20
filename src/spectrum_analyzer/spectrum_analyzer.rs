use super::{
    CalcMode, Command, Config, DspMode, InputStage, RadioModule, SetupInfo, Sweep, TrackingStatus,
    WifiBand,
};
use crate::rf_explorer::{
    self, ConnectionError, Error, Model, ParseFromBytes, RfExplorer, RfeResult, SerialNumber,
    SerialPortReader,
};
use serialport::SerialPortInfo;
use std::{
    fmt::Debug,
    io::{self, BufRead, ErrorKind},
    ops::RangeInclusive,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use uom::si::{
    f64::Frequency,
    frequency::{kilohertz, megahertz},
};

pub struct SpectrumAnalyzer {
    serial_port: Arc<Mutex<SerialPortReader>>,
    is_reading: Arc<Mutex<bool>>,
    read_thread_handle: Option<JoinHandle<()>>,
    config: Arc<Mutex<Config>>,
    last_sweep: Arc<Mutex<Sweep>>,
    dsp_mode: Arc<Mutex<Option<DspMode>>>,
    serial_number: Arc<Mutex<Option<SerialNumber>>>,
    tracking_status: Arc<Mutex<Option<TrackingStatus>>>,
    setup_info: SetupInfo,
}

impl SpectrumAnalyzer {
    const MIN_MAX_AMP_RANGE_DBM: RangeInclusive<i16> = -120..=35;
    const COMMAND_RESPONSE_TIMEOUT: Duration = Duration::from_secs(2);
    const READ_FIRST_MESSAGES_TIMEOUT: Duration = Duration::from_secs(2);

    /// Attempts to connect to an RF Explorer using the given serial port information.
    pub(crate) fn connect(port_info: &SerialPortInfo) -> Result<Self, ConnectionError> {
        let mut serial_port = rf_explorer::open(port_info)?;

        let (config, setup_info, sweep) = SpectrumAnalyzer::read_first_messages(&mut serial_port)?;

        let config = Arc::new(Mutex::new(config));
        let last_sweep = Arc::new(Mutex::new(sweep));
        let dsp_mode = Arc::new(Mutex::new(None));
        let serial_number = Arc::new(Mutex::new(None));
        let tracking_status = Arc::new(Mutex::new(None));

        let serial_port = Arc::new(Mutex::new(serial_port));
        let is_reading = Arc::new(Mutex::new(true));

        let read_thread_handle = Some(SpectrumAnalyzer::read_messages(
            Arc::clone(&serial_port),
            Arc::clone(&is_reading),
            Arc::clone(&config),
            Arc::clone(&last_sweep),
            Arc::clone(&dsp_mode),
            Arc::clone(&serial_number),
            Arc::clone(&tracking_status),
        ));

        Ok(SpectrumAnalyzer {
            serial_port,
            is_reading,
            read_thread_handle,
            setup_info,
            config,
            last_sweep,
            dsp_mode,
            serial_number,
            tracking_status,
        })
    }

    /// Attempts to read the messages sent by the spectrum analyzer when we connect to it.
    /// We need to receive Config, SetupInfo, and Sweep messages in order to create a valid SpectrumAnalyzer.
    fn read_first_messages(
        serial_port: &mut SerialPortReader,
    ) -> Result<(Config, SetupInfo, Sweep), ConnectionError> {
        let (mut initial_config, mut initial_setup_info, mut initial_sweep) = (None, None, None);

        let mut message_buf = Vec::new();

        let start_time = Instant::now();
        while start_time.elapsed() < SpectrumAnalyzer::READ_FIRST_MESSAGES_TIMEOUT {
            if initial_config.is_some() && initial_setup_info.is_some() && initial_sweep.is_some() {
                break;
            }

            serial_port.read_until(b'\n', &mut message_buf)?;

            if initial_config.is_none() {
                if let Ok((_, config)) = Config::parse_from_bytes(message_buf.as_slice()) {
                    initial_config = Some(config);
                    message_buf.clear();
                    continue;
                }
            }

            if initial_setup_info.is_none() {
                if let Ok((_, setup_info)) = SetupInfo::parse_from_bytes(message_buf.as_slice()) {
                    initial_setup_info = Some(setup_info);
                    message_buf.clear();
                    continue;
                }
            }

            if initial_sweep.is_none() {
                if let Ok((_, sweep)) = Sweep::parse_from_bytes(message_buf.as_slice()) {
                    initial_sweep = Some(sweep);
                    message_buf.clear();
                    continue;
                }
            }

            message_buf.clear();
        }

        match (initial_config, initial_setup_info, initial_sweep) {
            (Some(config), Some(setup_info), Some(sweep)) => Ok((config, setup_info, sweep)),
            _ => Err(ConnectionError::NotAnRfExplorer),
        }
    }

    /// Spawns a new thread to read messages from the spectrum analyzer.
    fn read_messages(
        serial_port: Arc<Mutex<SerialPortReader>>,
        is_reading: Arc<Mutex<bool>>,
        config: Arc<Mutex<Config>>,
        last_sweep: Arc<Mutex<Sweep>>,
        dsp_mode: Arc<Mutex<Option<DspMode>>>,
        serial_number: Arc<Mutex<Option<SerialNumber>>>,
        tracking_status: Arc<Mutex<Option<TrackingStatus>>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut message_buf = Vec::new();
            while *is_reading.lock().unwrap() {
                let read_message_result = serial_port
                    .lock()
                    .unwrap()
                    .read_until(b'\n', &mut message_buf);

                // Time out errors are recoverable so we should try to read again
                // Other errors are not recoverable and we should exit the thread
                if let Err(error) = read_message_result {
                    match error.kind() {
                        ErrorKind::TimedOut => continue,
                        _ => break,
                    }
                }

                // Try to parse a sweep from the message we received
                let parse_sweep_result = Sweep::parse_from_bytes(&message_buf);
                if let Ok((_, new_sweep)) = parse_sweep_result {
                    *last_sweep.lock().unwrap() = new_sweep;
                    message_buf.clear();
                    continue;
                } else if let Err(nom::Err::Incomplete(_)) = parse_sweep_result {
                    continue;
                }

                // Try to parse a config from the message we received
                if let Ok((_, new_config)) = Config::parse_from_bytes(&message_buf) {
                    *config.lock().unwrap() = new_config;
                    message_buf.clear();
                    continue;
                }

                // Try to parse a DSP mode message from the message we received
                if let Ok((_, new_dsp_mode)) = DspMode::parse_from_bytes(&message_buf) {
                    dsp_mode.lock().unwrap().replace(new_dsp_mode);
                    message_buf.clear();
                    continue;
                }

                // Try to parse a serial number message from the message we received
                if let Ok((_, new_serial_number)) = SerialNumber::parse_from_bytes(&message_buf) {
                    serial_number.lock().unwrap().replace(new_serial_number);
                    message_buf.clear();
                    continue;
                }

                // Try to parse a tracking status message from the message we received
                if let Ok((_, new_tracking_status)) = TrackingStatus::parse_from_bytes(&message_buf)
                {
                    tracking_status.lock().unwrap().replace(new_tracking_status);
                    message_buf.clear();
                    continue;
                }

                // We weren't able to parse the message we received so clear the message buffer and read again
                message_buf.clear();
            }

            *is_reading.lock().unwrap() = false;
        })
    }

    /// Returns the spectrum analyzer's configuration.
    pub fn config(&self) -> Config {
        *self.config.lock().unwrap()
    }

    /// Returns a copy of the last sweep received from the spectrum analyzer.
    pub fn last_sweep(&self) -> Sweep {
        self.last_sweep.lock().unwrap().clone()
    }

    /// Returns a copy of the next sweep received from the spectrum analyzer.
    pub fn next_sweep(&self, timeout: Duration) -> RfeResult<Sweep> {
        // Store a copy of the last sweep
        let last_sweep = self.last_sweep();

        // Check to see if we've received a new sweep by comparing the timestamps of the most
        // recent sweeps
        let start_time = Instant::now();
        while start_time.elapsed() <= timeout {
            if last_sweep.timestamp() != self.last_sweep.lock().unwrap().timestamp() {
                return Ok(self.last_sweep.lock().unwrap().clone());
            }
        }

        Err(Error::TimedOut(timeout))
    }

    /// Returns the model of the active RF Explorer radio module.
    pub fn active_model(&self) -> Model {
        match self.config().active_radio_module() {
            RadioModule::Main => self.setup_info.main_model(),
            RadioModule::Expansion => self
                .setup_info
                .expansion_model()
                .unwrap_or(self.setup_info.main_model()),
        }
    }

    /// Returns the model of the inactive RF Explorer radio module.
    pub fn inactive_model(&self) -> Option<Model> {
        match self.config().active_radio_module() {
            RadioModule::Main => self.setup_info.expansion_model(),
            RadioModule::Expansion => Some(self.setup_info.main_model()),
        }
    }

    /// Sets the start and stop frequency of sweeps measured by the spectrum analyzer.
    pub fn set_start_stop(
        &mut self,
        start_freq: Frequency,
        stop_freq: Frequency,
    ) -> RfeResult<Config> {
        let config = self.config();
        self.set_config(
            start_freq,
            stop_freq,
            config.min_amp_dbm(),
            config.max_amp_dbm(),
        )
    }

    /// Sets the center frequency and span of sweeps measured by the spectrum analyzer.
    pub fn set_center_span(
        &mut self,
        center_freq: Frequency,
        span: Frequency,
    ) -> RfeResult<Config> {
        self.set_start_stop(center_freq - span / 2., center_freq + span / 2.)
    }

    /// Sets the minimum and maximum amplitudes displayed on the RF Explorer's screen.
    pub fn set_min_max_amps(&mut self, min_amp_dbm: i16, max_amp_dbm: i16) -> RfeResult<Config> {
        let config = self.config();
        self.set_config(
            config.start_freq(),
            config.stop_freq(),
            min_amp_dbm,
            max_amp_dbm,
        )
    }

    /// Sets the spectrum analyzer's configuration.
    fn set_config(
        &mut self,
        start_freq: Frequency,
        stop_freq: Frequency,
        min_amp_dbm: i16,
        max_amp_dbm: i16,
    ) -> RfeResult<Config> {
        self.validate_start_stop(start_freq, stop_freq)?;
        self.validate_min_max_amps(min_amp_dbm, max_amp_dbm)?;

        // Store a copy of the original config to help determine when we've received a new config
        let original_config = *self.config.lock().unwrap();

        // Send the command to change the config
        self.send_command(
            Command::SetConfig {
                start_freq_khz: start_freq.get::<kilohertz>(),
                stop_freq_khz: stop_freq.get::<kilohertz>(),
                min_amp_dbm,
                max_amp_dbm,
            }
            .to_vec(),
        )?;

        // Wait to see if we receive a new config in response
        let start_time = Instant::now();
        while start_time.elapsed() < Self::COMMAND_RESPONSE_TIMEOUT {
            let new_config = *self.config.lock().unwrap();
            // If the new config is different than the old config it means we received a new config
            // in reponse to our command
            if new_config != original_config {
                return Ok(new_config);
            }
        }

        Err(Error::TimedOut(Self::COMMAND_RESPONSE_TIMEOUT))
    }

    /// Sets the number of points in each sweep measured by the spectrum analyzer.
    pub fn set_sweep_points(&mut self, sweep_points: u16) -> io::Result<()> {
        if sweep_points <= 4096 {
            self.send_command(Command::SetSweepPointsExt(sweep_points).to_vec())
        } else {
            self.send_command(Command::SetSweepPointsLarge(sweep_points).to_vec())
        }
    }

    /// Sets the spectrum analyzer's calculator mode.
    pub fn set_calc_mode(&mut self, calc_mode: CalcMode) -> io::Result<()> {
        self.send_command(Command::SetCalcMode(calc_mode).to_vec())
    }

    pub fn set_input_stage(&mut self, input_stage: InputStage) -> io::Result<()> {
        self.send_command(Command::SetInputStage(input_stage).to_vec())
    }

    /// Adds or subtracts an offset to the amplitudes in each sweep.
    pub fn set_offset_db(&mut self, offset_db: i8) -> io::Result<()> {
        self.send_command(Command::SetOffsetDB(offset_db).to_vec())
    }

    pub fn set_dsp_mode(&mut self, dsp_mode: DspMode) -> RfeResult<DspMode> {
        // Set the DSP mode to None so we can tell whether or not we've received a new DSP mode by
        // checking for Some
        *self.dsp_mode.lock().unwrap() = None;

        // Send the command to set the DSP mode
        self.send_command(Command::SetDsp(dsp_mode).to_vec())?;

        // Wait to see if we receive a DSP mode message in response
        let start_time = Instant::now();
        while start_time.elapsed() <= Self::COMMAND_RESPONSE_TIMEOUT {
            if let Some(&dsp_mode) = self.dsp_mode.lock().unwrap().as_ref() {
                return Ok(dsp_mode);
            }
        }

        Err(Error::TimedOut(Self::COMMAND_RESPONSE_TIMEOUT))
    }

    /// Switches the spectrum analyzer's active module to the main module.
    pub fn switch_module_main(&mut self) -> RfeResult<()> {
        self.send_command(Command::SwitchModuleMain.to_vec())?;

        let start_time = Instant::now();
        while start_time.elapsed() <= Self::COMMAND_RESPONSE_TIMEOUT {
            match self.config.lock().unwrap().active_radio_module() {
                RadioModule::Main => return Ok(()),
                RadioModule::Expansion => continue,
            }
        }

        Err(Error::TimedOut(Self::COMMAND_RESPONSE_TIMEOUT))
    }

    /// Switches the spectrum analyzer's active module to the expansion module.
    pub fn switch_module_exp(&mut self) -> RfeResult<()> {
        self.send_command(Command::SwitchModuleExp.to_vec())?;

        let start_time = Instant::now();
        while start_time.elapsed() <= Self::COMMAND_RESPONSE_TIMEOUT {
            match self.config.lock().unwrap().active_radio_module() {
                RadioModule::Expansion => return Ok(()),
                RadioModule::Main => continue,
            }
        }

        Err(Error::TimedOut(Self::COMMAND_RESPONSE_TIMEOUT))
    }

    pub fn start_wifi_analyzer(&mut self, wifi_band: WifiBand) -> io::Result<()> {
        self.send_command(Command::StartWifiAnalyzer(wifi_band).to_vec())
    }

    pub fn stop_wifi_analyzer(&mut self) -> io::Result<()> {
        self.send_command(Command::StopWifiAnalyzer.to_vec())
    }

    pub fn request_tracking(
        &mut self,
        start_freq: Frequency,
        freq_step: Frequency,
    ) -> RfeResult<TrackingStatus> {
        // Set the tracking status to None so we can tell whether or not we've received a new
        // tracking status message by checking for Some
        *self.tracking_status.lock().unwrap() = None;

        // Send the command to enter tracking mode
        self.send_command(
            Command::StartTracking {
                start_freq_khz: start_freq.get::<kilohertz>(),
                step_freq_khz: freq_step.get::<kilohertz>(),
            }
            .to_vec(),
        )?;

        // Wait to see if we receive a DSP mode message in response
        let start_time = Instant::now();
        while start_time.elapsed() <= Self::COMMAND_RESPONSE_TIMEOUT {
            if let Some(&tracking_status) = self.tracking_status.lock().unwrap().as_ref() {
                return Ok(tracking_status);
            }
        }

        Err(Error::TimedOut(Self::COMMAND_RESPONSE_TIMEOUT))
    }

    pub fn tracking_step(&mut self, step: u16) -> io::Result<()> {
        self.send_command(Command::TrackingStep(step).to_vec())
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
    fn send_command(&mut self, command: impl AsRef<[u8]>) -> io::Result<()> {
        self.serial_port
            .lock()
            .unwrap()
            .get_mut()
            .write_all(command.as_ref())
    }

    fn main_model(&self) -> Model {
        self.setup_info.main_model()
    }

    fn expansion_model(&self) -> Option<Model> {
        self.setup_info.expansion_model()
    }

    fn firmware_version(&self) -> &str {
        self.setup_info.firmware_version()
    }

    fn request_serial_number(&mut self) -> RfeResult<SerialNumber> {
        // If we've already received a serial number, return it without requesting the RF
        // Explorer sends it again
        if let Some(serial_number) = self.serial_number.lock().unwrap().as_ref() {
            return Ok(serial_number.clone());
        }

        // Send the command to request the RF Explorer's serial number
        self.send_command(crate::rf_explorer::Command::RequestSerialNumber)?;

        // Wait to see if we receive a serial number in response
        let start_time = Instant::now();
        while start_time.elapsed() <= Self::COMMAND_RESPONSE_TIMEOUT {
            if let Some(serial_number) = self.serial_number.lock().unwrap().as_ref() {
                return Ok(serial_number.clone());
            }
        }

        Err(Error::TimedOut(Self::COMMAND_RESPONSE_TIMEOUT))
    }
}

impl Drop for SpectrumAnalyzer {
    fn drop(&mut self) {
        *self.is_reading.lock().unwrap() = false;
        if let Some(read_handle) = self.read_thread_handle.take() {
            let _ = read_handle.join();
        }
    }
}

impl Debug for SpectrumAnalyzer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpectrumAnalyzer")
            .field("setup_info", &self.setup_info)
            .field("config", &self.config.lock().unwrap())
            .finish()
    }
}
