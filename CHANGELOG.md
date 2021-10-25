# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]


## [v0.3.2] - 2021-10-25

### Fixed
- Readd `Pin` type export as an alias to `SysfsPin` for compatibility with the `0.3.0` version.

## [v0.3.1] - 2021-09-27
### Added

- Added implementation of transactional SPI and I2C traits.
- `CountDown` implementation for `SysTimer`.
- `Default` implementation for `SysTimer`.

### Changed

- Set default features to build both sysfs and cdev pin types.
- Removed `Pin` export, use `CdevPin` or `SysfsPin`.
- Updated `embedded-hal` to version `0.2.6`.
- Updated `nb` to version `0.1.3`.
- Updated `gpio-cdev` to version `0.5`.
- Updated `i2cdev` to version `0.5`.
- Updated `spidev` to version `0.5`.
- Updated `sysfs-gpio` to version `0.6`.
- Updated `cast` to version `0.3`.

### Fixed

- Modified `OutputPin` behavior for active-low pins to match `InputPin` behavior.

## [v0.3.0] - 2019-11-25

### Added

- Added serial::Read/Write implementation.
- Added feature flag for Chardev GPIO

### Fixed

- Do write and read in one transaction in WriteRead implementation.
- Removed #[deny(warnings)]

### Changed

- Use embedded-hal::digital::v2 traits.
- Updated to i2cdev 0.4.3 (necessary for trasactional write-read).
- Updated to spidev 0.4
- Added feature flag for Sysfs GPIO

## [v0.2.2] - 2018-12-21

### Changed

- updated to i2cdev 0.4.1 (removes superflous dependencies)

## [v0.2.1] - 2018-10-25

### Added

- implementation of the unproven `embedded_hal::::digital::InputPin` trait.

## [v0.2.0] - 2018-05-14

### Changed

- [breaking-change] moved to v0.2.x of `embedded-hal`.

## [v0.1.1] - 2018-02-13

### Added

- implementation of `embedded_hal::blocking::Delay*` traits in the form of the `Delay` zero sized
  type.

- implementation of the `embedded_hal::blocking::i2c` traits in the form of the `I2cdev` newtype.

## v0.1.0 - 2018-01-17

Initial release

[Unreleased]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.3.2...HEAD
[v0.3.2]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.3.1...v0.3.2
[v0.3.1]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.2.2...v0.3.0
[v0.2.2]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.1.0...v0.1.1
