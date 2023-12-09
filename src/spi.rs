//! Implementation of [`embedded-hal`] SPI traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal
//!

use std::cmp::Ordering;
use std::fmt;
use std::io;
use std::ops;
use std::path::Path;

/// Spidev wrapper providing the embedded-hal [`SpiDevice`] trait.
///
/// Use this struct when you want a single spidev device, using a Linux-managed CS (chip-select) pin,
/// which is already defined in your device tree. Linux will handle sharing the bus
/// between different SPI devices, even between different processes.
///
/// You get an object that implements [`SpiDevice`], which is what most drivers require,
/// but does not implement [`SpiBus`]. In some rare cases, you may require [`SpiBus`]
/// instead; for that refer to [`SpidevBus`] below. You may also want to use [`SpiBus`]
/// if you want to handle all the CS pins yourself using GPIO.
///
/// This struct wraps a [`spidev::Spidev`] struct, so it can be constructed directly
/// and the inner struct accessed if needed, for example to (re)configure the SPI settings.
///
/// Note that [delay operations] on this device are capped to 65535 microseconds.
///
/// [`SpiDevice`]: embedded_hal::spi::SpiDevice
/// [`SpiBus`]: embedded_hal::spi::SpiBus
/// [`spidev::Spidev`]: spidev::Spidev
/// [delay operations]: embedded_hal::spi::Operation::DelayUs
pub struct SpidevDevice(pub spidev::Spidev);

/// Spidev wrapper providing the embedded-hal [`SpiBus`] trait.
///
/// Use this struct when you require direct access to the underlying SPI bus, for
/// example when you want to use GPIOs as software-controlled CS (chip-select) pins to share the
/// bus with multiple devices, or because a driver requires the entire bus (for
/// example to drive smart LEDs).
///
/// Do not use this struct if you're accessing SPI devices that already appear in your
/// device tree; you will not be able to drive CS pins that are already used by `spidev`
/// as GPIOs. Instead use [`SpidevDevice`].
///
/// This struct must still be created from a [`spidev::Spidev`] device, but there are two
/// important notes:
///
/// 1. The CS pin associated with this `spidev` device will be driven whenever any device accesses
///    this bus, so it should be an unconnected or unused pin.
/// 2. No other `spidev` device on the same bus may be used as long as this `SpidevBus` exists,
///    as Linux will _not_ do anything to ensure this bus has exclusive access.
///
/// It is recommended to use a dummy `spidev` device associated with an unused CS pin, and then use
/// regular GPIOs as CS pins if required. If you are planning to share this bus using GPIOs, the
/// [`embedded-hal-bus`] crate may be of interest.
///
/// If necessary, you can [configure] the underlying [`spidev::Spidev`] instance with the
/// [`SPI_NO_CS`] flag set to prevent any CS pin activity.
///
/// [`SpiDevice`]: embedded_hal::spi::SpiDevice
/// [`SpiBus`]: embedded_hal::spi::SpiBus
/// [`embedded-hal-bus`]: https://docs.rs/embedded-hal-bus/
/// [`spidev::Spidev`]: spidev::Spidev
/// [delay operations]: embedded_hal::spi::Operation::DelayUs
/// [configure]: spidev::Spidev::configure
/// [`SPI_NO_CS`]: spidev::SpiModeFlags::SPI_NO_CS
pub struct SpidevBus(pub spidev::Spidev);

impl SpidevDevice {
    /// See [`spidev::Spidev::open`] for details.
    ///
    /// The provided `path` is for the specific device you wish to access.
    /// Access to the bus is shared with other devices via the Linux kernel.
    pub fn open<P>(path: P) -> Result<Self, SPIError>
    where
        P: AsRef<Path>,
    {
        spidev::Spidev::open(path)
            .map(SpidevDevice)
            .map_err(|e| e.into())
    }
}

impl SpidevBus {
    /// See [`spidev::Spidev::open`] for details.
    ///
    /// The provided `path` must be the _only_ device in use on its bus,
    /// and note its own CS pin will be asserted for all device access,
    /// so the path should be to a dummy device used only to access
    /// the underlying bus.
    pub fn open<P>(path: P) -> Result<Self, SPIError>
    where
        P: AsRef<Path>,
    {
        spidev::Spidev::open(path)
            .map(SpidevBus)
            .map_err(|e| e.into())
    }
}

impl ops::Deref for SpidevDevice {
    type Target = spidev::Spidev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for SpidevDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ops::Deref for SpidevBus {
    type Target = spidev::Spidev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for SpidevBus {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

mod embedded_hal_impl {
    use super::*;
    use embedded_hal::spi::ErrorType;
    use embedded_hal::spi::{Operation as SpiOperation, SpiBus, SpiDevice};
    use spidev::SpidevTransfer;
    use std::convert::TryInto;
    use std::io::{Read, Write};

    impl ErrorType for SpidevDevice {
        type Error = SPIError;
    }

    impl ErrorType for SpidevBus {
        type Error = SPIError;
    }

    impl SpiBus<u8> for SpidevBus {
        fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            self.0.read_exact(words).map_err(|err| SPIError { err })
        }

        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            self.0.write_all(words).map_err(|err| SPIError { err })
        }

        fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
            let read_len = read.len();
            match read_len.cmp(&write.len()) {
                Ordering::Less => self.0.transfer_multiple(&mut [
                    SpidevTransfer::read_write(&write[..read_len], read),
                    SpidevTransfer::write(&write[read_len..]),
                ]),
                Ordering::Equal => self
                    .0
                    .transfer(&mut SpidevTransfer::read_write(write, read)),
                Ordering::Greater => {
                    let (read1, read2) = read.split_at_mut(write.len());
                    self.0.transfer_multiple(&mut [
                        SpidevTransfer::read_write(write, read1),
                        SpidevTransfer::read(read2),
                    ])
                }
            }
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

    impl SpiDevice for SpidevDevice {
        /// Perform a transaction against the device. [Read more][transaction]
        ///
        /// [Delay operations][delay] are capped to 65535 microseconds.
        ///
        /// [transaction]: SpiDevice::transaction
        /// [delay]: SpiOperation::DelayUs
        fn transaction(
            &mut self,
            operations: &mut [SpiOperation<'_, u8>],
        ) -> Result<(), Self::Error> {
            let mut transfers = Vec::with_capacity(operations.len());
            for op in operations {
                match op {
                    SpiOperation::Read(buf) => transfers.push(SpidevTransfer::read(buf)),
                    SpiOperation::Write(buf) => transfers.push(SpidevTransfer::write(buf)),
                    SpiOperation::Transfer(read, write) => match read.len().cmp(&write.len()) {
                        Ordering::Less => {
                            let n = read.len();
                            transfers.push(SpidevTransfer::read_write(&write[..n], read));
                            transfers.push(SpidevTransfer::write(&write[n..]));
                        }
                        Ordering::Equal => transfers.push(SpidevTransfer::read_write(write, read)),
                        Ordering::Greater => {
                            let (read1, read2) = read.split_at_mut(write.len());
                            transfers.push(SpidevTransfer::read_write(write, read1));
                            transfers.push(SpidevTransfer::read(read2));
                        }
                    },
                    SpiOperation::TransferInPlace(buf) => {
                        let tx = unsafe {
                            let p = buf.as_ptr();
                            std::slice::from_raw_parts(p, buf.len())
                        };
                        transfers.push(SpidevTransfer::read_write(tx, buf));
                    }
                    SpiOperation::DelayNs(ns) => {
                        let us = {
                            if *ns == 0 {
                                0
                            } else {
                                let us = *ns / 1000;
                                if us == 0 {
                                    1
                                } else {
                                    (us).try_into().unwrap_or(u16::MAX)
                                }
                            }
                        };
                        transfers.push(SpidevTransfer::delay(us));
                    }
                }
            }
            self.0
                .transfer_multiple(&mut transfers)
                .map_err(|err| SPIError { err })?;
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
