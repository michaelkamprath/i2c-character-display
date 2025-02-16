use bitfield::bitfield;
use core::marker::PhantomData;
use embedded_hal::{delay::DelayNs, i2c};

use crate::{
    driver::DeviceHardwareTrait, CharacterDisplayError, DeviceSetupConfig, LcdDisplayType,
};

use super::HD44780AdapterTrait;

bitfield! {
    pub struct AdafruitLCDBackpackBitField(u8);
    impl Debug;
    impl BitAnd;
    pub rs, set_rs: 1, 1;
    pub enable, set_enable: 2, 2;
    pub backlight, set_backlight: 7, 7;
    pub data, set_data: 6, 3;
}

impl Clone for AdafruitLCDBackpackBitField {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

/// AdaFruit branded HD44780 I2C adapter based on the MCP23008 I2C GPIO expander
pub struct AdafruitLCDBackpackAdapter<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    bits: AdafruitLCDBackpackBitField,
    config: DeviceSetupConfig<I2C, DELAY>,
    _marker: PhantomData<I2C>,
}

impl<I2C, DELAY> DeviceHardwareTrait<I2C, DELAY> for AdafruitLCDBackpackAdapter<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn new(config: DeviceSetupConfig<I2C, DELAY>) -> Self {
        Self {
            bits: AdafruitLCDBackpackBitField(0),
            config: config,
            _marker: PhantomData,
        }
    }

    fn default_i2c_address() -> u8 {
        0x20
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

    fn init(&mut self) -> Result<(u8, u8, u8), CharacterDisplayError<I2C>> {
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

impl<I2C, DELAY> HD44780AdapterTrait<I2C, DELAY> for AdafruitLCDBackpackAdapter<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn device_config(&mut self) -> &mut DeviceSetupConfig<I2C, DELAY> {
        &mut self.config
    }

    fn is_supported(display_type: LcdDisplayType) -> bool {
        display_type != LcdDisplayType::Lcd40x4
    }

    fn hardware_init(&mut self) -> Result<(), I2C::Error> {
        // Set the MCP23008 IODIR register to output
        self.config.i2c.write(self.config.address, &[0x00, 0x00])?;
        Ok(())
    }

    fn bits(&self) -> u8 {
        self.bits.0
    }

    fn set_rs(&mut self, value: bool) {
        self.bits.set_rs(value as u8);
    }

    fn set_rw(&mut self, _value: bool) {
        // adafruit backpack doesn't use RW
    }

    fn set_enable(
        &mut self,
        value: bool,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if controller != 0 {
            return Err(CharacterDisplayError::BadDeviceId);
        }
        self.bits.set_enable(value as u8);
        Ok(())
    }

    fn set_backlight(&mut self, value: bool) -> Result<(), CharacterDisplayError<I2C>> {
        self.bits.set_backlight(value as u8);
        self.write_bits_to_gpio()
    }

    fn set_data(&mut self, value: u8) {
        self.bits.set_data(value);
    }

    fn write_bits_to_gpio(&mut self) -> Result<(), CharacterDisplayError<I2C>> {
        // first byte is GPIO register address
        let data = [0x09, self.bits()];
        self.config
            .i2c
            .write(self.config.address, &data)
            .map_err(CharacterDisplayError::I2cError)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use embedded_hal_mock::eh1::{
        delay::NoopDelay,
        i2c::{Mock as I2cMock, Transaction as I2cTransaction},
    };

    #[test]
    fn test_adafruit_lcd_backpack_adapter() {
        let mut config = AdafruitLCDBackpackAdapter::new(DeviceSetupConfig {
            i2c: I2cMock::new(&[
                I2cTransaction::write(0x20, std::vec![0x09, 0b1_1010_110]),
                I2cTransaction::write(0x20, std::vec![0x09, 0b0_0101_000]),
            ]),
            address: 0x20,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        });
        config.set_rs(true);
        assert!(config.set_enable(true, 2).is_err());
        assert!(config.set_enable(true, 0).is_ok());
        config.set_data(0b1010);
        assert!(config.set_backlight(true).is_ok());

        // adafruit backpack doesn't use RW

        assert_eq!(config.bits(), 0b11010110);
        assert_eq!(
            AdafruitLCDBackpackAdapter::<I2cMock, NoopDelay>::default_i2c_address(),
            0x20
        );

        config.set_rs(false);
        assert!(config.set_enable(false, 0).is_ok());
        config.set_data(0b0101);
        assert!(config.set_backlight(false).is_ok());

        assert_eq!(config.bits(), 0b00101000);
        config.i2c().done();
    }

    #[test]
    fn test_adafruit_lcd_backpack_config_write_bits_to_gpio() {
        let expected_transactions = [I2cTransaction::write(0x20, std::vec![0x09, 0b11010110])];
        let mut config = AdafruitLCDBackpackAdapter::new(DeviceSetupConfig {
            i2c: I2cMock::new(&expected_transactions),
            address: 0x20,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        });
        config.set_rs(true);
        assert!(config.set_enable(true, 0).is_ok());
        config.set_data(0b1010);
        assert!(config.set_backlight(true).is_ok());
        config.i2c().done();
    }

    #[test]
    fn test_adafruit_init() {
        let expected_transactions = [I2cTransaction::write(0x20, std::vec![0x00, 0x00])];
        let mut config = AdafruitLCDBackpackAdapter::new(DeviceSetupConfig {
            i2c: I2cMock::new(&expected_transactions),
            address: 0x20,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        });

        config.hardware_init().unwrap();
        config.i2c().done();
    }
}
