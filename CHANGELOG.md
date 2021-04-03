# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Changed

- Implement `embedded_hal::digital::IoPin` for `CdevPin` and `SysfsPin`
- Set default features to build both sysfs and cdev pin types.
- Removed `Pin` export, use `CdevPin` or `SysfsPin`.
- Increased the Minimum Supported Rust Version to `1.36.0` due to an update of `gpio_cdev`.
- Adapted to `embedded-hal` `1.0.0-alpha.3` release.
- Updated `nb` to version `1`.

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

[Unreleased]: https://github.com/japaric/linux-embedded-hal/compare/v0.2.1...HEAD
[v0.2.2]: https://github.com/japaric/linux-embedded-hal/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/japaric/linux-embedded-hal/compare/v0.1.0...v0.1.1
