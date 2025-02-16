use bitfield::bitfield;
use core::marker::PhantomData;
use embedded_hal::{delay::DelayNs, i2c};

use crate::{
    driver::DeviceHardwareTrait, CharacterDisplayError, DeviceSetupConfig, LcdDisplayType,
};

use super::HD44780AdapterTrait;

// Configuration for the PCF8574T based 4-bit LCD interface sold
bitfield! {
    pub struct GenericPCF8574TBitField(u8);
    impl Debug;
    impl BitAnd;
    pub rs, set_rs: 0, 0;
    pub rw, set_rw: 1, 1;
    pub enable, set_enable: 2, 2;
    pub backlight, set_backlight: 3, 3;
    pub data, set_data: 7, 4;
}

impl Clone for GenericPCF8574TBitField {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

/// Adapter based on the PCF8574T I2C GPIO expander interfacing with the HD44780 LCD controller
/// via a 4-bit interface.
pub struct GenericPCF8574TAdapter<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    bits: GenericPCF8574TBitField,
    config: DeviceSetupConfig<I2C, DELAY>,
    _marker: PhantomData<I2C>,
}

impl<I2C, DELAY> DeviceHardwareTrait<I2C, DELAY> for GenericPCF8574TAdapter<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn new(config: DeviceSetupConfig<I2C, DELAY>) -> Self {
        Self {
            bits: GenericPCF8574TBitField(0),
            config: config,
            _marker: PhantomData,
        }
    }

    fn default_i2c_address() -> u8 {
        0x27
    }

    fn supports_reads() -> bool {
        true
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

    fn init(&mut self) -> Result<(u8, u8, u8), CharacterDisplayError<I2C>> {
        self.adapter_init()
    }

    fn write_bytes(
        &mut self,
        _rs_setting: bool,
        _data: &[u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        todo!()
    }
}
impl<I2C, DELAY> HD44780AdapterTrait<I2C, DELAY> for GenericPCF8574TAdapter<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    fn device_config(&mut self) -> &mut DeviceSetupConfig<I2C, DELAY> {
        &mut self.config
    }

    fn bits(&self) -> u8 {
        self.bits.0
    }

    fn read_bytes_from_controller(
        &mut self,
        controller: usize,
        rs_setting: bool,
        buffer: &mut [u8],
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if controller != 0 {
            return Err(CharacterDisplayError::BadDeviceId);
        }
        // wait for the BUSY flag to clear
        let i2c_address = self.config.address;
        while self.is_busy()? {
            // wait
        }

        // now we can read the data. Set up PCF8574T to read data
        let mut data_cntl = self.bits.clone();
        data_cntl.set_data(0b1111);
        data_cntl.set_enable(0);
        data_cntl.set_rs(rs_setting as u8);
        data_cntl.set_rw(1); // read
        self.config
            .i2c
            .write(i2c_address, &[data_cntl.0])
            .map_err(CharacterDisplayError::I2cError)?;

        // not that is is set up, read bytes into buffer
        let mut data_buf = [0];
        for byte in buffer {
            *byte = 0;
            // read high nibble
            data_cntl.set_enable(1);
            self.config
                .i2c
                .write(i2c_address, &[data_cntl.0])
                .map_err(CharacterDisplayError::I2cError)?;
            self.config
                .i2c
                .read(i2c_address, &mut data_buf)
                .map_err(CharacterDisplayError::I2cError)?;
            data_cntl.set_enable(0);
            self.config
                .i2c
                .write(i2c_address, &[data_cntl.0])
                .map_err(CharacterDisplayError::I2cError)?;
            *byte = GenericPCF8574TBitField(data_buf[0]).data() << 4;

            // read low nibble
            data_cntl.set_enable(1);
            self.config
                .i2c
                .write(i2c_address, &[data_cntl.0])
                .map_err(CharacterDisplayError::I2cError)?;
            self.config
                .i2c
                .read(i2c_address, &mut data_buf)
                .map_err(CharacterDisplayError::I2cError)?;
            data_cntl.set_enable(0);
            self.config
                .i2c
                .write(i2c_address, &[data_cntl.0])
                .map_err(CharacterDisplayError::I2cError)?;
            *byte |= GenericPCF8574TBitField(data_buf[0]).data() & 0x0F;
        }
        Ok(())
    }

    fn is_busy(&mut self) -> Result<bool, CharacterDisplayError<I2C>> {
        // need to set all data bits to HIGH to read, per PFC8574 data sheet description of Quasi-bidirectional I/Os
        let mut setup = self.bits.clone();
        setup.set_data(0b1111);
        setup.set_rs(0);
        setup.set_rw(1);
        setup.set_enable(0);
        self.config
            .i2c
            .write(self.config.address, &[setup.0])
            .map_err(CharacterDisplayError::I2cError)?;
        // need two enable cycles to read the data, but the busy flag is in the 4th bit of the first
        // nibble, so we only need to read the first nibble
        setup.set_enable(1);
        self.config
            .i2c
            .write(self.config.address, &[setup.0])
            .map_err(CharacterDisplayError::I2cError)?;
        let mut data = [0];
        self.config
            .i2c
            .read(self.config.address, &mut data)
            .map_err(CharacterDisplayError::I2cError)?;
        let read_data = GenericPCF8574TBitField(data[0]);
        // turn off the enable bit so next nibble can be read
        setup.set_enable(0);
        self.config
            .i2c
            .write(self.config.address, &[setup.0])
            .map_err(CharacterDisplayError::I2cError)?;
        // toggle enable one more time per the 4-bit interface for the HD44780
        setup.set_enable(1);
        self.config
            .i2c
            .write(self.config.address, &[setup.0])
            .map_err(CharacterDisplayError::I2cError)?;
        setup.set_enable(0);
        self.config
            .i2c
            .write(self.config.address, &[setup.0])
            .map_err(CharacterDisplayError::I2cError)?;

        Ok(read_data.data() & 0b1000 != 0)
    }

    fn set_rs(&mut self, value: bool) {
        self.bits.set_rs(value as u8);
    }

    fn set_rw(&mut self, value: bool) {
        self.bits.set_rw(value as u8);
    }

    fn set_enable(
        &mut self,
        value: bool,
        controller: usize,
    ) -> Result<(), CharacterDisplayError<I2C>> {
        if controller != 0 {
            return Err(CharacterDisplayError::BadDeviceId);
        }
        self.bits.set_enable(value as u8);
        Ok(())
    }

    fn set_backlight(&mut self, value: bool) -> Result<(), CharacterDisplayError<I2C>> {
        self.bits.set_backlight(value as u8);
        self.write_bits_to_gpio()
    }

    fn set_data(&mut self, value: u8) {
        self.bits.set_data(value);
    }

    fn is_supported(display_type: LcdDisplayType) -> bool {
        display_type != LcdDisplayType::Lcd40x4
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use crate::DeviceSetupConfig;

    use super::*;
    use embedded_hal_mock::eh1::{
        delay::NoopDelay,
        i2c::{Mock as I2cMock, Transaction as I2cTransaction},
    };

    #[test]
    fn test_generic_pcf8574t_compatiple_lcd_types() {
        // not exhaustive for compatible displays (is_supported == true)
        assert!(GenericPCF8574TAdapter::<I2cMock, NoopDelay>::is_supported(
            LcdDisplayType::Lcd16x2
        ));
        assert!(GenericPCF8574TAdapter::<I2cMock, NoopDelay>::is_supported(
            LcdDisplayType::Lcd20x4
        ));
        assert!(GenericPCF8574TAdapter::<I2cMock, NoopDelay>::is_supported(
            LcdDisplayType::Lcd40x2
        ));
        assert!(!GenericPCF8574TAdapter::<I2cMock, NoopDelay>::is_supported(
            LcdDisplayType::Lcd40x4
        ));
    }

    #[test]
    fn test_generic_pcf8574t_bits() {
        let expected_transactions = [
            I2cTransaction::write(0x27, std::vec![0b1010_1101]),
            I2cTransaction::write(0x27, std::vec![0b0101_0010]),
        ];
        let mut device = GenericPCF8574TAdapter::<I2cMock, NoopDelay>::new(DeviceSetupConfig {
            i2c: I2cMock::new(&expected_transactions),
            address: 0x27,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        });
        device.set_rs(true);
        device.set_rw(false);
        assert!(device.set_enable(true, 0).is_ok());
        device.set_data(0b1010);
        assert!(device.set_backlight(true).is_ok());

        assert_eq!(device.bits(), 0b10101101);
        assert_eq!(
            GenericPCF8574TAdapter::<I2cMock, NoopDelay>::default_i2c_address(),
            0x27
        );

        device.set_rs(false);
        device.set_rw(true);
        assert!(!device.set_enable(false, 1).is_ok());
        assert!(device.set_enable(false, 0).is_ok());
        device.set_data(0b0101);
        assert!(device.set_backlight(false).is_ok());

        assert_eq!(device.bits(), 0b01010010);
        device.i2c().done();
    }

    #[test]
    fn test_generic_pcf8574t_write_bits() {
        let expected_transactions = [I2cTransaction::write(0x27, std::vec![0b10100101])];
        let i2c = I2cMock::new(&expected_transactions);
        let config = DeviceSetupConfig {
            i2c: i2c,
            address: 0x27,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let mut adapter = GenericPCF8574TAdapter::<I2cMock, NoopDelay>::new(config);
        adapter.set_rs(true);
        adapter.set_rw(false);
        assert!(adapter.set_enable(true, 0).is_ok());
        adapter.set_data(0b1010);
        assert!(adapter.set_backlight(false).is_ok());
        adapter.i2c().done();
    }

    #[test]
    fn test_generic_pcf8574t_write_byte() {
        let expected_transactions = [
            // wrtie byte 0xDE with RS = 1
            // write high nibble
            I2cTransaction::write(0x27, std::vec![0b11010101]), // enable = 1, rs = 1
            I2cTransaction::write(0x27, std::vec![0b11010001]), // enable = 0, rs = 1
            // write low nibble
            I2cTransaction::write(0x27, std::vec![0b11100101]), // enable = 1, rs = 1
            I2cTransaction::write(0x27, std::vec![0b11100001]), // enable = 0, rs = 1
            // wrtie byte 0xAD with RS = 0
            // write high nibble
            I2cTransaction::write(0x27, std::vec![0b10100100]), // enable = 1, rs = 0
            I2cTransaction::write(0x27, std::vec![0b10100000]), // enable = 0, rs = 0
            // write low nibble
            I2cTransaction::write(0x27, std::vec![0b11010100]), // enable = 1, rs = 0
            I2cTransaction::write(0x27, std::vec![0b11010000]), // enable = 0, rs = 0
        ];
        let i2c = I2cMock::new(&expected_transactions);
        let config = DeviceSetupConfig {
            i2c: i2c,
            address: 0x27,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let mut adapter = GenericPCF8574TAdapter::<I2cMock, NoopDelay>::new(config);

        assert!(adapter.write_byte_to_controller(0, true, 0xDE).is_ok());
        assert!(adapter.write_byte_to_controller(0, false, 0xAD).is_ok());
        adapter.i2c().done();
    }

    #[test]
    fn test_generic_pcf8574t_config_read_bytes() {
        let expected_transactions = [
            // set up PCF8574T to read data for is busy check - true
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // read high nibble
            I2cTransaction::write(0x27, std::vec![0b11110110]),
            I2cTransaction::read(0x27, std::vec![0b10100110]),
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // read low nibble
            I2cTransaction::write(0x27, std::vec![0b11110110]),
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // set up PCF8574T to read data for is busy check - false
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // read high nibble
            I2cTransaction::write(0x27, std::vec![0b11110110]),
            I2cTransaction::read(0x27, std::vec![0b00100110]),
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // read low nibble
            I2cTransaction::write(0x27, std::vec![0b11110110]),
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // set up PCF8574T to read data for data read
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // Byte 0 = $DE
            // read high nibble
            I2cTransaction::write(0x27, std::vec![0b11110110]),
            I2cTransaction::read(0x27, std::vec![0b11010110]),
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // read low nibble
            I2cTransaction::write(0x27, std::vec![0b11110110]),
            I2cTransaction::read(0x27, std::vec![0b11100110]),
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // Byte 0 = $AD
            // read high nibble
            I2cTransaction::write(0x27, std::vec![0b11110110]),
            I2cTransaction::read(0x27, std::vec![0b10100110]),
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // read low nibble
            I2cTransaction::write(0x27, std::vec![0b11110110]),
            I2cTransaction::read(0x27, std::vec![0b11010110]),
            I2cTransaction::write(0x27, std::vec![0b11110010]),
        ];
        let i2c = I2cMock::new(&expected_transactions);
        let config = DeviceSetupConfig {
            i2c: i2c,
            address: 0x27,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let mut adapter = GenericPCF8574TAdapter::<I2cMock, NoopDelay>::new(config);

        let buffer = &mut [0u8; 2];
        assert!(adapter.read_bytes_from_controller(0, false, buffer).is_ok());
        assert_eq!(buffer, &[0xDE, 0xAD]);
        adapter.i2c().done();
    }

    #[test]
    fn test_generic_pcf8574t_is_not_busy() {
        let expected_transactions = [
            // set up PCF8574T to read data
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // read high nibble
            I2cTransaction::write(0x27, std::vec![0b11110110]),
            I2cTransaction::read(0x27, std::vec![0b00100110]),
            I2cTransaction::write(0x27, std::vec![0b11110010]),
            // read low nibble
            I2cTransaction::write(0x27, std::vec![0b11110110]),
            I2cTransaction::write(0x27, std::vec![0b11110010]),
        ];
        let i2c = I2cMock::new(&expected_transactions);
        let config = DeviceSetupConfig {
            i2c: i2c,
            address: 0x27,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let mut adapter = GenericPCF8574TAdapter::<I2cMock, NoopDelay>::new(config);

        let is_busy = adapter.is_busy().unwrap();

        assert_eq!(is_busy, false);
        adapter.i2c().done();
    }

    #[test]
    fn test_set_enable_controllor_out_of_range() {
        let i2c = I2cMock::new(&[]);
        let config = DeviceSetupConfig {
            i2c: i2c,
            address: 0x27,
            lcd_type: LcdDisplayType::Lcd16x4,
            delay: NoopDelay,
        };
        let mut adapter = GenericPCF8574TAdapter::<I2cMock, NoopDelay>::new(config);
        assert!(adapter.set_enable(true, 1).is_err());
        assert!(adapter.set_enable(true, 0).is_ok());
        adapter.i2c().done();
    }
}
