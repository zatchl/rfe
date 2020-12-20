use super::{Command, Model, SerialNumber};
use std::{fmt::Debug, io, time::Duration};
use thiserror::Error;

pub trait RfExplorer {
    /// Attempts to send a command to the RF Explorer.
    fn send_command(&mut self, command: impl AsRef<[u8]>) -> io::Result<()>;

    /// Returns the model of the RF Explorer's main module.
    fn main_model(&self) -> Model;

    /// Returns the model of the RF Explorer's expansion module.
    fn expansion_model(&self) -> Option<Model>;

    /// Returns the RF Explorer's firmware version.
    fn firmware_version(&self) -> &str;

    /// Requests the RF Explorer's serial number.
    fn request_serial_number(&mut self) -> RfeResult<SerialNumber>;

    /// Turns the RF Explorer's screen on.
    fn enable_lcd(&mut self) -> io::Result<()> {
        self.send_command(Command::EnableLcd)
    }

    /// Turns the RF Explorer's screen off.
    fn disable_lcd(&mut self) -> io::Result<()> {
        self.send_command(Command::DisableLcd)
    }

    fn enable_dump_screen(&mut self) -> io::Result<()> {
        self.send_command(Command::EnableDumpScreen)
    }

    fn disable_dump_screen(&mut self) -> io::Result<()> {
        self.send_command(Command::DisableDumpScreen)
    }

    fn hold(&mut self) -> io::Result<()> {
        self.send_command(Command::Hold)
    }

    /// Reboots the RF Explorer.
    fn reboot(&mut self) -> io::Result<()> {
        self.send_command(Command::Reboot)
    }

    /// Powers off the RF Explorer.
    fn power_off(&mut self) -> io::Result<()> {
        self.send_command(Command::PowerOff)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid input: {}", .0)]
    InvalidInput(String),

    #[error("Invalid operation: {}", .0)]
    InvalidOperation(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Failed to complete the operation within the timeout duration ({} ms)", .0.as_millis())]
    TimedOut(Duration),
}

pub(crate) type RfeResult<T> = Result<T, Error>;
