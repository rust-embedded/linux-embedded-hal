//! Implementation of [`embedded-hal`] SPI traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal
//!

use std::fmt;
use std::io;
use std::ops;
use std::path::Path;

use crate::Delay;
use spidev::SpidevTransfer;

/// Newtype around [`spidev::Spidev`] that implements the `embedded-hal` traits
///
/// [`spidev::Spidev`]: https://docs.rs/spidev/0.5.spidev/spidev/struct.Spidev.html
pub struct Spidev(pub spidev::Spidev, Delay);

impl Spidev {
    /// See [`spidev::Spidev::open`][0] for details.
    ///
    /// [0]: https://docs.rs/spidev/0.5.spidev/spidev/struct.Spidev.html#method.open
    pub fn open<P>(path: P) -> Result<Self, SPIError>
    where
        P: AsRef<Path>,
    {
        spidev::Spidev::open(path)
            .map(|spidev| Spidev(spidev, Delay {}))
            .map_err(|e| e.into())
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
    use embedded_hal::delay::DelayUs;
    use embedded_hal::spi::ErrorType;
    use embedded_hal::spi::{Operation as SpiOperation, SpiBus, SpiDevice};
    use spidev::SpidevTransfer;
    use std::io::{Read, Write};

    impl ErrorType for Spidev {
        type Error = SPIError;
    }

    impl SpiBus<u8> for Spidev {
        fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            self.0.read_exact(words).map_err(|err| SPIError { err })
        }

        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            self.0.write_all(words).map_err(|err| SPIError { err })
        }

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

        fn flush(&mut self) -> Result<(), Self::Error> {
            self.0.flush().map_err(|err| SPIError { err })
        }
    }

    impl SpiDevice for Spidev {
        fn transaction(
            &mut self,
            operations: &mut [SpiOperation<'_, u8>],
        ) -> Result<(), Self::Error> {
            let mut spidev_ops: Vec<Operation> = vec![];
            let mut spidev_transfers: Vec<SpidevTransfer> = vec![];

            for op in operations {
                match op {
                    SpiOperation::Read(buf) => spidev_transfers.push(SpidevTransfer::read(buf)),
                    SpiOperation::Write(buf) => spidev_transfers.push(SpidevTransfer::write(buf)),
                    SpiOperation::Transfer(read, write) => {
                        spidev_transfers.push(SpidevTransfer::read_write(write, read))
                    }
                    SpiOperation::TransferInPlace(buf) => {
                        let tx = unsafe {
                            let p = buf.as_ptr();
                            std::slice::from_raw_parts(p, buf.len())
                        };
                        spidev_transfers.push(SpidevTransfer::read_write(tx, buf))
                    }
                    SpiOperation::DelayUs(us) => {
                        if !spidev_transfers.is_empty() {
                            let mut transfers: Vec<SpidevTransfer> = vec![];
                            std::mem::swap(&mut transfers, &mut spidev_transfers);
                            spidev_ops.push(Operation::Transfers(transfers));
                        }
                        spidev_ops.push(Operation::Delay(us.to_owned()));
                    }
                }
            }

            if !spidev_transfers.is_empty() {
                spidev_ops.push(Operation::Transfers(spidev_transfers));
            }

            for op in spidev_ops {
                match op {
                    Operation::Transfers(mut transfers) => {
                        self.0
                            .transfer_multiple(&mut transfers)
                            .map_err(|err| SPIError { err })?;
                        self.flush()?;
                    }
                    Operation::Delay(us) => {
                        self.1.delay_us(us);
                    }
                }
            }

            Ok(())
        }
    }
}

enum Operation<'a, 'b> {
    Transfers(Vec<SpidevTransfer<'a, 'b>>),
    Delay(u32),
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
    #[allow(clippy::match_single_binding)]
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        use embedded_hal::spi::ErrorKind;
        // TODO: match any errors here if we can find any that are relevant
        ErrorKind::Other
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
