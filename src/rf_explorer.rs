use crate::{
    ParseSweepError, RfExplorerCalcMode, RfExplorerConfig, RfExplorerDspMode,
    RfExplorerSerialNumber, RfExplorerSetup, RfExplorerSweep, RfExplorerWifiMode,
};
use serialport::{
    ClearBuffer, DataBits, FlowControl, Parity, SerialPort, SerialPortInfo, SerialPortSettings,
    SerialPortType, StopBits, UsbPortInfo,
};
use std::{
    convert::TryFrom,
    fmt::{self, Debug},
    io::{self, BufRead, BufReader},
    ops::RangeInclusive,
    time::{Duration, SystemTime},
};
use thiserror::Error;

type SerialPortReader = BufReader<Box<dyn SerialPort>>;

pub struct RfExplorer {
    serial_port: SerialPortReader,
    setup: RfExplorerSetup,
    config: RfExplorerConfig,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("The attempted operation requires a more recent RF Explorer firmware")]
    IncompatibleFirmwareVersion,

    #[error("Invalid input: {}", .0)]
    InvalidInput(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Failed to receive a response from the RF Explorer within the timeout duration ({} ms)", .0.as_millis())]
    ResponseTimedOut(Duration),

    #[error(transparent)]
    SerialPort(#[from] serialport::Error),
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Failed to establish a connection to the RF Explorer within the timeout duration ({} ms)", .0.as_millis())]
    ConnectionTimedOut(Duration),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Attempted to connect to a device that is not an RF Explorer")]
    NotAnRfExplorer,

    #[error(transparent)]
    SerialPort(#[from] serialport::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl RfExplorer {
    const SERIAL_PORT_SETTIGNS: SerialPortSettings = SerialPortSettings {
        baud_rate: 500_000,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_secs(1),
    };
    const SERIAL_PORT_PID: u16 = 60000;
    const SERIAL_PORT_VID: u16 = 4292;
    const DEFAULT_NEXT_SWEEP_TIMEOUT: Duration = Duration::from_secs(2);
    const DEFAULT_REQUEST_CONFIG_TIMEOUT: Duration = Duration::from_secs(2);
    const DEFAULT_REQUEST_SERIAL_NUMBER_TIMEOUT: Duration = Duration::from_secs(2);
    const CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);
    const MIN_MAX_AMP_RANGE_DBM: RangeInclusive<i16> = -120..=35;

    pub fn setup(&self) -> RfExplorerSetup {
        self.setup.clone()
    }

    pub fn config(&self) -> RfExplorerConfig {
        self.config
    }

    pub fn next_sweep(&mut self) -> Result<RfExplorerSweep> {
        self.next_sweep_with_timeout(RfExplorer::DEFAULT_NEXT_SWEEP_TIMEOUT)
    }

    pub fn next_sweep_with_timeout(&mut self, timeout: Duration) -> Result<RfExplorerSweep> {
        // Before reading the next sweep, we should clear the serial port's input buffer
        // This will prevent us from reading a stale sweep
        self.serial_port.get_ref().clear(ClearBuffer::Input)?;

        let mut rfe_message_buf = Vec::new();
        let start_time = SystemTime::now();

        while start_time.elapsed().map_or(false, |e| e <= timeout) {
            self.serial_port.read_until(b'\n', &mut rfe_message_buf)?;

            // It's possible that the byte '\n' could be used to represent an amplitude (-5 dBm)
            // This would result in an invalid sweep with fewer amplitudes than indicated by the length field
            // If parsing the bytes fails with ParseSweepError::TooFewAmplitudes, do not clear the message buffer
            // This will give us another chance to find the real end of the sweep because read_until() appends to the message buffer
            if let Some(rfe_message) = rfe_message_buf.get(0..rfe_message_buf.len() - 2) {
                match RfExplorerSweep::try_from(rfe_message) {
                    Ok(sweep) => return Ok(sweep),
                    Err(ParseSweepError::TooFewAmplitudes { .. }) => continue,
                    Err(_) => (),
                }
            }

            // The line we read was not a sweep, so clear the message buffer before reading the next line
            rfe_message_buf.clear();
        }

        Err(Error::ResponseTimedOut(timeout))
    }

    pub fn write_command(&mut self, command: &[u8]) -> Result<()> {
        let mut command_buf = vec![
            b'#',
            u8::try_from(command.len() + 2).map_err(|_| {
                Error::InvalidInput("Command must be between 0 and 253 bytes long".to_string())
            })?,
        ];
        command_buf.append(&mut command.to_vec());
        Ok(self.serial_port.get_mut().write_all(&command_buf)?)
    }

    // PC to RF Explorer Any Model

    pub fn request_config(&mut self) -> Result<RfExplorerConfig> {
        self.request_config_with_timeout(RfExplorer::DEFAULT_REQUEST_CONFIG_TIMEOUT)
    }

    pub fn request_config_with_timeout(&mut self, timeout: Duration) -> Result<RfExplorerConfig> {
        // Before asking the RF Explorer to send its config, we should clear the serial port's input buffer
        // This will allow us to read the config without having to read a bunch of unrelated data first
        self.serial_port.get_ref().clear(ClearBuffer::Input)?;
        self.write_command(b"C0")?;

        self.config = self.wait_for_response(timeout)?;
        Ok(self.config)
    }

    pub fn request_shutdown(&mut self) -> Result<()> {
        self.write_command(b"S")
    }

    pub fn request_hold(&mut self) -> Result<()> {
        self.write_command(b"CH")
    }

    pub fn request_reboot(&mut self) -> Result<()> {
        self.write_command(b"r")
    }

    pub fn change_baudrate(&mut self) -> Result<()> {
        todo!()
    }

    pub fn enable_lcd(&mut self) -> Result<()> {
        self.write_command(b"L1")
    }

    pub fn disable_lcd(&mut self) -> Result<()> {
        self.write_command(b"L0")
    }

    pub fn enable_dump_screen(&mut self) -> Result<()> {
        self.write_command(b"D1")
    }

    pub fn disable_dump_screen(&mut self) -> Result<()> {
        self.write_command(b"D0")
    }

    pub fn request_serial_number(&mut self) -> Result<RfExplorerSerialNumber> {
        self.request_serial_number_with_timeout(RfExplorer::DEFAULT_REQUEST_SERIAL_NUMBER_TIMEOUT)
    }

    pub fn request_serial_number_with_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<RfExplorerSerialNumber> {
        // Before asking the RF Explorer to send its serial number, we should clear the serial port's input buffer
        // This will allow us to read the serial number without having to read a bunch of unrelated data first
        self.serial_port.get_ref().clear(ClearBuffer::Input)?;
        self.write_command(b"Cn")?;

        self.wait_for_response(timeout)
    }

    // PC to RF Explorer Spectrum Analyzer

    pub fn change_config(
        &mut self,
        start_freq_khz: f64,
        end_freq_khz: f64,
        amp_bottom_dbm: i16,
        amp_top_dbm: i16,
    ) -> Result<RfExplorerConfig> {
        self.validate_freq_range(start_freq_khz..=end_freq_khz)?;
        self.validate_amp_range(amp_bottom_dbm..=amp_top_dbm)?;

        let command = format!(
            "C2-F:{:07.0},{:07.0},{:04},{:04}",
            start_freq_khz, end_freq_khz, amp_top_dbm, amp_bottom_dbm
        );
        // Before asking the RF Explorer to change its config, we should clear the serial port's input buffer
        // This will allow us to read the RF Explorer's response without having to read a bunch of unrelated data first
        self.serial_port.get_ref().clear(ClearBuffer::Input)?;
        self.write_command(command.as_bytes())?;

        self.config = self.wait_for_response(RfExplorer::DEFAULT_REQUEST_CONFIG_TIMEOUT)?;
        Ok(self.config)
    }

    pub fn change_freq_range(
        &mut self,
        start_freq_khz: f64,
        end_freq_khz: f64,
    ) -> Result<RfExplorerConfig> {
        self.change_config(
            start_freq_khz,
            end_freq_khz,
            self.config.amp_bottom_dbm(),
            self.config.amp_top_dbm(),
        )
    }

    pub fn change_center_span(
        &mut self,
        center_freq_khz: f64,
        span_khz: f64,
    ) -> Result<RfExplorerConfig> {
        self.change_freq_range(
            center_freq_khz - span_khz / 2f64,
            center_freq_khz + span_khz / 2f64,
        )
    }

    pub fn change_amp_range(
        &mut self,
        amp_bottom_dbm: i16,
        amp_top_dbm: i16,
    ) -> Result<RfExplorerConfig> {
        self.change_config(
            self.config.start_freq_khz(),
            self.config.end_freq_khz(),
            amp_bottom_dbm,
            amp_top_dbm,
        )
    }

    pub fn switch_module_main(&mut self) -> Result<()> {
        self.write_command(&[b'C', b'M', 0])
    }

    pub fn switch_module_expansion(&mut self) -> Result<()> {
        self.write_command(&[b'C', b'M', 1])
    }

    pub fn set_wifi_mode(&mut self, wifi_mode: RfExplorerWifiMode) -> Result<()> {
        self.write_command(&[b'C', b'W', wifi_mode as u8])
    }

    pub fn set_calc_mode(&mut self, calc_mode: RfExplorerCalcMode) -> Result<()> {
        self.write_command(&[b'C', b'+', calc_mode as u8])
    }

    pub fn request_tracking(&mut self, start_freq_khz: f64, freq_step_khz: f64) -> Result<()> {
        let command = format!("C3-K:{:07.0},{:07.0}", start_freq_khz, freq_step_khz);
        self.write_command(command.as_bytes())
    }

    pub fn tracking_step(&mut self, step: u16) -> Result<()> {
        todo!()
    }

    pub fn set_dsp(&mut self, dsp_mode: RfExplorerDspMode) -> Result<()> {
        self.write_command(&[b'C', b'p', dsp_mode as u8])
    }

    pub fn set_offset_db(&mut self, offset_db: i8) -> Result<()> {
        self.write_command(&[b'C', b'O', offset_db as u8])
    }

    pub fn set_input_stage(&mut self) -> Result<()> {
        todo!()
    }

    pub fn set_sweep_points(&mut self, sweep_points: u16) -> Result<()> {
        if sweep_points <= 4096 {
            self.write_command(&[b'C', b'J', ((sweep_points / 16) - 1) as u8])
        } else {
            todo!()
        }
    }

    pub fn set_sweep_points_large(&mut self, sweep_points: u16) -> Result<()> {
        todo!()
    }

    fn wait_for_response<T>(&mut self, timeout: Duration) -> Result<T>
    where
        T: for<'a> TryFrom<&'a [u8]>,
    {
        let mut rfe_message_buf = Vec::new();
        let start_time = SystemTime::now();

        while start_time.elapsed().map_or(false, |e| e <= timeout) {
            self.serial_port.read_until(b'\n', &mut rfe_message_buf)?;

            // The last two bytes of each message are \r and \n
            // Try to create the response from a slice of bytes without \r\n
            if let Some(rfe_message) = rfe_message_buf.get(0..rfe_message_buf.len() - 2) {
                if let Ok(response) = T::try_from(rfe_message) {
                    return Ok(response);
                }
            }

            // The line we read was not the response, so clear the message buffer before reading the next line
            rfe_message_buf.clear();
        }

        Err(Error::ResponseTimedOut(timeout))
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
        if !RfExplorer::MIN_MAX_AMP_RANGE_DBM.contains(amp_range_dbm.start()) {
            return Err(Error::InvalidInput("".to_string()));
        } else if !RfExplorer::MIN_MAX_AMP_RANGE_DBM.contains(amp_range_dbm.end()) {
            return Err(Error::InvalidInput("".to_string()));
        }

        Ok(())
    }
}

impl TryFrom<&SerialPortInfo> for RfExplorer {
    type Error = ConnectionError;

    fn try_from(serial_port_info: &SerialPortInfo) -> std::result::Result<Self, Self::Error> {
        // Check the SerialPortInfo and make sure it's a USB port with the PID and VID of an RF Explorer
        match serial_port_info.port_type {
            SerialPortType::UsbPort(UsbPortInfo {
                pid: RfExplorer::SERIAL_PORT_PID,
                vid: RfExplorer::SERIAL_PORT_VID,
                ..
            }) => RfExplorer::try_from(serialport::open_with_settings(
                &serial_port_info.port_name,
                &RfExplorer::SERIAL_PORT_SETTIGNS,
            )?),
            _ => Err(ConnectionError::NotAnRfExplorer),
        }
    }
}

impl TryFrom<Box<dyn SerialPort>> for RfExplorer {
    type Error = ConnectionError;

    fn try_from(serial_port: Box<dyn SerialPort>) -> std::result::Result<Self, Self::Error> {
        let mut serial_port = BufReader::new(serial_port);

        // Request an RfExplorerConfig
        serial_port.get_mut().write_all(&[b'#', 4, b'C', b'0'])?;

        let (mut rfe_setup, mut rfe_config) = (None, None);
        let mut rfe_message_buf = Vec::new();
        let start_time = SystemTime::now();

        // Only create an RfExplorer object if we receive a valid RfExplorerSetup and RfExplorerConfig within the timeout duration
        while (rfe_setup.is_none() || rfe_config.is_none())
            && start_time
                .elapsed()
                .map_or(false, |e| e <= RfExplorer::CONNECTION_TIMEOUT)
        {
            serial_port.read_until(b'\n', &mut rfe_message_buf)?;

            // The last two bytes of each message are \r and \n
            // Create an RfExplorerSetup or RfExplorerConfig from a slice of bytes without \r\n
            if let Some(rfe_message) = rfe_message_buf.get(0..rfe_message_buf.len() - 2) {
                if let Ok(setup) = RfExplorerSetup::try_from(rfe_message) {
                    rfe_setup = Some(setup);
                } else if let Ok(config) = RfExplorerConfig::try_from(rfe_message) {
                    rfe_config = Some(config);
                }
            }

            rfe_message_buf.clear();
        }

        if let (Some(setup), Some(config)) = (rfe_setup, rfe_config) {
            Ok(RfExplorer {
                serial_port,
                setup,
                config,
            })
        } else {
            Err(ConnectionError::ConnectionTimedOut(
                RfExplorer::CONNECTION_TIMEOUT,
            ))
        }
    }
}

impl Debug for RfExplorer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RfExplorer")
            .field("setup", &self.setup)
            .field("config", &self.config)
            .finish()
    }
}
