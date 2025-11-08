//! Implementation of [`embedded-hal`] I2C traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use std::fmt;
use std::ops;
use std::path::{Path, PathBuf};

use embedded_hal::i2c::NoAcknowledgeSource;

/// Newtype around [`i2cdev::linux::LinuxI2CDevice`] that implements the `embedded-hal` traits
///
/// [`i2cdev::linux::LinuxI2CDevice`]: https://docs.rs/i2cdev/0.5.0/i2cdev/linux/struct.LinuxI2CDevice.html
pub struct I2cdev {
    inner: i2cdev::linux::LinuxI2CDevice,
    path: PathBuf,
    address: Option<u16>,
}

impl I2cdev {
    /// See [`i2cdev::linux::LinuxI2CDevice::new`][0] for details.
    ///
    /// [0]: https://docs.rs/i2cdev/0.5.0/i2cdev/linux/struct.LinuxI2CDevice.html#method.new
    pub fn new<P>(path: P) -> Result<Self, i2cdev::linux::LinuxI2CError>
    where
        P: AsRef<Path>,
    {
        let dev = I2cdev {
            path: path.as_ref().to_path_buf(),
            inner: i2cdev::linux::LinuxI2CDevice::new(path, 0)?,
            address: None,
        };
        Ok(dev)
    }

    fn set_address(&mut self, address: u16) -> Result<(), i2cdev::linux::LinuxI2CError> {
        if self.address != Some(address) {
            self.inner = i2cdev::linux::LinuxI2CDevice::new(&self.path, address)?;
            self.address = Some(address);
        }
        Ok(())
    }
}

impl fmt::Debug for I2cdev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("I2cdev")
            .field("path", &self.path)
            .field("address", &self.address)
            // For the inner field we provide a placeholder, i2cdev does not impl Debug for LinuxI2CDevice
            .field("inner", &format!("LinuxI2CDevice<path: {:?}>", self.path))
            .finish()
    }
}

impl ops::Deref for I2cdev {
    type Target = i2cdev::linux::LinuxI2CDevice;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ops::DerefMut for I2cdev {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

mod embedded_hal_impl {
    use super::*;
    use embedded_hal::i2c::ErrorType;
    use embedded_hal::i2c::{I2c, Operation as I2cOperation, SevenBitAddress, TenBitAddress};
    use i2cdev::core::{I2CMessage, I2CTransfer};
    use i2cdev::linux::LinuxI2CMessage;
    impl ErrorType for I2cdev {
        type Error = I2CError;
    }

    impl I2c<TenBitAddress> for I2cdev {
        fn transaction(
            &mut self,
            address: u16,
            operations: &mut [I2cOperation],
        ) -> Result<(), Self::Error> {
            // Map operations from generic to linux objects
            let mut messages: Vec<_> = operations
                .as_mut()
                .iter_mut()
                .map(|a| match a {
                    I2cOperation::Write(w) => LinuxI2CMessage::write(w),
                    I2cOperation::Read(r) => LinuxI2CMessage::read(r),
                })
                .collect();

            self.set_address(address)?;
            self.inner
                .transfer(&mut messages)
                .map(drop)
                .map_err(|err| I2CError { err })
        }
    }

    impl I2c<SevenBitAddress> for I2cdev {
        fn transaction(
            &mut self,
            address: u8,
            operations: &mut [I2cOperation],
        ) -> Result<(), Self::Error> {
            I2c::<TenBitAddress>::transaction(self, u16::from(address), operations)
        }
    }
}

/// Error type wrapping [LinuxI2CError](i2cdev::linux::LinuxI2CError) to implement [embedded_hal::i2c::ErrorKind]
#[derive(Debug)]
pub struct I2CError {
    err: i2cdev::linux::LinuxI2CError,
}

impl I2CError {
    /// Fetch inner (concrete) [`LinuxI2CError`]
    pub fn inner(&self) -> &i2cdev::linux::LinuxI2CError {
        &self.err
    }
}

impl From<i2cdev::linux::LinuxI2CError> for I2CError {
    fn from(err: i2cdev::linux::LinuxI2CError) -> Self {
        Self { err }
    }
}

impl fmt::Display for I2CError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl std::error::Error for I2CError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.err)
    }
}

impl embedded_hal::i2c::Error for I2CError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        use embedded_hal::i2c::ErrorKind;
        use nix::errno::Errno::*;

        let errno = match &self.err {
            i2cdev::linux::LinuxI2CError::Errno(e) => nix::Error::from_i32(*e),
            i2cdev::linux::LinuxI2CError::Io(e) => match e.raw_os_error() {
                Some(r) => nix::Error::from_i32(r),
                None => return ErrorKind::Other,
            },
        };

        // https://www.kernel.org/doc/html/latest/i2c/fault-codes.html
        match errno {
            EBUSY | EINVAL | EIO => ErrorKind::Bus,
            EAGAIN => ErrorKind::ArbitrationLoss,
            ENODEV => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data),
            ENXIO => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address),
            _ => ErrorKind::Other,
        }
    }
}
