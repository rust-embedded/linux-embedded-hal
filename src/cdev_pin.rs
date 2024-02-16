//! Implementation of [`embedded-hal`] digital input/output traits using a Linux cdev pin.
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use std::fmt;
use std::marker::PhantomData;
use std::path::Path;

use embedded_hal::digital::{Error, ErrorType, InputPin, OutputPin, PinState, StatefulOutputPin};
use gpiocdev::{
    line::{Config, Direction, Offset, Value},
    request::Request,
};
#[cfg(feature = "async-tokio")]
use gpiocdev::{
    line::{EdgeDetection, EdgeKind},
    tokio::AsyncRequest,
};

/// Marker type for a [`CdevPin`] in input mode.
#[non_exhaustive]
pub struct Input;

/// Marker type for a [`CdevPin`] in output mode.
#[non_exhaustive]
pub struct Output;

/// Wrapper around [`gpiocdev::request::Request`] that implements the `embedded-hal` traits.
#[derive(Debug)]
pub struct CdevPin<MODE> {
    #[cfg(not(feature = "async-tokio"))]
    req: Request,
    #[cfg(feature = "async-tokio")]
    req: AsyncRequest,
    line: Offset,
    line_config: Config,
    mode: PhantomData<MODE>,
}

impl CdevPin<Input> {
    /// Creates a new input pin for the given `line` on the given `chip`.
    ///
    /// ```
    /// use linux_embedded_hal::CdevPin;
    /// # use linux_embedded_hal::CdevPinError;
    ///
    /// # fn main() -> Result<(), CdevPinError> {
    /// let mut pin = CdevPin::new_input("/dev/gpiochip0", 4)?;
    /// pin.is_high()?;
    /// # }
    /// ```
    pub fn new_input<P>(chip: P, line: u32) -> Result<Self, CdevPinError>
    where
        P: AsRef<Path>,
    {
        let line_config = Config {
            direction: Some(Direction::Input),
            ..Default::default()
        };

        let req = Request::builder()
            .on_chip(chip.as_ref())
            .from_line_config(&line_config)
            .request()?;

        #[cfg(feature = "async-tokio")]
        let req = AsyncRequest::new(req);

        Ok(Self {
            req,
            line,
            line_config,
            mode: PhantomData,
        })
    }

    /// Converts this input pin into an output pin with the given `initial_state`.
    pub fn into_output<P>(self, initial_state: PinState) -> Result<CdevPin<Output>, CdevPinError> {
        let new_value = self.state_to_value(initial_state);

        let req = self.req;
        let mut new_config = req.as_ref().config();
        new_config.as_output(new_value);
        req.as_ref().reconfigure(&new_config)?;

        let line = self.line;
        let line_config = new_config.line_config(line).unwrap().clone();

        Ok(CdevPin {
            req,
            line,
            line_config,
            mode: PhantomData,
        })
    }
}

impl CdevPin<Output> {
    /// Creates a new output pin for the given `line` on the given `chip`,
    /// initialized with the given `initial_state`.
    ///
    /// ```
    /// use linux_embedded_hal::CdevPin;
    /// # use linux_embedded_hal::CdevPinError;
    ///
    /// # fn main() -> Result<(), CdevPinError> {
    /// let mut pin = CdevPin::new_output("/dev/gpiochip0", 4)?;
    /// pin.is_set_high()?;
    /// pin.set_high()?;
    /// # }
    /// ```
    pub fn new_output<P>(chip: P, line: u32, initial_state: PinState) -> Result<Self, CdevPinError>
    where
        P: AsRef<Path>,
    {
        let line_config = Config {
            direction: Some(Direction::Output),
            active_low: false,
            value: Some(match initial_state {
                PinState::High => Value::Active,
                PinState::Low => Value::Inactive,
            }),
            ..Default::default()
        };

        let req = Request::builder()
            .on_chip(chip.as_ref())
            .from_line_config(&line_config)
            .request()?;

        #[cfg(feature = "async-tokio")]
        let req = AsyncRequest::new(req);

        Ok(Self {
            req,
            line,
            line_config,
            mode: PhantomData,
        })
    }

    /// Converts this output pin into an input pin.
    pub fn into_input<P>(self) -> Result<CdevPin<Output>, CdevPinError> {
        let req = self.req;
        let mut new_config = req.as_ref().config();
        new_config.as_input();
        req.as_ref().reconfigure(&new_config)?;

        let line = self.line;
        let line_config = new_config.line_config(line).unwrap().clone();

        Ok(CdevPin {
            req,
            line,
            line_config,
            mode: PhantomData,
        })
    }
}

impl<MODE> CdevPin<MODE> {
    /// Converts a pin state to a value, depending on
    /// whether the pin is configured as active-low.
    fn state_to_value(&self, state: PinState) -> Value {
        if self.line_config.active_low {
            match state {
                PinState::High => Value::Inactive,
                PinState::Low => Value::Active,
            }
        } else {
            match state {
                PinState::High => Value::Active,
                PinState::Low => Value::Inactive,
            }
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

impl Error for CdevPinError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        use embedded_hal::digital::ErrorKind;
        ErrorKind::Other
    }
}

impl<MODE> ErrorType for CdevPin<MODE> {
    type Error = CdevPinError;
}

impl InputPin for CdevPin<Input> {
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        let low_value = self.state_to_value(PinState::Low);
        Ok(self.req.as_ref().value(self.line)? == low_value)
    }

    fn is_high(&mut self) -> Result<bool, Self::Error> {
        let high_value = self.state_to_value(PinState::High);
        Ok(self.req.as_ref().value(self.line)? == high_value)
    }
}

impl OutputPin for CdevPin<Output> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        let new_value = self.state_to_value(PinState::Low);

        self.req.as_ref().set_value(self.line, new_value)?;
        self.line_config.value = Some(new_value);

        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let new_value = self.state_to_value(PinState::High);

        self.req.as_ref().set_value(self.line, new_value)?;
        self.line_config.value = Some(new_value);

        Ok(())
    }
}

impl StatefulOutputPin for CdevPin<Output> {
    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        let low_value = self.state_to_value(PinState::Low);
        Ok(self.line_config.value == Some(low_value))
    }

    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        let high_value = self.state_to_value(PinState::High);
        Ok(self.line_config.value == Some(high_value))
    }
}

#[cfg(feature = "async-tokio")]
impl embedded_hal_async::digital::Wait for CdevPin<Input> {
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
        if !matches!(
            self.line_config.edge_detection,
            Some(EdgeDetection::RisingEdge | EdgeDetection::BothEdges)
        ) {
            let req = self.req.as_ref();
            let mut new_config = req.config();
            new_config.with_edge_detection(EdgeDetection::RisingEdge);
            req.reconfigure(&new_config)?;
            self.line_config.edge_detection = Some(EdgeDetection::RisingEdge);
        }

        loop {
            let event = self.req.read_edge_event().await?;
            if event.kind == EdgeKind::Rising {
                return Ok(());
            }
        }
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        if !matches!(
            self.line_config.edge_detection,
            Some(EdgeDetection::FallingEdge | EdgeDetection::BothEdges)
        ) {
            let mut new_config = self.req.as_ref().config();
            new_config.with_edge_detection(EdgeDetection::FallingEdge);
            self.req.as_ref().reconfigure(&new_config)?;
            self.line_config.edge_detection = Some(EdgeDetection::FallingEdge);
        }

        loop {
            let event = self.req.read_edge_event().await?;
            if event.kind == EdgeKind::Falling {
                return Ok(());
            }
        }
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        if !matches!(
            self.line_config.edge_detection,
            Some(EdgeDetection::BothEdges)
        ) {
            let mut new_config = self.req.as_ref().config();
            new_config.with_edge_detection(EdgeDetection::BothEdges);
            self.req.as_ref().reconfigure(&new_config)?;
            self.line_config.edge_detection = Some(EdgeDetection::BothEdges);
        }

        self.req.read_edge_event().await?;
        Ok(())
    }
}
