use super::{
    Attenuation, Command, Config, ConfigAmpSweep, ConfigCw, ConfigFreqSweep, PowerLevel, SetupInfo,
    Temperature,
};
use crate::rf_explorer::{
    self, ConnectionError, Error, Model, ParseFromBytes, RfExplorer, RfeResult, SerialNumber,
    SerialPortReader,
};
use serialport::SerialPortInfo;
use std::{
    fmt::Debug,
    io::{self, BufRead, ErrorKind},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use uom::si::{f64::Frequency, frequency::kilohertz};

pub struct SignalGenerator {
    serial_port: Arc<Mutex<SerialPortReader>>,
    is_reading: Arc<Mutex<bool>>,
    read_thread_handle: Option<JoinHandle<()>>,
    config: Arc<Mutex<Config>>,
    config_amp_sweep: Arc<Mutex<Option<ConfigAmpSweep>>>,
    config_cw: Arc<Mutex<Option<ConfigCw>>>,
    config_freq_sweep: Arc<Mutex<Option<ConfigFreqSweep>>>,
    serial_number: Arc<Mutex<Option<SerialNumber>>>,
    temperature: Arc<Mutex<Option<Temperature>>>,
    setup_info: SetupInfo,
}

impl SignalGenerator {
    const COMMAND_RESPONSE_TIMEOUT: Duration = Duration::from_secs(2);
    const READ_FIRST_MESSAGES_TIMEOUT: Duration = Duration::from_secs(2);

    /// Attempts to connect to an RF Explorer using the given serial port information.
    pub(crate) fn connect(port_info: &SerialPortInfo) -> Result<Self, ConnectionError> {
        let mut serial_port = rf_explorer::open(port_info)?;

        let (config, setup_info) = SignalGenerator::read_first_messages(&mut serial_port)?;

        let config = Arc::new(Mutex::new(config));
        let config_amp_sweep = Arc::new(Mutex::new(None));
        let config_cw = Arc::new(Mutex::new(None));
        let config_freq_sweep = Arc::new(Mutex::new(None));
        let serial_number = Arc::new(Mutex::new(None));
        let temperature = Arc::new(Mutex::new(None));

        let serial_port = Arc::new(Mutex::new(serial_port));
        let is_reading = Arc::new(Mutex::new(true));

        let read_thread_handle = Some(SignalGenerator::read_messages(
            Arc::clone(&serial_port),
            Arc::clone(&is_reading),
            Arc::clone(&config),
            Arc::clone(&config_amp_sweep),
            Arc::clone(&config_cw),
            Arc::clone(&config_freq_sweep),
            Arc::clone(&serial_number),
            Arc::clone(&temperature),
        ));

        Ok(SignalGenerator {
            serial_port,
            is_reading,
            read_thread_handle,
            setup_info,
            config,
            config_amp_sweep,
            config_cw,
            config_freq_sweep,
            serial_number,
            temperature,
        })
    }

    /// Attempts to read the messages sent by the signal generator when we connect to it.
    /// We need to receive Config and SetupInfo messages in order to create a valid SignalGenerator.
    fn read_first_messages(
        serial_port: &mut SerialPortReader,
    ) -> Result<(Config, SetupInfo), ConnectionError> {
        let (mut initial_config, mut initial_setup_info) = (None, None);

        let mut message_buf = Vec::new();

        let start_time = Instant::now();
        while start_time.elapsed() < SignalGenerator::READ_FIRST_MESSAGES_TIMEOUT {
            if initial_config.is_some() && initial_setup_info.is_some() {
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

            message_buf.clear();
        }

        match (initial_config, initial_setup_info) {
            (Some(config), Some(setup_info)) => Ok((config, setup_info)),
            _ => Err(ConnectionError::NotAnRfExplorer),
        }
    }

    /// Spawns a new thread to read messages from the signal generator.
    fn read_messages(
        serial_port: Arc<Mutex<SerialPortReader>>,
        is_reading: Arc<Mutex<bool>>,
        config: Arc<Mutex<Config>>,
        config_amp_sweep: Arc<Mutex<Option<ConfigAmpSweep>>>,
        config_cw: Arc<Mutex<Option<ConfigCw>>>,
        config_freq_sweep: Arc<Mutex<Option<ConfigFreqSweep>>>,
        serial_number: Arc<Mutex<Option<SerialNumber>>>,
        temperature: Arc<Mutex<Option<Temperature>>>,
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

                // Try to parse a config from the message we received
                if let Ok((_, new_config)) = Config::parse_from_bytes(&message_buf) {
                    *config.lock().unwrap() = new_config;
                    message_buf.clear();
                    continue;
                }

                // Try to parse a new amplitude sweep mode config from the message we received
                if let Ok((_, new_config)) = ConfigAmpSweep::parse_from_bytes(&message_buf) {
                    *config_amp_sweep.lock().unwrap() = Some(new_config);
                    message_buf.clear();
                    continue;
                }

                // Try to parse a new CW mode config from the message we received
                if let Ok((_, new_config)) = ConfigCw::parse_from_bytes(&message_buf) {
                    *config_cw.lock().unwrap() = Some(new_config);
                    message_buf.clear();
                    continue;
                }

                // Try to parse a new frequency sweep mode config from the message we received
                if let Ok((_, new_config)) = ConfigFreqSweep::parse_from_bytes(&message_buf) {
                    *config_freq_sweep.lock().unwrap() = Some(new_config);
                    message_buf.clear();
                    continue;
                }

                // Try to parse a serial number message from the message we received
                if let Ok((_, new_serial_number)) = SerialNumber::parse_from_bytes(&message_buf) {
                    *serial_number.lock().unwrap() = Some(new_serial_number);
                    message_buf.clear();
                    continue;
                }

                // Try to parse a temperature messagefrom the message we received
                if let Ok((_, new_temperature)) = Temperature::parse_from_bytes(&message_buf) {
                    *temperature.lock().unwrap() = Some(new_temperature);
                    message_buf.clear();
                    continue;
                }

                // We weren't able to parse the message we received so clear the message buffer and read again
                message_buf.clear();
            }

            *is_reading.lock().unwrap() = false;
        })
    }

    /// Returns the signal generator's configuration.
    pub fn config(&self) -> Config {
        *self.config.lock().unwrap()
    }

    /// Returns the signal generator's amplitude sweep mode configuration.
    pub fn config_amp_sweep(&self) -> Option<ConfigAmpSweep> {
        *self.config_amp_sweep.lock().unwrap()
    }

    /// Returns the signal generator's CW mode configuration.
    pub fn config_cw(&self) -> Option<ConfigCw> {
        *self.config_cw.lock().unwrap()
    }

    /// Returns the signal generator's frequency sweep mode configuration.
    pub fn config_freq_sweep(&self) -> Option<ConfigFreqSweep> {
        *self.config_freq_sweep.lock().unwrap()
    }

    /// Returns the signal generator's temperature.
    pub fn temperature(&self) -> Option<Temperature> {
        *self.temperature.lock().unwrap()
    }

    /// Starts the signal generator's amplitude sweep mode.
    pub fn start_amp_sweep(
        &mut self,
        cw_freq: Frequency,
        start_attenuation: Attenuation,
        start_power_level: PowerLevel,
        stop_attenuation: Attenuation,
        stop_power_level: PowerLevel,
        step_delay: Duration,
    ) -> io::Result<()> {
        self.send_command(
            Command::StartAmpSweep {
                cw_freq_khz: cw_freq.get::<kilohertz>(),
                start_attenuation,
                start_power_level,
                stop_attenuation,
                stop_power_level,
                step_delay,
            }
            .to_vec(),
        )
    }

    /// Starts the signal generator's amplitude sweep mode using the expansion module.
    pub fn start_amp_sweep_exp(
        &mut self,
        cw_freq: Frequency,
        start_power_dbm: f64,
        step_power_db: f64,
        stop_power_dbm: f64,
        step_delay: Duration,
    ) -> io::Result<()> {
        self.send_command(
            Command::StartAmpSweepExp {
                cw_freq_khz: cw_freq.get::<kilohertz>(),
                start_power_dbm,
                step_power_db,
                stop_power_dbm,
                step_delay,
            }
            .to_vec(),
        )
    }

    /// Starts the signal generator's CW mode.
    pub fn start_cw(
        &mut self,
        cw_freq: Frequency,
        attenuation: Attenuation,
        power_level: PowerLevel,
    ) -> io::Result<()> {
        self.send_command(
            Command::StartCw {
                cw_freq_khz: cw_freq.get::<kilohertz>(),
                attenuation,
                power_level,
            }
            .to_vec(),
        )
    }

    /// Starts the signal generator's CW mode using the expansion module.
    pub fn start_cw_exp(&mut self, cw_freq: Frequency, power_dbm: f64) -> io::Result<()> {
        self.send_command(
            Command::StartCwExp {
                cw_freq_khz: cw_freq.get::<kilohertz>(),
                power_dbm,
            }
            .to_vec(),
        )
    }

    /// Starts the signal generator's frequency sweep mode.
    pub fn start_freq_sweep(
        &mut self,
        start_freq: Frequency,
        attenuation: Attenuation,
        power_level: PowerLevel,
        sweep_steps: u16,
        freq_step: Frequency,
        step_delay: Duration,
    ) -> io::Result<()> {
        self.send_command(
            Command::StartFreqSweep {
                start_freq_khz: start_freq.get::<kilohertz>(),
                attenuation,
                power_level,
                sweep_steps,
                step_freq_khz: freq_step.get::<kilohertz>(),
                step_delay,
            }
            .to_vec(),
        )
    }

    /// Starts the signal generator's frequency sweep mode using the expansion module.
    pub fn start_freq_sweep_exp(
        &mut self,
        start_freq: Frequency,
        power_dbm: f64,
        sweep_steps: u16,
        freq_step: Frequency,
        step_delay: Duration,
    ) -> io::Result<()> {
        self.send_command(
            Command::StartFreqSweepExp {
                start_freq_khz: start_freq.get::<kilohertz>(),
                power_dbm,
                sweep_steps,
                step_freq_khz: freq_step.get::<kilohertz>(),
                step_delay,
            }
            .to_vec(),
        )
    }

    /// Starts the signal generator's tracking mode.
    pub fn start_tracking(
        &mut self,
        start_freq: Frequency,
        attenuation: Attenuation,
        power_level: PowerLevel,
        sweep_steps: u16,
        freq_step: Frequency,
    ) -> io::Result<()> {
        self.send_command(
            Command::StartTracking {
                start_freq_khz: start_freq.get::<kilohertz>(),
                attenuation,
                power_level,
                sweep_steps,
                step_freq_khz: freq_step.get::<kilohertz>(),
            }
            .to_vec(),
        )
    }

    /// Starts the signal generator's tracking mode using the expansion module.
    pub fn start_tracking_exp(
        &mut self,
        start_freq: Frequency,
        power_dbm: f64,
        sweep_steps: u16,
        freq_step: Frequency,
    ) -> io::Result<()> {
        self.send_command(
            Command::StartTrackingExp {
                start_freq_khz: start_freq.get::<kilohertz>(),
                power_dbm,
                sweep_steps,
                step_freq_khz: freq_step.get::<kilohertz>(),
            }
            .to_vec(),
        )
    }

    pub fn tracking_step(&mut self, steps: u16) -> io::Result<()> {
        self.send_command(Command::TrackingStep(steps).to_vec())
    }

    pub fn rf_power_on(&mut self) -> io::Result<()> {
        self.send_command(Command::RfPowerOn.to_vec())
    }

    pub fn rf_power_off(&mut self) -> io::Result<()> {
        self.send_command(Command::RfPowerOff.to_vec())
    }
}

impl RfExplorer for SignalGenerator {
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
        self.send_command(rf_explorer::Command::RequestSerialNumber)?;

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

impl Drop for SignalGenerator {
    fn drop(&mut self) {
        *self.is_reading.lock().unwrap() = false;
        if let Some(read_handle) = self.read_thread_handle.take() {
            let _ = read_handle.join();
        }
    }
}

impl Debug for SignalGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalGenerator")
            .field("setup_info", &self.setup_info)
            .field("config", &self.config)
            .finish()
    }
}
