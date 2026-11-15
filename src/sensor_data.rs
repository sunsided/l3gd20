use crate::{I16x3, Reading};
use l3gd20_registers::StatusRegister;

/// Sensor data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SensorData {
    /// The temperature reading
    pub temperature: i8,
    /// The X-axis reading.
    pub x: Reading<i16>,
    /// The Y-axis reading.
    pub y: Reading<i16>,
    /// The Z-axis reading.
    pub z: Reading<i16>,
}

impl SensorData {
    /// Maps sensor readings into a [`SensorData`] structure.
    pub fn new<S>(temperature: i8, x: i16, y: i16, z: i16, status: S) -> Self
    where
        S: core::borrow::Borrow<StatusRegister>,
    {
        let status = status.borrow();
        Self {
            temperature,
            x: Reading::map(x, status.x_da(), status.x_overrun()),
            y: Reading::map(y, status.y_da(), status.y_overrun()),
            z: Reading::map(z, status.z_da(), status.z_overrun()),
        }
    }

    /// Indicates whether any reading is stale.
    #[must_use]
    pub fn stale(&self) -> bool {
        self.x.stale() || self.y.stale() || self.z.stale()
    }

    /// Indicates whether all readings are fresh.
    #[must_use]
    pub fn fresh(&self) -> bool {
        self.x.fresh() && self.y.fresh() && self.z.fresh()
    }

    /// Indicates whether all readings are fresh or overrun.
    #[must_use]
    pub fn fresh_or_overrun(&self) -> bool {
        self.x.fresh_or_overrun() && self.y.fresh_or_overrun() && self.z.fresh_or_overrun()
    }

    /// Indicates whether any reading is an overrun reading.
    #[must_use]
    pub fn overrun(&self) -> bool {
        self.x.overrun() || self.y.overrun() || self.z.overrun()
    }
}

impl From<SensorData> for I16x3 {
    fn from(value: SensorData) -> Self {
        Self {
            x: *value.x,
            y: *value.y,
            z: *value.z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freshness_covers_all_axes() {
        // Status: X and Y have new data, Z does not; Y overran.
        let status = StatusRegister::from(0b0010_0011);
        let data = SensorData::new(0, 1, 2, 3, status);
        assert!(data.x.fresh() && data.y.overrun() && data.z.stale());
        assert!(data.stale());
        assert!(!data.fresh());
        assert!(!data.fresh_or_overrun());
        assert!(data.overrun());

        let data = SensorData::new(0, 1, 2, 3, StatusRegister::from(0b0000_0111));
        assert!(data.fresh() && data.fresh_or_overrun());
        assert!(!data.stale() && !data.overrun());
    }
}
