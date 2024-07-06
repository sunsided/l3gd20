# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

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
