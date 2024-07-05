/// Scale and noise characteristics of the sensor.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Characteristics {
    /// Full Scale selection in degrees/second.
    pub full_scale: u16,

    /// The sensitivity at the current selected full scale.
    pub sensitivity: f32,

    /// The zero-rate noise level in ±degrees/second.
    pub zero_rate_noise: f32,

    /// The temperature-related zero-rate noise level change, in degrees/second.
    /// * At FS=250 dps, ±0.03 dps/°C
    /// * At FS=2000 dps, ±0.04 dps/°C
    pub zero_rate_level_temp: f32,

    /// The frequency-dependent rate noise level in degrees/second.
    /// The rate noise density is at 0.03 dps/√Hz.
    pub rate_noise_density: f32,
}

impl Default for Characteristics {
    fn default() -> Self {
        #[allow(clippy::excessive_precision)]
        Self {
            full_scale: 250,
            sensitivity: 8.75 * 0.001,
            zero_rate_noise: 10.0,
            zero_rate_level_temp: 0.03,
            rate_noise_density: 0.03 * 3.5355339059327378,
        }
    }
}
