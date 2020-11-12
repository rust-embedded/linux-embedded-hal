//! Implementation of [`Serial`](https://docs.rs/embedded-hal/0.2.1/embedded_hal/serial/index.html)

use std::io::{ErrorKind as IoErrorKind, Read, Write};
use std::path::Path;

use nb;

use serial_core;
use serial_unix::TTYPort;

/// Newtype around [`serial_unix::TTYPort`] that implements
/// the `embedded-hal` traits.
pub struct Serial(pub TTYPort);

impl Serial {
    /// Wrapper for `serial_unix::TTYPort::open`
    pub fn open(path: impl AsRef<Path>) -> Result<Serial, serial_core::Error> {
        Ok(Serial(TTYPort::open(path.as_ref())?))
    }
}

/// Helper to convert std::io::Error to the nb::Error
fn translate_io_errors(err: std::io::Error) -> nb::Error<IoErrorKind> {
    match err.kind() {
        IoErrorKind::WouldBlock | IoErrorKind::TimedOut | IoErrorKind::Interrupted => {
            nb::Error::WouldBlock
        }
        err => nb::Error::Other(err),
    }
}

impl embedded_hal::serial::Read<u8> for Serial {
    type Error = IoErrorKind;

    fn try_read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut buffer = [0; 1];
        let bytes_read = self.0.read(&mut buffer).map_err(translate_io_errors)?;
        if bytes_read == 1 {
            Ok(buffer[0])
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl embedded_hal::serial::Write<u8> for Serial {
    type Error = IoErrorKind;

    fn try_write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.0.write(&[word]).map_err(translate_io_errors)?;
        Ok(())
    }

    fn try_flush(&mut self) -> nb::Result<(), Self::Error> {
        self.0.flush().map_err(translate_io_errors)
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use embedded_hal::serial::{Read, Write};
    use std::io::{Read as IoRead, Write as IoWrite};

    use super::*;

    fn create_pty_and_serial() -> (std::fs::File, Serial) {
        let (master, _slave, name) =
            openpty::openpty(None, None, None).expect("Creating pty failed");
        let serial = Serial::open(Path::new(&name)).expect("Creating TTYPort failed");
        (master, serial)
    }

    #[test]
    fn test_empty_read() {
        let (mut _master, mut serial) = create_pty_and_serial();
        assert_eq!(Err(nb::Error::WouldBlock), serial.try_read());
    }

    #[test]
    fn test_read() {
        let (mut master, mut serial) = create_pty_and_serial();
        master.write(&[1]).expect("Write failed");
        assert_eq!(Ok(1), serial.try_read());
    }

    #[test]
    fn test_write() {
        let (mut master, mut serial) = create_pty_and_serial();
        serial.try_write(2).expect("Write failed");
        let mut buf = [0; 2];
        assert_eq!(1, master.read(&mut buf).unwrap());
        assert_eq!(buf, [2, 0]);
    }
}
