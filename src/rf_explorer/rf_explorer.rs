use crate::rf_explorer::SerialNumber;
use num_enum::IntoPrimitive;
use serialport::{
    ClearBuffer, DataBits, FlowControl, Parity, SerialPort, SerialPortInfo, SerialPortSettings,
    StopBits,
};
use std::{
    convert::TryFrom,
    fmt::Debug,
    io::{self, BufRead, BufReader},
    time::{Duration, Instant},
};
use thiserror::Error;

pub trait RfExplorer: for<'a> TryFrom<&'a SerialPortInfo> {
    type Setup: for<'a> TryFrom<&'a [u8]>;
    type Config: for<'a> TryFrom<&'a [u8]>;

    const SERIAL_PORT_PID: u16 = 60000;
    const SERIAL_PORT_VID: u16 = 4292;
    const CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);
    const DEFAULT_REQUEST_SERIAL_NUMBER_TIMEOUT: Duration = Duration::from_secs(2);
    const DEFAULT_REQUEST_CONFIG_TIMEOUT: Duration = Duration::from_secs(2);
    const SERIAL_PORT_SETTIGNS: SerialPortSettings = SerialPortSettings {
        baud_rate: 500_000,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_secs(1),
    };

    fn reader(&mut self) -> &mut SerialPortReader;

    fn setup(&self) -> Self::Setup;

    fn wait_for_response<T>(&mut self, timeout: Duration) -> Result<T>
    where
        T: for<'a> TryFrom<&'a [u8]>;

    fn write_command(&mut self, command: &[u8]) -> Result<()> {
        let mut command_buf = vec![
            b'#',
            u8::try_from(command.len() + 2).map_err(|_| {
                Error::InvalidInput("Command must be between 0 and 253 bytes long".to_string())
            })?,
        ];
        command_buf.append(&mut command.to_vec());
        Ok(self.reader().get_mut().write_all(&command_buf)?)
    }

    fn request_config(&mut self) -> Result<Self::Config> {
        self.request_config_with_timeout(<Self as RfExplorer>::DEFAULT_REQUEST_CONFIG_TIMEOUT)
    }

    fn request_config_with_timeout(&mut self, timeout: Duration) -> Result<Self::Config> {
        self.reader().get_mut().clear(ClearBuffer::Input)?;
        self.write_command(b"C0")?;
        self.wait_for_response(timeout)
    }

    fn request_shutdown(&mut self) -> Result<()> {
        self.write_command(b"S")
    }

    fn request_hold(&mut self) -> Result<()> {
        self.write_command(b"CH")
    }

    fn request_reboot(&mut self) -> Result<()> {
        self.write_command(b"r")
    }

    fn set_baud_rate(&mut self, baud_rate: BaudRate) -> Result<()> {
        self.write_command(&[b'c', baud_rate.into()])?;
        Ok(self.reader().get_mut().set_baud_rate(baud_rate.bps())?)
    }

    fn enable_lcd(&mut self) -> Result<()> {
        self.write_command(b"L1")
    }

    fn disable_lcd(&mut self) -> Result<()> {
        self.write_command(b"L0")
    }

    fn enable_dump_screen(&mut self) -> Result<()> {
        self.write_command(b"D1")
    }

    fn disable_dump_screen(&mut self) -> Result<()> {
        self.write_command(b"D0")
    }

    fn request_serial_number(&mut self) -> Result<SerialNumber> {
        self.request_serial_number_with_timeout(
            <Self as RfExplorer>::DEFAULT_REQUEST_SERIAL_NUMBER_TIMEOUT,
        )
    }

    fn request_serial_number_with_timeout(&mut self, timeout: Duration) -> Result<SerialNumber> {
        self.reader().get_ref().clear(ClearBuffer::Input)?;
        self.write_command(b"Cn")?;
        self.wait_for_response(timeout)
    }
}

#[derive(Debug, Copy, Clone, IntoPrimitive)]
#[repr(u8)]
pub enum BaudRate {
    _500000bps = b'0',
    _1200bps = b'1',
    _2400bps = b'2',
    _4800bps = b'3',
    _9600bps = b'4',
    _19200bps = b'5',
    _38400bps = b'6',
    _57600bps = b'7',
    _115200bps = b'8',
}

impl BaudRate {
    pub fn bps(&self) -> u32 {
        match self {
            BaudRate::_500000bps => 500_000,
            BaudRate::_1200bps => 1_200,
            BaudRate::_2400bps => 2_400,
            BaudRate::_4800bps => 4_800,
            BaudRate::_9600bps => 9_600,
            BaudRate::_19200bps => 19_200,
            BaudRate::_38400bps => 38_400,
            BaudRate::_57600bps => 57_600,
            BaudRate::_115200bps => 115_200,
        }
    }
}

pub type SerialPortReader = BufReader<Box<dyn SerialPort>>;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);

pub(crate) fn try_read_setup_and_config<S, C>(
    serial_port: &mut Box<dyn SerialPort>,
) -> std::result::Result<(S, C), ConnectionError>
where
    S: for<'a> TryFrom<&'a [u8]>,
    C: for<'a> TryFrom<&'a [u8]>,
{
    let mut reader = BufReader::new(serial_port);

    // Request a config
    reader.get_mut().write_all(&[b'#', 4, b'C', b'0'])?;

    let (mut rfe_setup, mut rfe_config) = (None, None);
    let mut message_buf = Vec::new();
    let start_time = Instant::now();

    while (rfe_setup.is_none() || rfe_config.is_none())
        && start_time.elapsed() <= CONNECTION_TIMEOUT
    {
        reader.read_until(b'\n', &mut message_buf)?;

        // The last two bytes of each message are \r and \n
        // Create an RfExplorerSetup or RfExplorerConfig from a slice of bytes without \r\n
        if let Some(rfe_message) = message_buf.get(0..message_buf.len() - 2) {
            if let Ok(setup) = S::try_from(rfe_message) {
                rfe_setup = Some(setup);
            } else if let Ok(config) = C::try_from(rfe_message) {
                rfe_config = Some(config);
            }
        }

        message_buf.clear();
    }

    if let (Some(setup), Some(config)) = (rfe_setup, rfe_config) {
        Ok((setup, config))
    } else {
        Err(ConnectionError::ConnectionTimedOut(CONNECTION_TIMEOUT))
    }
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

macro_rules! impl_rf_explorer {
    ($type:ty, $setup:ty, $config:ty) => {
        impl crate::rf_explorer::RfExplorer for $type {
            type Setup = $setup;
            type Config = $config;

            fn reader(&mut self) -> &mut crate::rf_explorer::SerialPortReader {
                &mut self.reader
            }

            fn setup(&self) -> Self::Setup {
                self.setup.clone()
            }

            fn wait_for_response<T>(
                &mut self,
                timeout: std::time::Duration,
            ) -> crate::rf_explorer::Result<T>
            where
                T: for<'a> std::convert::TryFrom<&'a [u8]>,
            {
                self.message_buf.clear();
                let start_time = std::time::Instant::now();

                while start_time.elapsed() <= timeout {
                    std::io::BufRead::read_until(&mut self.reader, b'\n', &mut self.message_buf)?;

                    // The last two bytes of each message are \r and \n
                    // Try to create the response from a slice of bytes without \r\n
                    if let Some(rfe_message) = self.message_buf.get(0..self.message_buf.len() - 2) {
                        if let Ok(response) = T::try_from(rfe_message) {
                            return Ok(response);
                        }
                    }

                    // The line we read was not the response, so clear the message buffer before reading the next line
                    self.message_buf.clear();
                }

                Err(crate::rf_explorer::Error::ResponseTimedOut(timeout))
            }
        }

        impl std::convert::TryFrom<&serialport::SerialPortInfo> for $type {
            type Error = crate::rf_explorer::ConnectionError;

            fn try_from(
                serial_port_info: &serialport::SerialPortInfo,
            ) -> std::result::Result<Self, Self::Error> {
                // Check the SerialPortInfo and make sure it's a USB port with the PID and VID of an RF Explorer
                match serial_port_info.port_type {
                    serialport::SerialPortType::UsbPort(serialport::UsbPortInfo {
                        pid: <Self as crate::RfExplorer>::SERIAL_PORT_PID,
                        vid: <Self as crate::RfExplorer>::SERIAL_PORT_VID,
                        ..
                    }) => {
                        let mut serial_port = serialport::open_with_settings(
                            &serial_port_info.port_name,
                            &<Self as crate::RfExplorer>::SERIAL_PORT_SETTIGNS,
                        )?;
                        let (setup, config) =
                            crate::rf_explorer::rf_explorer::try_read_setup_and_config(
                                &mut serial_port,
                            )?;
                        Ok(Self {
                            reader: std::io::BufReader::new(serial_port),
                            setup,
                            config,
                            message_buf: Vec::new(),
                        })
                    }
                    _ => Err(crate::rf_explorer::ConnectionError::NotAnRfExplorer),
                }
            }
        }
    };
}
