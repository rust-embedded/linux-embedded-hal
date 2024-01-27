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

#[cfg(feature = "i2c")]
pub use i2cdev;
pub use nb;
pub use serialport;
#[cfg(feature = "spi")]
pub use spidev;

#[cfg(feature = "gpio_sysfs")]
pub use sysfs_gpio;

#[cfg(feature = "gpio_sysfs")]
/// Sysfs Pin wrapper module
mod sysfs_pin;

#[cfg(feature = "gpio_cdev")]
/// Cdev Pin wrapper module
mod cdev_pin;

#[cfg(feature = "gpio_cdev")]
/// Cdev pin re-export
pub use cdev_pin::{CdevPin, CdevPinError};

#[cfg(feature = "gpio_sysfs")]
/// Sysfs pin re-export
pub use sysfs_pin::{SysfsPin, SysfsPinError};

mod delay;
#[cfg(feature = "i2c")]
mod i2c;
mod serial;
#[cfg(feature = "spi")]
mod spi;
mod timer;

pub use crate::delay::Delay;
#[cfg(feature = "i2c")]
pub use crate::i2c::{I2CError, I2cdev};
pub use crate::serial::{Serial, SerialError};
#[cfg(feature = "spi")]
pub use crate::spi::{SPIError, SpidevBus, SpidevDevice};
pub use crate::timer::{CountDown, Periodic, SysTimer};
