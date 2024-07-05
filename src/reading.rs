use core::ops::{Deref, DerefMut};

/// A sensor reading that captures the notion of recent and outdated information.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Reading<T> {
    /// The reading is stale.
    Stale(T),
    /// The reading is recent.
    Fresh(T),
    /// New data was written before old data was read.
    Overrun(T),
}

impl<T> Deref for Reading<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Reading::Stale(x) => x,
            Reading::Fresh(x) => x,
            Reading::Overrun(x) => x,
        }
    }
}

impl<T> DerefMut for Reading<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Reading::Stale(x) => x,
            Reading::Fresh(x) => x,
            Reading::Overrun(x) => x,
        }
    }
}

impl<T> Reading<T> {
    /// Creates a stale reading.
    #[must_use]
    pub fn new_stale(value: T) -> Self {
        Self::Stale(value)
    }

    /// Creates a fresh reading.
    #[must_use]
    pub fn new_fresh(value: T) -> Self {
        Self::Fresh(value)
    }

    /// Creates a fresh reading.
    #[must_use]
    pub fn new_overrun(value: T) -> Self {
        Self::Overrun(value)
    }

    /// Consumes self and returns the inner value.
    #[must_use]
    pub fn into_inner(self) -> T {
        match self {
            Reading::Stale(x) => x,
            Reading::Fresh(x) => x,
            Reading::Overrun(x) => x,
        }
    }

    /// Indicates whether this is a stale reading.
    #[must_use]
    pub fn stale(&self) -> bool {
        matches!(self, Reading::Stale(_))
    }

    /// Indicates whether this is a fresh reading.
    #[must_use]
    pub fn fresh(&self) -> bool {
        matches!(self, Reading::Fresh(_))
    }

    /// Indicates whether this is a fresh or an overrun reading.
    #[must_use]
    pub fn fresh_or_overrun(&self) -> bool {
        self.fresh() || self.overrun()
    }

    /// Indicates whether this is an overrun reading.
    #[must_use]
    pub fn overrun(&self) -> bool {
        matches!(self, Reading::Overrun(_))
    }
}
