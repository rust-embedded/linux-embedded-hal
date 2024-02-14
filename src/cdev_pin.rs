//! Implementation of [`embedded-hal`] digital input/output traits using a Linux CDev pin
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use std::fmt;
use std::path::Path;

use embedded_hal::digital::InputPin;
#[cfg(feature = "async-tokio")]
use gpiocdev::{line::EdgeDetection, tokio::AsyncRequest};
use gpiocdev::{
    line::{Offset, Value},
    request::{Config, Request},
};

/// Newtype around [`gpiocdev::request::Request`] that implements the `embedded-hal` traits.
#[cfg_attr(not(feature = "async-tokio"), derive(Debug))]
pub struct CdevPin {
    #[cfg(not(feature = "async-tokio"))]
    req: Request,
    #[cfg(feature = "async-tokio")]
    req: AsyncRequest,
    line: Offset,
}

impl CdevPin {
    /// Creates a new pin for the given `line` on the given `chip`.
    ///
    /// ```
    /// use linux_embedded_hal::CdevPin;
    /// # use linux_embedded_hal::CdevPinError;
    ///
    /// # fn main() -> Result<(), CdevPinError> {
    /// let mut pin = CdevPin::new("/dev/gpiochip0", 4)?.into_output_pin()?;
    /// pin.set_high()?;
    /// # }
    /// ```
    pub fn new<P>(chip: P, line: u32) -> Result<Self, CdevPinError>
    where
        P: AsRef<Path>,
    {
        let req = Request::builder()
            .on_chip(chip.as_ref())
            .with_line(line)
            .request()?;

        #[cfg(feature = "async-tokio")]
        let req = AsyncRequest::new(req);

        Ok(Self { req, line })
    }

    /// Creates a new pin from a [`Request`](gpiocdev::request::Request).
    ///
    /// # Panics
    ///
    /// Panics if the [`Request`](gpiocdev::request::Request) does not contain exactly one line.
    pub fn from_request(req: Request) -> Result<Self, CdevPinError> {
        let config = req.config();
        let lines = config.lines();

        assert!(
            lines.len() == 1,
            "A `CdevPin` must correspond to a single GPIO line."
        );
        let line = lines[0];

        #[cfg(feature = "async-tokio")]
        let req = AsyncRequest::new(req);

        Ok(CdevPin { req, line })
    }

    #[inline]
    fn request(&self) -> &Request {
        #[cfg(not(feature = "async-tokio"))]
        {
            &self.req
        }

        #[cfg(feature = "async-tokio")]
        {
            self.req.as_ref()
        }
    }

    fn config(&self) -> Config {
        self.request().config()
    }

    fn is_active_low(&self) -> bool {
        self.line_config().active_low
    }

    fn line_config(&self) -> gpiocdev::line::Config {
        // Unwrapping is fine, since `self.line` comes from a `Request` and is guaranteed to exist.
        self.config().line_config(self.line).unwrap().clone()
    }

    /// Set this pin to input mode
    pub fn into_input_pin(self) -> Result<CdevPin, CdevPinError> {
        let line_config = self.line_config();

        if line_config.direction == Some(gpiocdev::line::Direction::Input) {
            return Ok(self);
        }

        let mut new_config = self.config();
        new_config.as_input();
        self.request().reconfigure(&new_config)?;

        Ok(self)
    }

    /// Set this pin to output mode
    pub fn into_output_pin(
        self,
        state: embedded_hal::digital::PinState,
    ) -> Result<CdevPin, CdevPinError> {
        let line_config = self.line_config();

        if line_config.direction == Some(gpiocdev::line::Direction::Output) {
            return Ok(self);
        }

        let mut new_config = self.config();
        new_config.as_output(state_to_value(state, line_config.active_low));
        self.request().reconfigure(&new_config)?;

        Ok(self)
    }

    #[cfg(feature = "async-tokio")]
    async fn wait_for_edge(&mut self, edge: EdgeDetection) -> Result<(), CdevPinError> {
        if self.line_config().edge_detection != Some(edge) {
            let mut new_config = self.config();
            new_config.with_edge_detection(edge);
            self.request().reconfigure(&new_config)?;
        }

        self.req.read_edge_event().await?;
        Ok(())
    }
}

/// Converts a pin state to the gpio_cdev compatible numeric value, accounting
/// for the active_low condition.
fn state_to_value(state: embedded_hal::digital::PinState, is_active_low: bool) -> Value {
    if is_active_low {
        match state {
            embedded_hal::digital::PinState::High => Value::Inactive,
            embedded_hal::digital::PinState::Low => Value::Active,
        }
    } else {
        match state {
            embedded_hal::digital::PinState::High => Value::Active,
            embedded_hal::digital::PinState::Low => Value::Inactive,
        }
    }
}

/// Error type wrapping [`gpiocdev::Error`] to implement [`embedded_hal::digital::Error`].
#[derive(Debug)]
pub struct CdevPinError {
    err: gpiocdev::Error,
}

impl From<gpiocdev::Error> for CdevPinError {
    fn from(err: gpiocdev::Error) -> Self {
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
        let line = self.line;
        let is_active_low = self.is_active_low();
        self.request()
            .set_value(
                line,
                state_to_value(embedded_hal::digital::PinState::Low, is_active_low),
            )
            .map(|_| ())
            .map_err(CdevPinError::from)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let line = self.line;
        let is_active_low = self.is_active_low();
        self.request()
            .set_value(
                line,
                state_to_value(embedded_hal::digital::PinState::High, is_active_low),
            )
            .map(|_| ())
            .map_err(CdevPinError::from)
    }
}

impl InputPin for CdevPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        let line = self.line;
        self.request()
            .value(line)
            .map(|val| {
                val == state_to_value(embedded_hal::digital::PinState::High, self.is_active_low())
            })
            .map_err(CdevPinError::from)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|val| !val)
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
        self.wait_for_edge(EdgeDetection::RisingEdge).await
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_edge(EdgeDetection::FallingEdge).await
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_edge(EdgeDetection::BothEdges).await
    }
}
