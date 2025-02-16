use bitfield::bitfield;
use core::marker::PhantomData;
use embedded_hal::{delay::DelayNs, i2c};

use crate::{driver::DeviceHardwareTrait, CharacterDisplayError, DeviceSetupConfig, LcdDisplayType};

use super::HD44780AdapterTrait;

// Configuration for the PCF8574T based 4-bit LCD interface soldto dual HD44780 controllers
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

impl Clone for DualHD44780_PCF8574TBitField {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

/// Adapter based on the PCF8574T I2C GPIO expander interfacing with two HD44780 LCD controller
/// via a 4-bit interface. The two controllers enable LCD screen sizes lik 40x4.
pub struct DualHD44780_PCF8574TAdapter<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    bits: DualHD44780_PCF8574TBitField,
    config: DeviceSetupConfig<I2C, DELAY>,
    _marker: PhantomData<I2C>,
}

impl<I2C, DELAY> DeviceHardwareTrait<I2C, DELAY> for DualHD44780_PCF8574TAdapter<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn new(config: DeviceSetupConfig<I2C, DELAY>) -> Self {
        Self {
            bits: DualHD44780_PCF8574TBitField(0),
            config: config,
            _marker: PhantomData,
        }
    }

    fn default_i2c_address() -> u8 {
        0x27
    }

    fn supports_reads() -> bool {
        false
    }

    fn lcd_type(&self) -> LcdDisplayType {
        self.config.lcd_type
    }

    fn i2c_address(&self) -> u8 {
        self.config.address
    }

    fn delay(&mut self) -> &mut DELAY {
        &mut self.config.delay
    }

    fn i2c(&mut self) -> &mut I2C {
        &mut self.config.i2c
    }

    fn init(
        &mut self,
    ) -> Result<(u8, u8, u8), CharacterDisplayError<I2C>> {
        self.adapter_init()
    }

    fn write_bytes(
        &mut self,
        _rs_setting: bool,
        _data: &[u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        todo!()
    }
}

impl<I2C, DELAY> HD44780AdapterTrait<I2C, DELAY> for DualHD44780_PCF8574TAdapter<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn device_config(&mut self) -> &mut DeviceSetupConfig<I2C, DELAY> {
        &mut self.config
    }

    fn bits(&self) -> u8 {
        self.bits.0
    }

    fn max_controller_count() -> usize {
        2
    }

    fn set_enable(
        &mut self,
        value: bool,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if controller == 0 {
            self.bits.set_enable1(value as u8);
        } else if controller == 1 {
            self.bits.set_enable2(value as u8);
        } else {
            return Err(CharacterDisplayError::BadDeviceId);
        }
        Ok(())
    }

    fn set_rs(&mut self, value: bool) {
        self.bits.set_rs(value as u8);
    }

    fn set_rw(&mut self, _value: bool) {
        // does nothing
    }

    fn set_backlight(&mut self, value: bool) ->Result<(), CharacterDisplayError<I2C>> {
        self.bits.set_backlight(value as u8);
        self.write_bits_to_gpio()
    }

    fn set_data(&mut self, value: u8) {
        self.bits.set_data(value);
    }

    fn is_supported(display_type: LcdDisplayType) -> bool {
        display_type == LcdDisplayType::Lcd40x4
    }

    fn controller_count() -> usize {
        2
    }

    fn row_to_controller_row(&self, row: u8) -> (usize, u8) {
        if row < 2 {
            (0, row)
        } else {
            (1, row - 2)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use embedded_hal_mock::eh1::{
        i2c::{Mock as I2cMock, Transaction as I2cTransaction},
        delay::NoopDelay,
    };

    #[test]
    fn test_bad_device_id() {
        let mut config = DualHD44780_PCF8574TAdapter::new(
            DeviceSetupConfig {
                i2c: I2cMock::new(&[]),
                address: 0x27,
                lcd_type: LcdDisplayType::Lcd40x4,
                delay: NoopDelay,
            },
        );
        assert!(config.set_enable(true, 2).is_err());
        assert!(config.bits() == 0);
        assert!(config.set_enable(true, 0).is_ok());
        assert!(config.bits() & 0b0000_0100 != 0);
        assert!(config.set_enable(false, 0).is_ok());
        assert!(config.bits() == 0);
        assert!(config.set_enable(true, 1).is_ok());
        assert!(config.bits() & 0b0000_0010 != 0);
        assert!(config.set_enable(false, 1).is_ok());
        assert!(config.bits() == 0);

        config.i2c().done();
    }

    #[test]
    fn test_dual_hd44780_adapter() {
        let mut config = DualHD44780_PCF8574TAdapter::new(
            DeviceSetupConfig {
                i2c: I2cMock::new(&[
                    I2cTransaction::write(0x27, std::vec![0b1010_1101]),
                    I2cTransaction::write(0x27, std::vec![0b0101_0010]),
                ]),
                address: 0x27,
                lcd_type: LcdDisplayType::Lcd40x4,
                delay: NoopDelay,
            },
        );
        config.set_rs(true);
        config.set_rw(false);
        assert!(config.set_enable(true, 0).is_ok());
        assert!(config.set_enable(false, 1).is_ok());
        config.set_data(0b1010);
        assert!(config.set_backlight(true).is_ok());

        assert_eq!(config.bits(), 0b10101101);
        assert_eq!(
            DualHD44780_PCF8574TAdapter::<I2cMock, NoopDelay>::default_i2c_address(),
            0x27
        );

        config.set_rs(false);
        config.set_rw(false);
        assert!(config.set_enable(false, 0).is_ok());
        assert!(config.set_enable(true, 1).is_ok());
        config.set_data(0b0101);
        assert!(config.set_backlight(false).is_ok());

        assert_eq!(config.bits(), 0b01010010);
        config.i2c().done();
    }

    #[test]
    fn test_dual_hd44780_adapter_write_bits_to_gpio() {
        let expected_transactions = [I2cTransaction::write(0x27, std::vec![0b10100011])];
        let mut config = DualHD44780_PCF8574TAdapter::new(
            DeviceSetupConfig {
                i2c: I2cMock::new(&expected_transactions),
                address: 0x27,
                lcd_type: LcdDisplayType::Lcd40x4,
                delay: NoopDelay,
            },
        );
        config.set_rs(true);
        config.set_rw(false);
        assert!(config.set_enable(false, 0).is_ok());
        assert!(config.set_enable(true, 1).is_ok());
        config.set_data(0b1010);
        assert!(config.set_backlight(false).is_ok());
        config.i2c().done();
    }

    #[test]
    fn test_row_to_controller_row() {
        let mut config = DualHD44780_PCF8574TAdapter::new(
            DeviceSetupConfig {
                i2c: I2cMock::new(&[]),
                address: 0x27,
                lcd_type: LcdDisplayType::Lcd40x4,
                delay: NoopDelay,
            },
        );
        assert_eq!(config.row_to_controller_row(0), (0, 0));
        assert_eq!(config.row_to_controller_row(1), (0, 1));
        assert_eq!(config.row_to_controller_row(2), (1, 0));
        assert_eq!(config.row_to_controller_row(3), (1, 1));
        config.i2c().done();
    }
}
