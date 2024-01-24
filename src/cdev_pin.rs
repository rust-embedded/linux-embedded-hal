//! Implementation of [`embedded-hal`] digital input/output traits using a Linux CDev pin
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use std::fmt;

use embedded_hal::digital::InputPin;
#[cfg(feature = "async-tokio")]
use gpiocdev::{line::EdgeDetection, request::Request, tokio::AsyncRequest};

/// Newtype around [`gpio_cdev::LineHandle`] that implements the `embedded-hal` traits
///
/// [`gpio_cdev::LineHandle`]: https://docs.rs/gpio-cdev/0.5.0/gpio_cdev/struct.LineHandle.html
#[derive(Debug)]
pub struct CdevPin(pub Option<gpio_cdev::LineHandle>, gpio_cdev::LineInfo);

#[cfg(feature = "async-tokio")]
#[derive(Debug)]
struct CdevPinEdgeWaiter<'a> {
    pin: &'a mut CdevPin,
    edge: EdgeDetection,
}

#[cfg(feature = "async-tokio")]
impl<'a> CdevPinEdgeWaiter<'a> {
    pub fn new(pin: &'a mut CdevPin, edge: EdgeDetection) -> Result<Self, gpiocdev::Error> {
        Ok(Self { pin, edge })
    }

    pub async fn wait(self) -> Result<(), gpiocdev::Error> {
        let line_handle = self.pin.0.take().unwrap();
        let line_info = &self.pin.1;

        let line = line_handle.line().clone();
        let flags = line_handle.flags();
        let chip = line.chip().path().to_owned();
        let offset = line.offset();
        let consumer = line_info.consumer().unwrap_or("").to_owned();
        let edge = self.edge;

        // Close line handle.
        drop(line_handle);

        let req = Request::builder()
            .on_chip(chip)
            .with_line(offset)
            .as_is()
            .with_consumer(consumer.clone())
            .with_edge_detection(edge)
            .request()?;

        let req = AsyncRequest::new(req);
        let event = req.read_edge_event().await;
        drop(req);

        // Recreate line handle.
        self.pin.0 = Some(line.request(flags, 0, &consumer).unwrap());

        event?;

        Ok(())
    }
}

impl CdevPin {
    /// See [`gpio_cdev::Line::request`][0] for details.
    ///
    /// [0]: https://docs.rs/gpio-cdev/0.5.0/gpio_cdev/struct.Line.html#method.request
    pub fn new(handle: gpio_cdev::LineHandle) -> Result<Self, gpio_cdev::errors::Error> {
        let info = handle.line().info()?;
        Ok(CdevPin(Some(handle), info))
    }

    fn line_handle(&self) -> &gpio_cdev::LineHandle {
        self.0.as_ref().unwrap()
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
        let line = self.line_handle().line().clone();
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

        let line = self.line_handle().line().clone();
        let output_flags = self.get_output_flags();
        let consumer = self.1.consumer().unwrap_or("").to_owned();

        // Drop self to free the line before re-requesting it in a new mode.
        std::mem::drop(self);

        let is_active_low = output_flags.intersects(gpio_cdev::LineRequestFlags::ACTIVE_LOW);
        CdevPin::new(line.request(
            output_flags,
            state_to_value(state, is_active_low),
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

/// Error type wrapping [gpio_cdev::errors::Error](gpio_cdev::errors::Error) to implement [embedded_hal::digital::Error]
#[derive(Debug)]
pub struct CdevPinError {
    err: gpio_cdev::errors::Error,
}

impl CdevPinError {
    /// Fetch inner (concrete) [`gpio_cdev::errors::Error`]
    pub fn inner(&self) -> &gpio_cdev::errors::Error {
        &self.err
    }
}

impl From<gpio_cdev::errors::Error> for CdevPinError {
    fn from(err: gpio_cdev::errors::Error) -> Self {
        Self { err }
    }
}

impl fmt::Display for CdevPinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl std::error::Error for CdevPinError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.err)
    }
}

impl embedded_hal::digital::Error for CdevPinError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        use embedded_hal::digital::ErrorKind;
        ErrorKind::Other
    }
}

impl embedded_hal::digital::ErrorType for CdevPin {
    type Error = CdevPinError;
}

impl embedded_hal::digital::OutputPin for CdevPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.line_handle()
            .set_value(state_to_value(
                embedded_hal::digital::PinState::Low,
                self.1.is_active_low(),
            ))
            .map_err(CdevPinError::from)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.line_handle()
            .set_value(state_to_value(
                embedded_hal::digital::PinState::High,
                self.1.is_active_low(),
            ))
            .map_err(CdevPinError::from)
    }
}

impl InputPin for CdevPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        self.line_handle()
            .get_value()
            .map(|val| {
                val == state_to_value(
                    embedded_hal::digital::PinState::High,
                    self.1.is_active_low(),
                )
            })
            .map_err(CdevPinError::from)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|val| !val)
    }
}

impl core::ops::Deref for CdevPin {
    type Target = gpio_cdev::LineHandle;

    fn deref(&self) -> &Self::Target {
        self.line_handle()
    }
}

impl core::ops::DerefMut for CdevPin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

#[cfg(feature = "async-tokio")]
impl embedded_hal_async::digital::Wait for CdevPin {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        if self.is_high()? {
            return Ok(());
        }

        self.wait_for_rising_edge().await
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        if self.is_low()? {
            return Ok(());
        }

        self.wait_for_falling_edge().await
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        let waiter = CdevPinEdgeWaiter::new(self, EdgeDetection::RisingEdge).unwrap();
        waiter.wait().await.unwrap();
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        let waiter = CdevPinEdgeWaiter::new(self, EdgeDetection::FallingEdge).unwrap();
        waiter.wait().await.unwrap();
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        let waiter = CdevPinEdgeWaiter::new(self, EdgeDetection::BothEdges).unwrap();
        waiter.wait().await.unwrap();
        Ok(())
    }
}
