//! Implementation of [`embedded-hal`] digital input/output traits using a Linux CDev pin
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

/// Newtype around [`gpio_cdev::LineHandle`] that implements the `embedded-hal` traits
///
/// [`gpio_cdev::LineHandle`]: https://docs.rs/gpio-cdev/0.5.0/gpio_cdev/struct.LineHandle.html
pub struct CdevPin(pub gpio_cdev::LineHandle, bool);

impl CdevPin {
    /// See [`gpio_cdev::Line::request`][0] for details.
    ///
    /// [0]: https://docs.rs/gpio-cdev/0.5.0/gpio_cdev/struct.Line.html#method.request
    pub fn new(handle: gpio_cdev::LineHandle) -> Result<Self, gpio_cdev::errors::Error> {
        let info = handle.line().info()?;
        Ok(CdevPin(handle, info.is_active_low()))
    }
}

impl embedded_hal::digital::blocking::OutputPin for CdevPin {
    type Error = gpio_cdev::errors::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        if self.1 {
            self.0.set_value(1)
        } else {
            self.0.set_value(0)
        }
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        if self.1 {
            self.0.set_value(0)
        } else {
            self.0.set_value(1)
        }
    }
}

impl embedded_hal::digital::blocking::InputPin for CdevPin {
    type Error = gpio_cdev::errors::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        if !self.1 {
            self.0.get_value().map(|val| val != 0)
        } else {
            self.0.get_value().map(|val| val == 0)
        }
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.is_high().map(|val| !val)
    }
}

impl core::ops::Deref for CdevPin {
    type Target = gpio_cdev::LineHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for CdevPin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
