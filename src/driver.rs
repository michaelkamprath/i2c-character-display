pub mod aip31068;
pub mod hd44780;
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
    /// Creates a new instance of the device hardware.
    ///
    /// # Parameters
    /// - `config`: A `DeviceSetupConfig` containing the I2C, delay, address, and LCD type configuration.
    ///
    /// # Returns
    /// - A new instance of the device hardware.
    fn new(config: DeviceSetupConfig<I2C, DELAY>) -> Self;

    /// Returns the default I2C address for the device.
    ///
    /// # Returns
    /// - The default I2C address as a `u8`.
    fn default_i2c_address() -> u8;

    /// Indicates whether the device supports read operations.
    ///
    /// # Returns
    /// - `true` if the device supports reads, `false` otherwise.
    fn supports_reads() -> bool;

    /// Returns the LCD type associated with the device.
    ///
    /// # Returns
    /// - The `LcdDisplayType` of the device.
    fn lcd_type(&self) -> LcdDisplayType;

    /// Returns the configured I2C address of the device.
    ///
    /// # Returns
    /// - The I2C address as a `u8`.
    fn i2c_address(&self) -> u8;

    /// Returns a mutable reference to the delay object.
    ///
    /// # Returns
    /// - A mutable reference to the `DELAY` object.
    fn delay(&mut self) -> &mut DELAY;

    /// Returns a mutable reference to the I2C object.
    ///
    /// # Returns
    /// - A mutable reference to the `I2C` object.
    ///
    /// This is primarily used for testing purposes.
    fn i2c(&mut self) -> &mut I2C;

    /// Initializes the device hardware.
    ///
    /// # Returns
    /// - `Ok((u8, u8, u8))` containing the initial configuration of the device:
    ///   - `display_function`: The display function settings.
    ///   - `display_control`: The display control settings.
    ///   - `display_mode`: The display mode settings.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during initialization.
    fn init(&mut self) -> Result<(u8, u8, u8), CharacterDisplayError<I2C>>;

    /// Writes a sequence of bytes to the device.
    ///
    /// # Parameters
    /// - `rs_setting`: A boolean indicating the register select setting (`true` for data, `false` for command).
    /// - `data`: A slice of bytes to write to the device.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
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
    /// Initializes the display state.
    ///
    /// # Parameters
    /// - `display_function`: The initial display function settings.
    /// - `display_control`: The initial display control settings.
    /// - `display_mode`: The initial display mode settings.
    ///
    /// # Returns
    /// - `Ok(())` if the initialization is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during initialization.
    ///
    /// This method is intended to be called once after the device is initialized and before any other operations are performed.
    fn init_display_state(
        &mut self,
        display_function: u8,
        display_control: u8,
        display_mode: u8,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Clears the display.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn clear(&mut self, device: &mut DEVICE) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the cursor to the home position.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn home(&mut self, device: &mut DEVICE) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the cursor position at the specified column and row.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    /// - `col`: The column position (zero-indexed).
    /// - `row`: The row position (zero-indexed).
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn set_cursor(
        &mut self,
        device: &mut DEVICE,
        col: u8,
        row: u8,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the cursor visibility.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    /// - `show_cursor`: A boolean indicating whether to show the cursor (`true`) or hide it (`false`).
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn show_cursor(
        &mut self,
        device: &mut DEVICE,
        show_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the cursor blinking.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    /// - `blink_cursor`: A boolean indicating whether to enable cursor blinking (`true`) or disable it (`false`).
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn blink_cursor(
        &mut self,
        device: &mut DEVICE,
        blink_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the display visibility.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    /// - `show_display`: A boolean indicating whether to show the display (`true`) or hide it (`false`).
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn show_display(
        &mut self,
        device: &mut DEVICE,
        show_display: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Scrolls the display to the left.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn scroll_left(&mut self, device: &mut DEVICE) -> Result<(), CharacterDisplayError<I2C>>;

    /// Scrolls the display to the right.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn scroll_right(&mut self, device: &mut DEVICE) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the text flow direction to left-to-right.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn left_to_right(&mut self, device: &mut DEVICE) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the text flow direction to right-to-left.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn right_to_left(&mut self, device: &mut DEVICE) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the auto-scroll mode.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    /// - `autoscroll`: A boolean indicating whether to enable auto-scroll (`true`) or disable it (`false`).
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn autoscroll(
        &mut self,
        device: &mut DEVICE,
        autoscroll: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Prints a string to the LCD at the current cursor position of the active device.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    /// - `text`: The string to print.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn print(&mut self, device: &mut DEVICE, text: &str) -> Result<(), CharacterDisplayError<I2C>>;

    /// Sets the backlight on or off.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    /// - `on`: A boolean indicating whether to turn the backlight on (`true`) or off (`false`).
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError::UnsupportedOperation)` if the device does not support backlight control.
    ///
    /// # Example
    /// ```rust
    /// let mut device = ...; // Initialize your device
    /// let mut display = ...; // Initialize your display actions
    /// display.backlight(&mut device, true).unwrap(); // Turn on the backlight
    /// display.backlight(&mut device, false).unwrap(); // Turn off the backlight
    /// ```
    ///
    /// # Notes
    /// Some devices, such as the AiP31068 controller, do not support backlight control. In such cases,
    /// this method will return an `Err(CharacterDisplayError::UnsupportedOperation)`.
    fn backlight(
        &mut self,
        device: &mut DEVICE,
        on: bool,
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Creates a new custom character.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    /// - `location`: The memory location (0-7) to store the custom character.
    /// - `charmap`: An array of 8 bytes representing the custom character. Note that the first
    ///    five bits of each byte are used to define the character shape, while the last three bits
    ///    are ignored. Byte 0 is the top row of the character, and byte 7 is the bottom row.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn create_char(
        &mut self,
        device: &mut DEVICE,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<(), CharacterDisplayError<I2C>>;

    /// Reads data from the active controller of the device.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    /// - `buffer`: A mutable buffer to store the read data. The size of the buffer determines the number of bytes to read.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    fn read_device_data(
        &self,
        _device: &mut DEVICE,
        _buffer: &mut [u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        #[cfg(feature = "defmt")]
        defmt::warn!("Reading data is not supported on this display");
        Err(CharacterDisplayError::UnsupportedOperationWithMessage(
            "read_device_data",
        ))
    }

    /// Reads the address counter from the display device.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    ///
    /// # Returns
    /// - `Ok(u8)` containing the address counter value if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    /// - `Err(CharacterDisplayError::UnsupportedOperation)` if the device does not support reading the address counter.
    fn read_address_counter(
        &mut self,
        _device: &mut DEVICE,
    ) -> Result<u8, CharacterDisplayError<I2C>> {
        #[cfg(feature = "defmt")]
        defmt::warn!("Reading the address counter is not supported on this display");
        Err(CharacterDisplayError::UnsupportedOperationWithMessage(
            "read_address_counter",
        ))
    }

    /// Sets the contrast of the display.
    ///
    /// # Parameters
    /// - `device`: A mutable reference to the device implementing `DeviceHardwareTrait`.
    /// - `contrast`: The contrast level to set (0-255).
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CharacterDisplayError<I2C>)` if an error occurs during the operation.
    /// - `Err(CharacterDisplayError::UnsupportedOperation)` if the device does not support contrast adjustment.
    fn set_contrast(
        &mut self,
        _device: &mut DEVICE,
        _contrast: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        #[cfg(feature = "defmt")]
        defmt::warn!("Setting contrast is not supported on this display");
        Err(CharacterDisplayError::UnsupportedOperationWithMessage(
            "set_contrast",
        ))
    }
}
