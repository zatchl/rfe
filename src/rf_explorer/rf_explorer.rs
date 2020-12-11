use crate::rf_explorer::{Message, ParseFromBytes, SerialNumber};
use num_enum::IntoPrimitive;
use serialport::{
    ClearBuffer, DataBits, FlowControl, Parity, SerialPort, SerialPortInfo, SerialPortSettings,
    SerialPortType, StopBits, UsbPortInfo,
};
use std::{
    convert::TryFrom,
    fmt::Debug,
    io::{self, BufRead, BufReader},
    time::{Duration, Instant},
};
use thiserror::Error;

const PID: u16 = 60000;
const VID: u16 = 4292;
const READ_SETUP_TIMEOUT: Duration = Duration::from_secs(1);
const DEFAULT_REQUEST_SERIAL_NUMBER_TIMEOUT: Duration = Duration::from_secs(2);
const DEFAULT_REQUEST_CONFIG_TIMEOUT: Duration = Duration::from_secs(2);
const SERIAL_PORT_SETTINGS: SerialPortSettings = SerialPortSettings {
    baud_rate: 500_000,
    data_bits: DataBits::Eight,
    flow_control: FlowControl::None,
    parity: Parity::None,
    stop_bits: StopBits::One,
    timeout: Duration::from_secs(1),
};

pub trait RfExplorer: Sized {
    type SetupInfo: Message;
    type Config: Message;

    fn new(reader: SerialPortReader, setup_info: Self::SetupInfo, config: Self::Config) -> Self;

    fn reader(&mut self) -> &mut SerialPortReader;

    fn setup_info(&self) -> Self::SetupInfo;

    fn connect(port_info: &SerialPortInfo) -> Result<Self, ConnectionError> {
        let (port_type, port_name) = (&port_info.port_type, &port_info.port_name);
        // Check the SerialPortInfo and make sure it's a USB port with the PID and VID of an RF Explorer
        if let SerialPortType::UsbPort(UsbPortInfo {
            pid: PID, vid: VID, ..
        }) = port_type
        {
            let mut reader = {
                let mut port = serialport::open_with_settings(port_name, &SERIAL_PORT_SETTINGS)?;
                // Request the RF Explorer's config to get the RF Explorer to start sending us bytes
                write_command(b"C0", &mut port)?;
                SerialPortReader::new(port)
            };

            let setup = read_message(&mut reader, READ_SETUP_TIMEOUT)?;

            // Request the RF Explorer's config again in case it was discarded while reading the setup message
            write_command(b"C0", &mut reader.get_mut())?;

            let config = read_message(&mut reader, DEFAULT_REQUEST_CONFIG_TIMEOUT)?;

            return Ok(Self::new(reader, setup, config));
        }

        return Err(ConnectionError::NotAnRfExplorer);
    }

    fn write_command(&mut self, command: &[u8]) -> WriteCommandResult<()> {
        write_command(command, self.reader().get_mut())
    }

    fn read_message<T: ParseFromBytes>(&mut self, timeout: Duration) -> ReadMessageResult<T> {
        read_message(self.reader(), timeout)
    }

    fn request_config(&mut self) -> RfeResult<Self::Config> {
        self.request_config_with_timeout(DEFAULT_REQUEST_CONFIG_TIMEOUT)
    }

    fn request_config_with_timeout(&mut self, timeout: Duration) -> RfeResult<Self::Config> {
        self.reader().get_mut().clear(ClearBuffer::Input)?;
        self.write_command(b"C0")?;
        Ok(self.read_message(timeout)?)
    }

    fn request_shutdown(&mut self) -> WriteCommandResult<()> {
        self.write_command(b"S")
    }

    fn request_hold(&mut self) -> WriteCommandResult<()> {
        self.write_command(b"CH")
    }

    fn request_reboot(&mut self) -> WriteCommandResult<()> {
        self.write_command(b"r")
    }

    fn set_baud_rate(&mut self, baud_rate: BaudRate) -> WriteCommandResult<()> {
        self.write_command(&[b'c', baud_rate.into()])?;
        Ok(self.reader().get_mut().set_baud_rate(baud_rate.bps())?)
    }

    fn enable_lcd(&mut self) -> WriteCommandResult<()> {
        self.write_command(b"L1")
    }

    fn disable_lcd(&mut self) -> WriteCommandResult<()> {
        self.write_command(b"L0")
    }

    fn enable_dump_screen(&mut self) -> WriteCommandResult<()> {
        self.write_command(b"D1")
    }

    fn disable_dump_screen(&mut self) -> WriteCommandResult<()> {
        self.write_command(b"D0")
    }

    fn request_serial_number(&mut self) -> RfeResult<SerialNumber> {
        self.request_serial_number_with_timeout(DEFAULT_REQUEST_SERIAL_NUMBER_TIMEOUT)
    }

    fn request_serial_number_with_timeout(&mut self, timeout: Duration) -> RfeResult<SerialNumber> {
        self.reader().get_ref().clear(ClearBuffer::Input)?;
        self.write_command(b"Cn")?;
        Ok(self.read_message(timeout)?)
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

pub(crate) type RfeResult<T> = Result<T, Error>;

pub(crate) type WriteCommandResult<T> = Result<T, WriteCommandError>;

pub(crate) type ReadMessageResult<T> = Result<T, ReadMessageError>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FailedToReadMessage(#[from] ReadMessageError),

    #[error(transparent)]
    FailedToWriteCommand(#[from] WriteCommandError),

    #[error("The attempted operation requires a more recent RF Explorer firmware")]
    IncompatibleFirmwareVersion,

    #[error("Invalid input: {}", .0)]
    InvalidInput(String),

    #[error("Invalid operation: {}", .0)]
    InvalidOperation(String),

    #[error("Invalid response: {}", .0)]
    InvalidResponse(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    SerialPort(#[from] serialport::Error),
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Failed to establish a connection to the RF Explorer within the timeout duration")]
    ConnectionTimedOut,

    #[error(transparent)]
    FailedToReadMessage(#[from] ReadMessageError),

    #[error(transparent)]
    FailedToWriteCommand(#[from] WriteCommandError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Attempted to connect to a device that is not an RF Explorer")]
    NotAnRfExplorer,

    #[error(transparent)]
    SerialPort(#[from] serialport::Error),
}

#[derive(Error, Debug)]
pub enum WriteCommandError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    SerialPort(#[from] serialport::Error),

    #[error("Commands must be between 0 and 253 bytes long")]
    CommandTooLong(#[from] std::num::TryFromIntError),
}

#[derive(Error, Debug)]
pub enum ReadMessageError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    SerialPort(#[from] serialport::Error),

    #[error("Failed to read the message from the RF Explorer within the timeout duration ({} ms)", .0.as_millis())]
    TimedOut(Duration),
}

fn write_command(command: &[u8], port: &mut impl io::Write) -> WriteCommandResult<()> {
    let mut command_buf = vec![b'#', u8::try_from(command.len() + 2)?];
    command_buf.append(&mut command.to_vec());
    Ok(port.write_all(&command_buf)?)
}

fn read_message<T: ParseFromBytes>(
    reader: &mut SerialPortReader,
    timeout: Duration,
) -> ReadMessageResult<T> {
    let mut message_buf = Vec::new();
    let start_time = Instant::now();

    while start_time.elapsed() <= timeout {
        // Every message from the RF Explorer ends with \r\n
        reader.read_until(b'\n', &mut message_buf)?;

        // Return the message if it's succesfully parsed
        // Continue reading if parsing is incomplete
        // Clear the message buffer and then continue reading if parsing fails
        match T::parse_from_bytes(message_buf.as_ref()) {
            Ok(response) => return Ok(response.1),
            Err(nom::Err::Incomplete(_)) => continue,
            _ => message_buf.clear(),
        }
    }

    Err(ReadMessageError::TimedOut(timeout))
}
