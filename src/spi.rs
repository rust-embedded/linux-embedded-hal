//! Implementation of [`embedded-hal`] SPI traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal
//!

use std::io;
use std::ops;
use std::path::Path;

/// Newtype around [`spidev::Spidev`] that implements the `embedded-hal` traits
///
/// [`spidev::Spidev`]: https://docs.rs/spidev/0.4.0/spidev/struct.Spidev.html
pub struct Spidev(pub spidev::Spidev);

impl Spidev {
    /// See [`spidev::Spidev::open`][0] for details.
    ///
    /// [0]: https://docs.rs/spidev/0.4.0/spidev/struct.Spidev.html#method.open
    pub fn open<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        spidev::Spidev::open(path).map(Spidev)
    }
}

impl ops::Deref for Spidev {
    type Target = spidev::Spidev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Spidev {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

mod embedded_hal_impl {
    use super::*;
    use embedded_hal::spi::blocking::{Operation as SpiOperation, Transactional, Transfer, Write};
    use spidev::SpidevTransfer;
    use std::io::Write as _;

    impl Transfer<u8> for Spidev {
        type Error = io::Error;

        fn transfer<'b>(&mut self, buffer: &'b mut [u8]) -> io::Result<()> {
            let tx = buffer.to_owned();
            self.0
                .transfer(&mut SpidevTransfer::read_write(&tx, buffer))
        }
    }

    impl Write<u8> for Spidev {
        type Error = io::Error;

        fn write(&mut self, buffer: &[u8]) -> io::Result<()> {
            self.0.write_all(buffer)
        }
    }

    /// Transactional implementation batches SPI operations into a single transaction
    impl Transactional<u8> for Spidev {
        type Error = io::Error;

        fn exec<'a>(&mut self, operations: &mut [SpiOperation<'a, u8>]) -> Result<(), Self::Error> {
            // Map types from generic to linux objects
            let mut messages: Vec<_> = operations
                .iter_mut()
                .map(|a| {
                    match a {
                        SpiOperation::Write(w) => SpidevTransfer::write(w),
                        SpiOperation::Transfer(r) => {
                            // Clone read to write pointer
                            // SPIdev is okay with having w == r but this is tricky to achieve in safe rust
                            let w = unsafe {
                                let p = r.as_ptr();
                                std::slice::from_raw_parts(p, r.len())
                            };

                            SpidevTransfer::read_write(w, r)
                        }
                    }
                })
                .collect();

            // Execute transfer
            self.0.transfer_multiple(&mut messages)
        }
    }
}
