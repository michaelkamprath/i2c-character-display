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

impl Clone for GenericPCF8574TBitField {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

#[derive(Clone)]
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

    fn supports_reads() -> bool {
        true
    }

    fn read_from_gpio(&self, i2c: &mut I2C, i2c_address: u8, rs_setting: bool) -> Result<u8, I2C::Error> {
        // wait for the BUSY flag to clear
        while self.is_busy(i2c, i2c_address)? {
            // wait
        }
        let mut data_read: u8;
        let mut data_buf = [0];
        // now we can read the data. Set up PCF8574T to read data
        let mut data_cntl = self.bits.clone();
        data_cntl.set_data(0b1111);
        data_cntl.set_enable(0);
        data_cntl.set_rs(rs_setting as u8);
        data_cntl.set_rw(1); // read
        i2c.write(i2c_address, &[data_cntl.0])?;
        // not that is is set up, set the enable bit to read the data
        data_cntl.set_enable(1);
        i2c.write(i2c_address, &[data_cntl.0])?;
        // read the data
        i2c.read(i2c_address, &mut data_buf)?;
        // turn off the enable bit so next nibble can be read
        data_cntl.set_enable(0);
        i2c.write(i2c_address, &[data_cntl.0])?;
        // first nibble read was high nibble, shift it to the left. using GenericPCF8574TBitField
        // to get the bit positions correct.
        data_read = GenericPCF8574TBitField(data_buf[0]).data() << 4;
        // now read the low nibble. activate the enable bit to read the low nibble
        data_cntl.set_enable(1);
        i2c.write(i2c_address, &[data_cntl.0])?;
        // read the data
        i2c.read(i2c_address, &mut data_buf)?;
        // turn off the enable bit so next nibble can be read
        data_cntl.set_enable(0);
        i2c.write(i2c_address, &[data_cntl.0])?;
        // combine the high and low nibbles
        data_read |= GenericPCF8574TBitField(data_buf[0]).data() & 0x0F;

        Ok(data_read)
    }
    fn is_busy(&self, i2c: &mut I2C, i2c_address: u8) -> Result<bool, I2C::Error> {
        // need to set all data bits to HIGH to read, per PFC8574 data sheet description of Quasi-bidirectional I/Os
        let mut setup = self.bits.clone();
        setup.set_data(0b1111);
        setup.set_rs(0);
        setup.set_rw(1);
        setup.set_enable(0);
        i2c.write(i2c_address, &[setup.0])?;
        // need two enable cycles to read the data, but the busy flag is in the 4th bit of the first
        // nibble, so we only need to read the first nibble
        setup.set_enable(1);
        i2c.write(i2c_address, &[setup.0])?;
        let mut data = [0];
        i2c.read(i2c_address, &mut data)?;
        let read_data = GenericPCF8574TBitField(data[0]);
        // turn off the enable bit so next nibble can be read
        setup.set_enable(0);
        i2c.write(i2c_address, &[setup.0])?;
        // toggle enable one more time per the 4-bit interface for the HD44780
        setup.set_enable(1);
        i2c.write(i2c_address, &[setup.0])?;
        setup.set_enable(0);
        i2c.write(i2c_address, &[setup.0])?;

        Ok(read_data.data() & 0b1000 != 0)

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
