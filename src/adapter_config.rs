pub mod adafruit_lcd_backpack;
pub mod dual_hd44780;
pub mod generic_pcf8574t;

use core::fmt::Display;

use crate::LcdDisplayType;
use embedded_hal::i2c;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AdapterError<I2C>
where
    I2C: i2c::I2c,
{
    /// The device ID was not recognized
    BadDeviceId,
    /// An I2C error occurred
    I2CError(I2C::Error),
}

#[cfg(feature = "defmt")]
impl<I2C> defmt::Format for AdapterError<I2C>
where
    I2C: i2c::I2c,
{
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            AdapterError::BadDeviceId => defmt::write!(fmt, "BadDeviceId"),
            AdapterError::I2CError(_) => defmt::write!(fmt, "I2CError"),
        }
    }
}

#[cfg(feature = "ufmt")]
impl<I2C> ufmt::uDisplay for AdapterError<I2C>
where
    I2C: i2c::I2c,
{
    fn fmt<W>(&self, w: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        match self {
            AdapterError::BadDeviceId => ufmt::uwrite!(w, "BadDeviceId"),
            AdapterError::I2CError(_) => ufmt::uwrite!(w, "I2CError"),
        }
    }
}

impl<I2C> Display for AdapterError<I2C>
where
    I2C: i2c::I2c,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AdapterError::BadDeviceId => write!(f, "BadDeviceId"),
            AdapterError::I2CError(_) => write!(f, "I2CError"),
        }
    }
}

pub trait AdapterConfigTrait<I2C>: Default
where
    I2C: i2c::I2c,
{
    /// Returns the bitfield value for the adapter
    fn bits(&self) -> u8;

    /// Returns the default I2C address for the adapter
    fn default_i2c_address() -> u8;

    /// Determines if reading from device is supported by this adapter
    fn supports_reads() -> bool {
        false
    }

    /// Sets the RS pin for the display. A value of `false` indicates an instruction is being sent, while
    /// a value of `true` indicates data is being sent.
    fn set_rs(&mut self, value: bool);

    /// Sets the RW pin for the display. A value of `false` indicates a write operation, while a value of
    /// `true` indicates a read operation. Not all displays support reading, so this method may not be
    /// implemented fully.
    fn set_rw(&mut self, value: bool);

    /// Sets the enable pin for the given device. Most displays only have one enable pin, so the device
    /// parameter is ignored. For displays with two enable pins, the device parameter is used to determine
    /// which enable pin to set.
    fn set_enable(&mut self, value: bool, device: usize) -> Result<(), AdapterError<I2C>>;

    /// Sets the backlight pin for the display. A value of `true` indicates the backlight is on, while a value
    /// of `false` indicates the backlight is off.
    fn set_backlight(&mut self, value: bool);

    fn set_data(&mut self, value: u8);

    fn init(&self, _i2c: &mut I2C, _i2c_address: u8) -> Result<(), I2C::Error> {
        Ok(())
    }

    fn write_bits_to_gpio(&self, i2c: &mut I2C, i2c_address: u8) -> Result<(), AdapterError<I2C>> {
        let data = [self.bits()];
        i2c.write(i2c_address, &data)
            .map_err(AdapterError::I2CError)?;
        Ok(())
    }

    /// writes a full byte to the indicated device. If `rs_setting` is `true`, the data is written to the data register,
    /// either the CGRAM or DDRAM, depending on prior command sent. If `rs_setting` is `false`, the data is written to
    /// command register.
    fn write_byte_to_device(
        &mut self,
        i2c: &mut I2C,
        i2c_address: u8,
        device: usize,
        rs_setting: bool,
        value: u8,
    ) -> Result<(), AdapterError<I2C>> {
        self.write_nibble_to_device(i2c, i2c_address, device, rs_setting, value >> 4)
            .and_then(|_| {
                self.write_nibble_to_device(i2c, i2c_address, device, rs_setting, value & 0x0F)
            })
    }

    /// writes the lower nibble of a `value` byte to the indicated device. Typically only used for device initialization in 4 bit mode.
    /// If `rs_setting` is `true`, the data is written to the data register,
    /// either the CGRAM or DDRAM, depending on prior command sent. If `rs_setting` is `false`, the data is written to
    /// command register.
    fn write_nibble_to_device(
        &mut self,
        i2c: &mut I2C,
        i2c_address: u8,
        device: usize,
        rs_setting: bool,
        value: u8,
    ) -> Result<(), AdapterError<I2C>> {
        self.set_rs(rs_setting);
        self.set_rw(false);

        // now write the low nibble
        self.set_data(value & 0x0F);
        self.set_enable(true, device)?;
        self.write_bits_to_gpio(i2c, i2c_address)?;
        self.set_enable(false, device)?;
        self.write_bits_to_gpio(i2c, i2c_address)?;

        Ok(())
    }

    /// read bytes from the indicated device. The size of the buffer is the number of bytes to read.
    /// What is read depends on the `rs_setting` parameter. If `rs_setting` is `true`, the data is read
    /// from the data register, either the CGRAM or DDRAM, depending on prior command sent. If `rs_setting`
    /// is `false`, the data is read from the busy flag and address register.
    /// Note that while nothing "breaks" passing a buffer size greater than one when `rs_setting` is `false`,
    /// the data returned will be the same for each byte read.
    fn read_bytes_from_device(
        &self,
        _i2c: &mut I2C,
        _i2c_address: u8,
        _device: usize,
        _rs_setting: bool,
        _buffer: &mut [u8],
    ) -> Result<(), AdapterError<I2C>> {
        unimplemented!("Reads are not supported for device");
    }

    fn is_busy(&self, _i2c: &mut I2C, _i2c_address: u8) -> Result<bool, AdapterError<I2C>> {
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
