//! Implementation of [`embedded-hal`] SPI traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal
//!

use std::io;
use std::ops;
use std::path::Path;

/// Newtype around [`spidev::Spidev`] that implements the `embedded-hal` traits
///
/// [`spidev::Spidev`]: https://docs.rs/spidev/0.5.0/spidev/struct.Spidev.html
pub struct Spidev(pub spidev::Spidev);

impl Spidev {
    /// See [`spidev::Spidev::open`][0] for details.
    ///
    /// [0]: https://docs.rs/spidev/0.5.0/spidev/struct.Spidev.html#method.open
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
    use embedded_hal::spi::blocking::{
        Operation as SpiOperation, Transactional, Transfer, TransferInplace, Write,
    };
    use spidev::SpidevTransfer;
    use std::io::Write as _;

    impl Transfer<u8> for Spidev {
        type Error = IoError;

        fn transfer<'b>(&mut self, read: &'b mut [u8], write: &[u8]) -> Result<(), Self::Error> {
            self.0
                .transfer(&mut SpidevTransfer::read_write(&write, read))
                .map_err(|err| IoError { err })
        }
    }

    impl TransferInplace<u8> for Spidev {
        type Error = IoError;

        fn transfer_inplace<'b>(&mut self, buffer: &'b mut [u8]) -> Result<(), Self::Error> {
            let tx = buffer.to_owned();
            self.0
                .transfer(&mut SpidevTransfer::read_write(&tx, buffer))
                .map_err(|err| IoError { err })
        }
    }

    impl Write<u8> for Spidev {
        type Error = IoError;

        fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.0.write_all(buffer).map_err(|err| IoError { err })
        }
    }

    /// Transactional implementation batches SPI operations into a single transaction
    impl Transactional<u8> for Spidev {
        type Error = IoError;

        fn exec<'a>(&mut self, operations: &mut [SpiOperation<'a, u8>]) -> Result<(), Self::Error> {
            // Map types from generic to linux objects
            let mut messages: Vec<_> = operations
                .iter_mut()
                .map(|a| {
                    match a {
                        SpiOperation::Read(w) => SpidevTransfer::read(w),
                        SpiOperation::Write(w) => SpidevTransfer::write(w),
                        SpiOperation::Transfer(r, w) => SpidevTransfer::read_write(w, r),
                        SpiOperation::TransferInplace(r) => {
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
            self.0
                .transfer_multiple(&mut messages)
                .map_err(|err| IoError { err })
        }
    }
}

#[derive(Debug)]
pub struct IoError {
    err: io::Error,
}

impl From<io::Error> for IoError {
    fn from(err: io::Error) -> Self {
        Self { err }
    }
}

impl embedded_hal::spi::Error for IoError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        use embedded_hal::spi::ErrorKind::*;
        match self.err.kind() {
            // io::ErrorKind::NotFound => todo!(),
            // io::ErrorKind::PermissionDenied => todo!(),
            // io::ErrorKind::ConnectionRefused => todo!(),
            // io::ErrorKind::ConnectionReset => todo!(),
            // io::ErrorKind::HostUnreachable => todo!(),
            // io::ErrorKind::NetworkUnreachable => todo!(),
            // io::ErrorKind::ConnectionAborted => todo!(),
            // io::ErrorKind::NotConnected => todo!(),
            // io::ErrorKind::AddrInUse => todo!(),
            // io::ErrorKind::AddrNotAvailable => todo!(),
            // io::ErrorKind::NetworkDown => todo!(),
            // io::ErrorKind::BrokenPipe => todo!(),
            // io::ErrorKind::AlreadyExists => todo!(),
            // io::ErrorKind::WouldBlock => todo!(),
            // io::ErrorKind::NotADirectory => todo!(),
            // io::ErrorKind::IsADirectory => todo!(),
            // io::ErrorKind::DirectoryNotEmpty => todo!(),
            // io::ErrorKind::ReadOnlyFilesystem => todo!(),
            // io::ErrorKind::FilesystemLoop => todo!(),
            // io::ErrorKind::StaleNetworkFileHandle => todo!(),
            // io::ErrorKind::InvalidInput => todo!(),
            // io::ErrorKind::InvalidData => todo!(),
            // io::ErrorKind::TimedOut => todo!(),
            // io::ErrorKind::WriteZero => todo!(),
            // io::ErrorKind::StorageFull => todo!(),
            // io::ErrorKind::NotSeekable => todo!(),
            // io::ErrorKind::FilesystemQuotaExceeded => todo!(),
            // io::ErrorKind::FileTooLarge => todo!(),
            // io::ErrorKind::ResourceBusy => todo!(),
            // io::ErrorKind::ExecutableFileBusy => todo!(),
            // io::ErrorKind::Deadlock => todo!(),
            // io::ErrorKind::CrossesDevices => todo!(),
            // io::ErrorKind::TooManyLinks => todo!(),
            // io::ErrorKind::FilenameTooLong => todo!(),
            // io::ErrorKind::ArgumentListTooLong => todo!(),
            // io::ErrorKind::Interrupted => todo!(),
            // io::ErrorKind::Unsupported => todo!(),
            // io::ErrorKind::UnexpectedEof => todo!(),
            // io::ErrorKind::OutOfMemory => todo!(),
            // io::ErrorKind::Other => todo!(),
            // io::ErrorKind::Uncategorized => todo!(),
            _ => Other,
        }
    }
}
