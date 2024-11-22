use bitfield::bitfield;
use core::marker::PhantomData;
use embedded_hal::i2c;

use super::{AdapterConfigTrait, AdapterError};
use crate::LcdDisplayType;

bitfield! {
    pub struct GenericAiP31068ControlByte(u8);
    impl Debug;
    impl BitAnd;
    pub co, set_co: 7, 7;
    pub rs, set_rs: 6, 6;
}

impl Default for GenericAiP31068ControlByte {
    fn default() -> Self {
        Self(0)
    }
}

impl Clone for GenericAiP31068ControlByte {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

pub struct GenericAiP31068<I2C>
where
    I2C: i2c::I2c,
{
    _marker: PhantomData<I2C>,
}

impl<I2C> Default for GenericAiP31068<I2C>
where
    I2C: i2c::I2c,
{
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<I2C> AdapterConfigTrait<I2C> for GenericAiP31068<I2C>
where
    I2C: i2c::I2c,
{
    /// Returns the bitfield value for the adapter
    fn bits(&self) -> u8 {
        0
    }

    fn default_i2c_address() -> u8 {
        0x3E
    }

    fn init(&self, _i2c: &mut I2C, _i2c_address: u8) -> Result<(), I2C::Error> {
        Ok(())
    }

    /// Sets the RS pin for the display. A value of `false` indicates an instruction is being sent, while
    /// a value of `true` indicates data is being sent.
    fn set_rs(&mut self, value: bool) {}

    /// Sets the RW pin for the display. A value of `false` indicates a write operation, while a value of
    /// `true` indicates a read operation. Not all displays support reading, so this method may not be
    /// implemented fully.
    fn set_rw(&mut self, value: bool) {}

    /// Sets the enable pin for the given device. Most displays only have one enable pin, so the device
    /// parameter is ignored. For displays with two enable pins, the device parameter is used to determine
    /// which enable pin to set.
    fn set_enable(&mut self, value: bool, device: usize) -> Result<(), AdapterError<I2C>> {
        Ok(())
    }

    /// Sets the backlight pin for the display. A value of `true` indicates the backlight is on, while a value
    /// of `false` indicates the backlight is off.
    fn set_backlight(&mut self, value: bool) {}

    fn set_data(&mut self, value: u8) {}

    fn is_supported(display_type: LcdDisplayType) -> bool {
        true
    }
}
