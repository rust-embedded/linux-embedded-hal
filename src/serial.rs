//! Implementation of [`Serial`](https://docs.rs/embedded-hal/0.2.1/embedded_hal/serial/index.html)

use std::io::{Error as IoError, Read, Write};

use nb;

use hal;
use serial_unix::TTYPort;

/// Newtype around [`serial_unix::TTYPort`] that implements
/// the `embedded-hal` traits.
pub struct Serial(pub TTYPort);

impl hal::serial::Read<u8> for Serial {
    type Error = IoError;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut buffer = [0; 1];
        let bytes_read = self
            .0
            .read(&mut buffer)
            .map_err(|err| nb::Error::Other(Self::Error::from(err)))?;
        if bytes_read == 1 {
            Ok(buffer[0])
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl hal::serial::Write<u8> for Serial {
    type Error = IoError;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.0
            .write(&[word])
            .map_err(|err| nb::Error::Other(Self::Error::from(err)))?;
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.0
            .flush()
            .map_err(|err| nb::Error::Other(Self::Error::from(err)))
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use hal::serial::Read;
    use std::io::Write;

    use super::*;

    #[test]
    fn test_empty() {
        let (mut master, _slave, name) =
            openpty::openpty(None, None, None).expect("Creating pty failed");
        println!("{:?}", name);
        let port = TTYPort::open(Path::new(&name)).unwrap();
        let mut serial = Serial(port);
        master.write(&[1]).unwrap();
        serial.read().unwrap();
    }
}
