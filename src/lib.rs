//! # STMicroelectronics L3GD20 SPI driver
//!
//! > A platform-agnostic SPI driver to interface with the L3GD20 gyroscope
//!
//! Do note that the sensor is discontinued and that documentation is scarce. Owners of an STM32F3 Discovery
//! board may still find this crate useful, among others.
//!
//! This driver was built using [`embedded-hal`] 1.0 and [`hardware-registers`] traits.
//!
//! [`embedded-hal`]: https://crates.io/crates/embedded-hal
//! [`hardware-registers`]: https://crates.io/crates/hardware-registers

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod characteristics;
mod error;
mod i16x3;
mod reading;
mod sensor_data;

use embedded_hal::spi::{Operation, SpiDevice};
use l3gd20_registers::prelude::{R1, RegisterAddress8, SPIRegister, TryFromBits};
use l3gd20_registers::*;

pub use characteristics::Characteristics;
pub use error::Error;
pub use i16x3::I16x3;
pub use reading::Reading;
pub use sensor_data::SensorData;

/// SPI Driver for the STMicroelectronics L3GD20 Gyroscope.
///
/// The [`SpiDevice`] owns the chip select. To share a bus, use one of the
/// `SpiDevice` implementations of [`embedded-hal-bus`](https://crates.io/crates/embedded-hal-bus).
pub struct L3GD20SPI<SPI> {
    spi: SPI,
}

impl<SPI> L3GD20SPI<SPI>
where
    SPI: SpiDevice,
{
    /// Mask for register addresses.
    const REG_ADDR_MASK: u8 = 0b0011_1111;

    /// Initialize the SPI connection.
    pub fn new(spi: SPI) -> Result<Self, Error<SPI::Error>> {
        let mut device = Self { spi };

        // Apply standard configuration.
        device.reset()?;
        Ok(device)
    }

    /// Consumes the driver and returns the SPI device.
    pub fn release(self) -> SPI {
        self.spi
    }

    /// Identifies this chip by querying the `WHO_AM_I` register.
    pub fn identify(&mut self) -> Result<bool, Error<SPI::Error>> {
        let ident = self.read_register::<WhoAmI>()?;
        if ident.ident() == L3GD20_WHO_AM_I {
            Ok(true)
        } else {
            #[cfg(feature = "defmt")]
            defmt::debug!(
                "L3GD20 sensor identification failed; got {:08b}",
                ident.ident()
            );
            Ok(false)
        }
    }

    /// Resets the device to reasonable defaults.
    pub fn reset(&mut self) -> Result<(), Error<SPI::Error>> {
        // Use a bulk write instead.
        self.write_register(
            ControlRegister1::default()
                .with_power_up(true)
                .with_x_enable(true)
                .with_y_enable(true)
                .with_z_enable(true)
                .with_output_data_rate(OutputDataRate::Hz95)
                .with_bandwidth(Bandwidth::Narrowest),
        )?;
        self.write_register(
            ControlRegister2::default()
                .with_hpm(HighpassFilterMode::NormalModeResetFilter)
                .with_hpcf(0),
        )?;
        self.write_register(
            ControlRegister3::default()
                .with_i1int1(false)
                .with_i1boot(false)
                .with_int1_low(false)
                .with_i2drdy(false)
                .with_i2wtm(false)
                .with_i2orun(false)
                .with_i2empty(false)
                .with_open_drain(false),
        )?;
        self.write_register(
            ControlRegister4::default()
                .with_block_data_update(false)
                .with_big_endian(false)
                .with_full_scale(Sensitivity::D250)
                .with_spi_serial_3wire(false),
        )?;
        self.write_register(ControlRegister5::default().with_boot(true))?; // toggle boot
        self.write_register(
            ControlRegister5::default()
                .with_boot(false)
                .with_fifo_enable(false)
                .with_hpen(false)
                .with_int1_sel(0)
                .with_out_sel(0),
        )?;

        Ok(())
    }

    /// Sets the be powered up and active.
    pub fn power_up(&mut self) -> Result<(), Error<SPI::Error>> {
        self.modify_register(|reg: ControlRegister1| {
            reg.with_power_up(true)
                .with_x_enable(true)
                .with_y_enable(true)
                .with_z_enable(true)
        })
    }

    /// Sets the device to sleep mode.
    pub fn sleep_mode(&mut self) -> Result<(), Error<SPI::Error>> {
        self.modify_register(|reg: ControlRegister1| {
            reg.with_power_up(true)
                .with_x_enable(false)
                .with_y_enable(false)
                .with_z_enable(false)
        })
    }

    /// Sets the device to be powered down.
    pub fn power_down(&mut self) -> Result<(), Error<SPI::Error>> {
        self.modify_register(|reg: ControlRegister1| reg.with_power_up(false))
    }

    /// Enables the data ready interrupt.
    pub fn enable_data_ready(&mut self, enabled: bool) -> Result<(), Error<SPI::Error>> {
        self.modify_register(|reg: ControlRegister3| reg.with_i2drdy(enabled))
    }

    /// Sets the output data rate.
    pub fn set_odr(&mut self, data_rate: OutputDataRate) -> Result<(), Error<SPI::Error>> {
        self.modify_register(|reg: ControlRegister1| reg.with_output_data_rate(data_rate))
    }

    /// Sets the sensitivity.
    pub fn set_sensitivity(&mut self, fs: Sensitivity) -> Result<(), Error<SPI::Error>> {
        self.modify_register(|reg: ControlRegister4| reg.with_full_scale(fs))
    }

    /// Sets the output data rate.
    pub fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<(), Error<SPI::Error>> {
        self.modify_register(|reg: ControlRegister1| reg.with_bandwidth(bandwidth))
    }

    /// Obtains sensor characteristics.
    /// The results of this call depend on the current configuration of the sensor and should
    /// be obtained when the sensor configuration was changed.
    pub fn characteristics(&mut self) -> Result<Characteristics, Error<SPI::Error>> {
        let data = self.temp_raw()?;
        let reg1 = self.read_register::<ControlRegister1>()?;
        let reg4 = self.read_register::<ControlRegister4>()?;

        let odr = reg1.output_data_rate();
        let bw = reg1.bandwidth();
        let fs = reg4.full_scale();

        Ok(Characteristics {
            full_scale: match fs {
                Sensitivity::D250 => 250,
                Sensitivity::D500 => 500,
                Sensitivity::D2000 => 2000,
                Sensitivity::D2000_11 => 2000,
            },
            sensitivity: match fs {
                Sensitivity::D250 => 8.75 * 0.001,     // mdeg/1000
                Sensitivity::D500 => 17.5 * 0.001,     // mdeg/1000
                Sensitivity::D2000 => 70.0 * 0.001,    // mdeg/1000
                Sensitivity::D2000_11 => 70.0 * 0.001, // mdeg/1000
            },
            zero_rate_noise: match fs {
                Sensitivity::D250 => 10.0,
                Sensitivity::D500 => 15.0,
                Sensitivity::D2000 => 75.0,
                Sensitivity::D2000_11 => 75.0,
            },
            zero_rate_level_temp: match fs {
                Sensitivity::D250 => 0.03 * f32::from(data),
                Sensitivity::D500 => 0.03 * f32::from(data),
                Sensitivity::D2000 => 0.04 * f32::from(data),
                Sensitivity::D2000_11 => 0.05 * f32::from(data),
            },
            rate_noise_density: 0.03 * bw.sqrt_hz_at(odr),
        })
    }

    /// Reads the raw temperature from the `OUT_TEMP` register.
    pub fn temp_raw(&mut self) -> Result<i8, Error<SPI::Error>> {
        let temp = self.read_register::<TemperatureRegister>()?;
        Ok(temp.temp())
    }

    /// Fetches X, Y and Z-axis data off the sensor.
    pub fn xyz_raw(&mut self) -> Result<I16x3, Error<SPI::Error>> {
        // The registers come in the order XL (0x28), XH, YL, YH, ZL, ZH (0x2D)
        let mut buffer = [0; 6];
        self.read_multi(*OutXLow::REGISTER_ADDRESS, &mut buffer)?;

        let xlo = OutXLow::from_bits(buffer[0]);
        let xhi = OutXHigh::from_bits(buffer[1]);
        let ylo = OutYLow::from_bits(buffer[2]);
        let yhi = OutYHigh::from_bits(buffer[3]);
        let zlo = OutZLow::from_bits(buffer[4]);
        let zhi = OutZHigh::from_bits(buffer[5]);

        let x = xhi + xlo;
        let y = yhi + ylo;
        let z = zhi + zlo;

        Ok(I16x3::new(x, y, z))
    }

    /// Fetches all data off the sensor.
    pub fn data_raw(&mut self) -> Result<SensorData, Error<SPI::Error>> {
        // The registers come in the order Temperature (0x26), Status (0x27), XL, XH, YL, YH, ZL, ZH (0x2D)
        let mut buffer = [0; 8];
        self.read_multi(*TemperatureRegister::REGISTER_ADDRESS, &mut buffer)?;

        let temp = TemperatureRegister::from_bits(buffer[0]);
        let status = StatusRegister::from_bits(buffer[1]);
        let xlo = OutXLow::from_bits(buffer[2]);
        let xhi = OutXHigh::from_bits(buffer[3]);
        let ylo = OutYLow::from_bits(buffer[4]);
        let yhi = OutYHigh::from_bits(buffer[5]);
        let zlo = OutZLow::from_bits(buffer[6]);
        let zhi = OutZHigh::from_bits(buffer[7]);

        let x = xhi + xlo;
        let y = yhi + ylo;
        let z = zhi + zlo;

        Ok(SensorData::new(temp.temp(), x, y, z, status))
    }

    /// Reads consecutive registers, starting at `address`, with address auto-increment.
    fn read_multi(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error<SPI::Error>> {
        let command = SPI_READ | SPI_AUTO_INCREMENT | (address & Self::REG_ADDR_MASK);
        self.spi
            .transaction(&mut [Operation::Write(&[command]), Operation::Read(buffer)])
            .map_err(Error::Spi)
    }

    /// Reads a single register.
    pub fn read_register<R>(&mut self) -> Result<R, Error<SPI::Error>>
    where
        R: ReadableRegister,
        Error<SPI::Error>: From<<R as TryFromBits<u8>>::Error>,
    {
        let command = SPI_READ
            | (*<R as SPIRegister<RegisterAddress8, R1>>::REGISTER_ADDRESS & Self::REG_ADDR_MASK);
        let mut buffer = [0];
        self.spi
            .transaction(&mut [Operation::Write(&[command]), Operation::Read(&mut buffer)])
            .map_err(Error::Spi)?;
        Ok(R::try_from_bits(buffer[0])?)
    }

    /// Writes a single register.
    pub fn write_register<B, R>(&mut self, register: B) -> Result<(), Error<SPI::Error>>
    where
        B: core::borrow::Borrow<R>,
        R: WritableRegister,
    {
        let byte = register.borrow().to_bits();
        let command =
            *<R as SPIRegister<RegisterAddress8, R1>>::REGISTER_ADDRESS & Self::REG_ADDR_MASK;
        self.spi.write(&[command, byte]).map_err(Error::Spi)
    }

    /// Modifies a single register.
    pub fn modify_register<F, R>(&mut self, f: F) -> Result<(), Error<SPI::Error>>
    where
        F: FnOnce(R) -> R,
        R: ReadableRegister + WritableRegister,
        Error<SPI::Error>: From<<R as TryFromBits<u8>>::Error>,
    {
        let register: R = self.read_register()?;
        let register = f(register);
        self.write_register(register)
    }
}
