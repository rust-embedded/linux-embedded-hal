//! Implementation of [`embedded-hal`] I2C traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use std::ops;
use std::path::{Path, PathBuf};

/// Newtype around [`i2cdev::linux::LinuxI2CDevice`] that implements the `embedded-hal` traits
///
/// [`i2cdev::linux::LinuxI2CDevice`]: https://docs.rs/i2cdev/0.5.0/i2cdev/linux/struct.LinuxI2CDevice.html
pub struct I2cdev {
    inner: i2cdev::linux::LinuxI2CDevice,
    path: PathBuf,
    address: Option<u8>,
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

    fn set_address(&mut self, address: u8) -> Result<(), i2cdev::linux::LinuxI2CError> {
        if self.address != Some(address) {
            self.inner = i2cdev::linux::LinuxI2CDevice::new(&self.path, u16::from(address))?;
            self.address = Some(address);
        }
        Ok(())
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
    use embedded_hal::i2c::blocking::{
        Operation as I2cOperation, Read, Transactional, Write, WriteRead,
    };
    use i2cdev::core::{I2CDevice, I2CMessage, I2CTransfer};
    use i2cdev::linux::LinuxI2CMessage;

    impl Read for I2cdev {
        type Error = i2cdev::linux::LinuxI2CError;

        fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
            self.set_address(address)?;
            self.inner.read(buffer)
        }
    }

    impl Write for I2cdev {
        type Error = i2cdev::linux::LinuxI2CError;

        fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
            self.set_address(address)?;
            self.inner.write(bytes)
        }
    }

    impl WriteRead for I2cdev {
        type Error = i2cdev::linux::LinuxI2CError;

        fn write_read(
            &mut self,
            address: u8,
            bytes: &[u8],
            buffer: &mut [u8],
        ) -> Result<(), Self::Error> {
            self.set_address(address)?;
            let mut messages = [LinuxI2CMessage::write(bytes), LinuxI2CMessage::read(buffer)];
            self.inner.transfer(&mut messages).map(drop)
        }
    }

    impl Transactional for I2cdev {
        type Error = i2cdev::linux::LinuxI2CError;

        fn exec(
            &mut self,
            address: u8,
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
            self.inner.transfer(&mut messages).map(drop)
        }
    }
}
