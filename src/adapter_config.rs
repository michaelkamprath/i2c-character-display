pub mod adafruit_lcd_backpack;
pub mod dual_hd44780;
pub mod generic_pcf8574t;

use crate::LcdDisplayType;
use embedded_hal::i2c;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AdapterError {
    /// The device ID was not recognized
    BadDeviceId,
}

#[cfg(feature = "defmt")]
impl defmt::Format for AdapterError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            AdapterError::BadDeviceId => defmt::write!(fmt, "BadDeviceId"),
        }
    }
}

#[cfg(feature = "ufmt")]
impl ufmt::uDisplay for AdapterError {
    fn fmt<W>(&self, w: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        match self {
            AdapterError::BadDeviceId => ufmt::uwrite!(w, "BadDeviceId"),
        }
    }
}

pub trait AdapterConfigTrait<I2C>: Default
where
    I2C: i2c::I2c,
{
    fn bits(&self) -> u8;
    fn default_i2c_address() -> u8;
    fn supports_reads() -> bool;

    fn set_rs(&mut self, value: bool);
    fn set_rw(&mut self, value: bool);
    /// Sets the enable pin for the given device. Most displays only have one enable pin, so the device
    /// parameter is ignored. For displays with two enable pins, the device parameter is used to determine
    /// which enable pin to set.
    fn set_enable(&mut self, value: bool, device: usize) -> Result<(), AdapterError>;
    fn set_backlight(&mut self, value: bool);
    fn set_data(&mut self, value: u8);

    fn init(&self, _i2c: &mut I2C, _i2c_address: u8) -> Result<(), I2C::Error> {
        Ok(())
    }

    fn write_bits_to_gpio(&self, i2c: &mut I2C, i2c_address: u8) -> Result<(), I2C::Error> {
        let data = [self.bits()];
        i2c.write(i2c_address, &data)?;
        Ok(())
    }

    fn read_from_gpio(&self, i2c: &mut I2C, i2c_address: u8, rs_setting: bool) -> Result<u8, I2C::Error>;

    fn is_busy(&self, _i2c: &mut I2C, _i2c_address: u8) -> Result<bool, I2C::Error> {
        Ok(false)
    }

    fn device_count(&self) -> usize {
        1
    }

    /// Convert a row number to the row number on the device
    fn row_to_device_row(&self, row: u8) -> (usize, u8) {
        (0, row)
    }

    /// Determines of display type is supported by this adapter
    fn is_supported(display_type: LcdDisplayType) -> bool;
}
