use embedded_hal::i2c::{
    blocking::{I2cBus, I2cBusBase as _, I2cDevice},
    Direction,
};
use embedded_hal_bus::i2c::blocking::ExclusiveDevice;
use linux_embedded_hal::I2cBus as LinuxI2cBus;

const ADDR: u8 = 0x12;

struct Driver<I2C> {
    i2c: I2C,
}

impl<I2C> Driver<I2C>
where
    I2C: I2cDevice,
    I2C::Bus: I2cBus,
{
    pub fn new(i2c: I2C) -> Self {
        Driver { i2c }
    }

    fn read_something(&mut self) -> Result<u8, I2C::Error> {
        let mut read_buffer = [0];
        self.i2c.transaction(|bus| {
            bus.start(ADDR, Direction::Write)?;
            bus.write(&[0xAB])?;
            bus.start(ADDR, Direction::Read)?;
            bus.read(&mut read_buffer)
        })?;
        Ok(read_buffer[0])
    }
}

fn main() {
    let bus = LinuxI2cBus::new("/dev/i2c-1").unwrap();
    let dev = ExclusiveDevice::new(bus);
    let mut driver = Driver::new(dev);
    let value = driver.read_something().unwrap();
    println!("Read value: {}", value);
}
