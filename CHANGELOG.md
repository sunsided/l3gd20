# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added

- `Error` combines SPI errors with `Error::InvalidFifoMode` for reserved FIFO modes.
- `L3GD20SPI::release` returns the SPI device.

### Changed

- [**breaking**] Ported to `embedded-hal` 1.0. `L3GD20SPI` takes an `SpiDevice`, which owns
  the chip select; the `chip-select` dependency is gone.
- [**breaking**] All methods return `Error<SPI::Error>` instead of the SPI error.
- [**breaking**] Uses [`l3gd20-registers`] version `0.3.0`. `read_register` and
  `modify_register` require `ReadableRegister`.
- [**breaking**] Edition 2024; the MSRV is now Rust 1.85.
- Updated `defmt` to 1.0.

### Removed

- [**breaking**] `wrapper::refcell::RefCellSPI`. Use the `SpiDevice` implementations of
  `embedded-hal-bus` to share a bus.

### Fixed

- `SensorData::fresh` and `SensorData::fresh_or_overrun` now require all axes;
  `SensorData::overrun` reports whether any axis overran.
- `Characteristics::rate_noise_density` was five times too high at 380 Hz with
  `Bandwidth::Narrow`.

## [0.1.0] - 2024-07-06

[0.1.0]: https://github.com/sunsided/l3gd20/releases/tag/v0.1.0

### Added

- Initial release.
- Uses [`l3gd20-registers`] version `0.2.0` for register mapping.
- Added `Characteristics` type for obtaining sensor characteristics from
  the current configuration.
- Added `xyz_raw`, `temp_raw` and `data_raw` to obtain data individually.

## [0.1.0-alpha.1] - 2024-07-05

[0.1.0-alpha.1]: https://github.com/sunsided/l3gd20/releases/tag/v0.1.0-alpha.1

### Added

- Initial release.

[`l3gd20-registers`]: https://crates.io/crates/l3gd20-registers
