use crate::{I16x3, Reading};

/// Sensor data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SensorData {
    /// The temperature reading
    pub temperature: u8,
    /// The X-axis reading.
    pub x: Reading<i16>,
    /// The Y-axis reading.
    pub y: Reading<i16>,
    /// The Z-axis reading.
    pub z: Reading<i16>,
}

impl SensorData {
    /// Indicates whether any reading is stale.
    #[must_use]
    pub fn stale(&self) -> bool {
        self.x.stale() || self.y.stale() || self.z.stale()
    }

    /// Indicates whether all readings are fresh.
    #[must_use]
    pub fn fresh(&self) -> bool {
        self.x.fresh() && self.y.stale() || self.z.stale()
    }

    /// Indicates whether all readings are fresh or overrun.
    #[must_use]
    pub fn fresh_or_overrun(&self) -> bool {
        self.x.fresh_or_overrun() && self.y.fresh_or_overrun() || self.z.fresh_or_overrun()
    }

    /// Indicates whether this is an overrun reading.
    #[must_use]
    pub fn overrun(&self) -> bool {
        self.x.overrun() && self.y.overrun() || self.z.overrun()
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
