use core::convert::Infallible;
use l3gd20_registers::InvalidFifoMode;

/// A driver error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// The SPI transfer failed.
    Spi(E),
    /// The `FIFO_CTRL_REG` register holds a reserved FIFO mode.
    InvalidFifoMode(InvalidFifoMode),
}

impl<E> From<InvalidFifoMode> for Error<E> {
    fn from(value: InvalidFifoMode) -> Self {
        Self::InvalidFifoMode(value)
    }
}

/// Lets registers that decode every bit pattern use the `?` operator.
impl<E> From<Infallible> for Error<E> {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}
