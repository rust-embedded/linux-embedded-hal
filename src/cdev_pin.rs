//! Implementation of [`embedded-hal`] digital input/output traits using a Linux CDev pin
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

/// Newtype around [`gpio_cdev::LineHandle`] that implements the `embedded-hal` traits
///
/// [`gpio_cdev::LineHandle`]: https://docs.rs/gpio-cdev/0.5.0/gpio_cdev/struct.LineHandle.html
pub struct CdevPin(pub gpio_cdev::LineHandle, gpio_cdev::LineInfo);

impl CdevPin {
    /// See [`gpio_cdev::Line::request`][0] for details.
    ///
    /// [0]: https://docs.rs/gpio-cdev/0.5.0/gpio_cdev/struct.Line.html#method.request
    pub fn new(handle: gpio_cdev::LineHandle) -> Result<Self, gpio_cdev::errors::Error> {
        let info = handle.line().info()?;
        Ok(CdevPin(handle, info))
    }

    fn get_input_flags(&self) -> gpio_cdev::LineRequestFlags {
        if self.1.is_active_low() {
            return gpio_cdev::LineRequestFlags::INPUT | gpio_cdev::LineRequestFlags::ACTIVE_LOW;
        }
        gpio_cdev::LineRequestFlags::INPUT
    }

    fn get_output_flags(&self) -> gpio_cdev::LineRequestFlags {
        let mut flags = gpio_cdev::LineRequestFlags::OUTPUT;
        if self.1.is_active_low() {
            flags.insert(gpio_cdev::LineRequestFlags::ACTIVE_LOW);
        }
        if self.1.is_open_drain() {
            flags.insert(gpio_cdev::LineRequestFlags::OPEN_DRAIN);
        } else if self.1.is_open_source() {
            flags.insert(gpio_cdev::LineRequestFlags::OPEN_SOURCE);
        }
        flags
    }

    /// Set this pin to input mode
    pub fn into_input_pin(self) -> Result<CdevPin, gpio_cdev::errors::Error> {
        if self.1.direction() == gpio_cdev::LineDirection::In {
            return Ok(self);
        }
        let line = self.0.line().clone();
        let input_flags = self.get_input_flags();
        let consumer = self.1.consumer().unwrap_or("").to_owned();

        // Drop self to free the line before re-requesting it in a new mode.
        std::mem::drop(self);

        CdevPin::new(line.request(input_flags, 0, &consumer)?)
    }

    /// Set this pin to output mode
    pub fn into_output_pin(
        self,
        state: embedded_hal::digital::PinState,
    ) -> Result<CdevPin, gpio_cdev::errors::Error> {
        if self.1.direction() == gpio_cdev::LineDirection::Out {
            return Ok(self);
        }

        let line = self.0.line().clone();
        let output_flags = self.get_output_flags();
        let consumer = self.1.consumer().unwrap_or("").to_owned();

        // Drop self to free the line before re-requesting it in a new mode.
        std::mem::drop(self);

        CdevPin::new(line.request(
            output_flags,
            state_to_value(
                state,
                output_flags.intersects(gpio_cdev::LineRequestFlags::ACTIVE_LOW),
            ),
            &consumer,
        )?)
    }
}

/// Converts a pin state to the gpio_cdev compatible numeric value, accounting
/// for the active_low condition.
fn state_to_value(state: embedded_hal::digital::PinState, is_active_low: bool) -> u8 {
    if is_active_low {
        match state {
            embedded_hal::digital::PinState::High => 0,
            embedded_hal::digital::PinState::Low => 1,
        }
    } else {
        match state {
            embedded_hal::digital::PinState::High => 1,
            embedded_hal::digital::PinState::Low => 0,
        }
    }
}

impl embedded_hal::digital::ErrorType for CdevPin {
    type Error = PinError;
}

impl embedded_hal::digital::OutputPin for CdevPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(state_to_value(
            embedded_hal::digital::PinState::Low,
            self.1.is_active_low(),
        ))?;
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(state_to_value(
            embedded_hal::digital::PinState::High,
            self.1.is_active_low(),
        ))?;
        Ok(())
    }
}

impl embedded_hal::digital::InputPin for CdevPin {
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.0.get_value().map(|val| {
            val == state_to_value(
                embedded_hal::digital::PinState::High,
                self.1.is_active_low(),
            )
        })?)
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

/// Error type wrapping [gpio_cdev::errors::Error](gpio_cdev::errors::Error) to implement [embedded_hal::digital::ErrorKind]
#[derive(Debug)]
pub struct PinError {
    err: gpio_cdev::errors::Error,
}

impl PinError {
    /// Fetch inner (concrete) [`gpio_cdev::errors::Error`]
    pub fn inner(&self) -> &gpio_cdev::errors::Error {
        &self.err
    }
}

impl From<gpio_cdev::errors::Error> for PinError {
    fn from(err: gpio_cdev::errors::Error) -> Self {
        Self { err }
    }
}

impl embedded_hal::digital::Error for PinError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        use embedded_hal::digital::ErrorKind;
        ErrorKind::Other
    }
}
