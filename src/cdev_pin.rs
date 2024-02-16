//! Implementation of [`embedded-hal`] digital input/output traits using a Linux cdev pin.
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use std::fmt;
use std::path::Path;

use embedded_hal::digital::InputPin;
#[cfg(feature = "async-tokio")]
use gpiocdev::{
    line::{EdgeDetection, EdgeKind},
    tokio::AsyncRequest,
};
use gpiocdev::{
    line::{Offset, Value},
    request::{Config, Request},
};

/// Wrapper around [`gpiocdev::request::Request`] that implements the `embedded-hal` traits.
#[derive(Debug)]
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

    /// Set this pin to input mode
    pub fn into_input_pin(self) -> Result<CdevPin, CdevPinError> {
        let config = self.config();
        let line_config = config.line_config(self.line).unwrap();

        if line_config.direction == Some(gpiocdev::line::Direction::Input) {
            return Ok(self);
        }

        let mut new_config = config;
        new_config.as_input();
        self.request().reconfigure(&new_config)?;

        Ok(self)
    }

    /// Set this pin to output mode
    pub fn into_output_pin(
        self,
        state: embedded_hal::digital::PinState,
    ) -> Result<CdevPin, CdevPinError> {
        let config = self.config();
        let line_config = config.line_config(self.line).unwrap();
        if line_config.direction == Some(gpiocdev::line::Direction::Output) {
            return Ok(self);
        }
        let is_active_low = line_config.active_low;

        let mut new_config = config;
        new_config.as_output(state_to_value(state, is_active_low));
        self.request().reconfigure(&new_config)?;

        Ok(self)
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
        let is_active_low = self.config().line_config(line).unwrap().active_low;
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
        let is_active_low = self.config().line_config(line).unwrap().active_low;
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
        let is_active_low = self.config().line_config(line).unwrap().active_low;
        self.request()
            .value(line)
            .map(|val| val == state_to_value(embedded_hal::digital::PinState::High, is_active_low))
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
        let config = self.config();
        let line_config = config.line_config(self.line).unwrap();
        if !matches!(
            line_config.edge_detection,
            Some(EdgeDetection::RisingEdge | EdgeDetection::BothEdges)
        ) {
            let mut new_config = config;
            new_config.with_edge_detection(EdgeDetection::RisingEdge);
            self.request().reconfigure(&new_config)?;
        }

        loop {
            let event = self.req.read_edge_event().await?;
            if event.kind == EdgeKind::Rising {
                return Ok(());
            }
        }
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        let config = self.config();
        let line_config = config.line_config(self.line).unwrap();
        if !matches!(
            line_config.edge_detection,
            Some(EdgeDetection::FallingEdge | EdgeDetection::BothEdges)
        ) {
            let mut new_config = config;
            new_config.with_edge_detection(EdgeDetection::FallingEdge);
            self.request().reconfigure(&new_config)?;
        }

        loop {
            let event = self.req.read_edge_event().await?;
            if event.kind == EdgeKind::Falling {
                return Ok(());
            }
        }
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        let config = self.config();
        let line_config = config.line_config(self.line).unwrap();
        if line_config.edge_detection != Some(EdgeDetection::BothEdges) {
            let mut new_config = config;
            new_config.with_edge_detection(EdgeDetection::BothEdges);
            self.request().reconfigure(&new_config)?;
        }

        self.req.read_edge_event().await?;
        Ok(())
    }
}
