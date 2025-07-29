// ABOUTME: Temperature and climate system for environmental simulation layer
// ABOUTME: Implements elevation-based temperature gradients with scale-aware parameters

use crate::scale::{ScaleAware, WorldScale};

/// Core temperature data layer
#[derive(Clone, Debug)]
pub struct TemperatureLayer {
    /// Temperature in Celsius at each cell
    pub temperature: Vec<Vec<f32>>,
    /// Seasonal temperature variation range at each cell
    pub seasonal_variation: Vec<Vec<f32>>,
    /// Width and height for bounds checking
    width: usize,
    height: usize,
}

impl TemperatureLayer {
    /// Create a new temperature layer with the given dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            temperature: vec![vec![0.0; width]; height],
            seasonal_variation: vec![vec![0.0; width]; height],
            width,
            height,
        }
    }

    /// Get temperature at a specific location (with bounds checking)
    pub fn get_temperature(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.temperature[y][x]
        } else {
            0.0 // Default temperature if out of bounds
        }
    }

    /// Get seasonal variation at a specific location (with bounds checking)
    pub fn get_seasonal_variation(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.seasonal_variation[y][x]
        } else {
            0.0
        }
    }

    /// Get current temperature considering seasonal effects
    pub fn get_current_temperature(&self, x: usize, y: usize, season_factor: f32) -> f32 {
        if x < self.width && y < self.height {
            let base_temp = self.temperature[y][x];
            let variation = self.seasonal_variation[y][x];
            // Season factor: 0.0 = winter, 0.5 = spring/fall, 1.0 = summer
            let seasonal_offset = variation * (season_factor - 0.5) * 2.0;
            base_temp + seasonal_offset
        } else {
            0.0
        }
    }

    /// Get average temperature across the entire map
    pub fn get_average_temperature(&self) -> f32 {
        let total: f32 = self.temperature.iter().flat_map(|row| row.iter()).sum();

        let cell_count = (self.width * self.height) as f32;
        if cell_count > 0.0 {
            total / cell_count
        } else {
            0.0
        }
    }
}

/// Raw climate parameters before scale adjustment
#[derive(Clone, Debug)]
pub struct ClimateParameters {
    /// Base temperature at sea level in Celsius
    pub base_temperature_c: f32,
    /// Temperature decrease per meter of elevation (°C/m)
    pub elevation_lapse_rate: f32,
    /// Seasonal temperature variation amplitude (°C)
    pub seasonal_amplitude: f32,
    /// Temperature change per degree of latitude (°C/degree)
    pub latitude_gradient: f32,
    /// Minimum temperature threshold (°C)
    pub min_temperature: f32,
    /// Maximum temperature threshold (°C)
    pub max_temperature: f32,
}

impl Default for ClimateParameters {
    fn default() -> Self {
        Self {
            // Temperate climate defaults (similar to mid-latitude continental)
            base_temperature_c: 15.0,     // 15°C at sea level
            elevation_lapse_rate: 0.0065, // Standard atmospheric lapse rate (6.5°C/km)
            seasonal_amplitude: 20.0,     // ±20°C seasonal variation
            latitude_gradient: 0.8,       // About 0.8°C per degree latitude
            min_temperature: -50.0,       // Extreme cold limit
            max_temperature: 50.0,        // Extreme heat limit
        }
    }
}

impl ScaleAware for ClimateParameters {
    fn derive_parameters(&self, scale: &WorldScale) -> Self {
        let _meters_per_pixel = scale.meters_per_pixel() as f32;
        let physical_extent_km = scale.physical_size_km as f32;

        Self {
            // Base temperature is intensive - doesn't scale
            base_temperature_c: self.base_temperature_c,

            // Lapse rate is a physical constant - doesn't scale
            elevation_lapse_rate: self.elevation_lapse_rate,

            // Seasonal amplitude might vary with map size (larger areas = more continental)
            seasonal_amplitude: self.seasonal_amplitude * (1.0 + physical_extent_km / 1000.0 * 0.1),

            // Latitude gradient scales with map coverage
            // Larger maps span more latitudes = more temperature variation
            latitude_gradient: self.latitude_gradient * (physical_extent_km / 100.0).min(5.0),

            // Temperature limits remain physical constants
            min_temperature: self.min_temperature,
            max_temperature: self.max_temperature,
        }
    }
}

/// Climate system with effective parameters
#[derive(Clone, Debug)]
pub struct ClimateSystem {
    /// Scale-derived climate parameters
    pub parameters: ClimateParameters,
    /// Current seasonal position (0.0 = winter, 0.5 = spring/fall, 1.0 = summer)
    pub current_season: f32,
    /// Seasonal cycle rate (cycles per simulation tick)
    pub seasonal_rate: f32,
}

impl ClimateSystem {
    /// Create a new climate system for the given world scale
    pub fn new_for_scale(scale: &WorldScale) -> Self {
        let parameters = ClimateParameters::default().derive_parameters(scale);

        Self {
            parameters,
            current_season: 0.0,         // Start in winter
            seasonal_rate: 1.0 / 3650.0, // One year = ~3650 ticks (10 ticks per day)
        }
    }

    /// Create climate system from custom parameters
    pub fn from_parameters(parameters: ClimateParameters, scale: &WorldScale) -> Self {
        let scaled_params = parameters.derive_parameters(scale);

        Self {
            parameters: scaled_params,
            current_season: 0.0,
            seasonal_rate: 1.0 / 3650.0,
        }
    }

    /// Advance seasonal cycle
    pub fn tick(&mut self) {
        self.current_season += self.seasonal_rate;
        // Keep season in 0.0-1.0 range
        if self.current_season >= 1.0 {
            self.current_season -= 1.0;
        }
    }

    /// Generate temperature layer from heightmap
    pub fn generate_temperature_layer(&self, heightmap: &[Vec<f32>]) -> TemperatureLayer {
        let height = heightmap.len();
        let width = if height > 0 { heightmap[0].len() } else { 0 };

        let mut temp_layer = TemperatureLayer::new(width, height);

        // Calculate temperature for each cell
        for y in 0..height {
            for x in 0..width {
                let elevation = heightmap[y][x];
                let latitude_factor = (y as f32) / (height as f32); // 0.0 = north, 1.0 = south

                // Base temperature calculation
                let mut temperature = self.parameters.base_temperature_c;

                // Apply elevation-based cooling (higher = colder)
                temperature -= elevation.max(0.0) * self.parameters.elevation_lapse_rate * 1000.0;

                // Apply latitude-based variation (assume symmetric about equator)
                let latitude_from_equator = (latitude_factor - 0.5).abs() * 2.0; // 0.0 = equator, 1.0 = pole
                temperature -= latitude_from_equator * self.parameters.latitude_gradient * 90.0; // 90 degrees from equator to pole

                // Clamp to reasonable limits
                temperature = temperature
                    .max(self.parameters.min_temperature)
                    .min(self.parameters.max_temperature);

                temp_layer.temperature[y][x] = temperature;

                // Seasonal variation (higher latitudes have more variation)
                temp_layer.seasonal_variation[y][x] =
                    self.parameters.seasonal_amplitude * (0.5 + latitude_from_equator * 0.5); // More variation at poles
            }
        }

        temp_layer
    }

    /// Get season name for display purposes
    pub fn get_season_name(&self) -> &'static str {
        match self.current_season {
            s if s < 0.25 => "Early Spring",
            s if s < 0.5 => "Late Spring",
            s if s < 0.75 => "Summer",
            _ => "Winter",
        }
    }

    /// Get temperature multiplier for water evaporation
    /// Uses Arrhenius-like temperature dependence
    pub fn get_evaporation_multiplier(&self, temperature_c: f32) -> f32 {
        // Reference temperature: 20°C
        let reference_temp = 20.0;
        let temp_kelvin = temperature_c + 273.15;
        let reference_kelvin = reference_temp + 273.15;

        // Simple exponential relationship: evaporation doubles every 10°C
        let temp_factor = (temp_kelvin - reference_kelvin) / reference_kelvin;
        let multiplier = (temp_factor * 0.1 * 2.0_f32.ln()).exp();

        // Clamp to reasonable bounds (0.1x to 10x normal rate)
        multiplier.max(0.1).min(10.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::{DetailLevel, WorldScale};

    #[test]
    fn temperature_layer_basic_operations() {
        let temp_layer = TemperatureLayer::new(10, 10);

        // Should handle bounds correctly
        assert_eq!(temp_layer.get_temperature(5, 5), 0.0);
        assert_eq!(temp_layer.get_temperature(15, 15), 0.0); // Out of bounds

        // Should calculate seasonal temperature
        assert_eq!(temp_layer.get_current_temperature(5, 5, 0.5), 0.0); // Spring/fall = base temp
    }

    #[test]
    fn climate_parameters_scaling() {
        let base_params = ClimateParameters::default();
        let small_scale = WorldScale::new(1.0, (50, 50), DetailLevel::Standard);
        let large_scale = WorldScale::new(1000.0, (500, 500), DetailLevel::Standard);

        let small_scaled = base_params.derive_parameters(&small_scale);
        let large_scaled = base_params.derive_parameters(&large_scale);

        // Base temperature should remain constant
        assert_eq!(
            small_scaled.base_temperature_c,
            large_scaled.base_temperature_c
        );

        // Larger maps should have more continental effects
        assert!(large_scaled.seasonal_amplitude > small_scaled.seasonal_amplitude);
        assert!(large_scaled.latitude_gradient > small_scaled.latitude_gradient);
    }

    #[test]
    fn temperature_generation_from_heightmap() {
        let heightmap = vec![
            vec![0.0, 0.5, 1.0], // Sea level, mid elevation, high elevation
            vec![0.0, 0.5, 1.0],
            vec![0.0, 0.5, 1.0],
        ];

        let scale = WorldScale::new(10.0, (3, 3), DetailLevel::Standard);
        let climate = ClimateSystem::new_for_scale(&scale);
        let temp_layer = climate.generate_temperature_layer(&heightmap);

        // Higher elevations should be cooler
        let sea_level_temp = temp_layer.get_temperature(0, 0);
        let high_elevation_temp = temp_layer.get_temperature(2, 0);
        assert!(high_elevation_temp < sea_level_temp);

        // Higher latitudes (toward poles) should be cooler
        let north_temp = temp_layer.get_temperature(0, 0); // Top row
        let south_temp = temp_layer.get_temperature(0, 2); // Bottom row
        assert!(north_temp < temp_layer.get_temperature(0, 1)); // Middle should be warmest
        assert!(south_temp < temp_layer.get_temperature(0, 1));
    }

    #[test]
    fn seasonal_cycling() {
        let scale = WorldScale::new(10.0, (10, 10), DetailLevel::Standard);
        let mut climate = ClimateSystem::new_for_scale(&scale);

        assert_eq!(climate.current_season, 0.0); // Start in winter

        // Advance through seasons
        for _ in 0..1000 {
            climate.tick();
        }

        assert!(climate.current_season > 0.0);
        assert!(climate.current_season < 1.0);
    }

    #[test]
    fn evaporation_temperature_dependence() {
        let scale = WorldScale::new(10.0, (10, 10), DetailLevel::Standard);
        let climate = ClimateSystem::new_for_scale(&scale);

        let cold_multiplier = climate.get_evaporation_multiplier(0.0); // 0°C
        let warm_multiplier = climate.get_evaporation_multiplier(20.0); // 20°C (reference)
        let hot_multiplier = climate.get_evaporation_multiplier(40.0); // 40°C

        // Should follow exponential relationship
        assert!(cold_multiplier < warm_multiplier);
        assert!(warm_multiplier < hot_multiplier);
        assert!((warm_multiplier - 1.0).abs() < 0.1); // Should be ~1.0 at reference temperature

        // Should be bounded
        assert!(cold_multiplier >= 0.1);
        assert!(hot_multiplier <= 10.0);
    }

    #[test]
    fn seasonal_temperature_variation() {
        let heightmap = vec![vec![0.0; 10]; 10]; // Flat terrain
        let scale = WorldScale::new(10.0, (10, 10), DetailLevel::Standard);
        let climate = ClimateSystem::new_for_scale(&scale);
        let temp_layer = climate.generate_temperature_layer(&heightmap);

        // Test seasonal variation
        let winter_temp = temp_layer.get_current_temperature(5, 5, 0.0); // Winter
        let summer_temp = temp_layer.get_current_temperature(5, 5, 1.0); // Summer
        let spring_temp = temp_layer.get_current_temperature(5, 5, 0.5); // Spring

        // Summer should be warmer than winter
        assert!(summer_temp > winter_temp);

        // Spring should be between winter and summer
        assert!(spring_temp > winter_temp);
        assert!(spring_temp < summer_temp);

        // Base temperature should be roughly spring temperature
        let base_temp = temp_layer.get_temperature(5, 5);
        assert!((spring_temp - base_temp).abs() < 1.0); // Should be close to base temperature
    }
}
