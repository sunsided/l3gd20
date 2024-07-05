//! Provides [`RefCell`] wrappers for SPI types.

use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use embedded_hal::blocking::spi::Transfer;

/// An SPI instance wrapped in a [`RefCell`].
pub struct RefCellSPI<SPI, E>(RefCell<SPI>)
where
    SPI: Transfer<u8, Error = E>;

impl<SPI, E> RefCellSPI<SPI, E>
where
    SPI: Transfer<u8, Error = E>,
{
    /// Initializes a new instance of the [`RefCellSPI`] type.
    pub const fn new(i2c: RefCell<SPI>) -> Self {
        Self(i2c)
    }

    /// Consumes self and returns the inner SPI instance.
    #[inline]
    pub fn into_inner(self) -> RefCell<SPI> {
        self.0
    }
}

impl<SPI, E> Transfer<u8> for RefCellSPI<SPI, E>
where
    SPI: Transfer<u8, Error = E>,
{
    type Error = E;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.0.borrow_mut().transfer(words)
    }
}

impl<SPI, E> Deref for RefCellSPI<SPI, E>
where
    SPI: Transfer<u8, Error = E>,
{
    type Target = RefCell<SPI>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<SPI, E> DerefMut for RefCellSPI<SPI, E>
where
    SPI: Transfer<u8, Error = E>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
