use super::Command;
use serialport::{
    DataBits, FlowControl, Parity, SerialPort, SerialPortInfo, SerialPortType, StopBits,
};
use std::{
    io::{self, BufReader},
    time::Duration,
};
use thiserror::Error;

pub(crate) fn open(port_info: &SerialPortInfo) -> ConnectionResult<SerialPortReader> {
    let mut serial_port = {
        let (port_type, port_name) = (&port_info.port_type, &port_info.port_name);
        if let SerialPortType::UsbPort(_) = port_type {
            Ok(serialport::new(port_name, 500_000)
                .data_bits(DataBits::Eight)
                .flow_control(FlowControl::None)
                .parity(Parity::None)
                .stop_bits(StopBits::One)
                .timeout(Duration::from_secs(1))
                .open()?)
        } else {
            Err(ConnectionError::NotAnRfExplorer)
        }
    }?;
    serial_port.write_all(Command::RequestConfig.as_ref())?;
    Ok(SerialPortReader::new(serial_port))
}

#[derive(Error, Debug)]
pub(crate) enum ConnectionError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Attempted to connect to a device that is not an RF Explorer")]
    NotAnRfExplorer,

    #[error(transparent)]
    SerialPort(#[from] serialport::Error),
}

pub(crate) type ConnectionResult<T> = Result<T, ConnectionError>;
pub(crate) type SerialPortReader = BufReader<Box<dyn SerialPort>>;
