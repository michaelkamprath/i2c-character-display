# `i2c-character-display` Change Log

## [Unreleased]

## [0.2.1] - 2024-10-26
* Added support for `ufmt`
* Added `display_type()` method to `BaseCharacterDisplay` to allow for querying the display type
* Improved documentation

## [0.2.0]  - 2024-10-19
* Added support for 40x4 character displays using two HD44780 controllers with a PCF8574T I2C adapter wired with two enable pins, one for each controller.
* Improved unit tests

## 0.1.0
Initial release. Support for both Generic PCF8574T I2C and Adafruit Backpack character display adapters.

[Unreleased]: https://github.com/michaelkamprath/bespokeasm/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/michaelkamprath/bespokeasm/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/michaelkamprath/bespokeasm/compare/v0.1.0...v0.2.0