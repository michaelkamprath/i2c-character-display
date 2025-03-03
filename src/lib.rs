//! This Rust `embedded-hal`-based library is a simple way to control a character display that has either a [HD44780](https://en.wikipedia.org/wiki/Hitachi_HD44780_LCD_controller)
//! or [AiP31068](https://support.newhavendisplay.com/hc/en-us/articles/4414486901783--AiP31068) controller with an I2C interface
//! in an embedded, `no_std` environment. A number of I2C interfaces are supported:
//!
//! - **[Adafruit I2C/SPI LCD Backpack](https://www.adafruit.com/product/292)** - This is a simple I2C adapter for HD44780 character displays that can be used with either I2C
//!   or SPI. It is available from Adafruit and other retailers. This library only supports the I2C interface of this adapter.
//! - **PCF8574-based I2C adapter** - These adapters are ubiquitous on eBay and AliExpress and have no clear branding. Furthermore, some HD44780-based character
//!   display makers, such as [Surenoo](https://www.surenoo.com), integrate a PCF8574T directly on the display board enabling I2C connections without a seperate adapter.
//!   The most common pin wiring uses 4 data pins and 3 control pins. Most models have the display's 4-bit mode data pins connected to P4-P7 of the PCF8574.
//!   This library supports that configuration, though it would be straightforward to add support for other pin configurations.
//! - **AiP31068** - This is a character display controller with a built-in I2C support. The command set is similar to the HD44780, but the controller
//!   operates in 8-bit mode and is initialized differently.  Examples of displays that use this controller include the [Surenoo SLC1602O](https://www.surenoo.com/products/8109143).
//! - **ST7032i** - This is an I2C character display controller used with LCD displays. It is similar to the HD44780, but with some differences in the command set.
//!   Examples of displays that use this controller include the [Surenoo SLC1602K3](https://www.surenoo.com/collections/81733622/products/8131705).
//!
//! Key features include:
//! - Convenient high-level API for controlling many types of character display
//! - Support for custom characters
//! - Backlight control on hardwarware that supports it
//! - `core::fmt::Write` implementation for easy use with the `write!` macro
//! - Compatible with the `embedded-hal` traits v1.0 and later
//! - Support for character displays that uses multiple HD44780 drivers, such as the 40x4 display
//! - Optional support for the `defmt` and `ufmt` logging frameworks
//! - Optional support for reading from the display on controllers and adapters that support it
//!
//! ## Usage
//! Add this to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! i2c-character-display = { version = "0.4", features = ["defmt"] }
//! ```
//! The `features = ["defmt"]` line is optional and enables the `defmt` feature, which allows the library's errors to be used with the `defmt` logging
//! framework. Another optional feature is `features = ["ufmt"]`, which enables the `ufmt` feature, allowing the `uwriteln!` and `uwrite!` macros to be used.
//!
//! Then select the appropriate adapter for your display:
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
//! // Adafruit backpack for a single HD44780 controller
//! let mut lcd = AdafruitLCDBackpack::new(i2c, LcdDisplayType::Lcd16x2, delay);
//! // PCF8574T adapter for a single HD44780 controller
//! let mut lcd = CharacterDisplayPCF8574T::new(i2c, LcdDisplayType::Lcd16x2, delay);
//! // Character display with dual HD44780 controllers using a single PCF8574T I2C adapter
//! let mut lcd = CharacterDisplayDualHD44780::new(i2c, LcdDisplayType::Lcd40x4, delay);
//! // Character display with the AiP31068 controller
//! let mut lcd = CharacterDisplayAIP31068::new(i2c, LcdDisplayType::Lcd16x2, delay);
//! // Character display with the ST7032i controller
//! let mut lcd = CharacterDisplayST7032i::new(i2c, LcdDisplayType::Lcd16x2, delay);
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
//! The optional `ufmt` feature enables the `ufmt` crate, which allows the `uwriteln!` and `uwrite!` macros to be used with the display:
//! ```rust
//! use ufmt::uwriteln;
//!
//! uwriteln!(lcd, "Hello, world!")?;
//! ```
//!
//! The various methods for controlling the LCD are also available. Each returns a `Result` that wraps the display object in `Ok()`, allowing for easy chaining
//! of commands. For example:
//! ```rust
//! lcd.backlight(true)?.clear()?.home()?.print("Hello, world!")?;
//! ```
//! ### Reading from the display
//! Some I2C adapters support reading data from the HD44780 controller. For the I2C adapters that support it, the `read_device_data` method can be used to read
//! from either the CGRAM or DDRAM at the current cursor position. The `read_address_counter` method can be used to read the address counter from the HD44780 controller.
//! In both cases, the specific meaning of the data depends on the prior commands sent to the display. See the HD44780 datasheet for more information.
//!
//! ### Backlight control
//! All HD44780 controllers support backlight control. The `backlight` method can be used to turn the backlight on or off. The AiP31068 controller does not support
//! backlight control, and calling the `backlight` method with a AiP31068 controller will return an error.
//!
//! ### Multiple HD44780 controller character displays
//! Some character displays, such as the 40x4 display, use two HD44780 controllers to drive the display. This library supports these displays by
//! treating them as one logical display with multiple HD44780 controllers. The `CharacterDisplayDualHD44780` type is used to control these displays.
//! Use the various methods to control the display as you would with a single HD44780 controller display. The `set_cursor` method sets the active HD44780
//! controller device based on the row number you select.
//!
#![no_std]
#![allow(dead_code, non_camel_case_types, non_upper_case_globals)]
use core::{fmt::Display, marker::PhantomData};

use embedded_hal::{delay::DelayNs, i2c};

/// HD44780 based character display using a generic PCF8574T I2C adapter.
pub type CharacterDisplayPCF8574T<I2C, DELAY> = BaseCharacterDisplay<
    I2C,
    DELAY,
    crate::driver::hd44780::adapter::generic_pcf8574t::GenericPCF8574TAdapter<I2C, DELAY>,
    crate::driver::hd44780::GenericHD44780PCF8574T<I2C, DELAY>,
>;

/// HD44780 based character display using an Adafruit I2C/SPI LCD backpack adapter.
pub type AdafruitLCDBackpack<I2C, DELAY> = BaseCharacterDisplay<
    I2C,
    DELAY,
    crate::driver::hd44780::adapter::adafruit_lcd_backpack::AdafruitLCDBackpackAdapter<I2C, DELAY>,
    crate::driver::hd44780::AdafruitLCDBackpack<I2C, DELAY>,
>;

/// Character display using dual HD44780 I2C drivers connected using a generic PCF8574T I2C adapter with a pinout that
/// has two enable pins, one for each HD44780 driver. Typically used for 40x4 character displays.
pub type CharacterDisplayDualHD44780<I2C, DELAY> = BaseCharacterDisplay<
    I2C,
    DELAY,
    crate::driver::hd44780::adapter::dual_controller_pcf8574t::DualHD44780_PCF8574TAdapter<
        I2C,
        DELAY,
    >,
    crate::driver::hd44780::DualHD44780PCF8574T<I2C, DELAY>,
>;

/// Character display using the AIP31068 controller with built-in I2C adapter.
pub type CharacterDisplayAIP31068<I2C, DELAY> = BaseCharacterDisplay<
    I2C,
    DELAY,
    crate::driver::aip31068::AIP31068<I2C, DELAY>,
    crate::driver::standard::StandardCharacterDisplayHandler,
>;

/// Character display using the ST7032i controller with built-in I2C adapter.
pub type CharacterDisplayST7032i<I2C, DELAY> = BaseCharacterDisplay<
    I2C,
    DELAY,
    crate::driver::st7032i::ST7032i<I2C, DELAY>,
    crate::driver::st7032i::ST7032iDisplayActions<I2C, DELAY>,
>;

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

mod driver;

const MAX_DEVICE_COUNT: usize = 2;

#[derive(Debug, PartialEq, Copy, Clone)]
/// Errors that can occur when using the LCD backpack
pub enum CharacterDisplayError<I2C>
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
    /// The discplay type is not compatible with specific adapter.
    UnsupportedDisplayType,
    /// The requested operation is not supported by the adapter or controller
    UnsupportedOperation,
    /// Read operation is not supported by the adapter
    ReadNotSupported,
    /// Internal error - bad device ID
    BadDeviceId,
    /// Internal error - buffer too small
    BufferTooSmall,
}

impl<I2C> From<core::fmt::Error> for CharacterDisplayError<I2C>
where
    I2C: i2c::I2c,
{
    fn from(err: core::fmt::Error) -> Self {
        CharacterDisplayError::FormattingError(err)
    }
}

impl<I2C> From<&CharacterDisplayError<I2C>> for &'static str
where
    I2C: i2c::I2c,
{
    fn from(err: &CharacterDisplayError<I2C>) -> Self {
        match err {
            CharacterDisplayError::I2cError(_) => "I2C error",
            CharacterDisplayError::RowOutOfRange => "Row out of range",
            CharacterDisplayError::ColumnOutOfRange => "Column out of range",
            CharacterDisplayError::FormattingError(_) => "Formatting error",
            CharacterDisplayError::UnsupportedDisplayType => "Unsupported display type",
            CharacterDisplayError::UnsupportedOperation => "Unsupported operation",
            CharacterDisplayError::ReadNotSupported => "Read operation not supported",
            CharacterDisplayError::BadDeviceId => "Bad device ID",
            CharacterDisplayError::BufferTooSmall => "Buffer too small",
        }
    }
}

#[cfg(feature = "defmt")]
impl<I2C> defmt::Format for CharacterDisplayError<I2C>
where
    I2C: i2c::I2c,
{
    fn format(&self, fmt: defmt::Formatter) {
        let msg: &'static str = From::from(self);
        defmt::write!(fmt, "{}", msg);
    }
}

#[cfg(feature = "ufmt")]
impl<I2C> ufmt::uDisplay for CharacterDisplayError<I2C>
where
    I2C: i2c::I2c,
{
    fn fmt<W>(&self, w: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let msg: &'static str = From::from(self);
        ufmt::uwrite!(w, "{}", msg)
    }
}

impl<I2C> Display for CharacterDisplayError<I2C>
where
    I2C: i2c::I2c,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg: &'static str = From::from(self);
        write!(f, "{}", msg)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
    /// 40x4 display. Should be used with a DualHD44780 adapter.
    Lcd40x4,
}

impl From<&LcdDisplayType> for &'static str {
    fn from(display_type: &LcdDisplayType) -> Self {
        match display_type {
            LcdDisplayType::Lcd20x4 => "20x4",
            LcdDisplayType::Lcd20x2 => "20x2",
            LcdDisplayType::Lcd16x2 => "16x2",
            LcdDisplayType::Lcd16x4 => "16x4",
            LcdDisplayType::Lcd8x2 => "8x2",
            LcdDisplayType::Lcd40x2 => "40x2",
            LcdDisplayType::Lcd40x4 => "40x4",
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for LcdDisplayType {
    fn format(&self, fmt: defmt::Formatter) {
        let msg: &'static str = From::from(self);
        defmt::write!(fmt, "{}", msg);
    }
}

#[cfg(feature = "ufmt")]
impl ufmt::uDisplay for LcdDisplayType {
    fn fmt<W>(&self, w: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let msg: &'static str = From::from(self);
        ufmt::uwrite!(w, "{}", msg)
    }
}

impl Display for LcdDisplayType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg: &'static str = From::from(self);
        write!(f, "{}", msg)
    }
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
            LcdDisplayType::Lcd40x4 => 4,
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
            LcdDisplayType::Lcd40x4 => 40,
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
            LcdDisplayType::Lcd40x4 => [0x00, 0x40, 0x00, 0x40],
        }
    }
}

pub struct DeviceSetupConfig<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    lcd_type: LcdDisplayType,
    i2c: I2C,
    address: u8,
    delay: DELAY,
}

pub struct BaseCharacterDisplay<I2C, DELAY, DEVICE, ACTIONS>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    DEVICE: driver::DeviceHardwareTrait<I2C, DELAY>,
    ACTIONS: driver::DisplayActionsTrait<I2C, DELAY, DEVICE>,
{
    device: DEVICE,
    actions: ACTIONS,
    _phantom_i2c: PhantomData<I2C>,
    _phantom_delay: PhantomData<DELAY>,
}

impl<I2C, DELAY, DEVICE, ACTIONS> BaseCharacterDisplay<I2C, DELAY, DEVICE, ACTIONS>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    DEVICE: driver::DeviceHardwareTrait<I2C, DELAY>,
    ACTIONS: driver::DisplayActionsTrait<I2C, DELAY, DEVICE>,
{
    /// Create a new character display object with the default I2C address for the adapter.
    pub fn new(i2c: I2C, lcd_type: LcdDisplayType, delay: DELAY) -> Self {
        Self::new_with_address(i2c, DEVICE::default_i2c_address(), lcd_type, delay)
    }

    /// Create a new character display object with a specific I2C address for the adapter.
    pub fn new_with_address(i2c: I2C, address: u8, lcd_type: LcdDisplayType, delay: DELAY) -> Self {
        let config = DeviceSetupConfig {
            lcd_type,
            i2c,
            address,
            delay,
        };
        Self {
            device: DEVICE::new(config),
            actions: ACTIONS::default(),
            _phantom_i2c: PhantomData,
            _phantom_delay: PhantomData,
        }
    }

    /// Initialize the display. This must be called before using the display.
    pub fn init(&mut self) -> Result<(), CharacterDisplayError<I2C>> {
        let (display_function, display_control, display_mode) = self.device.init()?;
        self.actions
            .init_display_state(display_function, display_control, display_mode)?;
        Ok(())
    }

    /// returns a reference to the I2C peripheral. mostly needed for testing
    fn i2c(&mut self) -> &mut I2C {
        self.device.i2c()
    }

    /// returns the `LcdDisplayType` used to create the display
    pub fn display_type(&self) -> LcdDisplayType {
        self.device.lcd_type()
    }

    /// Supports the ability to read from the display.
    pub fn supports_reads() -> bool {
        DEVICE::supports_reads()
    }

    // /// Writes a data byte to the display. Normally users do not need to call this directly.
    // /// For multiple devices, this writes the data to the currently active contoller device.
    // fn write_data(&mut self, data: u8) -> Result<&mut Self, CharacterDisplayError<I2C>> {
    //     self.device.write_data(&mut self.config, data)?;
    //     Ok(self)
    // }

    /// Reads into the buffer data from the display device either the CGRAM or DDRAM at the current cursor position.
    /// For multiple devices, this reads from the currently active device as set by the cursor position.
    /// The amount of data read is determined by the length of the buffer.
    /// Not all adapters support reads from the device. This will return an error if the adapter
    /// does not support reads.
    pub fn read_device_data(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.read_device_data(&mut self.device, buffer)?;

        Ok(self)
    }

    /// Reads the address counter from the display device. The ready bit is masked off.
    /// Not all adapters support reads from the device. This will return an error if the adapter
    /// does not support reads.
    pub fn read_address_counter(&mut self) -> Result<u8, CharacterDisplayError<I2C>> {
        self.actions.read_address_counter(&mut self.device)
    }

    //--------------------------------------------------------------------------------------------------
    // high level commands, for the user!
    //--------------------------------------------------------------------------------------------------

    /// Clear the display
    pub fn clear(&mut self) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.clear(&mut self.device)?;
        Ok(self)
    }

    /// Set the cursor to the home position.
    pub fn home(&mut self) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.home(&mut self.device)?;
        Ok(self)
    }

    /// Set the cursor position at specified column and row. Columns and rows are zero-indexed.
    pub fn set_cursor(
        &mut self,
        col: u8,
        row: u8,
    ) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.set_cursor(&mut self.device, col, row)?;
        Ok(self)
    }

    /// Set the cursor visibility.
    pub fn show_cursor(
        &mut self,
        show_cursor: bool,
    ) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.show_cursor(&mut self.device, show_cursor)?;
        Ok(self)
    }

    /// Set the cursor blinking.
    pub fn blink_cursor(
        &mut self,
        blink_cursor: bool,
    ) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.blink_cursor(&mut self.device, blink_cursor)?;
        Ok(self)
    }

    /// Set the display visibility.
    pub fn show_display(
        &mut self,
        show_display: bool,
    ) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.show_display(&mut self.device, show_display)?;
        Ok(self)
    }

    /// Scroll the display to the left.
    pub fn scroll_display_left(&mut self) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.scroll_left(&mut self.device)?;
        Ok(self)
    }

    /// Scroll the display to the right.
    pub fn scroll_display_right(&mut self) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.scroll_right(&mut self.device)?;
        Ok(self)
    }

    /// Set the text flow direction to left to right.
    pub fn left_to_right(&mut self) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.left_to_right(&mut self.device)?;
        Ok(self)
    }

    /// Set the text flow direction to right to left.
    pub fn right_to_left(&mut self) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.right_to_left(&mut self.device)?;
        Ok(self)
    }

    /// Set the auto scroll mode.
    pub fn autoscroll(
        &mut self,
        autoscroll: bool,
    ) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.autoscroll(&mut self.device, autoscroll)?;
        Ok(self)
    }

    /// Create a new custom character.
    pub fn create_char(
        &mut self,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions
            .create_char(&mut self.device, location, charmap)?;
        Ok(self)
    }

    /// Prints a string to the LCD at the current cursor position of the active device.
    pub fn print(&mut self, text: &str) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.print(&mut self.device, text)?;
        Ok(self)
    }

    /// Turn the backlight on or off.
    /// Note that the AIP31068 controller does not support backlight control.
    pub fn backlight(&mut self, on: bool) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.backlight(&mut self.device, on)?;
        Ok(self)
    }

    /// Set the contrast level of the display. This is only supported by the ST7032i controller.
    pub fn set_contrast(&mut self, contrast: u8) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.actions.set_contrast(&mut self.device, contrast)?;
        Ok(self)
    }
}

/// Implement the `core::fmt::Write` trait, allowing it to be used with the `write!` macro.
/// This is a convenience method for printing to the display. For multi-device, this will print to the active device as set by
/// `set_cursor`.
impl<I2C, DELAY, DEVICE, ACTIONS> core::fmt::Write
    for BaseCharacterDisplay<I2C, DELAY, DEVICE, ACTIONS>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    DEVICE: driver::DeviceHardwareTrait<I2C, DELAY>,
    ACTIONS: driver::DisplayActionsTrait<I2C, DELAY, DEVICE>,
{
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        if let Err(_e) = self.print(s) {
            return Err(core::fmt::Error);
        }
        Ok(())
    }
}

#[cfg(feature = "ufmt")]
/// Implement the `ufmt::uWrite` trait, allowing it to be used with the `uwriteln!` and `uwrite!` macros.
/// This is a convenience method for printing to the display. For multi-device, this will print to the active device as set by
/// `set_cursor`.
impl<I2C, DELAY, DEVICE, ACTIONS> ufmt::uWrite for BaseCharacterDisplay<I2C, DELAY, DEVICE, ACTIONS>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    DEVICE: driver::DeviceHardwareTrait<I2C, DELAY>,
    ACTIONS: driver::DisplayActionsTrait<I2C, DELAY, DEVICE>,
{
    fn write_str(&mut self, s: &str) -> Result<(), CharacterDisplayError<I2C>> {
        if let Err(e) = self.print(s) {
            return Err(e);
        }
        Ok(())
    }

    type Error = CharacterDisplayError<I2C>;
}

#[cfg(test)]
mod lib_tests {
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
            // I2cTransaction::write(i2c_address, std::vec![0b0000_1000]),    // backlight on
            // LCD_CMD_FUNCTIONSET | LCD_FLAG_4BITMODE | LCD_FLAG_5x8_DOTS | LCD_FLAG_2LINE
            // = 0x20 | 0x00 | 0x00 | 0x08 = 0x28
            I2cTransaction::write(i2c_address, std::vec![0b0010_0100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0010_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1000_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1000_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_DISPLAYCONTROL | LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF
            // = 0x08 | 0x04 | 0x00 | 0x00 = 0x0C
            I2cTransaction::write(i2c_address, std::vec![0b0000_0100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1100_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1100_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_ENTRYMODESET | LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT
            // = 0x04 | 0x02 | 0x00 = 0x06
            I2cTransaction::write(i2c_address, std::vec![0b0000_0100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0110_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0110_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_CLEARDISPLAY
            // = 0x01
            I2cTransaction::write(i2c_address, std::vec![0b0000_0100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0001_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0001_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_RETURNHOME
            // = 0x02
            I2cTransaction::write(i2c_address, std::vec![0b0000_0100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0010_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0010_0000]), // low nibble, rw=0, enable=0
            // Set Backlight
            I2cTransaction::write(i2c_address, std::vec![0b0010_1000]), // backlight on
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
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0010_100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0010_000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_1000_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_1000_000]), // low nibble, rw=0, enable=0
            // LCD_CMD_DISPLAYCONTROL | LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF
            // = 0x08 | 0x04 | 0x00 | 0x00 = 0x0C
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0000_100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0000_000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_1100_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_1100_000]), // low nibble, rw=0, enable=0
            // LCD_CMD_ENTRYMODESET | LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT
            // = 0x04 | 0x02 | 0x00 = 0x06
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0000_100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0000_000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0110_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0110_000]), // low nibble, rw=0, enable=0
            // LCD_CMD_CLEARDISPLAY
            // = 0x01
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0000_100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0000_000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0001_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0001_000]), // low nibble, rw=0, enable=0
            // LCD_CMD_RETURNHOME
            // = 0x02
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0000_100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0000_000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0010_100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b0_0010_000]), // low nibble, rw=0, enable=0
            // Set Backlight
            I2cTransaction::write(i2c_address, std::vec![0x09, 0b1_0010_000]), // backlight on
        ];

        let i2c = I2cMock::new(&expected_i2c_transactions);
        let mut lcd = AdafruitLCDBackpack::new(i2c, LcdDisplayType::Lcd16x2, NoopDelay::new());

        let result = lcd.init();
        assert!(result.is_ok());
        assert!(lcd.display_type() == LcdDisplayType::Lcd16x2);

        // finish the i2c mock
        lcd.i2c().done();
    }

    #[test]
    fn test_character_display_dual_hd44780_init() {
        let i2c_address = 0x27_u8;
        let expected_i2c_transactions = std::vec![
            // the PCF8574T has no adapter init sequence, so nothing to prepend
            // *** Device 0 ***
            // the LCD init sequence for device 0
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
            I2cTransaction::write(i2c_address, std::vec![0b0010_0100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0010_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1000_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1000_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_DISPLAYCONTROL | LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF
            // = 0x08 | 0x04 | 0x00 | 0x00 = 0x0C
            I2cTransaction::write(i2c_address, std::vec![0b0000_0100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1100_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1100_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_ENTRYMODESET | LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT
            // = 0x04 | 0x02 | 0x00 = 0x06
            I2cTransaction::write(i2c_address, std::vec![0b0000_0100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0110_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0110_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_CLEARDISPLAY
            // = 0x01
            I2cTransaction::write(i2c_address, std::vec![0b0000_0100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0001_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0001_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_RETURNHOME
            // = 0x02
            I2cTransaction::write(i2c_address, std::vec![0b0000_0100]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0010_0100]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0010_0000]), // low nibble, rw=0, enable=0
            // *** Device 1 ***
            // the LCD init sequence for device 0
            // write low nibble of 0x03 3 times
            I2cTransaction::write(i2c_address, std::vec![0b0011_0010]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0011_0000]), // low nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0011_0010]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0011_0000]), // low nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0011_0010]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0011_0000]), // low nibble, rw=0, enable=0
            // write high nibble of 0x02 one time
            I2cTransaction::write(i2c_address, std::vec![0b0010_0010]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0010_0000]), // high nibble, rw=0, enable=0
            // turn on the backlight
            // I2cTransaction::write(i2c_address, std::vec![0b0000_1000]),    // backlight on
            // LCD_CMD_FUNCTIONSET | LCD_FLAG_4BITMODE | LCD_FLAG_5x8_DOTS | LCD_FLAG_2LINE
            // = 0x20 | 0x00 | 0x00 | 0x08 = 0x28
            I2cTransaction::write(i2c_address, std::vec![0b0010_0010]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0010_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1000_0010]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1000_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_DISPLAYCONTROL | LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF
            // = 0x08 | 0x04 | 0x00 | 0x00 = 0x0C
            I2cTransaction::write(i2c_address, std::vec![0b0000_0010]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1100_0010]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1100_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_ENTRYMODESET | LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT
            // = 0x04 | 0x02 | 0x00 = 0x06
            I2cTransaction::write(i2c_address, std::vec![0b0000_0010]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0110_0010]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0110_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_CLEARDISPLAY
            // = 0x01
            I2cTransaction::write(i2c_address, std::vec![0b0000_0010]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0001_0010]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0001_0000]), // low nibble, rw=0, enable=0
            // LCD_CMD_RETURNHOME
            // = 0x02
            I2cTransaction::write(i2c_address, std::vec![0b0000_0010]), // high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0010_0010]), // low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0010_0000]), // low nibble, rw=0, enable=0
            // Set Backlight
            I2cTransaction::write(i2c_address, std::vec![0b0010_1000]), // backlight on
        ];

        let i2c = I2cMock::new(&expected_i2c_transactions);
        let mut lcd =
            CharacterDisplayDualHD44780::new(i2c, LcdDisplayType::Lcd40x4, NoopDelay::new());
        let result = lcd.init();
        assert!(result.is_ok());
        assert!(lcd.display_type() == LcdDisplayType::Lcd40x4);

        // finish the i2c mock
        lcd.i2c().done();
    }
}
