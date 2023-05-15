//! Implementation of [`embedded-hal`] SPI traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal
//!

use std::fmt;
use std::io;
use std::ops;
use std::path::Path;

/// Newtype around [`spidev::Spidev`] that implements the `embedded-hal` traits
///
/// [`spidev::Spidev`]: https://docs.rs/spidev/0.5.0/spidev/struct.Spidev.html
pub struct Spidev(pub spidev::Spidev);

impl Spidev {
    /// See [`spidev::Spidev::open`][0] for details.
    ///
    /// [0]: https://docs.rs/spidev/0.5.0/spidev/struct.Spidev.html#method.open
    pub fn open<P>(path: P) -> Result<Self, SPIError>
    where
        P: AsRef<Path>,
    {
        spidev::Spidev::open(path).map(Spidev).map_err(|e| e.into())
    }
}

impl ops::Deref for Spidev {
    type Target = spidev::Spidev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Spidev {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

mod embedded_hal_impl {
    use super::*;
    use embedded_hal::spi::ErrorType;
    use embedded_hal::spi::{
        Operation as SpiOperation, SpiBus, SpiBusFlush, SpiBusRead, SpiBusWrite, SpiDevice,
        SpiDeviceRead, SpiDeviceWrite,
    };
    use spidev::SpidevTransfer;
    use std::io::{Read, Write};

    impl ErrorType for Spidev {
        type Error = SPIError;
    }

    impl SpiBusFlush for Spidev {
        fn flush(&mut self) -> Result<(), Self::Error> {
            self.0.flush().map_err(|err| SPIError { err })
        }
    }

    impl SpiBusRead<u8> for Spidev {
        fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            self.0.read_exact(words).map_err(|err| SPIError { err })
        }
    }

    impl SpiBusWrite<u8> for Spidev {
        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            self.0.write_all(words).map_err(|err| SPIError { err })
        }
    }

    impl SpiBus<u8> for Spidev {
        fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
            self.0
                .transfer(&mut SpidevTransfer::read_write(write, read))
                .map_err(|err| SPIError { err })
        }

        fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            let tx = words.to_owned();
            self.0
                .transfer(&mut SpidevTransfer::read_write(&tx, words))
                .map_err(|err| SPIError { err })
        }
    }

    impl SpiDeviceRead for Spidev {
        fn read_transaction(&mut self, operations: &mut [&mut [u8]]) -> Result<(), Self::Error> {
            for buf in operations {
                SpiBusRead::read(self, buf)?;
            }
            self.flush()?;
            Ok(())
        }
    }

    impl SpiDeviceWrite for Spidev {
        fn write_transaction(&mut self, operations: &[&[u8]]) -> Result<(), Self::Error> {
            for buf in operations {
                SpiBusWrite::write(self, buf)?;
            }
            self.flush()?;
            Ok(())
        }
    }

    impl SpiDevice for Spidev {
        fn transaction(
            &mut self,
            operations: &mut [SpiOperation<'_, u8>],
        ) -> Result<(), Self::Error> {
            operations.iter_mut().try_for_each(|op| match op {
                SpiOperation::Read(buf) => SpiBusRead::read(self, buf),
                SpiOperation::Write(buf) => SpiBusWrite::write(self, buf),
                SpiOperation::Transfer(read, write) => SpiBus::transfer(self, read, write),
                SpiOperation::TransferInPlace(buf) => SpiBus::transfer_in_place(self, buf),
            })?;
            self.flush()?;
            Ok(())
        }
    }
}

/// Error type wrapping [io::Error](io::Error) to implement [embedded_hal::spi::ErrorKind]
#[derive(Debug)]
pub struct SPIError {
    err: io::Error,
}

impl SPIError {
    /// Fetch inner (concrete) [`LinuxI2CError`]
    pub fn inner(&self) -> &io::Error {
        &self.err
    }
}

impl From<io::Error> for SPIError {
    fn from(err: io::Error) -> Self {
        Self { err }
    }
}

impl embedded_hal::spi::Error for SPIError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        use embedded_hal::spi::ErrorKind;
        match self.err.kind() {
            // TODO: match any errors here if we can find any that are relevant
            _ => ErrorKind::Other,
        }
    }
}

impl fmt::Display for SPIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl std::error::Error for SPIError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.err)
    }
}
