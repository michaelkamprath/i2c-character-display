pub mod adafruit_lcd_backpack;
pub mod dual_controller_pcf8574t;
pub mod generic_pcf8574t;

use crate::{CharacterDisplayError, LcdDisplayType};
use embedded_hal::i2c;

/// Trait for implementing an I2C adapter for a specific HD44780 device. Assumes the connection
/// to the HD44780 controller from the adapter is via a 4 bit interface and the adapter has
/// 8 GPIO pins available for the 4 bit data interface, RS, RW, and enable pins.
pub trait HD44780AdapterTrait<I2C>: Default
where
    I2C: i2c::I2c,
{
    /// Returns the default I2C address for the adapter
    fn default_i2c_address() -> u8;

    /// Determines if reading from device is supported by this adapter
    fn supports_reads() -> bool {
        false
    }

    /// Determines of display type is supported by this adapter
    fn is_supported(display_type: LcdDisplayType) -> bool;

    /// Perform adapter specific initialization.
    fn init(&self, _i2c: &mut I2C, _i2c_address: u8) -> Result<(), I2C::Error> {
        Ok(())
    }

    /// Returns the bitfield value for the adapter
    fn bits(&self) -> u8;

    /// Sets the RS pin for the display. A value of `false` indicates an instruction is being sent, while
    /// a value of `true` indicates data is being sent.
    fn set_rs(&mut self, value: bool);

    /// Sets the RW pin for the display. A value of `false` indicates a write operation, while a value of
    /// `true` indicates a read operation. Not all displays support reading, so this method may not be
    /// implemented fully.
    fn set_rw(&mut self, value: bool);

    /// Sets the enable pin for the given controller. Most displays only have one enable pin, so the controller
    /// parameter is ignored. For displays with two enable pins, the controller parameter is used to determine
    /// which enable pin to set.
    fn set_enable(
        &mut self,
        value: bool,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the backlight pin for the display. A value of `true` indicates the backlight is on, while a value
    /// of `false` indicates the backlight is off.
    fn set_backlight(&mut self, value: bool);

    fn set_data(&mut self, value: u8);

    fn write_bits_to_gpio(
        &self,
        i2c: &mut I2C,
        i2c_address: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        let data = [self.bits()];
        i2c.write(i2c_address, &data)
            .map_err(CharacterDisplayError::I2cError)?;
        Ok(())
    }

    /// writes a full byte to the indicated controller on device. If `rs_setting` is `true`, the data is written to the data register,
    /// either the CGRAM or DDRAM, depending on prior command sent. If `rs_setting` is `false`, the data is written to
    /// command register.
    fn write_byte_to_controller(
        &mut self,
        i2c: &mut I2C,
        i2c_address: u8,
        controller: usize,
        rs_setting: bool,
        value: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.write_nibble_to_controller(i2c, i2c_address, controller, rs_setting, value >> 4)
            .and_then(|_| {
                self.write_nibble_to_controller(
                    i2c,
                    i2c_address,
                    controller,
                    rs_setting,
                    value & 0x0F,
                )
            })
    }

    /// writes the lower nibble of a `value` byte to the indicated controller on device. Typically only used for device initialization in 4 bit mode.
    /// If `rs_setting` is `true`, the data is written to the data register,
    /// either the CGRAM or DDRAM, depending on prior command sent. If `rs_setting` is `false`, the data is written to
    /// command register.
    fn write_nibble_to_controller(
        &mut self,
        i2c: &mut I2C,
        i2c_address: u8,
        controller: usize,
        rs_setting: bool,
        value: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.set_rs(rs_setting);
        self.set_rw(false);

        // now write the low nibble
        self.set_data(value & 0x0F);
        self.set_enable(true, controller)?;
        self.write_bits_to_gpio(i2c, i2c_address)?;
        self.set_enable(false, controller)?;
        self.write_bits_to_gpio(i2c, i2c_address)?;

        Ok(())
    }

    /// read bytes from the indicated controller on the device. The size of the buffer is the number of bytes to read.
    /// What is read depends on the `rs_setting` parameter. If `rs_setting` is `true`, the data is read
    /// from the data register, either the CGRAM or DDRAM, depending on prior command sent. If `rs_setting`
    /// is `false`, the data is read from the busy flag and address register.
    /// Note that while nothing "breaks" passing a buffer size greater than one when `rs_setting` is `false`,
    /// the data returned will be the same for each byte read.
    fn read_bytes_from_controller(
        &self,
        _i2c: &mut I2C,
        _i2c_address: u8,
        _controller: usize,
        _rs_setting: bool,
        _buffer: &mut [u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        unimplemented!("Reads are not supported for device");
    }

    fn is_busy(
        &self,
        _i2c: &mut I2C,
        _i2c_address: u8,
    ) -> Result<bool, CharacterDisplayError<I2C>> {
        Ok(false)
    }

    fn controller_count(&self) -> usize {
        1
    }

    /// Convert a row number to the row number for associated controller.
    /// return tuple is `( controller, row )`
    fn row_to_controller_row(&self, row: u8) -> (usize, u8) {
        (0, row)
    }
}
