mod adapter;

use core::marker::PhantomData;
use embedded_hal::{delay::DelayNs, i2c};

use crate::{
    driver::{
        hd44780::adapter::{
            adafruit_lcd_backpack::AdafruitLCDBackpackAdapter, generic_pcf8574t::GenericPCF8574TAdapter, dual_controller_pcf8574t::DualHD44780_PCF8574TAdapter,
            HD44780AdapterTrait,
        },
        DriverTrait,
    },
    CharacterDisplayError, DeviceSetupConfig,
};

pub type GenericHD44780PCF8574T<I2C> = HD44780<GenericPCF8574TAdapter<I2C>, I2C>;
pub type AdafruitLCDBackpack<I2C> = HD44780<AdafruitLCDBackpackAdapter<I2C>, I2C>;
pub type DualHD44780PCF8574T<I2C> = HD44780<DualHD44780_PCF8574TAdapter<I2C>, I2C>;

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

pub struct HD44780<ADAPTER, I2C>
where
    ADAPTER: HD44780AdapterTrait<I2C>,
    I2C: i2c::I2c,
{
    adapter: ADAPTER,
    display_function: [u8; MAX_CONTROLLER_COUNT],
    display_control: [u8; MAX_CONTROLLER_COUNT],
    display_mode: [u8; MAX_CONTROLLER_COUNT],
    active_controller: usize,
    _marker: PhantomData<I2C>,
}

impl<ADAPTER, I2C> Default for HD44780<ADAPTER, I2C>
where
    ADAPTER: HD44780AdapterTrait<I2C>,
    I2C: i2c::I2c,
{
    fn default() -> Self {
        Self {
            adapter: ADAPTER::default(),
            display_function: [0; MAX_CONTROLLER_COUNT],
            display_control: [LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF;
                MAX_CONTROLLER_COUNT],
            display_mode: [LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT; MAX_CONTROLLER_COUNT],
            active_controller: 0,
            _marker: PhantomData,
        }
    }
}

impl<ADAPTER, I2C, DELAY> DriverTrait<I2C, DELAY> for HD44780<ADAPTER, I2C>
where
    ADAPTER: HD44780AdapterTrait<I2C>,
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn default_i2c_address() -> u8 {
        ADAPTER::default_i2c_address()
    }

    fn supports_reads() -> bool {
        ADAPTER::supports_reads()
    }

    fn init(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if !ADAPTER::is_supported(device.lcd_type) {
            return Err(CharacterDisplayError::UnsupportedDisplayType);
        }

        self.adapter
            .init(&mut device.i2c, device.address)
            .map_err(CharacterDisplayError::I2cError)?;

        for controller in 0..self.adapter.controller_count() {
            if controller >= MAX_CONTROLLER_COUNT {
                return Err(CharacterDisplayError::BadDeviceId);
            }

            self.display_function[controller] =
                LCD_FLAG_4BITMODE | LCD_FLAG_5x8_DOTS | LCD_FLAG_2LINE;

            // Put LCD into 4 bit mode, device starts in 8 bit mode
            self.adapter.write_nibble_to_controller(
                &mut device.i2c,
                device.address,
                controller,
                false,
                0x03,
            )?;
            device.delay.delay_ms(5);
            self.adapter.write_nibble_to_controller(
                &mut device.i2c,
                device.address,
                controller,
                false,
                0x03,
            )?;
            device.delay.delay_ms(5);
            self.adapter.write_nibble_to_controller(
                &mut device.i2c,
                device.address,
                controller,
                false,
                0x03,
            )?;
            device.delay.delay_us(150);
            self.adapter.write_nibble_to_controller(
                &mut device.i2c,
                device.address,
                controller,
                false,
                0x02,
            )?;

            self.send_command_to_controller(
                device,
                controller,
                LCD_CMD_FUNCTIONSET | self.display_function[controller],
            )?;
            self.send_command_to_controller(
                device,
                controller,
                LCD_CMD_DISPLAYCONTROL | self.display_control[controller],
            )?;
            self.send_command_to_controller(
                device,
                controller,
                LCD_CMD_ENTRYMODESET | self.display_mode[controller],
            )?;
            self.clear_controller(device, controller)?;
            self.home_controller(device, controller)?;
        }
        // set up the display
        self.backlight(device, true)?;
        self.active_controller = 0;
        Ok(())
    }

    fn clear(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..self.adapter.controller_count() {
            self.clear_controller(device, controller)?;
        }
        Ok(())
    }

    fn home(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.home_controller(device, 0)?;
        self.active_controller = 0;
        Ok(())
    }

    fn set_cursor(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        col: u8,
        row: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if row >= device.lcd_type.rows() {
            return Err(CharacterDisplayError::RowOutOfRange);
        }
        if col >= device.lcd_type.cols() {
            return Err(CharacterDisplayError::ColumnOutOfRange);
        }

        let (controller, controller_row) = self.adapter.row_to_controller_row(row);
        self.active_controller = controller;
        self.set_cursor_controller(device, self.active_controller, col, controller_row)
    }

    fn show_cursor(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        show_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..self.adapter.controller_count() {
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
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        blink_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..self.adapter.controller_count() {
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
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        show_display: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..self.adapter.controller_count() {
            self.show_display_controller(device, controller, show_display)?;
        }
        Ok(())
    }

    fn scroll_left(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..self.adapter.controller_count() {
            self.scroll_display_left_controller(device, controller)?;
        }
        Ok(())
    }

    fn scroll_right(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..self.adapter.controller_count() {
            self.scroll_display_right_controller(device, controller)?;
        }
        Ok(())
    }

    fn left_to_right(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..self.adapter.controller_count() {
            self.left_to_right_controller(device, controller)?;
        }
        Ok(())
    }

    fn right_to_left(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..self.adapter.controller_count() {
            self.right_to_left_controller(device, controller)?;
        }
        Ok(())
    }

    fn autoscroll(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        autoscroll: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..self.adapter.controller_count() {
            self.autoscroll_controller(device, controller, autoscroll)?;
        }
        Ok(())
    }

    fn print(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        text: &str,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.print_controller(device, self.active_controller, text)
    }

    fn backlight(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        on: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.adapter.set_backlight(on);
        self.adapter
            .write_bits_to_gpio(&mut device.i2c, device.address)
    }

    fn create_char(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for controller in 0..self.adapter.controller_count() {
            self.create_char_controller(device, controller, location, charmap)?;
        }
        Ok(())
    }
}

impl<ADAPTER, I2C> HD44780<ADAPTER, I2C>
where
    ADAPTER: HD44780AdapterTrait<I2C>,
    I2C: i2c::I2c,
{
    fn send_command_to_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        controller: usize,
        command: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.adapter.write_byte_to_controller(
            &mut device.i2c,
            device.address,
            controller,
            false,
            command,
        )
    }

    pub fn clear_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.send_command_to_controller(device, controller, LCD_CMD_CLEARDISPLAY)?;
        device.delay.delay_ms(2);
        Ok(())
    }

    /// Set the cursor to the home position on a specific HD44780 controller device
    pub fn home_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.send_command_to_controller(device, controller, LCD_CMD_RETURNHOME)?;
        device.delay.delay_ms(2);
        Ok(())
    }

    /// Set the cursor position at specified column and row on a specific HD44780 controller device.
    /// Columns and rows are zero-indexed and in the frame of the specified device.
    pub fn set_cursor_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        controller: usize,
        col: u8,
        row: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if row >= device.lcd_type.rows() {
            return Err(CharacterDisplayError::RowOutOfRange);
        }
        if col >= device.lcd_type.cols() {
            return Err(CharacterDisplayError::ColumnOutOfRange);
        }

        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_SETDDRAMADDR | (col + device.lcd_type.row_offsets()[row as usize]),
        )?;
        Ok(())
    }

    /// Set the cursor visibility on a specific HD44780 controller.
    pub fn show_cursor_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
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
    pub fn blink_cursor_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
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

    pub fn show_display_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
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

    pub fn scroll_display_left_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVELEFT,
        )
    }

    pub fn scroll_display_right_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.send_command_to_controller(
            device,
            controller,
            LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVERIGHT,
        )
    }

    pub fn left_to_right_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
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

    pub fn right_to_left_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
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

    pub fn autoscroll_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
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

    pub fn create_char_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
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
            self.adapter.write_byte_to_controller(
                &mut device.i2c,
                device.address,
                controller,
                true,
                charmap_byte,
            )?;
        }
        Ok(self)
    }

    pub fn print_controller<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        controller: usize,
        text: &str,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        for c in text.chars() {
            self.adapter.write_byte_to_controller(
                &mut device.i2c,
                device.address,
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
    use crate::LcdDisplayType;

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
        let mut driver = GenericHD44780PCF8574T::default();
        let mut device = DeviceSetupConfig {
            i2c: i2c,
            address: i2c_address,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let result = driver.init(&mut device);
        assert!(result.is_ok());

        // finish the i2c mock
        device.i2c.done();
    }

    #[test]
    fn test_generic_hd44780_pcf8574t_set_backlight() {
        let i2c_address = 0x27_u8;
        let expected_i2c_transactions = std::vec![
            I2cTransaction::write(i2c_address, std::vec![0b0000_1000]), // backlight on
            I2cTransaction::write(i2c_address, std::vec![0b0000_0000]), // backlight off
        ];

        let i2c = I2cMock::new(&expected_i2c_transactions);
        let mut driver = GenericHD44780PCF8574T::default();

        let mut device = DeviceSetupConfig {
            i2c: i2c,
            address: i2c_address,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };

        assert!(driver.backlight(&mut device, true).is_ok());
        assert!(driver.backlight(&mut device, false).is_ok());

        // finish the i2c mock
        device.i2c.done();
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
        let mut driver = GenericHD44780PCF8574T::default();

        let mut device = DeviceSetupConfig {
            i2c: i2c,
            address: i2c_address,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };

        assert!(driver.print(&mut device, "hello").is_ok());

        // finish the i2c mock
        device.i2c.done();
    }
}
