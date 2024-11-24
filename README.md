# I2C Character Display
[![crates.io](https://img.shields.io/crates/v/i2c-character-display.svg)](https://crates.io/crates/i2c-character-display)
<!-- cargo-sync-readme start -->

This Rust `embedded-hal`-based library is a simple way to control a character display that has either a [HD44780](https://en.wikipedia.org/wiki/Hitachi_HD44780_LCD_controller)
or [AiP31068](https://support.newhavendisplay.com/hc/en-us/articles/4414486901783--AiP31068) contoller with an I2C interface
in an embedded, `no_std` environment. A number of I2C interfaces are supported:

- **[Adafruit I2C/SPI LCD Backpack](https://www.adafruit.com/product/292)** - This is a simple I2C adapter for HD44780 character displays that can be used with either I2C
  or SPI. It is available from Adafruit and other retailers. This library only supports the I2C interface of this adapter.
- **PCF8574-based I2C adapter** - These adapters are ubiquitous on eBay and AliExpress and have no clear branding. Furthermore, some HD44780-based character
  display makers, such as [Surenoo](https://www.surenoo.com), integrate a PCF8574T directly on the display board enabling I2C connections without a seperate adapter.
  The most common pin wiring uses 4 data pins and 3 control pins. Most models have the display's 4-bit mode data pins connected to P4-P7 of the PCF8574.
  This library supports that configuration, though it would be straightforward to add support for other pin configurations.
- **AiP31068** - This is a character display controller with a built-in I2C support. The command set is similar to the HD44780, but the controller
  operates in 8-bit mode and is initialized differently.

Key features include:
- Convenient high-level API for controlling many types of character display
- Support for custom characters
- Backlight control on hardwarware that supports it
- `core::fmt::Write` implementation for easy use with the `write!` macro
- Compatible with the `embedded-hal` traits v1.0 and later
- Support for character displays that uses multiple HD44780 drivers, such as the 40x4 display
- Optional support for the `defmt` and `ufmt` logging frameworks
- Optional support for reading from the display on controllers and adapters that support it

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
i2c-character-display = { version = "0.4", features = ["defmt"] }
```
The `features = ["defmt"]` line is optional and enables the `defmt` feature, which allows the library's errors to be used with the `defmt` logging
framework. Another optional feature is `features = ["ufmt"]`, which enables the `ufmt` feature, allowing the `uwriteln!` and `uwrite!` macros to be used.

Then select the appropriate adapter for your display:
```rust
use i2c_character_display::{AdafruitLCDBackpack, CharacterDisplayPCF8574T, LcdDisplayType};
use embedded_hal::delay::DelayMs;
use embedded_hal::i2c::I2c;

// board setup
let i2c = ...; // I2C peripheral
let delay = ...; // DelayMs implementation

// It is recommended that the `i2c` object be wrapped in an `embedded_hal_bus::i2c::CriticalSectionDevice` so that it can be shared between
// multiple peripherals.

// Adafruit backpack for a single HD44780 controller
let mut lcd = AdafruitLCDBackpack::new(i2c, LcdDisplayType::Lcd16x2, delay);
// PCF8574T adapter for a single HD44780 controller
let mut lcd = CharacterDisplayPCF8574T::new(i2c, LcdDisplayType::Lcd16x2, delay);
// Character display with dual HD44780 controllers using a single PCF8574T I2C adapter
let mut lcd = CharacterDisplayDualHD44780::new(i2c, LcdDisplayType::Lcd40x4, delay);
// Character display with the AiP31068 controller
let mut lcd = CharacterDisplayAIP31068::new(i2c, LcdDisplayType::Lcd16x2, delay);
```
When creating the display object, you can choose the display type from the `LcdDisplayType` enum. The display type should match the physical
display you are using. This display type configures the number of rows and columns, and the internal row offsets for the display.

Initialize the display:
```rust
if let Err(e) = lcd.init() {
   panic!("Error initializing LCD: {}", e);
}
```
Use the display:
```rust
// set up the display
lcd.backlight(true)?.clear()?.home()?;
// print a message
lcd.print("Hello, world!")?;
// can also use the `core::fmt::write!` macro
use core::fmt::Write;

write!(lcd, "Hello, world!")?;
```
The optional `ufmt` feature enables the `ufmt` crate, which allows the `uwriteln!` and `uwrite!` macros to be used with the display:
```rust
use ufmt::uwriteln;

uwriteln!(lcd, "Hello, world!")?;
```

The various methods for controlling the LCD are also available. Each returns a `Result` that wraps the display object in `Ok()`, allowing for easy chaining
of commands. For example:
```rust
lcd.backlight(true)?.clear()?.home()?.print("Hello, world!")?;
```
### Reading from the display
Some I2C adapters support reading data from the HD44780 controller. For the I2C adapters that support it, the `read_device_data` method can be used to read
from either the CGRAM or DDRAM at the current cursor position. The `read_address_counter` method can be used to read the address counter from the HD44780 controller.
In both cases, the specific meaning of the data depends on the prior commands sent to the display. See the HD44780 datasheet for more information.

### Backlight control
All HD44780 controllers support backlight control. The `backlight` method can be used to turn the backlight on or off. The AiP31068 controller does not support
backlight control, and calling the `backlight` method with a AiP31068 controller will return an error.

### Multiple HD44780 controller character displays
Some character displays, such as the 40x4 display, use two HD44780 controllers to drive the display. This library supports these displays by
treating them as one logical display with multiple HD44780 controllers. The `CharacterDisplayDualHD44780` type is used to control these displays.
Use the various methods to control the display as you would with a single HD44780 controller display. The `set_cursor` method sets the active HD44780
conmtroller device based on the row number you select.


<!-- cargo-sync-readme end -->

## License
Licensed under the [MIT](LICENSE) license.