pub mod adafruit_lcd_backpack;
pub mod dual_controller_pcf8574t;
pub mod generic_pcf8574t;

use crate::{
    driver::DeviceHardwareTrait, CharacterDisplayError, DeviceSetupConfig, LcdDisplayType,
};
use embedded_hal::{delay::DelayNs, i2c};

use super::{
    LCD_FLAG_5x8_DOTS, LCD_CMD_CLEARDISPLAY, LCD_CMD_DISPLAYCONTROL, LCD_CMD_ENTRYMODESET,
    LCD_CMD_FUNCTIONSET, LCD_CMD_RETURNHOME, LCD_FLAG_2LINE, LCD_FLAG_4BITMODE, LCD_FLAG_BLINKOFF,
    LCD_FLAG_CURSOROFF, LCD_FLAG_DISPLAYON, LCD_FLAG_ENTRYLEFT, LCD_FLAG_ENTRYSHIFTDECREMENT,
};

/// Trait for implementing an I2C adapter for a specific HD44780 device. Assumes the connection
/// to the HD44780 controller from the adapter is via a 4 bit interface and the adapter has
/// 8 GPIO pins available for the 4 bit data interface, RS, RW, and enable pins.
pub trait HD44780AdapterTrait<I2C, DELAY>: DeviceHardwareTrait<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn adapter_init(&mut self) -> Result<(u8, u8, u8), CharacterDisplayError<I2C>> {
        if !Self::is_supported(self.lcd_type()) {
            return Err(CharacterDisplayError::UnsupportedDisplayType);
        }

        self.hardware_init()
            .map_err(CharacterDisplayError::I2cError)?;

        let display_function: u8 = LCD_FLAG_4BITMODE | LCD_FLAG_2LINE | LCD_FLAG_5x8_DOTS;
        let display_control: u8 = LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF;
        let display_mode: u8 = LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT;

        for controller in 0..Self::controller_count() {
            if controller >= Self::max_controller_count() {
                return Err(CharacterDisplayError::BadDeviceId);
            }

            // Put LCD into 4 bit mode, device starts in 8 bit mode
            self.write_nibble_to_controller(controller, false, 0x03)?;
            self.device_config().delay.delay_ms(5);
            self.write_nibble_to_controller(controller, false, 0x03)?;
            self.device_config().delay.delay_ms(5);
            self.write_nibble_to_controller(controller, false, 0x03)?;
            self.device_config().delay.delay_us(150);
            self.write_nibble_to_controller(controller, false, 0x02)?;

            self.send_command_to_controller(controller, LCD_CMD_FUNCTIONSET | display_function)?;
            self.send_command_to_controller(controller, LCD_CMD_DISPLAYCONTROL | display_control)?;
            self.send_command_to_controller(controller, LCD_CMD_ENTRYMODESET | display_mode)?;
            self.send_command_to_controller(controller, LCD_CMD_CLEARDISPLAY)?;
            self.send_command_to_controller(controller, LCD_CMD_RETURNHOME)?;
        }
        // set up the display
        self.set_backlight(true)?;
        Ok((display_function, display_control, display_mode))
    }

    /// Returns the maximum number of controllers supported by the adapter. Most adapters only support one.
    fn max_controller_count() -> usize {
        1
    }

    fn hardware_init(&mut self) -> Result<(), I2C::Error> {
        Ok(())
    }

    fn device_config(&mut self) -> &mut DeviceSetupConfig<I2C, DELAY>;

    /// Determines of display type is supported by this adapter
    fn is_supported(display_type: LcdDisplayType) -> bool;

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
    fn set_backlight(&mut self, value: bool) -> Result<(), CharacterDisplayError<I2C>>;

    fn set_data(&mut self, value: u8);

    fn write_bits_to_gpio(&mut self) -> Result<(), CharacterDisplayError<I2C>> {
        let data = [self.bits()];
        let i2c_address = self.i2c_address();
        self.device_config()
            .i2c
            .write(i2c_address, &data)
            .map_err(CharacterDisplayError::I2cError)?;
        Ok(())
    }

    fn send_command_to_controller(
        &mut self,
        controller: usize,
        command: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.write_byte_to_controller(controller, false, command)
    }

    /// writes a full byte to the indicated controller on device. If `rs_setting` is `true`, the data is written to the data register,
    /// either the CGRAM or DDRAM, depending on prior command sent. If `rs_setting` is `false`, the data is written to
    /// command register.
    fn write_byte_to_controller(
        &mut self,
        controller: usize,
        rs_setting: bool,
        value: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.write_nibble_to_controller(controller, rs_setting, value >> 4)
            .and_then(|_| self.write_nibble_to_controller(controller, rs_setting, value & 0x0F))
    }

    /// writes the lower nibble of a `value` byte to the indicated controller on device. Typically only used for device initialization in 4 bit mode.
    /// If `rs_setting` is `true`, the data is written to the data register,
    /// either the CGRAM or DDRAM, depending on prior command sent. If `rs_setting` is `false`, the data is written to
    /// command register.
    fn write_nibble_to_controller(
        &mut self,
        controller: usize,
        rs_setting: bool,
        value: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.set_rs(rs_setting);
        self.set_rw(false);

        // now write the low nibble
        self.set_data(value & 0x0F);
        self.set_enable(true, controller)?;
        self.write_bits_to_gpio()?;
        self.set_enable(false, controller)?;
        self.write_bits_to_gpio()?;

        Ok(())
    }

    /// read bytes from the indicated controller on the device. The size of the buffer is the number of bytes to read.
    /// What is read depends on the `rs_setting` parameter. If `rs_setting` is `true`, the data is read
    /// from the data register, either the CGRAM or DDRAM, depending on prior command sent. If `rs_setting`
    /// is `false`, the data is read from the busy flag and address register.
    /// Note that while nothing "breaks" passing a buffer size greater than one when `rs_setting` is `false`,
    /// the data returned will be the same for each byte read.
    fn read_bytes_from_controller(
        &mut self,
        _controller: usize,
        _rs_setting: bool,
        _buffer: &mut [u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        unimplemented!("Reads are not supported for device");
    }

    fn is_busy(&mut self) -> Result<bool, CharacterDisplayError<I2C>> {
        Ok(false)
    }

    fn controller_count() -> usize {
        1
    }

    /// Convert a row number to the row number for associated controller.
    /// return tuple is `( controller, row )`
    fn row_to_controller_row(&self, row: u8) -> (usize, u8) {
        (0, row)
    }
}
