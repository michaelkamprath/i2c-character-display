// HD44780 Support
// This module provides support for HD44780-based character displays.
// The BaseCharacterDisplay generic requires a ACTIONS object that implements the DisplayActionsTrait
// and a DEVICE object that implements the DeviceHardwareTrait. The HD44780 struct is the main
// object that implements the DisplayActionsTrait for the HD44780 display.
// The HD44780Adapter struct implements the DeviceHardwareTrait, but contains an object that
// implements the HD44780AdapterTrait trait. The HD44780AdapterTrait is where specific I2C
// hardware adapters are implemented for HD44780 displays. There are three implementations provided:
//      * AdafruitLCDBackpackAdapter
//      * DualHD44780_PCF8574TAdapter
//      * GenericPCF8574TAdapter.
//

pub mod adapter;

use core::marker::PhantomData;
use embedded_hal::{delay::DelayNs, i2c};

use crate::{
    driver::{
        hd44780::adapter::{
            adafruit_lcd_backpack::AdafruitLCDBackpackAdapter,
            dual_controller_pcf8574t::DualHD44780_PCF8574TAdapter,
            generic_pcf8574t::GenericPCF8574TAdapter, HD44780AdapterTrait,
        },
        DisplayActionsTrait,
    },
    CharacterDisplayError, DeviceSetupConfig,
};

pub type GenericHD44780PCF8574T<I2C, DELAY> = HD44780<I2C, DELAY, GenericPCF8574TAdapter<I2C, DELAY>>;
pub type AdafruitLCDBackpack<I2C, DELAY> = HD44780<I2C, DELAY, AdafruitLCDBackpackAdapter<I2C, DELAY>>;
pub type DualHD44780PCF8574T<I2C, DELAY> = HD44780<I2C, DELAY, DualHD44780_PCF8574TAdapter<I2C, DELAY>>;

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

/// The number of HD44780 controllers that can be supported on one device
const MAX_CONTROLLER_COUNT: usize = 2;

pub struct HD44780<I2C, DELAY, DEVICE>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    DEVICE:  HD44780AdapterTrait<I2C, DELAY>,
{
    display_function: [u8; MAX_CONTROLLER_COUNT],
    display_control: [u8; MAX_CONTROLLER_COUNT],
    display_mode: [u8; MAX_CONTROLLER_COUNT],
    active_controller: usize,
    _marker: PhantomData<I2C>,
    _delay: PhantomData<DELAY>,
    _device: PhantomData<DEVICE>,
}

impl<I2C, DELAY, DEVICE> HD44780<I2C, DELAY, DEVICE>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    DEVICE: HD44780AdapterTrait<I2C, DELAY>,
{
    pub fn new_adapter(config: DeviceSetupConfig<I2C, DELAY>) -> DEVICE {
        DEVICE::new(config)
    }
}

impl<I2C, DELAY, DEVICE> Default for HD44780<I2C, DELAY, DEVICE>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    DEVICE: HD44780AdapterTrait<I2C, DELAY>,
{
    fn default() -> Self {
        Self {
            display_function: [0; MAX_CONTROLLER_COUNT],
            display_control: [0; MAX_CONTROLLER_COUNT],
            display_mode: [0; MAX_CONTROLLER_COUNT],
            active_controller: 0,
            _marker: PhantomData,
            _delay: PhantomData,
            _device: PhantomData,
        }
    }
}

impl< I2C, DELAY, DEVICE> DisplayActionsTrait<I2C, DELAY, DEVICE> for HD44780<I2C, DELAY, DEVICE>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    DEVICE: HD44780AdapterTrait<I2C, DELAY>,
{

    fn init_display_state(
        &mut self,
        display_function: u8,
        display_control: u8,
        display_mode: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..DEVICE::controller_count() {
            self.display_function[controller] = display_function;
            self.display_control[controller] = display_control;
            self.display_mode[controller] = display_mode;
        }
        self.active_controller = 0;
        Ok(())
    }


    fn clear(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..DEVICE::controller_count() {
            self.clear_controller(device, controller)?;
        }
        Ok(())
    }

    fn home(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.home_controller(device, 0)?;
        self.active_controller = 0;
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

        let (controller, controller_row) = device.row_to_controller_row(row);
        self.active_controller = controller;
        self.set_cursor_controller(device, self.active_controller, col, controller_row)
    }

    fn show_cursor(
        &mut self,
        device: &mut DEVICE,
        show_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..DEVICE::controller_count() {
            let local_show_cursor = if controller == self.active_controller {
                show_cursor
            } else {
                false
            };
            self.show_cursor_controller(device, controller, local_show_cursor)?;
        }
        Ok(())
    }

    fn blink_cursor(
        &mut self,
        device: &mut DEVICE,
        blink_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..DEVICE::controller_count() {
            let local_blink_cursor = if controller == self.active_controller {
                blink_cursor
            } else {
                false
            };
            self.blink_cursor_controller(device, controller, local_blink_cursor)?;
        }
        Ok(())
    }

    fn show_display(
        &mut self,
        device: &mut DEVICE,
        show_display: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..DEVICE::controller_count() {
            self.show_display_controller(device, controller, show_display)?;
        }
        Ok(())
    }

    fn scroll_left(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..DEVICE::controller_count() {
            self.scroll_display_left_controller(device, controller)?;
        }
        Ok(())
    }

    fn scroll_right(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..DEVICE::controller_count() {
            self.scroll_display_right_controller(device, controller)?;
        }
        Ok(())
    }

    fn left_to_right(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..DEVICE::controller_count() {
            self.left_to_right_controller(device, controller)?;
        }
        Ok(())
    }

    fn right_to_left(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..DEVICE::controller_count() {
            self.right_to_left_controller(device, controller)?;
        }
        Ok(())
    }

    fn autoscroll(
        &mut self,
        device: &mut DEVICE,
        autoscroll: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..DEVICE::controller_count() {
            self.autoscroll_controller(device, controller, autoscroll)?;
        }
        Ok(())
    }

    fn print(
        &mut self,
        device: &mut DEVICE,
        text: &str,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.print_controller(device, self.active_controller, text)
    }

    fn backlight(
        &mut self,
        device: &mut DEVICE,
        on: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        device.set_backlight(on)
    }

    fn create_char(
        &mut self,
        device: &mut DEVICE,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..DEVICE::controller_count() {
            self.create_char_controller(device, controller, location, charmap)?;
        }
        Ok(())
    }

    fn read_device_data(
        &self,
        device: &mut DEVICE,
        buffer: &mut [u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if !DEVICE::supports_reads() {
            return Err(CharacterDisplayError::ReadNotSupported);
        }

        device.read_bytes_from_controller(
            self.active_controller,
            true,
            buffer,
        )
    }

    fn read_address_counter(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<u8, CharacterDisplayError<I2C>> {
        if !DEVICE::supports_reads() {
            return Err(CharacterDisplayError::ReadNotSupported);
        }
        let mut buffer = [0];

        device.read_bytes_from_controller(
            self.active_controller,
            false,
            &mut buffer,
        )?;
        // mask off the busy flag
        Ok(buffer[0] & 0x7F)
    }
}

impl<I2C, DELAY, DEVICE> HD44780<I2C, DELAY, DEVICE>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
    DEVICE: HD44780AdapterTrait<I2C, DELAY>,
{
    fn send_command_to_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
        command: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        device.write_byte_to_controller(
            controller,
            false,
            command,
        )
    }

    pub fn clear_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.send_command_to_controller(device, controller, LCD_CMD_CLEARDISPLAY)?;
        device.delay().delay_ms(2);
        Ok(())
    }

    /// Set the cursor to the home position on a specific HD44780 controller device
    pub fn home_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.send_command_to_controller(device, controller, LCD_CMD_RETURNHOME)?;
        device.delay().delay_ms(2);
        Ok(())
    }

    /// Set the cursor position at specified column and row on a specific HD44780 controller device.
    /// Columns and rows are zero-indexed and in the frame of the specified device.
    pub fn set_cursor_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
        col: u8,
        row: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if row >= device.lcd_type().rows() {
            return Err(CharacterDisplayError::RowOutOfRange);
        }
        if col >= device.lcd_type().cols() {
            return Err(CharacterDisplayError::ColumnOutOfRange);
        }

        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_SETDDRAMADDR | (col + device.lcd_type().row_offsets()[row as usize]),
        )?;
        Ok(())
    }

    /// Set the cursor visibility on a specific HD44780 controller.
    pub fn show_cursor_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
        show_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if show_cursor {
            self.display_control[controller] |= LCD_FLAG_CURSORON;
        } else {
            self.display_control[controller] &= !LCD_FLAG_CURSORON;
        }
        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_DISPLAYCONTROL | self.display_control[controller],
        )
    }

    /// Set the cursor blinking on a specific HD44780 controller device.
    pub fn blink_cursor_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
        blink_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if blink_cursor {
            self.display_control[controller] |= LCD_FLAG_BLINKON;
        } else {
            self.display_control[controller] &= !LCD_FLAG_BLINKON;
        }
        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_DISPLAYCONTROL | self.display_control[controller],
        )
    }

    pub fn show_display_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
        show_display: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if show_display {
            self.display_control[controller] |= LCD_FLAG_DISPLAYON;
        } else {
            self.display_control[controller] &= !LCD_FLAG_DISPLAYON;
        }
        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_DISPLAYCONTROL | self.display_control[controller],
        )
    }

    pub fn scroll_display_left_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVELEFT,
        )
    }

    pub fn scroll_display_right_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVERIGHT,
        )
    }

    pub fn left_to_right_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        // TODO revisit this function's logic
        self.display_mode[controller] |= LCD_FLAG_ENTRYLEFT;
        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_ENTRYMODESET | self.display_mode[controller],
        )
    }

    pub fn right_to_left_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        // TODO revisit this function's logic
        self.display_mode[controller] |= LCD_FLAG_ENTRYRIGHT;
        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_ENTRYMODESET | self.display_mode[controller],
        )
    }

    pub fn autoscroll_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
        autoscroll: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if autoscroll {
            self.display_mode[controller] |= LCD_FLAG_ENTRYSHIFTINCREMENT;
        } else {
            self.display_mode[controller] &= !LCD_FLAG_ENTRYSHIFTINCREMENT;
        }
        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_ENTRYMODESET | self.display_mode[controller],
        )?;
        Ok(())
    }

    pub fn create_char_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<&mut Self, CharacterDisplayError<I2C>> {
        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_SETCGRAMADDR | ((location & 0x7) << 3),
        )?;
        for &charmap_byte in charmap.iter() {
            device.write_byte_to_controller(
                controller,
                true,
                charmap_byte,
            )?;
        }
        Ok(self)
    }

    pub fn print_controller(
        &mut self,
        device: &mut DEVICE,
        controller: usize,
        text: &str,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for c in text.chars() {
            device.write_byte_to_controller(
                controller,
                true,
                c as u8,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod lib_tests {
    extern crate std;
    use crate::{LcdDisplayType, driver::DeviceHardwareTrait};

    use super::*;
    use embedded_hal_mock::eh1::{
        delay::NoopDelay,
        i2c::{Mock as I2cMock, Transaction as I2cTransaction},
    };

    #[test]
    fn test_generic_hd44780_pcf8574t_init() {
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
        let device: DeviceSetupConfig<I2cMock, NoopDelay> = DeviceSetupConfig {
            i2c: i2c,
            address: i2c_address,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let mut driver = GenericHD44780PCF8574T::new_adapter(device);
        let result = driver.init();
        assert!(result.is_ok());

        // finish the i2c mock
        driver.i2c().done();
    }

    #[test]
    fn test_generic_hd44780_pcf8574t_set_backlight() {
        let i2c_address = 0x27_u8;
        let expected_i2c_transactions = std::vec![
            I2cTransaction::write(i2c_address, std::vec![0b0000_1000]), // backlight on
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // backlight off
        ];

        let i2c = I2cMock::new(&expected_i2c_transactions);
        let mut driver = GenericHD44780PCF8574T::new_adapter(
            DeviceSetupConfig {
                i2c: i2c,
                address: i2c_address,
                lcd_type: LcdDisplayType::Lcd16x4,
                delay: NoopDelay,
            }
        );

        let mut actions = GenericHD44780PCF8574T::default();


        assert!(actions.backlight(&mut driver, true).is_ok());
        assert!(actions.backlight(&mut driver,false).is_ok());

        // finish the i2c mock
        driver.i2c().done();
    }

    #[test]
    fn test_generic_hd44780_pcf8574t_print() {
        let i2c_address = 0x27_u8;
        let expected_i2c_transactions = std::vec![
            // print "hello" to the display
            I2cTransaction::write(i2c_address, std::vec![0b0110_0101]), // 'h' 0x68 - high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0110_0001]), // 'h' 0x68 - high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1000_0101]), // 'h' 0x68 - low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1000_0001]), // 'h' 0x68 - low nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0110_0101]), // 'e' 0x65 - high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0110_0001]), // 'e' 0x65 - high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0101_0101]), // 'e' 0x65 - low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0101_0001]), // 'e' 0x65 - low nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0110_0101]), // 'l' 0x6C - high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0110_0001]), // 'l' 0x6C - high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1100_0101]), // 'l' 0x6C - low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1100_0001]), // 'l' 0x6C - low nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0110_0101]), // 'l' 0x6C - high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0110_0001]), // 'l' 0x6C - high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1100_0101]), // 'l' 0x6C - low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1100_0001]), // 'l' 0x6C - low nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b0110_0101]), // 'o' 0x6F - high nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b0110_0001]), // 'o' 0x6F - high nibble, rw=0, enable=0
            I2cTransaction::write(i2c_address, std::vec![0b1111_0101]), // 'o' 0x6F - low nibble, rw=0, enable=1
            I2cTransaction::write(i2c_address, std::vec![0b1111_0001]), // 'o' 0x6F - low nibble, rw=0, enable=0
        ];

        let i2c = I2cMock::new(&expected_i2c_transactions);
        let mut driver = GenericHD44780PCF8574T::new_adapter(
            DeviceSetupConfig {
                i2c: i2c,
                address: i2c_address,
                lcd_type: LcdDisplayType::Lcd16x4,
                delay: NoopDelay,
            }
        );
        let mut actions = GenericHD44780PCF8574T::default();

        assert!(actions.print(&mut driver, "hello").is_ok());

        // finish the i2c mock
        driver.i2c().done();
    }

    #[test]
    fn test_set_cursor_out_of_range() {
        let i2c_address = 0x27_u8;
        let i2c = I2cMock::new(&[]);
        let mut driver = GenericHD44780PCF8574T::new_adapter(
            DeviceSetupConfig {
                i2c: i2c,
                address: i2c_address,
                lcd_type: LcdDisplayType::Lcd16x4,
                delay: NoopDelay,
            }
        );
        let mut actions = GenericHD44780PCF8574T::default();

        assert!(actions.set_cursor(&mut driver, 20, 0).is_err());
        assert!(actions.set_cursor(&mut driver, 0, 20).is_err());

        // finish the i2c mock
        driver.i2c().done();
    }

    #[test]
    fn test_set_cursor_dual_controller() {
        let i2c_address = 0x27_u8;
        let i2c = I2cMock::new(&[
            // first set cursor to (20,1), which on first controller
            // command = LCD_CMD_SETDDRAMADDR = 0x80
            // cursor = 20 + (row 1 offset =0x40) = 0x14 + 0x40 = 0x54)
            // byte to send = 0x80 | 0x54 = 0xD4
            I2cTransaction::write(i2c_address, std::vec![0b1101_0100]), // high nibble 0xD, rw=0, enable1=1, enabl2=0
            I2cTransaction::write(i2c_address, std::vec![0b1101_0000]), // high nibble 0xD, rw=0, enable1=0, enabl2=0
            I2cTransaction::write(i2c_address, std::vec![0b0100_0100]), // low nibble 0x4, rw=0, enable1=1, enabl2=0
            I2cTransaction::write(i2c_address, std::vec![0b0100_0000]), // low nibble 0x4, rw=0, enable1=0, enabl2=0
            // now set cursor to (10,2), which is in the second controller
            // command = LCD_CMD_SETDDRAMADDR = 0x80
            // cursor = 10 + (2 row offset = 0x00) = 0x0A + 0x00
            // byte to send = 0x80 | 0x0A = 0x8A
            I2cTransaction::write(i2c_address, std::vec![0b1000_0010]), // high nibble 0x8, rw=0, enable1=0, enabl2=1
            I2cTransaction::write(i2c_address, std::vec![0b1000_0000]), // high nibble 0x8, rw=0, enable1=0, enabl2=0
            I2cTransaction::write(i2c_address, std::vec![0b1010_0010]), // low nibble 0xA, rw=0, enable1=0, enabl2=1
            I2cTransaction::write(i2c_address, std::vec![0b1010_0000]), // low nibble 0xA, rw=0, enable1=0, enabl2=0

        ]);
        let mut driver = DualHD44780PCF8574T::new_adapter(
            DeviceSetupConfig {
                i2c: i2c,
                address: i2c_address,
                lcd_type: LcdDisplayType::Lcd40x4,
                delay: NoopDelay,
            }
        );
        let mut actions = DualHD44780PCF8574T::default();

        assert!(actions.set_cursor(&mut driver, 20, 1).is_ok());
        assert!(actions.set_cursor(&mut driver, 10, 2).is_ok());

        // finish the i2c mock
        driver.i2c().done();
    }
}
