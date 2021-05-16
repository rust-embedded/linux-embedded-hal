//! Linux CDev pin type

/// Newtype around [`gpio_cdev::LineHandle`] that implements the `embedded-hal` traits
///
/// [`gpio_cdev::LineHandle`]: https://docs.rs/gpio-cdev/0.2.0/gpio_cdev/struct.LineHandle.html
pub struct CdevPin(pub gpio_cdev::LineHandle, gpio_cdev::LineInfo);

impl CdevPin {
    /// See [`gpio_cdev::Line::request`][0] for details.
    ///
    /// [0]: https://docs.rs/gpio-cdev/0.2.0/gpio_cdev/struct.Line.html#method.request
    pub fn new(handle: gpio_cdev::LineHandle) -> Result<Self, gpio_cdev::errors::Error> {
        let info = handle.line().info()?;
        Ok(CdevPin(handle, info))
    }

    fn get_input_flags(&self) -> gpio_cdev::LineRequestFlags {
        let mut flags = gpio_cdev::LineRequestFlags::INPUT;
        if self.1.is_active_low() {
            flags.insert(gpio_cdev::LineRequestFlags::ACTIVE_LOW);
        }
        return flags;
    }

    fn get_output_flags(&self) -> gpio_cdev::LineRequestFlags {
        let mut flags = gpio_cdev::LineRequestFlags::OUTPUT;
        if self.1.is_open_drain() {
            flags.insert(gpio_cdev::LineRequestFlags::OPEN_DRAIN);
        } else if self.1.is_open_source() {
            flags.insert(gpio_cdev::LineRequestFlags::OPEN_SOURCE);
        }
        return flags;
    }
}

impl embedded_hal::digital::OutputPin for CdevPin {
    type Error = gpio_cdev::errors::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(0)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(1)
    }
}

impl embedded_hal::digital::InputPin for CdevPin {
    type Error = gpio_cdev::errors::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        if !self.1.is_active_low() {
            self.0.get_value().map(|val| val != 0)
        } else {
            self.0.get_value().map(|val| val == 0)
        }
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.is_high().map(|val| !val)
    }
}

impl embedded_hal::digital::IoPin<CdevPin, CdevPin> for CdevPin {
    type Error = gpio_cdev::errors::Error;

    fn into_input_pin(self) -> Result<CdevPin, Self::Error> {
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

    fn into_output_pin(
        self,
        state: embedded_hal::digital::PinState,
    ) -> Result<CdevPin, Self::Error> {
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
            match state {
                embedded_hal::digital::PinState::High => 1,
                embedded_hal::digital::PinState::Low => 0,
            },
            &consumer,
        )?)
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
