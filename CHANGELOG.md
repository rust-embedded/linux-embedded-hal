# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Fixed

- Fix UB (and remove unsafe block) in handling of SpiOperation::TransferInPlace

## [v0.4.0] - 2024-01-10

### Changed
- Updated to `embedded-hal` `1.0.0` release ([API changes](https://github.com/rust-embedded/embedded-hal/blob/master/embedded-hal/CHANGELOG.md#v100---2023-12-28))
- Updated to `embedded-hal-nb` `1.0.0` release ([API changes](https://github.com/rust-embedded/embedded-hal/blob/master/embedded-hal-nb/CHANGELOG.md#v100---2023-12-28))

## [v0.4.0-alpha.4] - 2024-01-03

### Changed
- [breaking-change] Replace serial-rs with the serialport-rs crate. `Serial::open` now needs a baud-rate argument as well.
- [breaking-change] Split `Spidev` into `SpidevDevice` and `SpidevBus`, implementing the respective `SpiDevice` and `SpiBus` traits (#100)
- Updated to `embedded-hal` `1.0.0-rc.3` release ([API changes](https://github.com/rust-embedded/embedded-hal/blob/master/embedded-hal/CHANGELOG.md#v100-rc3---2023-12-14))
- Updated to `embedded-hal-nb` `1.0.0-rc.3` release ([API changes](https://github.com/rust-embedded/embedded-hal/blob/master/embedded-hal-nb/CHANGELOG.md#v100-rc3---2023-12-14))
- Updated to `spidev` `0.6.0` release([API changes](https://github.com/rust-embedded/rust-spidev/blob/master/CHANGELOG.md#060--2023-08-03))
- Updated to `i2cdev` `0.6.0` release([API changes](https://github.com/rust-embedded/rust-i2cdev/blob/master/CHANGELOG.md#v060---2023-08-03))
- Updated to `gpio_cdev` `0.6.0` release([API changes](https://github.com/rust-embedded/gpio-cdev/blob/master/CHANGELOG.md#v060--2023-09-11))
- Updated to `nix` `0.27.1`
- MSRV is now 1.65.0.

### Fixed
- Fix using SPI transfer with unequal buffer sizes (#97, #98).

## [v0.4.0-alpha.3] - 2022-08-04

### Added

- Added feature flag for `spi` and `i2c`

### Changed

- Updated to `embedded-hal` `1.0.0-alpha.8` release ([API changes](https://github.com/rust-embedded/embedded-hal/blob/master/embedded-hal/CHANGELOG.md#v100-alpha8---2022-04-15))

## [v0.4.0-alpha.2] - 2022-02-15

### Added

- Mappings for `embedded-hal` error kinds
### Changed

- Updated to `embedded-hal` `1.0.0-alpha.7` release (significant [API changes](https://github.com/rust-embedded/embedded-hal/blob/master/embedded-hal/CHANGELOG.md#v100-alpha7---2022-02-09))
- Updated dependencies to force use of newer nix version
  - `spidev` to version `0.5.1`
  - `i2cdev` to version `0.5.1`
  - `gpio-cdev` to version `0.5.1`
  - `sysfs_gpio` to version `0.6.1`

## [v0.4.0-alpha.1] - 2021-10-07

### Added

- Implement `embedded_hal::digital::blocking::IoPin` for `CdevPin` and `SysfsPin`
- `CountDown` implementation for `SysTimer`.
- `Default` implementation for `SysTimer`.

### Changed

- Modified `OutputPin` behavior for active-low pins to match `InputPin` behavior.
- Set default features to build both sysfs and cdev pin types.
- Removed `Pin` export, use `CdevPin` or `SysfsPin`.
- Adapted to `embedded-hal` `1.0.0-alpha.5` release.
- Increased the Minimum Supported Rust Version to `1.46.0` due to an update of `bitflags`.
- Updated `spidev` to version `0.5`.
- Updated `i2cdev` to version `0.5`.
- Updated `gpio-cdev` to version `0.5`.
- Updated `sysfs_gpio` to version `0.6`.
- Updated `nb` to version `1`.

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

[Unreleased]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.4.0...HEAD
[v0.4.0]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.4.0-alpha.4...v0.4.0
[v0.4.0-alpha.4]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.4.0-alpha.3...v0.4.0-alpha.4
[v0.4.0-alpha.3]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.4.0-alpha.2...v0.4.0-alpha.3
[v0.4.0-alpha.2]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.4.0-alpha.1...v0.4.0-alpha.2
[v0.4.0-alpha.1]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.3.0...v0.4.0-alpha.1
[v0.3.2]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.3.1...v0.3.2
[v0.3.1]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.2.2...v0.3.0
[v0.2.2]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/rust-embedded/linux-embedded-hal/compare/v0.1.0...v0.1.1
