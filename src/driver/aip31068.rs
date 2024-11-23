
use core::marker::PhantomData;
use embedded_hal::{delay::DelayNs, i2c};

use crate::{
    driver::DriverTrait,
    CharacterDisplayError, DeviceSetupConfig,
};

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
const LCD_FLAG_2LINE: u8 = 0x08; //  LCD 2 line mode
const LCD_FLAG_1LINE: u8 = 0x00; //  LCD 1 line mode
const LCD_FLAG_5x10_DOTS: u8 = 0x04; //  10 pixel high font mode
const LCD_FLAG_5x8_DOTS: u8 = 0x00; //  8 pixel high font mode

const MAX_BUFFER_SIZE: usize = 82;      // 80 bytes of data + 2 control bytes.
pub struct AIP31068<I2C>
where
    I2C: i2c::I2c,
{
    display_function: u8,
    display_control: u8,
    display_mode: u8,
    buffer: [u8; MAX_BUFFER_SIZE],  // buffer for I2C data
    _marker: PhantomData<I2C>,
}

impl<I2C> Default for AIP31068<I2C>
where
    I2C: i2c::I2c,
{
    fn default() -> Self {
        AIP31068 {
            display_function: 0,
            display_control: 0,
            display_mode: 0,
            buffer: [0; MAX_BUFFER_SIZE],
            _marker: PhantomData,
        }
    }
}

impl<I2C, DELAY> DriverTrait<I2C, DELAY> for AIP31068<I2C>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn default_i2c_address() -> u8 {
        0x3e
    }

    fn supports_reads() -> bool {
        false
    }

    fn init(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {

        #[cfg(feature = "defmt")]
        defmt::debug!("Initializing AIP31068");
        // wait 15 ms for power on
        device.delay.delay_ms(15);

        // send function set command
        self.display_function = LCD_FLAG_2LINE | LCD_FLAG_5x8_DOTS;
        self.write_bytes(device, false, &[LCD_CMD_FUNCTIONSET | self.display_function])?;

        // wait 39 us
        device.delay.delay_us(39);

        // display on/off control
        self.display_control = LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF;
        self.write_bytes(device, false, &[LCD_CMD_DISPLAYCONTROL | self.display_control])?;

        // wait 39 us
        device.delay.delay_us(39);

        // clear display
        self.write_bytes(device, false, &[LCD_CMD_CLEARDISPLAY])?;

        // wait 1.53 ms
        device.delay.delay_us(1530);

        // entry mode set
        self.display_mode = LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT;
        self.write_bytes(device, false, &[LCD_CMD_ENTRYMODESET | self.display_mode])?;

        Ok(())
    }

    fn clear(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.write_bytes(device, false, &[LCD_CMD_CLEARDISPLAY])?;
        // wait for command to complete
        device.delay.delay_us(1530);
        Ok(())
    }

    fn home(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.write_bytes(device, false, &[LCD_CMD_RETURNHOME])?;
        // wait for command to complete
        device.delay.delay_us(1530);
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

        self.write_bytes(
            device,
            false,
            &[LCD_CMD_SETDDRAMADDR | (col + device.lcd_type.row_offsets()[row as usize])],
        )?;
        // wait for command to complete
        device.delay.delay_us(39);
        Ok(())
    }

    fn show_cursor(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        show_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if show_cursor {
            self.display_control |= LCD_FLAG_CURSORON;
        } else {
            self.display_control &= !LCD_FLAG_CURSORON;
        }
        self.write_bytes(device, false, &[LCD_CMD_DISPLAYCONTROL | self.display_control])?;
        // wait for command to complete
        device.delay.delay_us(39);
        Ok(())
    }

    fn blink_cursor(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        blink_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if blink_cursor {
            self.display_control |= LCD_FLAG_BLINKON;
        } else {
            self.display_control &= !LCD_FLAG_BLINKON;
        }
        self.write_bytes(device, false, &[LCD_CMD_DISPLAYCONTROL | self.display_control])?;
        // wait for command to complete
        device.delay.delay_us(39);
        Ok(())
    }

    fn show_display(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        show_display: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if show_display {
            self.display_control |= LCD_FLAG_DISPLAYON;
        } else {
            self.display_control &= !LCD_FLAG_DISPLAYON;
        }
        self.write_bytes(device, false, &[LCD_CMD_DISPLAYCONTROL | self.display_control])?;
        // wait for command to complete
        device.delay.delay_us(39);
        Ok(())
    }

    fn scroll_left(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.write_bytes(device, false, &[LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVELEFT])?;
        // wait for command to complete
        device.delay.delay_us(39);
        Ok(())
    }

    fn scroll_right(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.write_bytes(device, false, &[LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVERIGHT])?;
        // wait for command to complete
        device.delay.delay_us(39);
        Ok(())
    }

    fn left_to_right(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        todo!()
    }

    fn right_to_left(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        todo!()
    }

    fn autoscroll(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        autoscroll: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if autoscroll {
            self.display_mode |= LCD_FLAG_ENTRYSHIFTINCREMENT;
        } else {
            self.display_mode &= !LCD_FLAG_ENTRYSHIFTINCREMENT;
        }
        self.write_bytes(
            device,
            false,
            &[LCD_CMD_ENTRYMODESET | self.display_mode],
        )?;
        // wait for command to complete
        device.delay.delay_us(39);
        Ok(())
    }

    fn print(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        text: &str,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.write_bytes(device, true, text.as_bytes())?;
        // wait for command to complete
        device.delay.delay_us(43);
        Ok(())
    }

    fn backlight(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        on: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {

        Ok(())
    }

    fn create_char(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.write_bytes(device, false, &[LCD_CMD_SETCGRAMADDR | ((location & 0x7) << 3)])?;
        self.write_bytes(device, true, &charmap)?;
        // wait for command to complete
        device.delay.delay_us(39);
        Ok(())
    }

}

impl<I2C> AIP31068<I2C>
where
    I2C: i2c::I2c,
{
    const CONTROL_NOT_LAST_BYTE: u8 = 0b1000_0000;  // Last control byte. Only a stream of data bytes will follow.
    const CONTROL_LAST_BYTE: u8 = 0b0000_0000;      // Another control byte will follow the next data byte.
    const CONTROL_RS_DATA: u8 = 0b0100_0000;
    const CONTROL_RS_COMMAND: u8 = 0b0000_0000;

    /// write one or more bytes to the display.
    /// The `rs_setting` parameter indcate if the data is a command or data. `true` for data, `false` for command.
    fn write_bytes<DELAY: DelayNs>(
        &mut self,
        device: &mut DeviceSetupConfig<I2C, DELAY>,
        rs_setting: bool,
        data: &[u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if data.is_empty() {
            return Ok(());
        }
        let control_byte = if rs_setting {
            Self::CONTROL_RS_DATA
        } else {
            Self::CONTROL_RS_COMMAND
        };

        // build the data to send
        let mut idx: usize = 0;
        self.buffer[idx] = control_byte | Self::CONTROL_LAST_BYTE;
        idx += 1;
        for byte in &data[..data.len()] {
            if idx + 2 > MAX_BUFFER_SIZE {
                return Err(CharacterDisplayError::BufferTooSmall);
            }
            self.buffer[idx] = *byte;
            idx += 1;
        }
        // send the dat
        device.i2c.write(device.address, &self.buffer[..idx]).map_err(CharacterDisplayError::I2cError)?;

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
    fn test_write_bytes() {
        let i2c_address = 0x3e;
        let expected_i2c_transactions = std::vec![
            I2cTransaction::write(i2c_address, std::vec![
                0b0100_0000,
                0x01,
                0x02,
                0x03,
            ]),
            I2cTransaction::write(i2c_address, std::vec![
                0b0100_0000,
                0x04,
            ]),
            I2cTransaction::write(i2c_address, std::vec![
                0b0000_0000,
                0xAB,
            ]),
        ];

        let i2c = I2cMock::new(&expected_i2c_transactions);
        let mut driver = AIP31068::default();
        let mut device = DeviceSetupConfig {
            i2c: i2c,
            address: i2c_address,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };

        driver.write_bytes(&mut device, true, &[0x01, 0x02, 0x03]).unwrap();
        driver.write_bytes(&mut device, true, &[0x04]).unwrap();
        driver.write_bytes(&mut device, false, &[0xAB]).unwrap();
        device.i2c.done();
    }
}