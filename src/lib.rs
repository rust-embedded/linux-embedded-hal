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

pub use i2cdev;
pub use nb;
pub use serial_core;
pub use serial_unix;
pub use spidev;

#[cfg(feature = "gpio_sysfs")]
pub use sysfs_gpio;

#[cfg(feature = "gpio_cdev")]
pub use gpio_cdev;
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

mod delay;
mod i2c;
mod serial;
mod spi;
mod timer;

pub use crate::delay::Delay;
pub use crate::i2c::{I2CError, I2cdev};
pub use crate::serial::{Serial, SerialError};
pub use crate::spi::{SPIError, Spidev};
pub use crate::timer::{CountDown, Periodic, SysTimer};
