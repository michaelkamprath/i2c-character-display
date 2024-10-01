//! This Rust `embedded-hal`-based library is a simple way to control a [HD44780](https://en.wikipedia.org/wiki/Hitachi_HD44780_LCD_controller)
//! compatible character display with an "I2C backpack" interface in an embedded, `no_std` environment. A number of I2C backpack interfaces
//! are supported:
//!
//! - **[Adafruit I2C/SPI LCD Backpack](https://www.adafruit.com/product/292)** - This is a simple I2C backpack that can be used with either I2C
//! or SPI. It is available from Adafruit and other retailers. This library only supports the I2C interface.
//! - **PCF8574-based I2C adapter** - These adapters are ubiquitous on eBay and AliExpress and have no clear branding. The most common pin
//! wiring uses 4 data pins and 3 control pins. Most models have the display 4-bit data pins connected to P4-P7 of the PCF8574. This library
//! supports that configuration, though it would be straightforward to add support for other configurations.
//!
//! Key features include:
//! - Convenient high-level API for controlling the display
//! - Support for custom characters
//! - Backlight control
//! - `core::fmt::Write` implementation for easy use with the `write!` macro
//! - Compatible with the `embedded-hal` traits v1.0 and later
//!
//! ## Usage
//! Add this to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! i2c-character-display = { version = "0.1", features = ["defmt"] }
//! ```
//! The `features = ["defmt"]` line is optional and enables the `defmt` feature, which allows the library's errors to be used with the `defmt` logging
//! framework. Then select the appropriate adapter for your display:
//! ```rust
//! use i2c_character_display::{AdafruitLCDBackpack, CharacterDisplayPCF8574T, LcdDisplayType};
//! use embedded_hal::delay::DelayMs;
//! use embedded_hal::i2c::I2c;
//!
//! // board setup
//! let i2c = ...; // I2C peripheral
//! let delay = ...; // DelayMs implementation
//!
//! // It is recommended that the `i2c` object be wrapped in an `embedded_hal_bus::i2c::CriticalSectionDevice` so that it can be shared between
//! // multiple peripherals.
//!
//! // Adafruit backpack
//! let mut lcd = AdafruitLCDBackpack::new(i2c, LcdDisplayType::Lcd16x2, delay);
//! // PCF8574T adapter
//! let mut lcd = CharacterDisplayPCF8574T::new(i2c, LcdDisplayType::Lcd16x2, delay);
//! ```
//! When creating the display object, you can choose the display type from the `LcdDisplayType` enum. The display type should match the physical
//! display you are using. This display type configures the number of rows and columns, and the internal row offsets for the display.
//!
//! Initialize the display:
//! ```rust
//! if let Err(e) = lcd.init() {
//!    panic!("Error initializing LCD: {}", e);
//! }
//! ```
//! Use the display:
//! ```rust
//! // set up the display
//! lcd.backlight(true)?.clear()?.home()?;
//! // print a message
//! lcd.print("Hello, world!")?;
//! // can also use the `core::fmt::write!` macro
//! use core::fmt::Write;
//!
//! write!(lcd, "Hello, world!")?;
//! ```
//! The various methods for controlling the LCD are also available. Each returns a `Result` that wraps the display object in `Ok()`, allowing for easy chaining
//! of commands. For example:
//! ```rust
//! lcd.backlight(true)?.clear()?.home()?.print("Hello, world!")?;
//! ```
//!
#![no_std]
#![allow(dead_code, non_camel_case_types, non_upper_case_globals)]
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

mod adapter_config;

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

pub struct BaseCharacterDisplay<I2C, DELAY, BITS> {
    lcd_type: LcdDisplayType,
    i2c: I2C,
    address: u8,
    bits: BITS,
    delay: DELAY,
    display_function: u8,
    display_control: u8,
    display_mode: u8,
}

pub type CharacterDisplayPCF8574T<I2C, DELAY> =
    BaseCharacterDisplay<I2C, DELAY, adapter_config::GenericPCF8574TConfig<I2C>>;
pub type AdafruitLCDBackpack<I2C, DELAY> =
    BaseCharacterDisplay<I2C, DELAY, adapter_config::AdafruitLCDBackpackConfig<I2C>>;

impl<I2C, DELAY, BITS> BaseCharacterDisplay<I2C, DELAY, BITS>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    BITS: adapter_config::AdapterConfigTrait<I2C>,
{
    pub fn new(i2c: I2C, lcd_type: LcdDisplayType, delay: DELAY) -> Self {
        Self::new_with_address(i2c, BITS::default_i2c_address(), lcd_type, delay)
    }

    pub fn new_with_address(i2c: I2C, address: u8, lcd_type: LcdDisplayType, delay: DELAY) -> Self {
        Self {
            lcd_type,
            i2c,
            address,
            bits: BITS::default(),
            delay,
            display_function: LCD_FLAG_4BITMODE | LCD_FLAG_5x8_DOTS | LCD_FLAG_2LINE,
            display_control: LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF,
            display_mode: LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT,
        }
    }

    pub fn init(&mut self) -> Result<(), Error<I2C>> {
        self.bits
            .init(&mut self.i2c, self.address)
            .map_err(Error::I2cError)?;

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

    /// returns a reference to the I2C peripheral. mostly needed for testing
    fn i2c(&mut self) -> &mut I2C {
        &mut self.i2c
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
        self.bits
            .write_bits_to_gpio(&mut self.i2c, self.address)
            .map_err(Error::I2cError)?;
        self.delay.delay_us(1);
        self.bits.set_enable(0);
        self.bits
            .write_bits_to_gpio(&mut self.i2c, self.address)
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
        self.bits
            .write_bits_to_gpio(&mut self.i2c, self.address)
            .map_err(Error::I2cError)?;
        Ok(self)
    }
}

/// Implement the `core::fmt::Write` trait for the LCD backpack, allowing it to be used with the `write!` macro.
impl<I2C, DELAY, BITS> core::fmt::Write for BaseCharacterDisplay<I2C, DELAY, BITS>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    BITS: adapter_config::AdapterConfigTrait<I2C>,
{
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        if let Err(_error) = self.print(s) {
            return Err(core::fmt::Error);
        }
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
    fn test_character_display_pcf8574t_init() {
        let i2c_address = 0x27_u8;
        let expected_i2c_transactions = std::vec![
            // the PCF8574T has no adapter init sequence, so nothing to prepend
            // the LCD init sequence
            // write low nibble of 0x03 3 times
            I2cTransaction::write(i2c_address, std::vec![0b0011_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0011_0000]), // low nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0011_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0011_0000]), // low nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0011_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0011_0000]), // low nibble, rw=0, enable=0
            // write high nibble of 0x02 one time
            I2cTransaction::write(i2c_address, std::vec![0b0010_0100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0010_0000]), // high nibble, rw=0, enable=0
            // turn on the backlight
            // I2cTransaction::write(i2c_address, std::vec![0b0000_1000]),    // backlight on
            // LCD_CMD_FUNCTIONSET | LCD_FLAG_4BITMODE | LCD_FLAG_5x8_DOTS | LCD_FLAG_2LINE
            // = 0x20 | 0x00 | 0x00 | 0x08 = 0x28
            I2cTransaction::write(i2c_address, std::vec![0b0010_1100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0010_1000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1000_1100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1000_1000]), // low nibble, rw=0, enable=0
            // LCD_CMD_DISPLAYCONTROL | LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF
            // = 0x08 | 0x04 | 0x00 | 0x00 = 0x0C
            I2cTransaction::write(i2c_address, std::vec![0b0000_1100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_1000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1100_1100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1100_1000]), // low nibble, rw=0, enable=0
            // LCD_CMD_ENTRYMODESET | LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT
            // = 0x04 | 0x02 | 0x00 = 0x06
            I2cTransaction::write(i2c_address, std::vec![0b0000_1100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_1000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0110_1100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0110_1000]), // low nibble, rw=0, enable=0
            // LCD_CMD_CLEARDISPLAY
            // = 0x01
            I2cTransaction::write(i2c_address, std::vec![0b0000_1100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_1000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0001_1100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0001_1000]), // low nibble, rw=0, enable=0
            // LCD_CMD_RETURNHOME
            // = 0x02
            I2cTransaction::write(i2c_address, std::vec![0b0000_1100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_1000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0010_1100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0010_1000]), // low nibble, rw=0, enable=0
        ];

        let i2c = I2cMock::new(&expected_i2c_transactions);
        let mut lcd = CharacterDisplayPCF8574T::new(i2c, LcdDisplayType::Lcd16x2, NoopDelay::new());
        let result = lcd.init();
        assert!(result.is_ok());

        // finish the i2c mock
        lcd.i2c().done();
    }

    #[test]
    fn test_adafruit_lcd_backpack_init() {
        let i2c_address = 0x20_u8;
        let expected_i2c_transactions = std::vec![
            // the Adafruit Backpack need to init the adapter IC first
            // write 0x00 to the MCP23008 IODIR register to set all pins as outputs
            I2cTransaction::write(i2c_address, std::vec![0x00, 0x00]),
            // the LCD init sequence
            // write low nibble of 0x03 3 times
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0011_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0011_000]), // low nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0011_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0011_000]), // low nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0011_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0011_000]), // low nibble, rw=0, enable=0
            // write high nibble of 0x02 one time
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0010_100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0010_000]), // high nibble, rw=0, enable=0
            // turn on the backlight
            // I2cTransaction::write(i2c_address, std::vec![0b0000_1000]),    // backlight on
            // LCD_CMD_FUNCTIONSET | LCD_FLAG_4BITMODE | LCD_FLAG_5x8_DOTS | LCD_FLAG_2LINE
            // = 0x20 | 0x00 | 0x00 | 0x08 = 0x28
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0010_100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0010_000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_1000_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_1000_000]), // low nibble, rw=0, enable=0
            // LCD_CMD_DISPLAYCONTROL | LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF
            // = 0x08 | 0x04 | 0x00 | 0x00 = 0x0C
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0000_100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0000_000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_1100_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_1100_000]), // low nibble, rw=0, enable=0
            // LCD_CMD_ENTRYMODESET | LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT
            // = 0x04 | 0x02 | 0x00 = 0x06
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0000_100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0000_000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0110_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0110_000]), // low nibble, rw=0, enable=0
            // LCD_CMD_CLEARDISPLAY
            // = 0x01
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0000_100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0000_000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0001_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0001_000]), // low nibble, rw=0, enable=0
            // LCD_CMD_RETURNHOME
            // = 0x02
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0000_100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0000_000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0010_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0010_000]), // low nibble, rw=0, enable=0
        ];

        let i2c = I2cMock::new(&expected_i2c_transactions);
        let mut lcd = AdafruitLCDBackpack::new(i2c, LcdDisplayType::Lcd16x2, NoopDelay::new());
        let result = lcd.init();
        assert!(result.is_ok());

        // finish the i2c mock
        lcd.i2c().done();
    }
}
