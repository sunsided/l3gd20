//! # STMicroelectronics L3GD20 SPI driver
//!
//! > A platform-agnostic SPI driver to interface with the L3GD20 gyroscope
//!
//! Do note that the sensor is discontinued and that documentation is scarce. Owners of an STM32F3 Discovery
//! board may still find this crate useful, among others.
//!
//! This driver was built using [`embedded-hal`] and [`hardware-registers`] traits.
//!
//! [`embedded-hal`]: https://crates.io/crates/embedded-hal
//! [`hardware-registers`]: https://crates.io/crates/hardware-registers

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod characteristics;
mod i16x3;
mod reading;
mod sensor_data;
pub mod wrapper;

use chip_select::{ChipSelect, ChipSelectGuarded};
use embedded_hal::blocking::spi::Transfer;
use l3gd20_registers::prelude::SPIRegister;
use l3gd20_registers::*;

pub use characteristics::Characteristics;
pub use i16x3::I16x3;
pub use reading::Reading;
pub use sensor_data::SensorData;

/// SPI Driver for Bosch Sensortec L3GD20 Gyroscope
#[allow(non_snake_case)]
pub struct L3GD20SPI<CS, SPI> {
    cs: CS,
    spi: SPI,
}

impl<CS, SPI, E> L3GD20SPI<CS, SPI>
where
    CS: ChipSelect,
    SPI: Transfer<u8, Error = E>,
{
    /// Bit flag for a read command.
    const READ: u8 = 0b1000_0000;

    /// Bit flag for a write command.
    const WRITE: u8 = 0b0000_0000;

    /// Bit flag for a multi-address command; auto-increments addresses after each transfer.
    const MULTI: u8 = 0b0100_0000;

    /// Bit flag for a single-address command.
    const SINGLE: u8 = 0b0000_0000;

    /// Mask for register addresses.
    const REG_ADDR_MASK: u8 = 0b0011_1111;

    /// Initialize the SPI connection.
    #[allow(clippy::too_many_arguments)]
    pub fn new(spi: SPI, chip_select: CS) -> Result<Self, E>
    where
        CS: ChipSelectGuarded,
    {
        let mut device = Self {
            cs: chip_select,
            spi,
        };

        // Apply standard configuration.
        device.reset()?;
        Ok(device)
    }

    /// Identifies this chip by querying the `WHO_AM_I` register.
    pub fn identify(&mut self) -> Result<bool, E>
    where
        CS: ChipSelectGuarded,
    {
        let ident = self.read_register::<WhoAmI>()?;
        if ident.ident() == 0b11010100 {
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
    pub fn reset(&mut self) -> Result<(), E>
    where
        CS: ChipSelectGuarded,
    {
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
    pub fn power_up(&mut self) -> Result<(), E>
    where
        CS: ChipSelectGuarded,
    {
        self.modify_register(|reg: ControlRegister1| {
            reg.with_power_up(true)
                .with_x_enable(true)
                .with_y_enable(true)
                .with_z_enable(true)
        })
    }

    /// Sets the device to sleep mode.
    pub fn sleep_mode(&mut self) -> Result<(), E>
    where
        CS: ChipSelectGuarded,
    {
        self.modify_register(|reg: ControlRegister1| {
            reg.with_power_up(true)
                .with_x_enable(false)
                .with_y_enable(false)
                .with_z_enable(false)
        })
    }

    /// Sets the device to be powered down.
    pub fn power_down(&mut self) -> Result<(), E>
    where
        CS: ChipSelectGuarded,
    {
        self.modify_register(|reg: ControlRegister1| reg.with_power_up(false))
    }

    /// Enables the data ready interrupt.
    pub fn enable_data_ready(&mut self, enabled: bool) -> Result<(), E>
    where
        CS: ChipSelectGuarded,
    {
        self.modify_register(|reg: ControlRegister3| reg.with_i2drdy(enabled))
    }

    /// Sets the output data rate.
    pub fn set_odr(&mut self, data_rate: OutputDataRate) -> Result<(), E>
    where
        CS: ChipSelectGuarded,
    {
        self.modify_register(|reg: ControlRegister1| reg.with_output_data_rate(data_rate))
    }

    /// Sets the output data rate.
    pub fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<(), E>
    where
        CS: ChipSelectGuarded,
    {
        self.modify_register(|reg: ControlRegister1| reg.with_bandwidth(bandwidth))
    }

    /// Obtains sensor characteristics.
    /// The results of this call depend on the current configuration of the sensor and should
    /// be obtained when the sensor configuration was changed.
    pub fn characteristics(&mut self) -> Result<Characteristics, E>
    where
        CS: ChipSelectGuarded,
    {
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
            #[allow(clippy::excessive_precision)]
            rate_noise_density: 0.03
                * match bw {
                    // TODO: Use bw.sqrt_hz_at(odr)
                    Bandwidth::Narrowest => match odr {
                        OutputDataRate::Hz95 => 3.5355339059327378,  // √(12.5 Hz)
                        OutputDataRate::Hz190 => 3.5355339059327378, // √(12.5 Hz)
                        OutputDataRate::Hz380 => 4.47213595499958,   // √(20.0 Hz)
                        OutputDataRate::Hz760 => 5.477225575051661,  // √(30.0 Hz)
                    },
                    Bandwidth::Narrow => match odr {
                        OutputDataRate::Hz95 => 5.0,                // √(25.0 Hz)
                        OutputDataRate::Hz190 => 5.0,               // √(25.0 Hz)
                        OutputDataRate::Hz380 => 25.0,              // √(25.0 Hz)
                        OutputDataRate::Hz760 => 5.916079783099616, // √(35.0 Hz)
                    },
                    Bandwidth::Medium => match odr {
                        OutputDataRate::Hz95 => 5.0,                 // √(25.0 Hz)
                        OutputDataRate::Hz190 => 7.0710678118654755, // √(50.0 Hz)
                        OutputDataRate::Hz380 => 7.0710678118654755, // √(50.0 Hz)
                        OutputDataRate::Hz760 => 7.0710678118654755, // √(50.0 Hz)
                    },
                    Bandwidth::Wide => match odr {
                        OutputDataRate::Hz95 => 5.0,                // √(25.0 Hz)
                        OutputDataRate::Hz190 => 8.366600265340756, // √(70.0 Hz)
                        OutputDataRate::Hz380 => 10.0,              // √(100.0 Hz)
                        OutputDataRate::Hz760 => 10.0,              // √(100.0 Hz)
                    },
                },
        })
    }

    /// Identifies this chip by querying the `WHO_AM_I` register.
    pub fn temp_raw(&mut self) -> Result<u8, E>
    where
        CS: ChipSelectGuarded,
    {
        let ident = self.read_register::<TemperatureRegister>()?;
        Ok(ident.temp())
    }

    /// Fetches X, Y and Z-axis data off the sensor.
    pub fn xyz_raw(&mut self) -> Result<I16x3, E>
    where
        CS: ChipSelectGuarded,
    {
        let _guard = self.cs.select_guard();

        // The registers come in the order XL (0x28), XH, YL, YH, ZL, ZH (0x2D)
        let command = Self::read_multi_cmd(*OutXLow::REGISTER_ADDRESS);
        let mut buffer = [command, 0, 0, 0, 0, 0, 0];
        self.spi.transfer(&mut buffer)?;

        // skip the command byte [0].
        let xlo = OutXLow::from_bits(buffer[1]);
        let xhi = OutXHigh::from_bits(buffer[2]);
        let ylo = OutYLow::from_bits(buffer[3]);
        let yhi = OutYHigh::from_bits(buffer[4]);
        let zlo = OutZLow::from_bits(buffer[5]);
        let zhi = OutZHigh::from_bits(buffer[6]);

        let x = xhi + xlo;
        let y = yhi + ylo;
        let z = zhi + zlo;

        Ok(I16x3::new(x, y, z))
    }

    /// Fetches all data off the sensor.
    pub fn data_raw(&mut self) -> Result<SensorData, E>
    where
        CS: ChipSelectGuarded,
    {
        let _guard = self.cs.select_guard();

        // The registers come in the order Temperature (0x26), Status (0x27), XL, XH, YL, YH, ZL, ZH (0x2D)
        let command = Self::read_multi_cmd(*TemperatureRegister::REGISTER_ADDRESS);
        let mut buffer = [command, 0, 0, 0, 0, 0, 0, 0, 0];
        self.spi.transfer(&mut buffer)?;

        // skip the command byte at [0].
        let temp = TemperatureRegister::from_bits(buffer[1]);
        let status = StatusRegister::from_bits(buffer[2]);
        let xlo = OutXLow::from_bits(buffer[3]);
        let xhi = OutXHigh::from_bits(buffer[4]);
        let ylo = OutYLow::from_bits(buffer[5]);
        let yhi = OutYHigh::from_bits(buffer[6]);
        let zlo = OutZLow::from_bits(buffer[7]);
        let zhi = OutZHigh::from_bits(buffer[8]);

        let x = xhi + xlo;
        let y = yhi + ylo;
        let z = zhi + zlo;

        Ok(SensorData::new(temp.temp(), x, y, z, status))
    }

    /// Creates a read command for a given address. Does not auto-increment the address afterward.
    fn read_single_cmd(address: u8) -> u8 {
        Self::READ | Self::SINGLE | (address & Self::REG_ADDR_MASK)
    }

    /// Creates a read command for a given address. Does not auto-increment the address afterward.
    fn read_multi_cmd(address: u8) -> u8 {
        Self::READ | Self::MULTI | (address & Self::REG_ADDR_MASK)
    }

    /// Creates a write command for a given address. Does not auto-increment the address afterward.
    fn write_single_cmd(address: u8) -> u8 {
        Self::WRITE | Self::SINGLE | (address & Self::REG_ADDR_MASK)
    }

    /// Reads a single register. Assumes the chip is selected.
    pub fn read_register<R>(&mut self) -> Result<R, E>
    where
        R: Register,
        CS: ChipSelectGuarded,
    {
        let _guard = self.cs.select_guard();
        let command = Self::read_single_cmd(*R::REGISTER_ADDRESS);
        let mut buffer = [command, 0];
        self.spi.transfer(&mut buffer)?;
        Ok(R::from_bits(buffer[1]))
    }

    /// Writes a single register. Assumes the chip is selected.
    pub fn write_register<B, R>(&mut self, register: B) -> Result<(), E>
    where
        B: core::borrow::Borrow<R>,
        R: WritableRegister,
        CS: ChipSelectGuarded,
    {
        let _guard = self.cs.select_guard();
        let byte = register.borrow().to_bits();
        let command = Self::write_single_cmd(*R::REGISTER_ADDRESS);
        let mut buffer = [command, byte];
        self.spi.transfer(&mut buffer)?;
        Ok(())
    }

    /// Modifies a single register. Assumes the chip is selected.
    pub fn modify_register<F, R>(&mut self, f: F) -> Result<(), E>
    where
        F: FnOnce(R) -> R,
        R: WritableRegister,
        CS: ChipSelectGuarded,
    {
        let register: R = self.read_register()?;
        let register = f(register);
        self.write_register(register)
    }
}
