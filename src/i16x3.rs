/// XYZ triple
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct I16x3 {
    /// X component
    pub x: i16,
    /// Y component
    pub y: i16,
    /// Z component
    pub z: i16,
}

impl I16x3 {
    /// Creates a new instance of the [`I16x3`] struct from its components.
    #[must_use]
    pub fn new(x: i16, y: i16, z: i16) -> Self {
        Self { x, y, z }
    }
}

#[cfg(feature = "defmt")]
#[cfg_attr(docsrs, doc(cfg(feature = "defmt")))]
impl defmt::Format for I16x3 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "({}, {}, {})", self.x, self.y, self.z);
    }
}

impl core::fmt::Debug for I16x3 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;
        f.write_char('(')?;
        core::fmt::Debug::fmt(&self.x, f)?;
        f.write_str(", ")?;
        core::fmt::Debug::fmt(&self.y, f)?;
        f.write_str(", ")?;
        core::fmt::Debug::fmt(&self.z, f)?;
        f.write_char(')')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i16x3_debug() {
        let value = I16x3 {
            x: 10,
            y: 20,
            z: 30,
        };
        test_format::assert_debug_fmt!(value, "(10, 20, 30)");
    }
}
