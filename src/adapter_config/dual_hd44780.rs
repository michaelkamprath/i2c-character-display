use bitfield::bitfield;
use core::marker::PhantomData;
use embedded_hal::i2c;

use super::{AdapterConfigTrait, AdapterError, LcdDisplayType};

bitfield! {
    pub struct DualHD44780_PCF8574TBitField(u8);
    impl Debug;
    impl BitAnd;
    pub rs, set_rs: 0, 0;
    pub enable2, set_enable2: 1, 1;
    pub enable1, set_enable1: 2, 2;
    pub backlight, set_backlight: 3, 3;
    pub data, set_data: 7, 4;
}

pub struct DualHD44780_PCF8574TConfig<I2C> {
    bits: DualHD44780_PCF8574TBitField,
    _marker: PhantomData<I2C>,
}

impl<I2C> Default for DualHD44780_PCF8574TConfig<I2C>
where
    I2C: i2c::I2c,
{
    fn default() -> Self {
        Self {
            bits: DualHD44780_PCF8574TBitField(0),
            _marker: PhantomData,
        }
    }
}

impl<I2C> AdapterConfigTrait<I2C> for DualHD44780_PCF8574TConfig<I2C>
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

    /// Dual HD44780 displays have two enable pins and do not use the RW pin
    fn set_rw(&mut self, _value: bool) {
        // does nothing
    }

    fn set_enable(&mut self, value: bool, device: usize) -> Result<(), AdapterError> {
        if device == 0 {
            self.bits.set_enable1(value as u8);
        } else if device == 1 {
            self.bits.set_enable2(value as u8);
        } else {
            return Err(AdapterError::BadDeviceId);
        }
        Ok(())
    }

    fn set_backlight(&mut self, value: bool) {
        self.bits.set_backlight(value as u8);
    }

    fn set_data(&mut self, value: u8) {
        self.bits.set_data(value);
    }

    fn device_count(&self) -> usize {
        2
    }

    fn row_to_device_row(&self, row: u8) -> (usize, u8) {
        if row < 2 {
            (0, row)
        } else {
            (1, row - 2)
        }
    }

    fn is_supported(_display_type: LcdDisplayType) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    #[test]
    fn test_bad_device_id() {
        let mut config = DualHD44780_PCF8574TConfig::<I2cMock>::default();
        assert_eq!(config.set_enable(true, 2), Err(AdapterError::BadDeviceId));
    }

    #[test]
    fn test_dual_hd44780_config() {
        let mut config = DualHD44780_PCF8574TConfig::<I2cMock>::default();
        config.set_rs(true);
        config.set_rw(true);
        assert!(config.set_enable(true, 0).is_ok());
        assert!(config.set_enable(false, 1).is_ok());
        config.set_backlight(true);
        config.set_data(0b1010);

        assert_eq!(config.bits(), 0b10101101);
        assert_eq!(
            DualHD44780_PCF8574TConfig::<I2cMock>::default_i2c_address(),
            0x27
        );

        config.set_rs(false);
        config.set_rw(true);
        assert!(config.set_enable(false, 0).is_ok());
        assert!(config.set_enable(true, 1).is_ok());
        config.set_backlight(false);
        config.set_data(0b0101);

        assert_eq!(config.bits(), 0b01010010);
    }

    #[test]
    fn test_dual_hd44780_config_write_bits_to_gpio() {
        let mut config = DualHD44780_PCF8574TConfig::<I2cMock>::default();
        config.set_rs(true);
        config.set_rw(false);
        assert!(config.set_enable(false, 0).is_ok());
        assert!(config.set_enable(true, 1).is_ok());
        config.set_backlight(false);
        config.set_data(0b1010);

        let expected_transactions = [I2cTransaction::write(0x27, std::vec![0b10100011])];
        let mut i2c = I2cMock::new(&expected_transactions);

        config.write_bits_to_gpio(&mut i2c, 0x27).unwrap();
        i2c.done();
    }
}
