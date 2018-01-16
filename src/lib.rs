//! Implementation of [`embedded-hal`] traits for Linux devices
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

#![deny(missing_docs)]
#![deny(warnings)]

extern crate embedded_hal as hal;
pub extern crate spidev;
pub extern crate sysfs_gpio;

use std::io::{self, Write};
use std::ops;
use std::path::Path;

use spidev::SpidevTransfer;

/// Newtype around [`sysfs_gpio::Pin`] that implements the `embedded-hal` traits
///
/// [`sysfs_gpio::Pin`]: https://docs.rs/sysfs_gpio/0.5.1/sysfs_gpio/struct.Pin.html
pub struct Pin(pub sysfs_gpio::Pin);

impl Pin {
    /// See [`sysfs_gpio::Pin::new`][0]
    ///
    /// [0]: https://docs.rs/sysfs_gpio/0.5.1/sysfs_gpio/struct.Pin.html#method.new
    pub fn new(pin_num: u64) -> Pin {
        Pin(sysfs_gpio::Pin::new(pin_num))
    }

    /// See [`sysfs_gpio::Pin::from_path`][0]
    ///
    /// [0]: https://docs.rs/sysfs_gpio/0.5.1/sysfs_gpio/struct.Pin.html#method.from_path
    pub fn from_path<P>(path: P) -> sysfs_gpio::Result<Pin>
    where
        P: AsRef<Path>,
    {
        sysfs_gpio::Pin::from_path(path).map(Pin)
    }
}

impl hal::digital::OutputPin for Pin {
    fn is_low(&self) -> bool {
        unimplemented!()
    }

    fn is_high(&self) -> bool {
        unimplemented!()
    }

    fn set_low(&mut self) {
        self.0.set_value(0).unwrap()
    }

    fn set_high(&mut self) {
        self.0.set_value(1).unwrap()
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

/// Newtype around [`spidev::Spidev`] that implements the `embedded-hal` traits
///
/// [`spidev::Spidev`]: https://docs.rs/spidev/0.3.0/spidev/struct.Spidev.html
pub struct Spidev(pub spidev::Spidev);

impl Spidev {
    /// See [`spidev::Spidev::open`][0]
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
