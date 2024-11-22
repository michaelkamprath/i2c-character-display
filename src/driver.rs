pub mod hd44780;

use embedded_hal::{delay::DelayNs, i2c};

use crate::{CharacterDisplayError, DeviceSetupConfig, LcdDisplayType};


pub trait DriverTrait<I2C, DELAY>: Default
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    /// returns the default I2C address for the device
    fn default_i2c_address() -> u8;

    /// returns whether reads are supported by the device
    fn supports_reads() -> bool;

    /// Initialize the display
    fn init(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Clear the display
    fn clear(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the cursor to the home position.
    fn home(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the cursor position at specified column and row. Columns and rows are zero-indexed.
    fn set_cursor(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        col: u8,
        row: u8,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the cursor visibility.
    fn show_cursor(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        show_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the cursor blinking.
    fn blink_cursor(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        blink_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the display visibility.
    fn show_display(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        show_display: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Scroll display left.
    fn scroll_left(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Scroll display right.
    fn scroll_right(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the text flow direction to left to right.
    fn left_to_right(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the text flow direction to right to left.
    fn right_to_left(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the auto scroll mode.
    fn autoscroll(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        autoscroll: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Prints a string to the LCD at the current cursor position of the active device.
    fn print(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        text: &str,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the backlight on or off
    fn backlight(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        on: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// creates a new custom character
    fn create_char(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<(), CharacterDisplayError<I2C>>;

}
