//! Implementation of [`embedded-hal`] delay traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use cast::u64;
use embedded_hal::delay::DelayUs;
use std::thread;
use std::time::Duration;

/// Empty struct that provides delay functionality on top of `thread::sleep`
pub struct Delay;

impl DelayUs for Delay {
    fn delay_us(&mut self, n: u32) {
        let secs = n / 1_000_000;
        let nsecs = (n % 1_000_000) * 1_000;

        thread::sleep(Duration::new(u64(secs), nsecs));
    }
}
