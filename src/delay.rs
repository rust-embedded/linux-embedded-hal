//! Implementation of [`embedded-hal`] delay traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use cast::{u32, u64};
use core::convert::Infallible;
use embedded_hal::delay::blocking::{DelayMs, DelayUs};
use std::thread;
use std::time::Duration;

/// Empty struct that provides delay functionality on top of `thread::sleep`
pub struct Delay;

impl DelayUs<u8> for Delay {
    type Error = Infallible;

    fn delay_us(&mut self, n: u8) -> Result<(), Self::Error> {
        thread::sleep(Duration::new(0, u32(n) * 1000));
        Ok(())
    }
}

impl DelayUs<u16> for Delay {
    type Error = Infallible;

    fn delay_us(&mut self, n: u16) -> Result<(), Self::Error> {
        thread::sleep(Duration::new(0, u32(n) * 1000));
        Ok(())
    }
}

impl DelayUs<u32> for Delay {
    type Error = Infallible;

    fn delay_us(&mut self, n: u32) -> Result<(), Self::Error> {
        let secs = n / 1_000_000;
        let nsecs = (n % 1_000_000) * 1_000;

        thread::sleep(Duration::new(u64(secs), nsecs));
        Ok(())
    }
}

impl DelayUs<u64> for Delay {
    type Error = Infallible;

    fn delay_us(&mut self, n: u64) -> Result<(), Self::Error> {
        let secs = n / 1_000_000;
        let nsecs = ((n % 1_000_000) * 1_000) as u32;

        thread::sleep(Duration::new(secs, nsecs));
        Ok(())
    }
}

impl DelayMs<u8> for Delay {
    type Error = Infallible;

    fn delay_ms(&mut self, n: u8) -> Result<(), Self::Error> {
        thread::sleep(Duration::from_millis(u64(n)));
        Ok(())
    }
}

impl DelayMs<u16> for Delay {
    type Error = Infallible;

    fn delay_ms(&mut self, n: u16) -> Result<(), Self::Error> {
        thread::sleep(Duration::from_millis(u64(n)));
        Ok(())
    }
}

impl DelayMs<u32> for Delay {
    type Error = Infallible;

    fn delay_ms(&mut self, n: u32) -> Result<(), Self::Error> {
        thread::sleep(Duration::from_millis(u64(n)));
        Ok(())
    }
}

impl DelayMs<u64> for Delay {
    type Error = Infallible;

    fn delay_ms(&mut self, n: u64) -> Result<(), Self::Error> {
        thread::sleep(Duration::from_millis(n));
        Ok(())
    }
}
