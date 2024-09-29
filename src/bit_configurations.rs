use bitfield::bitfield;
use embedded_hal::i2c;
use core::marker::PhantomData;

pub trait LCDBitsTrait<I2C> : Default
where I2C: i2c::I2c
{
    fn bits(&self) -> u8;
    fn default_i2c_address() -> u8;

    fn set_rs(&mut self, value: u8);
    fn set_rw(&mut self, value: u8);
    fn set_enable(&mut self, value: u8);
    fn set_backlight(&mut self, value: u8);
    fn set_data(&mut self, value: u8);

    fn init(&self, _i2c: &mut I2C, _i2c_address: u8) -> Result<(), I2C::Error> {
        Ok(())
    }

    fn write_bits_to_gpio(&self, i2c: &mut I2C, i2c_address: u8) -> Result<(), I2C::Error> {
        let data = [self.bits()];
        i2c.write(i2c_address, &data)?;
        Ok(())
    }
}

// Configuration for the PCF8574T based 4-bit LCD interface sold by Buy Display and others
bitfield! {
    pub struct BuyDisplayBrandedLCDBitsImpl(u8);
    impl Debug;
    impl BitAnd;
    pub rs, set_rs: 0, 0;
    pub rw, set_rw: 1, 1;
    pub enable, set_enable: 2, 2;
    pub backlight, set_backlight: 3, 3;
    pub data, set_data: 7, 4;
}


pub struct BuyDisplayBrandedLCDBits<I2C> {
    bits: BuyDisplayBrandedLCDBitsImpl,
    _marker: PhantomData<I2C>,
}

impl<I2C> Default for BuyDisplayBrandedLCDBits<I2C>
where I2C: i2c::I2c
{
    fn default() -> Self {
        Self {
            bits: BuyDisplayBrandedLCDBitsImpl(0),
            _marker: PhantomData,
        }
    }
}

impl<I2C> LCDBitsTrait<I2C> for BuyDisplayBrandedLCDBits<I2C>
where I2C: i2c::I2c
{
    fn bits(&self) -> u8 {
        self.bits.0
    }

    fn default_i2c_address() -> u8 {
        0x27
    }

    fn set_rs(&mut self, value: u8) {
        self.bits.set_rs(value);
    }

    fn set_rw(&mut self, value: u8) {
        self.bits.set_rw(value);
    }

    fn set_enable(&mut self, value: u8) {
        self.bits.set_enable(value);
    }

    fn set_backlight(&mut self, value: u8) {
        self.bits.set_backlight(value);
    }

    fn set_data(&mut self, value: u8) {
        self.bits.set_data(value);
    }
}

// Configuration for the MCP23008 based LCD backpack from Adafruit
bitfield! {
    pub struct AdafruitLCDBackpackLCDBitsImpl(u8);
    impl Debug;
    impl BitAnd;
    pub rs, set_rs: 1, 1;
    pub enable, set_enable: 2, 2;
    pub backlight, set_backlight: 7, 7;
    pub data, set_data: 6, 3;
}

pub struct AdafruitLCDBackpackLCDBits<I2C> {
    bits: AdafruitLCDBackpackLCDBitsImpl,
    _marker: PhantomData<I2C>,
}

impl<I2C> Default for AdafruitLCDBackpackLCDBits<I2C>
where I2C: i2c::I2c
{
    fn default() -> Self {
        Self {
            bits: AdafruitLCDBackpackLCDBitsImpl(0),
            _marker: PhantomData,
        }
    }
}
/// Configuration for the MCP23008 based LCD backpack from Adafruit
impl<I2C> LCDBitsTrait<I2C> for AdafruitLCDBackpackLCDBits<I2C>
where I2C: i2c::I2c
{

    fn bits(&self) -> u8 {
        self.bits.0
    }

    fn default_i2c_address() -> u8 {
        0x20
    }

    fn set_rs(&mut self, value: u8) {
        self.bits.set_rs(value);
    }

    /// Adafruit LCD Backpack doesn't use RW
    fn set_rw(&mut self, _value: u8) {
        // Not used
    }

    fn set_enable(&mut self, value: u8) {
        self.bits.set_enable(value);
    }

    fn set_backlight(&mut self, value: u8) {
        self.bits.set_backlight(value);
    }

    fn set_data(&mut self, value: u8) {
        self.bits.set_data(value);
    }

    fn init(&self, i2c: &mut I2C, i2c_address: u8) -> Result<(), I2C::Error> {
        // Set the MCP23008 IODIR register to output
        i2c.write(i2c_address, &[0x00, 0x00])?;
        Ok(())
    }

    fn write_bits_to_gpio(&self, i2c: &mut I2C, i2c_address: u8) -> Result<(), I2C::Error> {
        // first byte is GPIO register address
        let data = [0x09, self.bits.0];
        i2c.write(i2c_address, &data)?;
        Ok(())
    }
}