
use embedded_hal::{delay::DelayNs, i2c};

use crate::{
    driver::DeviceHardwareTrait,
    CharacterDisplayError, DeviceSetupConfig, LcdDisplayType,
};

use crate::driver::standard::{
    LCD_FLAG_8BITMODE, LCD_FLAG_2LINE, LCD_FLAG_5x8_DOTS, LCD_CMD_FUNCTIONSET,
    LCD_FLAG_DISPLAYON, LCD_FLAG_CURSOROFF, LCD_FLAG_BLINKOFF, LCD_CMD_DISPLAYCONTROL,
    LCD_CMD_CLEARDISPLAY,
    LCD_FLAG_ENTRYLEFT, LCD_FLAG_ENTRYSHIFTDECREMENT, LCD_CMD_ENTRYMODESET,
};

use super::standard::StandardCharacterDisplayHandler;
use super::DisplayActionsTrait;

const CONTROL_NOT_LAST_BYTE: u8 = 0b1000_0000;  // Another control byte will follow the next data byte.
const CONTROL_LAST_BYTE: u8 = 0b0000_0000;      // Last control byte. Only a stream of data bytes will follow.
const CONTROL_RS_DATA: u8 = 0b0100_0000;
const CONTROL_RS_COMMAND: u8 = 0b0000_0000;

const LCD_FLAG_INSTUCTION_EXTENSION: u8 = 0x01;
const LCD_FLAG_INSTRUCTION_NORMAL: u8 = 0x00;
const LCD_CMD_SET_CONTRAST_LOW: u8 = 0x70;
const LCD_CMD_SET_PWR_ICON_CONTRAST_HI: u8 = 0x50;

const MAX_BUFFER_SIZE: usize = 82;      // 80 bytes of data + 2 control bytes.

/// AIP31068 device driver implementation
pub struct ST7032i<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    buffer: [u8; MAX_BUFFER_SIZE],  // buffer for I2C data
    config: DeviceSetupConfig<I2C, DELAY>,
}


impl<I2C, DELAY> DeviceHardwareTrait<I2C, DELAY> for ST7032i<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn new(config: DeviceSetupConfig<I2C, DELAY>) -> Self {
        ST7032i {
            buffer: [0; MAX_BUFFER_SIZE],
            config: config,
        }
    }

    fn default_i2c_address() -> u8 {
        0x3e
    }

    fn supports_reads() -> bool {
        false
    }

    fn lcd_type(&self) -> LcdDisplayType {
        self.config.lcd_type
    }

    fn i2c_address(&self) -> u8 {
        self.config.address
    }

    fn delay(&mut self) -> &mut DELAY {
        &mut self.config.delay
    }

    fn i2c(&mut self) -> &mut I2C {
        &mut self.config.i2c
    }

    fn init(
        &mut self,
    ) -> Result<(u8, u8, u8), CharacterDisplayError<I2C>> {
        // wait 40 ms for power on
        self.config.delay.delay_ms(40);

        // send function set command
        let display_function: u8 =  LCD_FLAG_8BITMODE | LCD_FLAG_2LINE | LCD_FLAG_5x8_DOTS | LCD_FLAG_INSTRUCTION_NORMAL;
        self.write_bytes(false, &[LCD_CMD_FUNCTIONSET | display_function])?;
        self.config.delay.delay_us(27);

        // place the device into extended instruction mode
        self.write_bytes(false, &[LCD_CMD_FUNCTIONSET | display_function | LCD_FLAG_INSTUCTION_EXTENSION])?;
        self.config.delay.delay_us(27);

        // set internal OSC frequency
        //   - 0x14 sets to 149 Hz(5V) or 144 Hz(3.3V), with 1/5 bias
        self.write_bytes(false, &[0x14])?;
        self.config.delay.delay_us(27);

        // set contrast
        let contrast_low: u8 =0x08;
        self.write_bytes(false, &[LCD_CMD_SET_CONTRAST_LOW | contrast_low])?;
        self.config.delay.delay_us(27);

        // set power/icon/contrast control
        self.write_bytes(false, &[0x5E])?;
        self.config.delay.delay_us(27);

        // set follower control
        self.write_bytes(false, &[0x6A])?;

        // wait 200 ms
        self.config.delay.delay_ms(200);

        // return to normal instructions
        self.write_bytes(false, &[LCD_CMD_FUNCTIONSET | display_function])?;
        self.config.delay.delay_us(27);

        // display on/off control
        let display_control: u8 = LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF;
        self.write_bytes( false, &[LCD_CMD_DISPLAYCONTROL | display_control])?;
        self.config.delay.delay_us(27);

        // clear display
        self.write_bytes(false, &[LCD_CMD_CLEARDISPLAY])?;
        self.config.delay.delay_ms(2);

        // entry mode set
        let display_mode: u8 = LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT;
        self.write_bytes(false, &[LCD_CMD_ENTRYMODESET | display_mode])?;
        self.config.delay.delay_us(27);

        Ok((display_function, display_control, display_mode))
    }

    /// write one or more bytes to the display.
    /// The `rs_setting` parameter indcate if the data is a command or data. `true` for data, `false` for command.
    fn write_bytes(
        &mut self,
        rs_setting: bool,
        data: &[u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if data.is_empty() {
            return Ok(());
        }
        let control_byte = if rs_setting {
            CONTROL_RS_DATA
        } else {
            CONTROL_RS_COMMAND
        };

        // build the data to send
        let mut idx: usize = 0;
        self.buffer[idx] = control_byte | CONTROL_LAST_BYTE;
        idx += 1;
        for byte in &data[..data.len()] {
            if idx > MAX_BUFFER_SIZE {
                return Err(CharacterDisplayError::BufferTooSmall);
            }
            self.buffer[idx] = *byte;
            idx += 1;
        }
        // send the data
        self.config.i2c.write(self.config.address, &self.buffer[..idx]).map_err(CharacterDisplayError::I2cError)?;
        Ok(())
    }
}

pub struct ST7032iDisplayActions<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    base: StandardCharacterDisplayHandler,
    power_icon_contrast_hi: u8,
    contrast_low: u8,
    _i2c: core::marker::PhantomData<I2C>,
    _delay: core::marker::PhantomData<DELAY>,
}

impl<I2C, DELAY> Default for ST7032iDisplayActions<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn default() -> Self {
        ST7032iDisplayActions {
            // rather than reimplementing the display actions of StandardCharacterDisplayHandler, we will use it as a "base class"
            // we just need to add the contrast settings to the display actions
            base: StandardCharacterDisplayHandler::default(),
            power_icon_contrast_hi: 0x0C,
            contrast_low: 0,
            _i2c: core::marker::PhantomData,
            _delay: core::marker::PhantomData,
        }
    }
}
impl<I2C, DELAY, DEVICE> DisplayActionsTrait<I2C, DELAY, DEVICE> for ST7032iDisplayActions<I2C, DELAY>
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
        <StandardCharacterDisplayHandler as DisplayActionsTrait<I2C, DELAY, DEVICE>>::init_display_state(
            &mut self.base,
            display_function,
            display_control,
            display_mode,
        )
    }

    fn clear(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.clear(device)
    }

    fn home(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.home(device)
    }

    fn set_cursor(
        &mut self,
        device: &mut DEVICE,
        col: u8,
        row: u8,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.set_cursor(device, col, row)
    }

    fn show_cursor(
        &mut self,
        device: &mut DEVICE,
        show_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.show_cursor(device, show_cursor)
    }

    fn blink_cursor(
        &mut self,
        device: &mut DEVICE,
        blink_cursor: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.blink_cursor(device, blink_cursor)
    }

    fn show_display(
        &mut self,
        device: &mut DEVICE,
        show_display: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.show_display(device, show_display)
    }

    fn scroll_left(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.scroll_left(device)
    }

    fn scroll_right(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.scroll_right(device)
    }

    fn left_to_right(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.left_to_right(device)
    }

    fn right_to_left(
        &mut self,
        device: &mut DEVICE,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.right_to_left(device)
    }

    fn autoscroll(
        &mut self,
        device: &mut DEVICE,
        autoscroll: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.autoscroll(device, autoscroll)
    }

    fn print(
        &mut self,
        device: &mut DEVICE,
        text: &str,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.print(device, text)
    }

    fn backlight(
        &mut self,
        device: &mut DEVICE,
        on: bool,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.backlight(device, on)
    }

    fn create_char(
        &mut self,
        device: &mut DEVICE,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        self.base.create_char(device, location, charmap)
    }

    /// Sets the contrast setting of the display (if supported).
    /// The constrat value os a 6-bit value, and will be masked to 0x3F.
    fn set_contrast(&mut self, device: &mut DEVICE, contrast: u8) -> Result<(), CharacterDisplayError<I2C>> {
        self.contrast_low = contrast&0x0F;
        self.power_icon_contrast_hi = ((contrast >> 4) & 0x03) | self.power_icon_contrast_hi & 0x0C;

        // first set device into extended instruction mode
        device.write_bytes(false, &[LCD_CMD_FUNCTIONSET | self.base.get_display_function() | LCD_FLAG_INSTUCTION_EXTENSION])?;
        device.delay().delay_us(27);

        // set the lower 4 bits of the contrast
        device.write_bytes(false, &[LCD_CMD_SET_CONTRAST_LOW | self.contrast_low])?;
        device.delay().delay_us(27);

        // set the higher 2 bits of the contrast
        device.write_bytes(false, &[LCD_CMD_SET_PWR_ICON_CONTRAST_HI | self.power_icon_contrast_hi])?;
        device.delay().delay_us(27);

        // return to normal instructions
        device.write_bytes(false, &[LCD_CMD_FUNCTIONSET | self.base.get_display_function()])?;
        device.delay().delay_us(27);
        Ok(())
    }
}



#[cfg(test)]
mod lib_tests {
    extern crate std;
    use crate::{driver::DisplayActionsTrait, LcdDisplayType};
    use crate::driver::standard::StandardCharacterDisplayHandler;

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
        let device = DeviceSetupConfig {
            i2c: i2c,
            address: i2c_address,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let mut driver = ST7032i::new(device);


        driver.write_bytes(true, &[0x01, 0x02, 0x03]).unwrap();
        driver.write_bytes(true, &[0x04]).unwrap();
        driver.write_bytes(false, &[0xAB]).unwrap();
        driver.config.i2c.done();
    }

    #[test]
    fn test_clear() {
        let i2c_address = 0x3e;
        let expected_i2c_transactions = std::vec![
            I2cTransaction::write(i2c_address, std::vec![
                0b0000_0000,
                0x01,
            ]),
        ];

        let i2c = I2cMock::new(&expected_i2c_transactions);
        let config = DeviceSetupConfig {
            i2c: i2c,
            address: i2c_address,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let mut device = ST7032i::new(config);
        let mut display = StandardCharacterDisplayHandler::default();

        assert!(display.clear(&mut device).is_ok());
        device.config.i2c.done();
    }

    #[test]
    fn test_print() {
        let i2c_address = 0x3e;
        let expected_i2c_transactions = std::vec![
            I2cTransaction::write(i2c_address, std::vec![
                0b0100_0000,
                0x48,
                0x65,
                0x6c,
                0x6c,
                0x6f,
                0x20,
                0x57,
                0x6f,
                0x72,
                0x6c,
                0x64,
            ]),
        ];

        let i2c = I2cMock::new(&expected_i2c_transactions);
        let config = DeviceSetupConfig {
            i2c: i2c,
            address: i2c_address,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let mut device = ST7032i::new(config);
        let mut display = StandardCharacterDisplayHandler::default();

        assert!(display.print(&mut device, "Hello World").is_ok());
        device.config.i2c.done();
    }

    #[test]
    fn test_create_char() {
        let i2c_address = 0x3e;
        let expected_i2c_transactions = std::vec![
            // send set CGRAM address command for location 2
            I2cTransaction::write(i2c_address, std::vec![
                0b0000_0000,    // control byte
                0x40 | (2 << 3),
            ]),
            // send the character data
            I2cTransaction::write(i2c_address, std::vec![
                0b0100_0000,    // control byte
                0b11011,
                0b10001,
                0b11011,
                0b00000,
                0b00000,
                0b00100,
                0b01110,
                0b10001,
            ]),
        ];
        let i2c = I2cMock::new(&expected_i2c_transactions);
        let config = DeviceSetupConfig {
            i2c: i2c,
            address: i2c_address,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let mut device = ST7032i::new(config);
        let mut display = StandardCharacterDisplayHandler::default();

        assert!(display.create_char(&mut device, 2, [0b11011, 0b10001, 0b11011, 0b00000, 0b00000, 0b00100, 0b01110, 0b10001]).is_ok());
        device.config.i2c.done();
    }

    #[test]
    fn test_set_contrast() {
        let contrast_value = 0x24;
        let i2c_address = 0x3e;
        let expected_i2c_transactions = std::vec![
            // send function set command to set to extended instruction mode
            I2cTransaction::write(i2c_address, std::vec![
                0b0000_0000,    // control byte - command
                0x39,   //  put device into extended instruction mode
            ]),
            // set the lower 4 bits of the contrast
            I2cTransaction::write(i2c_address, std::vec![
                0b0000_0000,    // control byte -  command
                0x70 | (contrast_value & 0x0F),
            ]),
            // set the higher 2 bits of the contrast
            I2cTransaction::write(i2c_address, std::vec![
                0b0000_0000,    // control byte -  command
                0x50 | 0x0C | ((contrast_value >> 4) & 0x03),
            ]),
            // return to normal instructions
            I2cTransaction::write(i2c_address, std::vec![
                0b0000_0000,    // control byte -  command
                0x38,
            ]),
        ];
        let i2c = I2cMock::new(&expected_i2c_transactions);
        let config = DeviceSetupConfig {
            i2c: i2c,
            address: i2c_address,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let mut device = ST7032i::new(config);
        let mut display: ST7032iDisplayActions<I2cMock, NoopDelay> = ST7032iDisplayActions::<I2cMock, NoopDelay>::default();
        assert!( <ST7032iDisplayActions<I2cMock, NoopDelay> as DisplayActionsTrait<I2cMock, NoopDelay, ST7032i<I2cMock, NoopDelay>>>::init_display_state(
            &mut display,
            0x18,
            0x04,
            0x02,
        ).is_ok());
        assert!(display.set_contrast(&mut device, contrast_value).is_ok());
        device.i2c().done();
    }
}