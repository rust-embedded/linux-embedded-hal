//! Implementation of [`embedded-hal`] I2C traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal

use std::marker::PhantomData;
use std::ops;
use std::path::Path;

use embedded_hal::i2c::Direction;
use i2cdev::linux::{LinuxI2CBus, LinuxI2CError, LinuxI2CMessage};

/// Error type
#[derive(Debug)]
pub enum I2cError {
    /// Error coming from the Linux kernel.
    Linux(i2cdev::linux::LinuxI2CError),
    /// Invalid transaction.
    ///
    /// Possibly a transaction was attempted but it had not been properly started or stopped.
    InvalidTransaction,
    /// Wrong operation.
    ///
    /// This may happen if a stop is issued without a previous transaction.
    WrongOperation,
    /// Internal error.
    ///
    /// There is an error in this crate. Please report it.
    InternalError,
}

/// I2C bus type that implements the `embedded-hal` traits
///
/// [`i2cdev::linux::LinuxI2CDevice`]: https://docs.rs/i2cdev/0.5.0/i2cdev/linux/struct.LinuxI2CDevice.html
pub struct I2cBus<'a> {
    bus: LinuxI2CBus,
    started_transaction: Option<(u16, bool, Direction)>,
    last_transaction: Option<(u16, bool, Direction)>,
    pending_transactions: Vec<LinuxI2CMessage<'a>>,
    _lifetime: PhantomData<&'a LinuxI2CMessage<'a>>,
}

impl<'a> I2cBus<'a> {
    /// See [`i2cdev::linux::LinuxI2CDevice::new`][0] for details.
    ///
    /// [0]: https://docs.rs/i2cdev/0.5.0/i2cdev/linux/struct.LinuxI2CDevice.html#method.new
    pub fn new<P>(path: P) -> Result<Self, I2cError>
    where
        P: AsRef<Path>,
    {
        let dev = Self {
            bus: LinuxI2CBus::new(path).map_err(I2cError::Linux)?,
            started_transaction: None,
            last_transaction: None,
            pending_transactions: vec![],
            _lifetime: PhantomData,
        };
        Ok(dev)
    }

    fn start_transaction(&mut self, address: u16, is_ten_bit_addr: bool, direction: Direction) {
        self.started_transaction = Some((address, is_ten_bit_addr, direction));
    }
}

impl<'a> ops::Deref for I2cBus<'a> {
    type Target = LinuxI2CBus;

    fn deref(&self) -> &Self::Target {
        &self.bus
    }
}

impl<'a> ops::DerefMut for I2cBus<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bus
    }
}

impl From<LinuxI2CError> for I2cError {
    fn from(err: LinuxI2CError) -> Self {
        Self::Linux(err)
    }
}

mod embedded_hal_impl {
    use super::*;
    use embedded_hal::i2c::{
        blocking::{I2cBus as EhI2cBus, I2cBusBase},
        ErrorKind, ErrorType, NoAcknowledgeSource, SevenBitAddress, TenBitAddress,
    };
    use i2cdev::core::{I2CMessage, I2CTransfer};
    use i2cdev::linux::{I2CMessageFlags, LinuxI2CMessage};

    impl<'a> ErrorType for I2cBus<'a> {
        type Error = I2cError;
    }

    impl<'a> I2cBusBase for I2cBus<'a> {
        fn read(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
            let address;
            let is_ten_bit;
            let mut flags = I2CMessageFlags::empty();
            if let Some((addr, ten_bit, direction)) = self.started_transaction {
                address = addr;
                is_ten_bit = ten_bit;
                match direction {
                    Direction::Read => (), // start is automatically sent
                    Direction::Write => return Err(I2cError::InvalidTransaction), // write transaction was started but a read was issued.
                }
            } else if let Some((addr, ten_bit, direction)) = self.last_transaction {
                address = addr;
                is_ten_bit = ten_bit;
                match direction {
                    Direction::Read => flags |= I2CMessageFlags::NO_START, // same-direction transactions back-to-back
                    Direction::Write => (), // restart is automatically sent
                }
            } else {
                return Err(I2cError::InvalidTransaction); // first transaction but start was not called.
            }

            if is_ten_bit {
                flags |= I2CMessageFlags::TEN_BIT_ADDRESS;
            }
            let msg = LinuxI2CMessage::read(buffer)
                .with_address(address)
                .with_flags(flags | I2CMessageFlags::READ);
            self.last_transaction = Some((address, is_ten_bit, Direction::Write));
            self.pending_transactions.push(msg);
            self.started_transaction = None;
            Ok(())
        }

        fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
            let address;
            let is_ten_bit;
            let mut flags = I2CMessageFlags::empty();
            if let Some((addr, ten_bit, direction)) = self.started_transaction {
                address = addr;
                is_ten_bit = ten_bit;
                match direction {
                    Direction::Write => (), // start is automatically sent
                    Direction::Read => return Err(I2cError::InvalidTransaction), // read transaction was started but a write was issued.
                }
            } else if let Some((addr, ten_bit, direction)) = self.last_transaction {
                address = addr;
                is_ten_bit = ten_bit;
                match direction {
                    Direction::Write => flags |= I2CMessageFlags::NO_START, // same-direction transactions back-to-back
                    Direction::Read => (), // restart is automatically sent
                }
            } else {
                return Err(I2cError::InvalidTransaction); // first transaction but start was not called.
            }
            if is_ten_bit {
                flags |= I2CMessageFlags::TEN_BIT_ADDRESS;
            }
            let msg = LinuxI2CMessage::write(bytes)
                .with_address(address)
                .with_flags(flags);
            self.last_transaction = Some((address, is_ten_bit, Direction::Write));
            self.pending_transactions.push(msg);
            self.started_transaction = None;
            Ok(())
        }

        fn stop(&mut self) -> Result<(), Self::Error> {
            self.bus.transfer(&mut self.pending_transactions)?; // a stop at the end is automatically sent
            self.pending_transactions.clear();
            self.started_transaction = None;
            self.last_transaction = None;
            Ok(())
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl<'a> EhI2cBus<SevenBitAddress> for I2cBus<'a> {
        fn start(
            &mut self,
            address: SevenBitAddress,
            direction: embedded_hal::i2c::Direction,
        ) -> Result<(), Self::Error> {
            self.start_transaction(u16::from(address), false, direction);
            Ok(())
        }
    }

    impl<'a> EhI2cBus<TenBitAddress> for I2cBus<'a> {
        fn start(
            &mut self,
            address: TenBitAddress,
            direction: embedded_hal::i2c::Direction,
        ) -> Result<(), Self::Error> {
            self.start_transaction(address, true, direction);
            Ok(())
        }
    }

    impl embedded_hal::i2c::Error for I2cError {
        fn kind(&self) -> ErrorKind {
            use nix::errno::Errno::*;

            match &self {
                Self::Linux(err) => {
                    let errno = match &err {
                        i2cdev::linux::LinuxI2CError::Nix(e) => *e,
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
                _ => ErrorKind::Other,
            }
        }
    }
}
