# STMicroelectronics L3GD20 SPI driver

[![Crates.io][crates-image]][crates-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![MSRV][msrv-image]
[![EUPL 1.2 licensed][license-eupl-image]][license-eupl-link]
[![Apache 2.0 licensed][license-apache-image]][license-apache-link]
[![MIT licensed][license-mit-image]][license-mit-link]

> A platform-agnostic driver to interface with the L3GD20 gyroscope.

Do note that the sensor is discontinued and that documentation is scarce. Owners of an STM32F3 Discovery
board may still find this crate useful, among others.

## What works

- Reading the gyroscope in SPI blocking mode
- Direct access to registers via [`l3gd20-registers`] structs and [`hardware-registers`] traits

## License

Licensed under either of

- European Union Public Licence, Version 1.2, ([LICENSE-EUPL](LICENSE-EUPL)
  or https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be triple licensed as above, without any
additional terms or conditions.

## Code of Conduct

We abide by the [Contributor Covenant][cc] and ask that you do as well.

[crates-image]: https://img.shields.io/crates/v/lsm303dlhc-ng

[crates-link]: https://crates.io/crates/lsm303dlhc-ng

[docs-image]: https://docs.rs/lsm303dlhc-ng/badge.svg

[docs-link]: https://docs.rs/lsm303dlhc-ng/

[build-image]: https://github.com/sunsided/l3gd20/workflows/Rust/badge.svg

[build-link]: https://github.com/sunsided/l3gd20/actions

[msrv-image]: https://img.shields.io/badge/rustc-1.64+-blue.svg

[license-eupl-image]: https://img.shields.io/badge/license-EUPL_1.2-blue.svg

[license-apache-image]: https://img.shields.io/badge/license-Apache_2.0-blue.svg

[license-mit-image]: https://img.shields.io/badge/license-MIT-blue.svg

[license-apache-link]: https://github.com/sunsided/l3gd20/blob/develop/LICENSE-APACHE

[license-mit-link]: https://github.com/sunsided/l3gd20/blob/develop/LICENSE-MIT

[license-eupl-link]: https://github.com/sunsided/l3gd20/blob/develop/LICENSE-EUPL

[cc]: https://contributor-covenant.org

[`l3gd20-registers`]: https://crates.io/crates/l3gd20-registers

[`hardware-registers`]: https://crates.io/crates/hardware-registers
