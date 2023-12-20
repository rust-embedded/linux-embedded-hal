//! Implementation of [`embedded-hal`] digital input/output traits using a Linux Sysfs pin
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use std::fmt;
use std::path::Path;

/// Newtype around [`sysfs_gpio::Pin`] that implements the `embedded-hal` traits
///
/// [`sysfs_gpio::Pin`]: https://docs.rs/sysfs_gpio/0.6.0/sysfs_gpio/struct.Pin.html
pub struct SysfsPin(pub sysfs_gpio::Pin);

impl SysfsPin {
    /// See [`sysfs_gpio::Pin::new`][0] for details.
    ///
    /// [0]: https://docs.rs/sysfs_gpio/0.6.0/sysfs_gpio/struct.Pin.html#method.new
    pub fn new(pin_num: u64) -> Self {
        SysfsPin(sysfs_gpio::Pin::new(pin_num))
    }

    /// See [`sysfs_gpio::Pin::from_path`][0] for details.
    ///
    /// [0]: https://docs.rs/sysfs_gpio/0.6.0/sysfs_gpio/struct.Pin.html#method.from_path
    pub fn from_path<P>(path: P) -> sysfs_gpio::Result<Self>
    where
        P: AsRef<Path>,
    {
        sysfs_gpio::Pin::from_path(path).map(SysfsPin)
    }

    /// Convert this pin to an input pin
    pub fn into_input_pin(self) -> Result<SysfsPin, sysfs_gpio::Error> {
        self.set_direction(sysfs_gpio::Direction::In)?;
        Ok(self)
    }

    /// Convert this pin to an output pin
    pub fn into_output_pin(
        self,
        state: embedded_hal::digital::PinState,
    ) -> Result<SysfsPin, sysfs_gpio::Error> {
        self.set_direction(match state {
            embedded_hal::digital::PinState::High => sysfs_gpio::Direction::High,
            embedded_hal::digital::PinState::Low => sysfs_gpio::Direction::Low,
        })?;
        Ok(self)
    }
}

/// Error type wrapping [sysfs_gpio::Error](sysfs_gpio::Error) to implement [embedded_hal::digital::Error]
#[derive(Debug)]
pub struct SysfsPinError {
    err: sysfs_gpio::Error,
}

impl SysfsPinError {
    /// Fetch inner (concrete) [`sysfs_gpio::Error`]
    pub fn inner(&self) -> &sysfs_gpio::Error {
        &self.err
    }
}

impl From<sysfs_gpio::Error> for SysfsPinError {
    fn from(err: sysfs_gpio::Error) -> Self {
        Self { err }
    }
}

impl fmt::Display for SysfsPinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl std::error::Error for SysfsPinError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.err)
    }
}

impl embedded_hal::digital::Error for SysfsPinError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        use embedded_hal::digital::ErrorKind;
        ErrorKind::Other
    }
}

impl embedded_hal::digital::ErrorType for SysfsPin {
    type Error = SysfsPinError;
}

impl embedded_hal::digital::OutputPin for SysfsPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        if self.0.get_active_low().map_err(SysfsPinError::from)? {
            self.0.set_value(1).map_err(SysfsPinError::from)
        } else {
            self.0.set_value(0).map_err(SysfsPinError::from)
        }
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        if self.0.get_active_low().map_err(SysfsPinError::from)? {
            self.0.set_value(0).map_err(SysfsPinError::from)
        } else {
            self.0.set_value(1).map_err(SysfsPinError::from)
        }
    }
}

impl embedded_hal::digital::InputPin for SysfsPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        if !self.0.get_active_low().map_err(SysfsPinError::from)? {
            self.0
                .get_value()
                .map(|val| val != 0)
                .map_err(SysfsPinError::from)
        } else {
            self.0
                .get_value()
                .map(|val| val == 0)
                .map_err(SysfsPinError::from)
        }
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|val| !val).map_err(SysfsPinError::from)
    }
}

impl core::ops::Deref for SysfsPin {
    type Target = sysfs_gpio::Pin;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for SysfsPin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
