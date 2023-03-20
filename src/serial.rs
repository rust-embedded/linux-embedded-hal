//! Implementation of [`embedded-hal`] serial traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use nb;
use serialport::{SerialPortBuilder, TTYPort};
use std::io::{ErrorKind as IoErrorKind, Read, Write};

/// Newtype around [`serialport::TTYPort`] that implements
/// the `embedded-hal` traits.
pub struct Serial(pub TTYPort);

impl Serial {
    /// Open a `serialport::TTYPort` by providing the port path and baud rate
    pub fn open(path: String, baud_rate: u32) -> Result<Serial, serialport::Error> {
        Ok(Serial(serialport::new(path, baud_rate).open_native()?))
    }

    /// Open a `serialport::TTYPort` by providing `serialport::SerialPortBuilder`
    pub fn open_from_builder(builder: SerialPortBuilder) -> Result<Serial, serialport::Error> {
        Ok(Serial(builder.open_native()?))
    }
}

/// Helper to convert std::io::Error to the nb::Error
fn translate_io_errors(err: std::io::Error) -> nb::Error<SerialError> {
    match err.kind() {
        IoErrorKind::WouldBlock | IoErrorKind::TimedOut | IoErrorKind::Interrupted => {
            nb::Error::WouldBlock
        }
        err => nb::Error::Other(SerialError { err }),
    }
}

impl embedded_hal::serial::ErrorType for Serial {
    type Error = SerialError;
}

impl embedded_hal_nb::serial::Read<u8> for Serial {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut buffer = [0; 1];
        let bytes_read = self.0.read(&mut buffer).map_err(translate_io_errors)?;
        if bytes_read == 1 {
            Ok(buffer[0])
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl embedded_hal_nb::serial::Write<u8> for Serial {
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.0.write(&[word]).map_err(translate_io_errors)?;
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.0.flush().map_err(translate_io_errors)
    }
}

/// Error type wrapping [io::ErrorKind](IoErrorKind) to implement [embedded_hal::serial::ErrorKind]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SerialError {
    err: IoErrorKind,
}

impl SerialError {
    /// Fetch inner (concrete) [`IoErrorKind`]
    pub fn inner(&self) -> &IoErrorKind {
        &self.err
    }
}

impl embedded_hal::serial::Error for SerialError {
    fn kind(&self) -> embedded_hal::serial::ErrorKind {
        use embedded_hal::serial::ErrorKind::*;
        match &self.err {
            // TODO: match any errors here if we can find any that are relevant
            _ => Other,
        }
    }
}

#[cfg(test)]
mod test {
    use embedded_hal_nb::serial::{Read, Write};
    use std::io::{Read as IoRead, Write as IoWrite};

    use super::*;

    fn create_pty_and_serial() -> (std::fs::File, Serial) {
        let (master, _slave, name) =
            openpty::openpty(None, None, None).expect("Creating pty failed");
        let serial = Serial::open(name, 9600).expect("Creating TTYPort failed");
        (master, serial)
    }

    #[test]
    fn create_serial_from_builder() {
        let (_master, _slave, name) =
            openpty::openpty(None, None, None).expect("Creating pty failed");
        let builder = serialport::new(name, 9600);
        let _serial = Serial::open_from_builder(builder).expect("Creating TTYPort failed");
    }

    #[test]
    fn test_empty_read() {
        let (mut _master, mut serial) = create_pty_and_serial();
        assert_eq!(Err(nb::Error::WouldBlock), serial.read());
    }

    #[test]
    fn test_read() {
        let (mut master, mut serial) = create_pty_and_serial();
        master.write(&[1]).expect("Write failed");
        assert_eq!(Ok(1), serial.read());
    }

    #[test]
    fn test_write() {
        let (mut master, mut serial) = create_pty_and_serial();
        serial.write(2).expect("Write failed");
        let mut buf = [0; 2];
        assert_eq!(1, master.read(&mut buf).unwrap());
        assert_eq!(buf, [2, 0]);
    }
}
