[![crates.io](https://img.shields.io/crates/d/linux-embedded-hal.svg)](https://crates.io/crates/linux-embedded-hal)
[![crates.io](https://img.shields.io/crates/v/linux-embedded-hal.svg)](https://crates.io/crates/linux-embedded-hal)
[![Documentation](https://docs.rs/linux-embedded-hal/badge.svg)](https://docs.rs/linux-embedded-hal)
![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.84+-blue.svg)

# `linux-embedded-hal`

> Implementation of the [`embedded-hal`] traits for Linux devices

This project is developed and maintained by the [Embedded Linux team][team].

[`embedded-hal`]: https://crates.io/crates/embedded-hal

## [Documentation](https://docs.rs/linux-embedded-hal)

## GPIO character device

Since Linux kernel v4.4 the use of sysfs GPIO was deprecated and replaced by the character device GPIO.
See [gpio-cdev documentation](https://github.com/rust-embedded/gpio-cdev#sysfs-gpio-vs-gpio-character-device) for details.

This crate includes feature flag `gpio_cdev` that exposes `CdevPin` as wrapper around `LineHandle` from [gpio-cdev](https://crates.io/crates/gpio-cdev).
To enable it update your Cargo.toml. Please note that in order to prevent `LineHandle` fd from closing you should
assign to a variable, see [cdev issue](https://github.com/rust-embedded/gpio-cdev/issues/29) for more details.
```
linux-embedded-hal = { version = "0.4", features = ["gpio_cdev"] }
```

`SysfsPin` can be still used with feature flag `gpio_sysfs`.

With `default-features = false` you can enable the features `gpio_cdev`, `gpio_sysfs`, `i2c`, and `spi` as needed.

## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.84.0 and up. It *might*
compile with older versions but that may change in any new patch release.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Code of Conduct

Contribution to this crate is organized under the terms of the [Rust Code of
Conduct][CoC], the maintainer of this crate, the [HAL team][team], promises
to intervene to uphold that code of conduct.

[CoC]: CODE_OF_CONDUCT.md
[team]: https://github.com/rust-embedded/wg/#the-embedded-linux-team
