
use embedded_hal::{delay::DelayNs, i2c};

use crate::{
    driver::DeviceHardwareTrait,
    CharacterDisplayError, DeviceSetupConfig, LcdDisplayType,
};

const CONTROL_NOT_LAST_BYTE: u8 = 0b1000_0000;  // Another control byte will follow the next data byte.
const CONTROL_LAST_BYTE: u8 = 0b0000_0000;      // Last control byte. Only a stream of data bytes will follow.
const CONTROL_RS_DATA: u8 = 0b0100_0000;
const CONTROL_RS_COMMAND: u8 = 0b0000_0000;


const MAX_BUFFER_SIZE: usize = 82;      // 80 bytes of data + 2 control bytes.

/// AIP31068 device driver implementation
pub struct AIP31068<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    buffer: [u8; MAX_BUFFER_SIZE],  // buffer for I2C data
    config: DeviceSetupConfig<I2C, DELAY>,
}


impl<I2C, DELAY> DeviceHardwareTrait<I2C, DELAY> for AIP31068<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn new(config: DeviceSetupConfig<I2C, DELAY>) -> Self {
        AIP31068 {
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
        use crate::driver::standard::{
            LCD_FLAG_2LINE, LCD_FLAG_5x8_DOTS, LCD_CMD_FUNCTIONSET,
            LCD_FLAG_DISPLAYON, LCD_FLAG_CURSOROFF, LCD_FLAG_BLINKOFF, LCD_CMD_DISPLAYCONTROL,
            LCD_CMD_CLEARDISPLAY,
            LCD_FLAG_ENTRYLEFT, LCD_FLAG_ENTRYSHIFTDECREMENT, LCD_CMD_ENTRYMODESET,
        };

        #[cfg(feature = "defmt")]
        defmt::debug!("Initializing AIP31068");
        // wait 15 ms for power on
        self.config.delay.delay_ms(15);

        // send function set command
        let display_function: u8 = LCD_FLAG_2LINE | LCD_FLAG_5x8_DOTS;
        self.write_bytes(false, &[LCD_CMD_FUNCTIONSET | display_function])?;

        // wait 39 us
        self.config.delay.delay_us(39);

        // display on/off control
        let display_control: u8 = LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF;
        self.write_bytes( false, &[LCD_CMD_DISPLAYCONTROL | display_control])?;

        // wait 39 us
        self.config.delay.delay_us(39);

        // clear display
        self.write_bytes(false, &[LCD_CMD_CLEARDISPLAY])?;

        // wait 1.53 ms
        self.config.delay.delay_us(1530);

        // entry mode set
        let display_mode: u8 = LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT;
        self.write_bytes(false, &[LCD_CMD_ENTRYMODESET | display_mode])?;

        Ok((display_function, display_control, display_mode))
    }

    /// write one or more bytes to the display.
    /// The `rs_setting` parameter indcate if the data is a command or data. `true` for data, `false` for command.
    fn write_bytes(
        &mut self,
        rs_setting: bool,
        data: &[u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        #[cfg(feature = "defmt")]
        defmt::debug!("Writing byte: {}", data);
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
        #[cfg(feature = "defmt")]
        defmt::debug!("Built data to send: {}", &self.buffer[..idx]);
        self.config.i2c.write(self.config.address, &self.buffer[..idx]).map_err(CharacterDisplayError::I2cError)?;
        #[cfg(feature = "defmt")]
        defmt::debug!("Data sent");
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
        let mut driver = AIP31068::new(device);


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
        let mut device = AIP31068::new(config);
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
        let mut device = AIP31068::new(config);
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
        let mut device = AIP31068::new(config);
        let mut display = StandardCharacterDisplayHandler::default();

        assert!(display.create_char(&mut device, 2, [0b11011, 0b10001, 0b11011, 0b00000, 0b00000, 0b00100, 0b01110, 0b10001]).is_ok());
        device.config.i2c.done();
    }

}