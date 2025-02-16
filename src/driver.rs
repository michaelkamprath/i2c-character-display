pub mod hd44780;
pub mod aip31068;
pub mod st7032i;
pub mod standard;

use embedded_hal::{delay::DelayNs, i2c};

use crate::{CharacterDisplayError, DeviceSetupConfig, LcdDisplayType};

/// Trait for device hardware implementations. Embodies the hardware-specific
/// functionality of the device driver IC. The trait is intended to be implemented
/// for the specific device driver ICs.
pub trait DeviceHardwareTrait<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn new(config: DeviceSetupConfig<I2C, DELAY>) -> Self;
    /// returns the default I2C address for the device
    fn default_i2c_address() -> u8;

    /// returns whether reads are supported by the device
    fn supports_reads() -> bool;

    /// returns LCD type
    fn lcd_type(&self) -> LcdDisplayType;

    /// returns configured i2c address
    fn i2c_address(&self) -> u8;

    /// return a immutable reference to the delay object
    fn delay(&mut self) -> &mut DELAY;

    /// returns the i2c object. mostly used for testing
    fn i2c(&mut self) -> &mut I2C;

    /// initializes the device hardware. On `Ok`, returns the initial configuration
    /// of the device. The configuration is a tuple of three bytes:
    /// (display_function, display_control, display_mode)
    fn init(
        &mut self,
    ) -> Result<(u8, u8, u8), CharacterDisplayError<I2C>>;

    fn write_bytes(
        &mut self,
        rs_setting: bool,
        data: &[u8],
    ) -> Result<(), CharacterDisplayError<I2C>>;
}

/// Trait for display actions. Embodies the display commnands that can be performed on the device.
/// Works with the `DeviceHardwareTrait` to perform the actions on the device to effect the desire
/// display operation.
pub trait DisplayActionsTrait<I2C, DELAY, DEVICE>: Default
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    DEVICE: DeviceHardwareTrait<I2C, DELAY>,
{
    /// Initialize the display state. Intended to be called once after the device is initialized
    /// and before any other operations are performed. The three parameters are the initial values
    /// and are recieved from the `DeviceHardwareTrait::init` method of the device.
    fn init_display_state(
        &mut self,
        display_function: u8,
        display_control: u8,
        display_mode: u8,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Clear the display
    fn clear(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the cursor to the home position.
    fn home(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the cursor position at specified column and row. Columns and rows are zero-indexed.
    fn set_cursor(
        &mut self,
        device: &mut DEVICE,
        col: u8,
        row: u8,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the cursor visibility.
    fn show_cursor(
        &mut self,
        device: &mut DEVICE,
        show_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the cursor blinking.
    fn blink_cursor(
        &mut self,
        device: &mut DEVICE,
        blink_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the display visibility.
    fn show_display(
        &mut self,
        device: &mut DEVICE,
        show_display: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Scroll display left.
    fn scroll_left(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Scroll display right.
    fn scroll_right(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the text flow direction to left to right.
    fn left_to_right(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the text flow direction to right to left.
    fn right_to_left(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Set the auto scroll mode.
    fn autoscroll(
        &mut self,
        device: &mut DEVICE,
        autoscroll: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Prints a string to the LCD at the current cursor position of the active device.
    fn print(
        &mut self,
        device: &mut DEVICE,
        text: &str,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the backlight on or off
    fn backlight(
        &mut self,
        device: &mut DEVICE,
        on: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// creates a new custom character
    fn create_char(
        &mut self,
        device: &mut DEVICE,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// read bytes from the active controller of the device. The size of the buffer is the number of bytes to read.
    fn read_device_data(
        &self,
        _device: &mut DEVICE,
        _buffer: &mut [u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        unimplemented!("Reads are not supported for device");
    }

    fn read_address_counter(
        &mut self,
        _device: &mut DEVICE,
    ) -> Result<u8, CharacterDisplayError<I2C>> {
        unimplemented!("Reads are not supported for device");
    }
}
