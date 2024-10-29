use bitfield::bitfield;
use core::marker::PhantomData;
use embedded_hal::i2c;

use super::{AdapterConfigTrait, AdapterError, LcdDisplayType};

// Configuration for the MCP23008 based LCD backpack from Adafruit
bitfield! {
    pub struct AdafruitLCDBackpackBitField(u8);
    impl Debug;
    impl BitAnd;
    pub rs, set_rs: 1, 1;
    pub enable, set_enable: 2, 2;
    pub backlight, set_backlight: 7, 7;
    pub data, set_data: 6, 3;
}

pub struct AdafruitLCDBackpackConfig<I2C> {
    bits: AdafruitLCDBackpackBitField,
    _marker: PhantomData<I2C>,
}

impl<I2C> Default for AdafruitLCDBackpackConfig<I2C>
where
    I2C: i2c::I2c,
{
    fn default() -> Self {
        Self {
            bits: AdafruitLCDBackpackBitField(0),
            _marker: PhantomData,
        }
    }
}
/// Configuration for the MCP23008 based LCD backpack from Adafruit
impl<I2C> AdapterConfigTrait<I2C> for AdafruitLCDBackpackConfig<I2C>
where
    I2C: i2c::I2c,
{
    fn bits(&self) -> u8 {
        self.bits.0
    }

    fn default_i2c_address() -> u8 {
        0x20
    }

    fn supports_reads() -> bool {
        false
    }

    fn set_rs(&mut self, value: bool) {
        self.bits.set_rs(value as u8);
    }

    /// Adafruit LCD Backpack doesn't use RW
    fn set_rw(&mut self, _value: bool) {
        // Not used
    }

    fn set_enable(&mut self, value: bool, _device: usize) -> Result<(), AdapterError> {
        self.bits.set_enable(value as u8);
        Ok(())
    }

    fn set_backlight(&mut self, value: bool) {
        self.bits.set_backlight(value as u8);
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

    fn read_from_gpio(&self, _i2c: &mut I2C, _i2c_address: u8, _rs_setting: bool) -> Result<u8, I2C::Error> {
        // reads are not supported
        unimplemented!("Reads are not supported for Adafruit LCD Backpack");
    }

    fn is_supported(display_type: LcdDisplayType) -> bool {
        display_type != LcdDisplayType::Lcd40x4
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    #[test]
    fn test_adafruit_lcd_backpack_config() {
        let mut config = AdafruitLCDBackpackConfig::<I2cMock>::default();
        config.set_rs(true);
        assert!(config.set_enable(true, 2).is_ok());
        config.set_backlight(true);
        config.set_data(0b1010);
        // adafruit backpack doesn't use RW

        assert_eq!(config.bits(), 0b11010110);
        assert_eq!(
            AdafruitLCDBackpackConfig::<I2cMock>::default_i2c_address(),
            0x20
        );

        config.set_rs(false);
        assert!(config.set_enable(false, 0).is_ok());
        config.set_backlight(false);
        config.set_data(0b0101);

        assert_eq!(config.bits(), 0b00101000);
    }

    #[test]
    fn test_adafruit_lcd_backpack_config_write_bits_to_gpio() {
        let mut config = AdafruitLCDBackpackConfig::<I2cMock>::default();
        config.set_rs(true);
        assert!(config.set_enable(true, 1).is_ok());
        config.set_backlight(true);
        config.set_data(0b1010);

        let expected_transactions = [I2cTransaction::write(0x20, std::vec![0x09, 0b11010110])];
        let mut i2c = I2cMock::new(&expected_transactions);

        config.write_bits_to_gpio(&mut i2c, 0x20).unwrap();
        i2c.done();
    }

    #[test]
    fn test_adafruit_init() {
        let config = AdafruitLCDBackpackConfig::<I2cMock>::default();

        let expected_transactions = [I2cTransaction::write(0x20, std::vec![0x00, 0x00])];
        let mut i2c = I2cMock::new(&expected_transactions);

        config.init(&mut i2c, 0x20).unwrap();
        i2c.done();
    }
}
