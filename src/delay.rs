//! Implementation of [`embedded-hal`] delay traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use cast::u64;
use embedded_hal::delay::DelayNs;
use std::thread;
use std::time::Duration;

/// Empty struct that provides delay functionality on top of `thread::sleep`
pub struct Delay;

impl DelayNs for Delay {
    fn delay_ns(&mut self, n: u32) {
        thread::sleep(Duration::from_nanos(u64(n)));
    }
}
