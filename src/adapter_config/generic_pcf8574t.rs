use bitfield::bitfield;
use core::marker::PhantomData;
use embedded_hal::i2c;

use crate::LcdDisplayType;

use super::{AdapterConfigTrait, AdapterError};

// Configuration for the PCF8574T based 4-bit LCD interface sold
bitfield! {
    pub struct GenericPCF8574TBitField(u8);
    impl Debug;
    impl BitAnd;
    pub rs, set_rs: 0, 0;
    pub rw, set_rw: 1, 1;
    pub enable, set_enable: 2, 2;
    pub backlight, set_backlight: 3, 3;
    pub data, set_data: 7, 4;
}

pub struct GenericPCF8574TConfig<I2C> {
    bits: GenericPCF8574TBitField,
    _marker: PhantomData<I2C>,
}

impl<I2C> Default for GenericPCF8574TConfig<I2C>
where
    I2C: i2c::I2c,
{
    fn default() -> Self {
        Self {
            bits: GenericPCF8574TBitField(0),
            _marker: PhantomData,
        }
    }
}

impl<I2C> AdapterConfigTrait<I2C> for GenericPCF8574TConfig<I2C>
where
    I2C: i2c::I2c,
{
    fn bits(&self) -> u8 {
        self.bits.0
    }

    fn default_i2c_address() -> u8 {
        0x27
    }

    fn set_rs(&mut self, value: bool) {
        self.bits.set_rs(value as u8);
    }

    fn set_rw(&mut self, value: bool) {
        self.bits.set_rw(value as u8);
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
    fn test_generic_pcf8574t_compatiple_lcd_types() {
        // not exhaustive for compatible displays (is_supported == true)
        assert!(GenericPCF8574TConfig::<I2cMock>::is_supported(
            LcdDisplayType::Lcd16x2
        ));
        assert!(GenericPCF8574TConfig::<I2cMock>::is_supported(
            LcdDisplayType::Lcd20x4
        ));
        assert!(GenericPCF8574TConfig::<I2cMock>::is_supported(
            LcdDisplayType::Lcd40x2
        ));
        assert!(!GenericPCF8574TConfig::<I2cMock>::is_supported(
            LcdDisplayType::Lcd40x4
        ));
    }

    #[test]
    fn test_generic_pcf8574t_config() {
        let mut config = GenericPCF8574TConfig::<I2cMock>::default();
        config.set_rs(true);
        config.set_rw(false);
        assert!(config.set_enable(true, 0).is_ok());
        config.set_backlight(true);
        config.set_data(0b1010);

        assert_eq!(config.bits(), 0b10101101);
        assert_eq!(
            GenericPCF8574TConfig::<I2cMock>::default_i2c_address(),
            0x27
        );

        config.set_rs(false);
        config.set_rw(true);
        assert!(config.set_enable(false, 1).is_ok());
        config.set_backlight(false);
        config.set_data(0b0101);

        assert_eq!(config.bits(), 0b01010010);
    }

    #[test]
    fn test_generic_pcf8574t_config_write_bits_to_gpio() {
        let mut config = GenericPCF8574TConfig::<I2cMock>::default();
        config.set_rs(true);
        config.set_rw(false);
        assert!(config.set_enable(true, 0).is_ok());
        config.set_backlight(false);
        config.set_data(0b1010);

        let expected_transactions = [I2cTransaction::write(0x27, std::vec![0b10100101])];
        let mut i2c = I2cMock::new(&expected_transactions);

        config.write_bits_to_gpio(&mut i2c, 0x27).unwrap();
        i2c.done();
    }
}
