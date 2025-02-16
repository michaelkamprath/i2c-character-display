use embedded_hal::{delay::DelayNs, i2c};
use crate::{
    driver::{DisplayActionsTrait, DeviceHardwareTrait},
    CharacterDisplayError,
};

// commands
pub const LCD_CMD_CLEARDISPLAY: u8 = 0x01; //  Clear display, set cursor position to zero
pub const LCD_CMD_RETURNHOME: u8 = 0x02; //  Set cursor position to zero
pub const LCD_CMD_ENTRYMODESET: u8 = 0x04; //  Sets the entry mode
pub const LCD_CMD_DISPLAYCONTROL: u8 = 0x08; //  Controls the display; does stuff like turning it off and on
pub const LCD_CMD_CURSORSHIFT: u8 = 0x10; //  Lets you move the cursor
pub const LCD_CMD_FUNCTIONSET: u8 = 0x20; //  Used to send the function to set to the display
pub const LCD_CMD_SETCGRAMADDR: u8 = 0x40; //  Used to set the CGRAM (character generator RAM) with characters
pub const LCD_CMD_SETDDRAMADDR: u8 = 0x80; //  Used to set the DDRAM (Display Data RAM)

// flags for display entry mode
pub const LCD_FLAG_ENTRYRIGHT: u8 = 0x00; //  Used to set text to flow from right to left
pub const LCD_FLAG_ENTRYLEFT: u8 = 0x02; //  Uset to set text to flow from left to right
pub const LCD_FLAG_ENTRYSHIFTINCREMENT: u8 = 0x01; //  Used to 'right justify' text from the cursor
pub const LCD_FLAG_ENTRYSHIFTDECREMENT: u8 = 0x00; //  Used to 'left justify' text from the cursor

// flags for display on/off control
pub const LCD_FLAG_DISPLAYON: u8 = 0x04; //  Turns the display on
pub const LCD_FLAG_DISPLAYOFF: u8 = 0x00; //  Turns the display off
pub const LCD_FLAG_CURSORON: u8 = 0x02; //  Turns the cursor on
pub const LCD_FLAG_CURSOROFF: u8 = 0x00; //  Turns the cursor off
pub const LCD_FLAG_BLINKON: u8 = 0x01; //  Turns on the blinking cursor
pub const LCD_FLAG_BLINKOFF: u8 = 0x00; //  Turns off the blinking cursor

// flags for display/cursor shift
pub const LCD_FLAG_DISPLAYMOVE: u8 = 0x08; //  Flag for moving the display
pub const LCD_FLAG_CURSORMOVE: u8 = 0x00; //  Flag for moving the cursor
pub const LCD_FLAG_MOVERIGHT: u8 = 0x04; //  Flag for moving right
pub const LCD_FLAG_MOVELEFT: u8 = 0x00; //  Flag for moving left

// flags for function set
pub const LCD_FLAG_2LINE: u8 = 0x08; //  LCD 2 line mode
pub const LCD_FLAG_1LINE: u8 = 0x00; //  LCD 1 line mode
pub const LCD_FLAG_5x10_DOTS: u8 = 0x04; //  10 pixel high font mode
pub const LCD_FLAG_5x8_DOTS: u8 = 0x00; //  8 pixel high font mode


/// `StandardActionsHandler`` is a struct that implements the `DisplayActionsTrait` trait. Most of the
/// character displays use a standard set of commands to control the display. This struct implements
/// for those standard commands.
pub struct StandardCharacterDisplayHandler {
    display_function: u8,
    display_control: u8,
    display_mode: u8,
}


impl Default for StandardCharacterDisplayHandler
{
    fn default() -> Self {
        StandardCharacterDisplayHandler {
            display_function: 0,
            display_control: 0,
            display_mode: 0,
        }
    }
}

impl<I2C, DELAY, DEVICE> DisplayActionsTrait<I2C, DELAY, DEVICE> for StandardCharacterDisplayHandler
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    DEVICE: DeviceHardwareTrait<I2C, DELAY>,
{
    fn init_display_state(
        &mut self,
        display_function: u8,
        display_control: u8,
        display_mode: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.display_function = display_function;
        self.display_control = display_control;
        self.display_mode = display_mode;
        Ok(())
    }

    fn clear(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        device.write_bytes(false, &[LCD_CMD_CLEARDISPLAY])?;
        // wait for command to complete
        device.delay().delay_us(1530);
        Ok(())
    }

    fn home(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        device.write_bytes(false, &[LCD_CMD_RETURNHOME])?;
        // wait for command to complete
        device.delay().delay_us(1530);
        Ok(())
    }

    fn set_cursor(
        &mut self,
        device: &mut DEVICE,
        col: u8,
        row: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if row >= device.lcd_type().rows() {
            return Err(CharacterDisplayError::RowOutOfRange);
        }
        if col >= device.lcd_type().cols() {
            return Err(CharacterDisplayError::ColumnOutOfRange);
        }

        device.write_bytes(
            false,
            &[LCD_CMD_SETDDRAMADDR | (col + device.lcd_type().row_offsets()[row as usize])],
        )?;
        // wait for command to complete
        device.delay().delay_us(39);
        Ok(())
    }

    fn show_cursor(
        &mut self,
        device: &mut DEVICE,
        show_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if show_cursor {
            self.display_control |= LCD_FLAG_CURSORON;
        } else {
            self.display_control &= !LCD_FLAG_CURSORON;
        }
        device.write_bytes(false, &[LCD_CMD_DISPLAYCONTROL | self.display_control])?;
        // wait for command to complete
        device.delay().delay_us(39);
        Ok(())
    }

    fn blink_cursor(
        &mut self,
        device: &mut DEVICE,
        blink_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if blink_cursor {
            self.display_control |= LCD_FLAG_BLINKON;
        } else {
            self.display_control &= !LCD_FLAG_BLINKON;
        }
        device.write_bytes(false, &[LCD_CMD_DISPLAYCONTROL | self.display_control])?;
        // wait for command to complete
        device.delay().delay_us(39);
        Ok(())
    }

    fn show_display(
        &mut self,
        device: &mut DEVICE,
        show_display: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if show_display {
            self.display_control |= LCD_FLAG_DISPLAYON;
        } else {
            self.display_control &= !LCD_FLAG_DISPLAYON;
        }
        device.write_bytes(false, &[LCD_CMD_DISPLAYCONTROL | self.display_control])?;
        // wait for command to complete
        device.delay().delay_us(39);
        Ok(())
    }

    fn scroll_left(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        device.write_bytes(false, &[LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVELEFT])?;
        // wait for command to complete
        device.delay().delay_us(39);
        Ok(())
    }

    fn scroll_right(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        device.write_bytes(false, &[LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVERIGHT])?;
        // wait for command to complete
        device.delay().delay_us(39);
        Ok(())
    }

    fn left_to_right(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        // TODO revisit this function's logic
        self.display_mode |= LCD_FLAG_ENTRYLEFT;
        device.write_bytes(
            false,
            &[LCD_CMD_ENTRYMODESET | self.display_mode],
        )?;
        // wait for command to complete
        device.delay().delay_us(39);
        Ok(())
    }

    fn right_to_left(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        // TODO revisit this function's logic
        self.display_mode |= LCD_FLAG_ENTRYRIGHT;
        device.write_bytes(
            false,
            &[LCD_CMD_ENTRYMODESET | self.display_mode],
        )?;
        // wait for command to complete
        device.delay().delay_us(39);
        Ok(())
    }

    fn autoscroll(
        &mut self,
        device: &mut DEVICE,
        autoscroll: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if autoscroll {
            self.display_mode |= LCD_FLAG_ENTRYSHIFTINCREMENT;
        } else {
            self.display_mode &= !LCD_FLAG_ENTRYSHIFTINCREMENT;
        }
        device.write_bytes(
            false,
            &[LCD_CMD_ENTRYMODESET | self.display_mode],
        )?;
        // wait for command to complete
        device.delay().delay_us(39);
        Ok(())
    }

    fn print(
        &mut self,
        device: &mut DEVICE,
        text: &str,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        #[cfg(feature = "defmt")]
        defmt::debug!("Printing: {}", text);
        device.write_bytes(true, text.as_bytes())?;
        #[cfg(feature = "defmt")]
        defmt::debug!("Printed ... now waiting");
        // wait for command to complete
        device.delay().delay_us(43);
        #[cfg(feature = "defmt")]
        defmt::debug!("done waiting");
        Ok(())
    }

    fn backlight(
        &mut self,
        _device: &mut DEVICE,
        _on: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        #[cfg(feature = "defmt")]
        defmt::warn!("Backlight not supported");
        Err(CharacterDisplayError::UnsupportedOperation)
    }

    fn create_char(
        &mut self,
        device: &mut DEVICE,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        device.write_bytes(false, &[LCD_CMD_SETCGRAMADDR | ((location & 0x7) << 3)])?;
        device.write_bytes(true, &charmap)?;
        // wait for command to complete
        device.delay().delay_us(39);
        Ok(())
    }

    /// Read the device data into the buffer.
    /// This function is not supported by the AIP31068 driver.
    fn read_device_data(
        &self,
        _device: &mut DEVICE,
        _buffer: &mut [u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        Err(CharacterDisplayError::UnsupportedOperation)
    }

    /// Read the address counter.
    /// This function is not supported by the AIP31068 driver.
    fn read_address_counter(
        &mut self,
        _device: &mut DEVICE,
    ) -> Result<u8, CharacterDisplayError<I2C>> {
        Err(CharacterDisplayError::UnsupportedOperation)
    }
}