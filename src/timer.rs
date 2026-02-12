//! Implementation of [`embedded-hal`] timer traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use core::convert::Infallible;
use std::time::{Duration, Instant};

/// Marker trait that indicates that a timer is periodic
pub trait Periodic {}

/// A count down timer
///
/// Note that this is borrowed from `embedded-hal` 0.2.x and will be in use until the `1.x` version provides one.
///
/// # Contract
///
/// - `self.start(count); block!(self.wait());` MUST block for AT LEAST the time specified by
///   `count`.
///
/// *Note* that the implementer doesn't necessarily have to be a *downcounting* timer; it could also
/// be an *upcounting* timer as long as the above contract is upheld.
///
/// # Examples
///
/// You can use this timer to create delays
///
/// ```
/// use std::time::Duration;
/// use nb::block;
/// use linux_embedded_hal::{CountDown, SysTimer};
///
/// fn main() {
///     let mut led: Led = {
///         // ..
/// #       Led
///     };
///     let mut timer = SysTimer::new();
///
///     Led.on();
///     timer.start(Duration::from_millis(1000)).unwrap();
///     block!(timer.wait()); // blocks for 1 second
///     Led.off();
/// }
///
/// # use core::convert::Infallible;
/// # struct Seconds(u32);
/// # trait U32Ext { fn s(self) -> Seconds; }
/// # impl U32Ext for u32 { fn s(self) -> Seconds { Seconds(self) } }
/// # struct Led;
/// # impl Led {
/// #     pub fn off(&mut self) {}
/// #     pub fn on(&mut self) {}
/// # }
/// ```
pub trait CountDown {
    /// An enumeration of `CountDown` errors.
    ///
    /// For infallible implementations, will be `Infallible`
    type Error: core::fmt::Debug;

    /// The unit of time used by this timer
    type Time;

    /// Starts a new count down
    fn start<T>(&mut self, count: T) -> Result<(), Self::Error>
    where
        T: Into<Self::Time>;

    /// Non-blockingly "waits" until the count down finishes
    ///
    /// # Contract
    ///
    /// - If `Self: Periodic`, the timer will start a new count down right after the last one
    ///   finishes.
    /// - Otherwise the behavior of calling `wait` after the last call returned `Ok` is UNSPECIFIED.
    ///   Implementers are suggested to panic on this scenario to signal a programmer error.
    fn wait(&mut self) -> nb::Result<(), Self::Error>;
}

impl<T: CountDown> CountDown for &mut T {
    type Error = T::Error;

    type Time = T::Time;

    fn start<TIME>(&mut self, count: TIME) -> Result<(), Self::Error>
    where
        TIME: Into<Self::Time>,
    {
        T::start(self, count)
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        T::wait(self)
    }
}

/// A periodic timer based on [`std::time::Instant`][instant], which is a
/// monotonically nondecreasing clock.
///
/// [instant]: https://doc.rust-lang.org/std/time/struct.Instant.html
pub struct SysTimer {
    start: Instant,
    duration: Duration,
}

impl SysTimer {
    /// Create a new timer instance.
    ///
    /// The `duration` will be initialized to 0, so make sure to call `start`
    /// with your desired timer duration before calling `wait`.
    pub fn new() -> SysTimer {
        SysTimer {
            start: Instant::now(),
            duration: Duration::from_millis(0),
        }
    }
}

impl Default for SysTimer {
    fn default() -> SysTimer {
        SysTimer::new()
    }
}

impl CountDown for SysTimer {
    type Error = Infallible;
    type Time = Duration;

    fn start<T>(&mut self, count: T) -> Result<(), Self::Error>
    where
        T: Into<Self::Time>,
    {
        self.start = Instant::now();
        self.duration = count.into();
        Ok(())
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        if (Instant::now() - self.start) >= self.duration {
            // Restart the timer to fulfill the contract by `Periodic`
            self.start = Instant::now();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Periodic for SysTimer {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensure that a 100 ms delay takes at least 100 ms,
    /// but not longer than 500 ms.
    #[test]
    fn test_delay() {
        let mut timer = SysTimer::new();
        let before = Instant::now();
        timer.start(Duration::from_millis(100)).unwrap();
        nb::block!(timer.wait()).unwrap();
        let after = Instant::now();
        let duration_ms = (after - before).as_millis();
        assert!(duration_ms >= 100);
        assert!(duration_ms < 500);
    }

    /// Ensure that the timer is periodic.
    #[test]
    fn test_periodic() {
        let mut timer = SysTimer::new();
        let before = Instant::now();
        timer.start(Duration::from_millis(100)).unwrap();
        nb::block!(timer.wait()).unwrap();
        let after1 = Instant::now();
        let duration_ms_1 = (after1 - before).as_millis();
        assert!(duration_ms_1 >= 100);
        assert!(duration_ms_1 < 500);
        nb::block!(timer.wait()).unwrap();
        let after2 = Instant::now();
        let duration_ms_2 = (after2 - after1).as_millis();
        assert!(duration_ms_2 >= 100);
        assert!(duration_ms_2 < 500);
    }
}
