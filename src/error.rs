use core::convert::Infallible;
use l3gd20_registers::InvalidFifoMode;

/// A driver error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error<E> {
    /// The SPI transfer failed.
    Spi(E),
    /// The `FIFO_CTRL_REG` register holds a reserved FIFO mode.
    InvalidFifoMode(InvalidFifoMode),
}

impl<E: core::fmt::Debug> core::fmt::Display for Error<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Spi(e) => write!(f, "SPI transfer failed: {e:?}"),
            Self::InvalidFifoMode(e) => e.fmt(f),
        }
    }
}

impl<E: core::fmt::Debug> core::error::Error for Error<E> {}

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
