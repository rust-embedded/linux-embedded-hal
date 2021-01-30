//! Implementation of [`embedded-hal`] traits for Linux devices
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal
//!
//! # Drivers
//!
//! This crate lets you use a bunch of platform agnostic drivers that are based on the
//! `embedded-hal` traits. You can find them on crates.io by [searching for the embedded-hal
//! keyword][0].
//!
//! [0]: https://crates.io/keywords/embedded-hal

#![deny(missing_docs)]

use cast;
pub use i2cdev;
pub use nb;
pub use serial_core;
pub use serial_unix;
pub use spidev;

#[cfg(feature = "gpio_sysfs")]
pub use sysfs_gpio;

#[cfg(feature = "gpio_cdev")]
pub use gpio_cdev;

use core::convert::Infallible;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{ops, thread};

use cast::{u32, u64};
use embedded_hal::blocking::i2c::Operation as I2cOperation;
use i2cdev::core::{I2CDevice, I2CMessage, I2CTransfer};
use i2cdev::linux::LinuxI2CMessage;
use spidev::SpidevTransfer;

mod serial;
mod timer;

pub use serial::Serial;
pub use timer::SysTimer;

#[cfg(feature = "gpio_sysfs")]
/// Sysfs Pin wrapper module
mod sysfs_pin;

#[cfg(feature = "gpio_cdev")]
/// Cdev Pin wrapper module
mod cdev_pin;

#[cfg(feature = "gpio_cdev")]
/// Cdev pin re-export
pub use cdev_pin::CdevPin;
#[cfg(feature = "gpio_sysfs")]
/// Sysfs pin re-export
pub use sysfs_pin::SysfsPin;

/// Empty struct that provides delay functionality on top of `thread::sleep`
pub struct Delay;

impl embedded_hal::blocking::delay::DelayUs<u8> for Delay {
    type Error = Infallible;

    fn try_delay_us(&mut self, n: u8) -> Result<(), Self::Error> {
        thread::sleep(Duration::new(0, u32(n) * 1000));
        Ok(())
    }
}

impl embedded_hal::blocking::delay::DelayUs<u16> for Delay {
    type Error = Infallible;

    fn try_delay_us(&mut self, n: u16) -> Result<(), Self::Error> {
        thread::sleep(Duration::new(0, u32(n) * 1000));
        Ok(())
    }
}

impl embedded_hal::blocking::delay::DelayUs<u32> for Delay {
    type Error = Infallible;

    fn try_delay_us(&mut self, n: u32) -> Result<(), Self::Error> {
        let secs = n / 1_000_000;
        let nsecs = (n % 1_000_000) * 1_000;

        thread::sleep(Duration::new(u64(secs), nsecs));
        Ok(())
    }
}

impl embedded_hal::blocking::delay::DelayUs<u64> for Delay {
    type Error = Infallible;

    fn try_delay_us(&mut self, n: u64) -> Result<(), Self::Error> {
        let secs = n / 1_000_000;
        let nsecs = ((n % 1_000_000) * 1_000) as u32;

        thread::sleep(Duration::new(secs, nsecs));
        Ok(())
    }
}

impl embedded_hal::blocking::delay::DelayMs<u8> for Delay {
    type Error = Infallible;

    fn try_delay_ms(&mut self, n: u8) -> Result<(), Self::Error> {
        thread::sleep(Duration::from_millis(u64(n)));
        Ok(())
    }
}

impl embedded_hal::blocking::delay::DelayMs<u16> for Delay {
    type Error = Infallible;

    fn try_delay_ms(&mut self, n: u16) -> Result<(), Self::Error> {
        thread::sleep(Duration::from_millis(u64(n)));
        Ok(())
    }
}

impl embedded_hal::blocking::delay::DelayMs<u32> for Delay {
    type Error = Infallible;

    fn try_delay_ms(&mut self, n: u32) -> Result<(), Self::Error> {
        thread::sleep(Duration::from_millis(u64(n)));
        Ok(())
    }
}

impl embedded_hal::blocking::delay::DelayMs<u64> for Delay {
    type Error = Infallible;

    fn try_delay_ms(&mut self, n: u64) -> Result<(), Self::Error> {
        thread::sleep(Duration::from_millis(n));
        Ok(())
    }
}

/// Newtype around [`i2cdev::linux::LinuxI2CDevice`] that implements the `embedded-hal` traits
///
/// [`i2cdev::linux::LinuxI2CDevice`]: https://docs.rs/i2cdev/0.3.1/i2cdev/linux/struct.LinuxI2CDevice.html
pub struct I2cdev {
    inner: i2cdev::linux::LinuxI2CDevice,
    path: PathBuf,
    address: Option<u8>,
}

impl I2cdev {
    /// See [`i2cdev::linux::LinuxI2CDevice::new`][0] for details.
    ///
    /// [0]: https://docs.rs/i2cdev/0.3.1/i2cdev/linux/struct.LinuxI2CDevice.html#method.new
    pub fn new<P>(path: P) -> Result<Self, i2cdev::linux::LinuxI2CError>
    where
        P: AsRef<Path>,
    {
        let dev = I2cdev {
            path: path.as_ref().to_path_buf(),
            inner: i2cdev::linux::LinuxI2CDevice::new(path, 0)?,
            address: None,
        };
        Ok(dev)
    }

    fn set_address(&mut self, address: u8) -> Result<(), i2cdev::linux::LinuxI2CError> {
        if self.address != Some(address) {
            self.inner = i2cdev::linux::LinuxI2CDevice::new(&self.path, u16::from(address))?;
            self.address = Some(address);
        }
        Ok(())
    }
}

impl embedded_hal::blocking::i2c::Read for I2cdev {
    type Error = i2cdev::linux::LinuxI2CError;

    fn try_read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.set_address(address)?;
        self.inner.read(buffer)
    }
}

impl embedded_hal::blocking::i2c::Write for I2cdev {
    type Error = i2cdev::linux::LinuxI2CError;

    fn try_write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.set_address(address)?;
        self.inner.write(bytes)
    }
}

impl embedded_hal::blocking::i2c::WriteRead for I2cdev {
    type Error = i2cdev::linux::LinuxI2CError;

    fn try_write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.set_address(address)?;
        let mut messages = [LinuxI2CMessage::write(bytes), LinuxI2CMessage::read(buffer)];
        self.inner.transfer(&mut messages).map(drop)
    }
}

impl embedded_hal::blocking::i2c::Transactional for I2cdev {
    type Error = i2cdev::linux::LinuxI2CError;

    fn try_exec(
        &mut self,
        address: u8,
        operations: &mut [I2cOperation],
    ) -> Result<(), Self::Error> {
        // Map operations from generic to linux objects
        let mut messages: Vec<_> = operations
            .as_mut()
            .iter_mut()
            .map(|a| match a {
                I2cOperation::Write(w) => LinuxI2CMessage::write(w),
                I2cOperation::Read(r) => LinuxI2CMessage::read(r),
            })
            .collect();

        self.set_address(address)?;
        self.inner.transfer(&mut messages).map(drop)
    }
}

impl ops::Deref for I2cdev {
    type Target = i2cdev::linux::LinuxI2CDevice;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ops::DerefMut for I2cdev {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Newtype around [`spidev::Spidev`] that implements the `embedded-hal` traits
///
/// [`spidev::Spidev`]: https://docs.rs/spidev/0.4.0/spidev/struct.Spidev.html
pub struct Spidev(pub spidev::Spidev);

impl Spidev {
    /// See [`spidev::Spidev::open`][0] for details.
    ///
    /// [0]: https://docs.rs/spidev/0.4.0/spidev/struct.Spidev.html#method.open
    pub fn open<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        spidev::Spidev::open(path).map(Spidev)
    }
}

impl embedded_hal::blocking::spi::Transfer<u8> for Spidev {
    type Error = io::Error;

    fn try_transfer<'b>(&mut self, buffer: &'b mut [u8]) -> io::Result<&'b [u8]> {
        let tx = buffer.to_owned();
        self.0
            .transfer(&mut SpidevTransfer::read_write(&tx, buffer))?;
        Ok(buffer)
    }
}

impl embedded_hal::blocking::spi::Write<u8> for Spidev {
    type Error = io::Error;

    fn try_write(&mut self, buffer: &[u8]) -> io::Result<()> {
        self.0.write_all(buffer)
    }
}

pub use embedded_hal::blocking::spi::Operation as SpiOperation;

/// Transactional implementation batches SPI operations into a single transaction
impl embedded_hal::blocking::spi::Transactional<u8> for Spidev {
    type Error = io::Error;

    fn try_exec<'a>(&mut self, operations: &mut [SpiOperation<'a, u8>]) -> Result<(), Self::Error> {
        // Map types from generic to linux objects
        let mut messages: Vec<_> = operations
            .iter_mut()
            .map(|a| {
                match a {
                    SpiOperation::Write(w) => SpidevTransfer::write(w),
                    SpiOperation::Transfer(r) => {
                        // Clone read to write pointer
                        // SPIdev is okay with having w == r but this is tricky to achieve in safe rust
                        let w = unsafe {
                            let p = r.as_ptr();
                            std::slice::from_raw_parts(p, r.len())
                        };

                        SpidevTransfer::read_write(w, r)
                    }
                }
            })
            .collect();

        // Execute transfer
        self.0.transfer_multiple(&mut messages)
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
