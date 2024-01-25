//! Implementation of [`embedded-hal`] delay traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use embedded_hal::delay::DelayNs;
use std::thread;
use std::time::Duration;

/// Empty struct that provides delay functionality on top of `thread::sleep`,
/// and `tokio::time::sleep` if the `async-tokio` feature is enabled.
pub struct Delay;

impl DelayNs for Delay {
    fn delay_ns(&mut self, n: u32) {
        thread::sleep(Duration::from_nanos(n.into()));
    }

    fn delay_us(&mut self, n: u32) {
        thread::sleep(Duration::from_micros(n.into()));
    }

    fn delay_ms(&mut self, n: u32) {
        thread::sleep(Duration::from_millis(n.into()));
    }
}

#[cfg(feature = "async-tokio")]
impl embedded_hal_async::delay::DelayNs for Delay {
    async fn delay_ns(&mut self, n: u32) {
        tokio::time::sleep(Duration::from_nanos(n.into())).await;
    }

    async fn delay_us(&mut self, n: u32) {
        tokio::time::sleep(Duration::from_micros(n.into())).await;
    }

    async fn delay_ms(&mut self, n: u32) {
        tokio::time::sleep(Duration::from_millis(n.into())).await;
    }
}
