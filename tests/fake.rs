//! Driver tests against a register-file model of the L3GD20 SPI protocol.

use core::cell::RefCell;
use fake_bus::{FakeSpi, SpiTarget};
use l3gd20_ng::{Error, L3GD20SPI};
use l3gd20_registers::*;

/// Models the register file and the SPI command byte: `RW | MS | AD5..AD0`.
struct Model {
    regs: [u8; 64],
    /// The current address and whether the transfer reads, once the command byte arrived.
    state: Option<(u8, bool, bool)>,
}

impl Model {
    fn new() -> RefCell<Self> {
        let mut regs = [0; 64];
        regs[0x0F] = L3GD20_WHO_AM_I;
        regs[0x20] = 0x07;
        RefCell::new(Self { regs, state: None })
    }
}

impl SpiTarget for Model {
    fn select(&mut self) {
        self.state = None;
    }

    fn transfer(&mut self, mosi: u8) -> u8 {
        let Some((addr, read, increment)) = self.state else {
            self.state = Some((
                mosi & 0x3F,
                mosi & SPI_READ != 0,
                mosi & SPI_AUTO_INCREMENT != 0,
            ));
            return 0;
        };
        let miso = if read {
            self.regs[addr as usize]
        } else {
            self.regs[addr as usize] = mosi;
            0
        };
        let next = if increment { (addr + 1) & 0x3F } else { addr };
        self.state = Some((next, read, increment));
        miso
    }
}

#[test]
fn new_applies_default_configuration() {
    let model = Model::new();
    model.borrow_mut().regs[0x24] = 0xFF;
    let _gyro = L3GD20SPI::new(FakeSpi::new(&model)).unwrap();

    let regs = model.borrow().regs;
    // Powered up, all axes, 95 Hz, narrowest bandwidth.
    assert_eq!(regs[0x20], 0b0000_1111);
    assert_eq!(regs[0x23], 0x00);
    // The boot toggle ends with boot cleared.
    assert_eq!(regs[0x24], 0x00);
}

#[test]
fn identify() {
    let model = Model::new();
    let mut gyro = L3GD20SPI::new(FakeSpi::new(&model)).unwrap();
    assert!(gyro.identify().unwrap());

    model.borrow_mut().regs[0x0F] = 0xD3;
    assert!(!gyro.identify().unwrap());
}

#[test]
fn modify_keeps_other_bits() {
    let model = Model::new();
    let mut gyro = L3GD20SPI::new(FakeSpi::new(&model)).unwrap();
    gyro.set_odr(OutputDataRate::Hz760).unwrap();
    assert_eq!(model.borrow().regs[0x20], 0b1100_1111);
    gyro.sleep_mode().unwrap();
    assert_eq!(model.borrow().regs[0x20], 0b1100_1000);
}

#[test]
fn reads_little_endian_axes() {
    let model = Model::new();
    let mut gyro = L3GD20SPI::new(FakeSpi::new(&model)).unwrap();
    model.borrow_mut().regs[0x26..=0x2D].copy_from_slice(&[
        0x19,
        0b0000_1111,
        0x34,
        0x12,
        0xFF,
        0xFF,
        0x00,
        0x80,
    ]);

    let xyz = gyro.xyz_raw().unwrap();
    assert_eq!((xyz.x, xyz.y, xyz.z), (0x1234, -1, i16::MIN));

    let data = gyro.data_raw().unwrap();
    assert_eq!(data.temperature, 0x19);
    assert!(data.fresh());
    assert_eq!((*data.x, *data.y, *data.z), (0x1234, -1, i16::MIN));
}

#[test]
fn reserved_fifo_mode_is_an_error() {
    let model = Model::new();
    let mut gyro = L3GD20SPI::new(FakeSpi::new(&model)).unwrap();

    model.borrow_mut().regs[0x2E] = 0b0100_0011;
    let fifo = gyro.read_register::<FifoControlRegister>().unwrap();
    assert_eq!(fifo.fifo_mode(), FifoMode::Stream);

    model.borrow_mut().regs[0x2E] = 0b1010_0000;
    assert_eq!(
        gyro.read_register::<FifoControlRegister>(),
        Err(Error::InvalidFifoMode(InvalidFifoMode(0b101)))
    );
}

#[test]
fn characteristics_follow_configuration() {
    let model = Model::new();
    let mut gyro = L3GD20SPI::new(FakeSpi::new(&model)).unwrap();
    gyro.set_odr(OutputDataRate::Hz380).unwrap();
    gyro.set_bandwidth(Bandwidth::Narrow).unwrap();
    gyro.set_sensitivity(Sensitivity::D500).unwrap();

    let c = gyro.characteristics().unwrap();
    assert_eq!(c.full_scale, 500);
    assert!((c.rate_noise_density - 0.03 * 5.0).abs() < 1e-6);
}
