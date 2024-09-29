#![no_std]
#![allow(dead_code, non_camel_case_types, non_upper_case_globals)]
use bitfield::bitfield;
use embedded_hal::{delay::DelayNs, i2c};

// commands
const LCD_CMD_CLEARDISPLAY: u8 = 0x01; //  Clear display, set cursor position to zero
const LCD_CMD_RETURNHOME: u8 = 0x02; //  Set cursor position to zero
const LCD_CMD_ENTRYMODESET: u8 = 0x04; //  Sets the entry mode
const LCD_CMD_DISPLAYCONTROL: u8 = 0x08; //  Controls the display; does stuff like turning it off and on
const LCD_CMD_CURSORSHIFT: u8 = 0x10; //  Lets you move the cursor
const LCD_CMD_FUNCTIONSET: u8 = 0x20; //  Used to send the function to set to the display
const LCD_CMD_SETCGRAMADDR: u8 = 0x40; //  Used to set the CGRAM (character generator RAM) with characters
const LCD_CMD_SETDDRAMADDR: u8 = 0x80; //  Used to set the DDRAM (Display Data RAM)

// flags for display entry mode
const LCD_FLAG_ENTRYRIGHT: u8 = 0x00; //  Used to set text to flow from right to left
const LCD_FLAG_ENTRYLEFT: u8 = 0x02; //  Uset to set text to flow from left to right
const LCD_FLAG_ENTRYSHIFTINCREMENT: u8 = 0x01; //  Used to 'right justify' text from the cursor
const LCD_FLAG_ENTRYSHIFTDECREMENT: u8 = 0x00; //  Used to 'left justify' text from the cursor

// flags for display on/off control
const LCD_FLAG_DISPLAYON: u8 = 0x04; //  Turns the display on
const LCD_FLAG_DISPLAYOFF: u8 = 0x00; //  Turns the display off
const LCD_FLAG_CURSORON: u8 = 0x02; //  Turns the cursor on
const LCD_FLAG_CURSOROFF: u8 = 0x00; //  Turns the cursor off
const LCD_FLAG_BLINKON: u8 = 0x01; //  Turns on the blinking cursor
const LCD_FLAG_BLINKOFF: u8 = 0x00; //  Turns off the blinking cursor

// flags for display/cursor shift
const LCD_FLAG_DISPLAYMOVE: u8 = 0x08; //  Flag for moving the display
const LCD_FLAG_CURSORMOVE: u8 = 0x00; //  Flag for moving the cursor
const LCD_FLAG_MOVERIGHT: u8 = 0x04; //  Flag for moving right
const LCD_FLAG_MOVELEFT: u8 = 0x00; //  Flag for moving left

// flags for function set
const LCD_FLAG_8BITMODE: u8 = 0x10; //  LCD 8 bit mode
const LCD_FLAG_4BITMODE: u8 = 0x00; //  LCD 4 bit mode
const LCD_FLAG_2LINE: u8 = 0x08; //  LCD 2 line mode
const LCD_FLAG_1LINE: u8 = 0x00; //  LCD 1 line mode
const LCD_FLAG_5x10_DOTS: u8 = 0x04; //  10 pixel high font mode
const LCD_FLAG_5x8_DOTS: u8 = 0x00; //  8 pixel high font mode

bitfield! {
    pub struct LCDBits(u8);
    impl Debug;
    impl BitAnd;
    pub rs, set_rs: 0, 0;
    pub rw, set_rw: 1, 1;
    pub enable, set_enable: 2, 2;
    pub backlight, set_backlight: 3, 3;
    pub data, set_data: 7, 4;
}

/// Errors that can occur when using the LCD backpack
pub enum Error<I2C>
where
    I2C: i2c::I2c,
{
    /// I2C error returned from the underlying I2C implementation
    I2cError(I2C::Error),
    /// Row is out of range
    RowOutOfRange,
    /// Column is out of range
    ColumnOutOfRange,
    /// Formatting error
    FormattingError(core::fmt::Error),
}

impl<I2C> From<core::fmt::Error> for Error<I2C>
where
    I2C: i2c::I2c,
{
    fn from(err: core::fmt::Error) -> Self {
        Error::FormattingError(err)
    }
}

#[cfg(feature = "defmt")]
impl<I2C> defmt::Format for Error<I2C>
where
    I2C: i2c::I2c,
{
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Error::I2cError(_e) => defmt::write!(fmt, "I2C error"),
            Error::RowOutOfRange => defmt::write!(fmt, "Row out of range"),
            Error::ColumnOutOfRange => defmt::write!(fmt, "Column out of range"),
            Error::FormattingError(_e) => defmt::write!(fmt, "Formatting error"),
        }
    }
}
/// The type of LCD display. This is used to determine the number of rows and columns, and the row offsets.
pub enum LcdDisplayType {
    /// 20x4 display
    Lcd20x4,
    /// 20x2 display
    Lcd20x2,
    /// 16x2 display
    Lcd16x2,
    /// 16x4 display
    Lcd16x4,
    /// 8x2 display
    Lcd8x2,
    /// 40x2 display
    Lcd40x2,
}

impl LcdDisplayType {
    /// Get the number of rows for the display type
    const fn rows(&self) -> u8 {
        match self {
            LcdDisplayType::Lcd20x4 => 4,
            LcdDisplayType::Lcd20x2 => 2,
            LcdDisplayType::Lcd16x2 => 2,
            LcdDisplayType::Lcd16x4 => 4,
            LcdDisplayType::Lcd8x2 => 2,
            LcdDisplayType::Lcd40x2 => 2,
        }
    }

    /// Get the number of columns for the display type
    const fn cols(&self) -> u8 {
        match self {
            LcdDisplayType::Lcd20x4 => 20,
            LcdDisplayType::Lcd20x2 => 20,
            LcdDisplayType::Lcd16x2 => 16,
            LcdDisplayType::Lcd16x4 => 16,
            LcdDisplayType::Lcd8x2 => 8,
            LcdDisplayType::Lcd40x2 => 40,
        }
    }

    /// Get the row offsets for the display type. This always returns an array of length 4.
    /// For displays with less than 4 rows, the unused rows will be set to offsets offscreen.
    const fn row_offsets(&self) -> [u8; 4] {
        match self {
            LcdDisplayType::Lcd20x4 => [0x00, 0x40, 0x14, 0x54],
            LcdDisplayType::Lcd20x2 => [0x00, 0x40, 0x00, 0x40],
            LcdDisplayType::Lcd16x2 => [0x00, 0x40, 0x10, 0x50],
            LcdDisplayType::Lcd16x4 => [0x00, 0x40, 0x10, 0x50],
            LcdDisplayType::Lcd8x2 => [0x00, 0x40, 0x00, 0x40],
            LcdDisplayType::Lcd40x2 => [0x00, 0x40, 0x00, 0x40],
        }
    }
}

pub struct CharacterDisplay<I2C, DELAY> {
    lcd_type: LcdDisplayType,
    i2c: I2C,
    address: u8,
    bits: LCDBits,
    delay: DELAY,
    display_function: u8,
    display_control: u8,
    display_mode: u8,
}

impl<I2C, DELAY> CharacterDisplay<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    pub fn new(i2c: I2C, lcd_type: LcdDisplayType, delay: DELAY) -> Self {
        Self::new_with_address(i2c, 0x27, lcd_type, delay)
    }

    pub fn new_with_address(i2c: I2C, address: u8, lcd_type: LcdDisplayType, delay: DELAY) -> Self {
        Self {
            lcd_type,
            i2c,
            address,
            bits: LCDBits(0),
            delay,
            display_function: LCD_FLAG_4BITMODE | LCD_FLAG_5x8_DOTS | LCD_FLAG_2LINE,
            display_control: LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF,
            display_mode: LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT,
        }
    }

    pub fn init(&mut self) -> Result<(), Error<I2C>> {
        // Put LCD into 4 bit mode, device starts in 8 bit mode
        self.write_4_bits(0x03)?;
        self.delay.delay_ms(5);
        self.write_4_bits(0x03)?;
        self.delay.delay_ms(5);
        self.write_4_bits(0x03)?;
        self.delay.delay_us(150);
        self.write_4_bits(0x02)?;

        // set up the display
        self.bits.set_backlight(1);
        self.send_command(LCD_CMD_FUNCTIONSET | self.display_function)?;
        self.send_command(LCD_CMD_DISPLAYCONTROL | self.display_control)?;
        self.send_command(LCD_CMD_ENTRYMODESET | self.display_mode)?;
        self.clear()?.home()?;
        Ok(())
    }

    fn send_command(&mut self, command: u8) -> Result<(), Error<I2C>> {
        self.bits.set_rs(0);
        self.write_8_bits(command)?;
        Ok(())
    }

    fn write_data(&mut self, data: u8) -> Result<(), Error<I2C>> {
        self.bits.set_rs(1);
        self.write_8_bits(data)?;
        Ok(())
    }

    fn write_8_bits(&mut self, value: u8) -> Result<(), Error<I2C>> {
        self.write_4_bits(value >> 4)?;
        self.write_4_bits(value & 0x0F)?;
        Ok(())
    }
    fn write_4_bits(&mut self, value: u8) -> Result<(), Error<I2C>> {
        self.bits.set_data(value & 0x0F);
        self.bits.set_rw(0);
        self.bits.set_enable(1);
        self.i2c
            .write(self.address, &[self.bits.0])
            .map_err(Error::I2cError)?;
        self.delay.delay_us(1);
        self.bits.set_enable(0);
        self.i2c
            .write(self.address, &[self.bits.0])
            .map_err(Error::I2cError)?;
        self.delay.delay_us(1);
        Ok(())
    }

    //--------------------------------------------------------------------------------------------------
    // high level commands, for the user!
    //--------------------------------------------------------------------------------------------------

    /// Clear the display
    pub fn clear(&mut self) -> Result<&mut Self, Error<I2C>> {
        self.send_command(LCD_CMD_CLEARDISPLAY)?;
        self.delay.delay_ms(2);
        Ok(self)
    }

    /// Set the cursor to the home position
    pub fn home(&mut self) -> Result<&mut Self, Error<I2C>> {
        self.send_command(LCD_CMD_RETURNHOME)?;
        self.delay.delay_ms(2);
        Ok(self)
    }

    /// Set the cursor position at specified column and row
    pub fn set_cursor(&mut self, col: u8, row: u8) -> Result<&mut Self, Error<I2C>> {
        if row >= self.lcd_type.rows() {
            return Err(Error::RowOutOfRange);
        }
        if col >= self.lcd_type.cols() {
            return Err(Error::ColumnOutOfRange);
        }

        self.send_command(
            LCD_CMD_SETDDRAMADDR | (col + self.lcd_type.row_offsets()[row as usize]),
        )?;
        Ok(self)
    }

    /// Set the cursor visibility
    pub fn show_cursor(&mut self, show_cursor: bool) -> Result<&mut Self, Error<I2C>> {
        if show_cursor {
            self.display_control |= LCD_FLAG_CURSORON;
        } else {
            self.display_control &= !LCD_FLAG_CURSORON;
        }
        self.send_command(LCD_CMD_DISPLAYCONTROL | self.display_control)?;
        Ok(self)
    }

    /// Set the cursor blinking
    pub fn blink_cursor(&mut self, blink_cursor: bool) -> Result<&mut Self, Error<I2C>> {
        if blink_cursor {
            self.display_control |= LCD_FLAG_BLINKON;
        } else {
            self.display_control &= !LCD_FLAG_BLINKON;
        }
        self.send_command(LCD_CMD_DISPLAYCONTROL | self.display_control)?;
        Ok(self)
    }

    /// Set the display visibility
    pub fn show_display(&mut self, show_display: bool) -> Result<&mut Self, Error<I2C>> {
        if show_display {
            self.display_control |= LCD_FLAG_DISPLAYON;
        } else {
            self.display_control &= !LCD_FLAG_DISPLAYON;
        }
        self.send_command(LCD_CMD_DISPLAYCONTROL | self.display_control)?;
        Ok(self)
    }

    /// Scroll the display to the left
    pub fn scroll_display_left(&mut self) -> Result<&mut Self, Error<I2C>> {
        self.send_command(LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVELEFT)?;
        Ok(self)
    }

    /// Scroll the display to the right
    pub fn scroll_display_right(&mut self) -> Result<&mut Self, Error<I2C>> {
        self.send_command(LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVERIGHT)?;
        Ok(self)
    }

    /// Set the text flow direction to left to right
    pub fn left_to_right(&mut self) -> Result<&mut Self, Error<I2C>> {
        self.display_mode |= LCD_FLAG_ENTRYLEFT;
        self.send_command(LCD_CMD_ENTRYMODESET | self.display_mode)?;
        Ok(self)
    }

    /// Set the text flow direction to right to left
    pub fn right_to_left(&mut self) -> Result<&mut Self, Error<I2C>> {
        self.display_mode &= !LCD_FLAG_ENTRYLEFT;
        self.send_command(LCD_CMD_ENTRYMODESET | self.display_mode)?;
        Ok(self)
    }

    /// Set the auto scroll mode
    pub fn autoscroll(&mut self, autoscroll: bool) -> Result<&mut Self, Error<I2C>> {
        if autoscroll {
            self.display_mode |= LCD_FLAG_ENTRYSHIFTINCREMENT;
        } else {
            self.display_mode &= !LCD_FLAG_ENTRYSHIFTINCREMENT;
        }
        self.send_command(LCD_CMD_ENTRYMODESET | self.display_mode)?;
        Ok(self)
    }

    /// Create a new custom character
    pub fn create_char(&mut self, location: u8, charmap: [u8; 8]) -> Result<&mut Self, Error<I2C>> {
        self.send_command(LCD_CMD_SETCGRAMADDR | ((location & 0x7) << 3))?;
        for &charmap_byte in charmap.iter() {
            self.write_data(charmap_byte)?;
        }
        Ok(self)
    }

    /// Prints a string to the LCD at the current cursor position
    pub fn print(&mut self, text: &str) -> Result<&mut Self, Error<I2C>> {
        for c in text.chars() {
            self.write_data(c as u8)?;
        }
        Ok(self)
    }

    /// Turn the backlight on or off
    pub fn backlight(&mut self, on: bool) -> Result<&mut Self, Error<I2C>> {
        self.bits.set_backlight(on as u8);
        self.i2c
            .write(self.address, &[self.bits.0])
            .map_err(Error::I2cError)?;
        Ok(self)
    }
}

/// Implement the `core::fmt::Write` trait for the LCD backpack, allowing it to be used with the `write!` macro.
impl<I2C, DELAY> core::fmt::Write for CharacterDisplay<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        if let Err(_error) = self.print(s) {
            return Err(core::fmt::Error);
        }
        Ok(())
    }
}
