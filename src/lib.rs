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

extern crate cast;
extern crate embedded_hal as hal;
pub extern crate i2cdev;
pub extern crate spidev;
pub extern crate sysfs_gpio;
pub extern crate serial_unix;
pub extern crate serial_core;
pub extern crate nb;

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{ops, thread};

use cast::{u32, u64};
use i2cdev::core::{I2CDevice, I2CMessage, I2CTransfer};
use i2cdev::linux::LinuxI2CMessage;
use spidev::SpidevTransfer;

mod serial;

pub use serial::Serial;

/// Empty struct that provides delay functionality on top of `thread::sleep`
pub struct Delay;

impl hal::blocking::delay::DelayUs<u8> for Delay {
    fn delay_us(&mut self, n: u8) {
        thread::sleep(Duration::new(0, u32(n) * 1000))
    }
}

impl hal::blocking::delay::DelayUs<u16> for Delay {
    fn delay_us(&mut self, n: u16) {
        thread::sleep(Duration::new(0, u32(n) * 1000))
    }
}

impl hal::blocking::delay::DelayUs<u32> for Delay {
    fn delay_us(&mut self, n: u32) {
        let secs = n / 1_000_000;
        let nsecs = (n % 1_000_000) * 1_000;

        thread::sleep(Duration::new(u64(secs), nsecs))
    }
}

impl hal::blocking::delay::DelayUs<u64> for Delay {
    fn delay_us(&mut self, n: u64) {
        let secs = n / 1_000_000;
        let nsecs = ((n % 1_000_000) * 1_000) as u32;

        thread::sleep(Duration::new(secs, nsecs))
    }
}

impl hal::blocking::delay::DelayMs<u8> for Delay {
    fn delay_ms(&mut self, n: u8) {
        thread::sleep(Duration::from_millis(u64(n)))
    }
}

impl hal::blocking::delay::DelayMs<u16> for Delay {
    fn delay_ms(&mut self, n: u16) {
        thread::sleep(Duration::from_millis(u64(n)))
    }
}

impl hal::blocking::delay::DelayMs<u32> for Delay {
    fn delay_ms(&mut self, n: u32) {
        thread::sleep(Duration::from_millis(u64(n)))
    }
}

impl hal::blocking::delay::DelayMs<u64> for Delay {
    fn delay_ms(&mut self, n: u64) {
        thread::sleep(Duration::from_millis(n))
    }
}

/// Newtype around [`sysfs_gpio::Pin`] that implements the `embedded-hal` traits
///
/// [`sysfs_gpio::Pin`]: https://docs.rs/sysfs_gpio/0.5.1/sysfs_gpio/struct.Pin.html
pub struct Pin(pub sysfs_gpio::Pin);

impl Pin {
    /// See [`sysfs_gpio::Pin::new`][0] for details.
    ///
    /// [0]: https://docs.rs/sysfs_gpio/0.5.1/sysfs_gpio/struct.Pin.html#method.new
    pub fn new(pin_num: u64) -> Pin {
        Pin(sysfs_gpio::Pin::new(pin_num))
    }

    /// See [`sysfs_gpio::Pin::from_path`][0] for details.
    ///
    /// [0]: https://docs.rs/sysfs_gpio/0.5.1/sysfs_gpio/struct.Pin.html#method.from_path
    pub fn from_path<P>(path: P) -> sysfs_gpio::Result<Pin>
    where
        P: AsRef<Path>,
    {
        sysfs_gpio::Pin::from_path(path).map(Pin)
    }
}

impl hal::digital::v2::OutputPin for Pin {
    type Error = sysfs_gpio::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(0)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(1)
    }
}

impl hal::digital::v2::InputPin for Pin {
    type Error = sysfs_gpio::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        if !self.0.get_active_low()? {
            self.0.get_value().map(|val| val != 0)
        } else {
            self.0.get_value().map(|val| val == 0)
        }
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.is_high().map(|val| !val)
    }
}

impl ops::Deref for Pin {
    type Target = sysfs_gpio::Pin;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Pin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

impl hal::blocking::i2c::Read for I2cdev {
    type Error = i2cdev::linux::LinuxI2CError;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.set_address(address)?;
        self.inner.read(buffer)
    }
}

impl hal::blocking::i2c::Write for I2cdev {
    type Error = i2cdev::linux::LinuxI2CError;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.set_address(address)?;
        self.inner.write(bytes)
    }
}

impl hal::blocking::i2c::WriteRead for I2cdev {
    type Error = i2cdev::linux::LinuxI2CError;

    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.set_address(address)?;
        let mut messages = [
            LinuxI2CMessage::write(bytes),
            LinuxI2CMessage::read(buffer),
        ];
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
/// [`spidev::Spidev`]: https://docs.rs/spidev/0.3.0/spidev/struct.Spidev.html
pub struct Spidev(pub spidev::Spidev);

impl Spidev {
    /// See [`spidev::Spidev::open`][0] for details.
    ///
    /// [0]: https://docs.rs/spidev/0.3.0/spidev/struct.Spidev.html#method.open
    pub fn open<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        spidev::Spidev::open(path).map(Spidev)
    }
}

impl hal::blocking::spi::Transfer<u8> for Spidev {
    type Error = io::Error;

    fn transfer<'b>(&mut self, buffer: &'b mut [u8]) -> io::Result<&'b [u8]> {
        let tx = buffer.to_owned();
        self.0
            .transfer(&mut SpidevTransfer::read_write(&tx, buffer))?;
        Ok(buffer)
    }
}

impl hal::blocking::spi::Write<u8> for Spidev {
    type Error = io::Error;

    fn write(&mut self, buffer: &[u8]) -> io::Result<()> {
        self.0.write_all(buffer)
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
