// ABOUTME: Temperature and climate system for environmental simulation layer
// ABOUTME: Implements elevation-based temperature gradients with scale-aware parameters

use super::super::core::scale::{ScaleAware, WorldScale};
use super::water::Vec2;

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

/// Atmospheric pressure data layer
/// Pressure drives wind patterns through horizontal pressure gradients
#[derive(Clone, Debug)]
pub struct AtmosphericPressureLayer {
    /// Pressure in Pascals at each cell (sea level equivalent)
    pub pressure: Vec<Vec<f32>>,
    /// Pressure gradient vector (∇P) in Pa/m at each cell
    pub pressure_gradient: Vec<Vec<Vec2>>,
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

    /// Get width of temperature layer
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get height of temperature layer
    pub fn height(&self) -> usize {
        self.height
    }
}

impl AtmosphericPressureLayer {
    /// Create a new atmospheric pressure layer with the given dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pressure: vec![vec![101325.0; width]; height], // Standard sea level pressure (Pa)
            pressure_gradient: vec![vec![Vec2::zero(); width]; height],
            width,
            height,
        }
    }

    /// Get pressure at a specific location (with bounds checking)
    pub fn get_pressure(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.pressure[y][x]
        } else {
            101325.0 // Standard sea level pressure if out of bounds
        }
    }

    /// Get pressure gradient at a specific location (with bounds checking)
    pub fn get_pressure_gradient(&self, x: usize, y: usize) -> Vec2 {
        if x < self.width && y < self.height {
            self.pressure_gradient[y][x].clone()
        } else {
            Vec2::zero()
        }
    }

    /// Calculate pressure gradients using finite differences
    /// ∇P = (∂P/∂x, ∂P/∂y) computed using central differences where possible
    pub fn calculate_pressure_gradients(&mut self, meters_per_pixel: f32) {
        for y in 0..self.height {
            for x in 0..self.width {
                let mut gradient = Vec2::zero();

                // Calculate ∂P/∂x using central differences (or forward/backward at boundaries)
                if x > 0 && x < self.width - 1 {
                    // Central difference: (P[x+1] - P[x-1]) / (2 * dx)
                    let dp_dx = (self.pressure[y][x + 1] - self.pressure[y][x - 1])
                        / (2.0 * meters_per_pixel);
                    gradient.x = dp_dx;
                } else if x == 0 && self.width > 1 {
                    // Forward difference: (P[x+1] - P[x]) / dx
                    let dp_dx = (self.pressure[y][x + 1] - self.pressure[y][x]) / meters_per_pixel;
                    gradient.x = dp_dx;
                } else if x == self.width - 1 && self.width > 1 {
                    // Backward difference: (P[x] - P[x-1]) / dx
                    let dp_dx = (self.pressure[y][x] - self.pressure[y][x - 1]) / meters_per_pixel;
                    gradient.x = dp_dx;
                }

                // Calculate ∂P/∂y using central differences (or forward/backward at boundaries)
                if y > 0 && y < self.height - 1 {
                    // Central difference: (P[y+1] - P[y-1]) / (2 * dy)
                    let dp_dy = (self.pressure[y + 1][x] - self.pressure[y - 1][x])
                        / (2.0 * meters_per_pixel);
                    gradient.y = dp_dy;
                } else if y == 0 && self.height > 1 {
                    // Forward difference: (P[y+1] - P[y]) / dy
                    let dp_dy = (self.pressure[y + 1][x] - self.pressure[y][x]) / meters_per_pixel;
                    gradient.y = dp_dy;
                } else if y == self.height - 1 && self.height > 1 {
                    // Backward difference: (P[y] - P[y-1]) / dy
                    let dp_dy = (self.pressure[y][x] - self.pressure[y - 1][x]) / meters_per_pixel;
                    gradient.y = dp_dy;
                }

                self.pressure_gradient[y][x] = gradient;
            }
        }
    }

    /// Get average pressure across the entire map
    pub fn get_average_pressure(&self) -> f32 {
        let total: f32 = self.pressure.iter().flat_map(|row| row.iter()).sum();

        let cell_count = (self.width * self.height) as f32;
        if cell_count > 0.0 {
            total / cell_count
        } else {
            101325.0
        }
    }

    /// Get maximum pressure gradient magnitude for stability analysis
    pub fn get_max_pressure_gradient_magnitude(&self) -> f32 {
        self.pressure_gradient
            .iter()
            .flat_map(|row| row.iter())
            .map(|grad| grad.magnitude())
            .fold(0.0, f32::max)
    }

    /// Get width of pressure layer
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get height of pressure layer
    pub fn height(&self) -> usize {
        self.height
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

    // Atmospheric pressure parameters
    /// Base pressure at sea level in Pascals
    pub base_pressure_pa: f32,
    /// Pressure variation amplitude due to temperature differences (Pa)
    pub pressure_temperature_coupling: f32,
    /// Pressure variation due to seasonal effects (Pa)
    pub seasonal_pressure_amplitude: f32,
    /// Random pressure perturbation strength for weather systems (Pa)
    pub pressure_noise_amplitude: f32,
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

            // Atmospheric pressure defaults
            base_pressure_pa: 101325.0, // Standard sea level pressure (1013.25 hPa)
            pressure_temperature_coupling: 500.0, // ~5 hPa pressure change per 10°C temperature difference
            seasonal_pressure_amplitude: 300.0,   // ~3 hPa seasonal pressure variation
            pressure_noise_amplitude: 200.0,      // ~2 hPa random weather perturbations
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

            // Continental-scale temperature gradient (north-south)
            // For continental domains, use realistic gradients: ~0.1°C/km = 10°C per 100km
            // Scale appropriately for domain size, avoiding global-scale assumptions
            latitude_gradient: {
                let continental_gradient_per_km = 0.1; // °C per kilometer (realistic continental gradient)
                let domain_half_extent_km = physical_extent_km / 2.0; // Half domain = center to edge distance
                let total_temperature_variation =
                    continental_gradient_per_km * domain_half_extent_km;
                // Clamp to reasonable bounds for continental domains (5-25°C variation)
                total_temperature_variation.max(5.0).min(25.0)
            },

            // Temperature limits remain physical constants
            min_temperature: self.min_temperature,
            max_temperature: self.max_temperature,

            // Pressure parameters
            // Base pressure is intensive - doesn't scale
            base_pressure_pa: self.base_pressure_pa,

            // Temperature-pressure coupling scales with temperature gradients
            pressure_temperature_coupling: self.pressure_temperature_coupling
                * (physical_extent_km / 100.0).min(3.0),

            // Seasonal pressure variation scales with continental effects
            seasonal_pressure_amplitude: self.seasonal_pressure_amplitude
                * (1.0 + physical_extent_km / 1000.0 * 0.2),

            // Weather noise scales with map size to maintain realistic pressure gradients
            // Scale minimum threshold with domain size for appropriate weather patterns
            pressure_noise_amplitude: {
                let base_scaling = (physical_extent_km / 100.0).min(4.0); // Increased max scaling
                // Scale minimum from 200Pa (50km) to 1000Pa (200km+) for appropriate weather visualization
                let weather_minimum =
                    (200.0 + (physical_extent_km - 50.0).max(0.0) * 4.0).min(1000.0);
                let calculated_noise = self.pressure_noise_amplitude * base_scaling;
                calculated_noise.max(weather_minimum) // Ensure minimum weather-scale variations
            },
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
    /// Random seed for pressure perturbations (for reproducible weather)
    pub pressure_seed: u64,
}

impl ClimateSystem {
    /// Create a new climate system for the given world scale
    pub fn new_for_scale(scale: &WorldScale) -> Self {
        let parameters = ClimateParameters::default().derive_parameters(scale);

        Self {
            parameters,
            current_season: 0.5, // Start in late spring/early summer for reasonable temperatures
            seasonal_rate: 1.0 / 3650.0, // One year = ~3650 ticks (10 ticks per day)
            pressure_seed: 12345, // Default seed for reproducible weather
        }
    }

    /// Create climate system from custom parameters
    pub fn from_parameters(parameters: ClimateParameters, scale: &WorldScale) -> Self {
        let scaled_params = parameters.derive_parameters(scale);

        Self {
            parameters: scaled_params,
            current_season: 0.5, // Start in late spring/early summer for reasonable temperatures
            seasonal_rate: 1.0 / 3650.0,
            pressure_seed: 12345,
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

    /// Generate temperature layer from heightmap with scale-aware continental climate
    /// This version uses the climate system's pre-scaled parameters
    pub fn generate_temperature_layer(&self, heightmap: &[Vec<f32>]) -> TemperatureLayer {
        let height = heightmap.len();
        let width = if height > 0 { heightmap[0].len() } else { 0 };

        let mut temp_layer = TemperatureLayer::new(width, height);

        // Calculate temperature for each cell with continental-scale gradients
        for y in 0..height {
            for x in 0..width {
                let elevation = heightmap[y][x];

                // Base temperature calculation
                let mut temperature = self.parameters.base_temperature_c;

                // Apply elevation-based cooling (higher = colder)
                temperature -= elevation.max(0.0) * self.parameters.elevation_lapse_rate * 1000.0;

                // Apply continental-scale north-south temperature gradient
                // Use normalized position within domain (0.0 = north edge, 1.0 = south edge)
                let north_south_position = (y as f32) / (height as f32).max(1.0);
                // Apply symmetric gradient around center (maximum cooling at edges)
                let distance_from_center = (north_south_position - 0.5).abs() * 2.0; // 0.0 = center, 1.0 = edge
                // Scale by domain-appropriate gradient (already scaled by ScaleAware)
                temperature -= distance_from_center * self.parameters.latitude_gradient;

                // Clamp to reasonable limits
                temperature = temperature
                    .max(self.parameters.min_temperature)
                    .min(self.parameters.max_temperature);

                temp_layer.temperature[y][x] = temperature;

                // Seasonal variation scales with distance from center (continental effect)
                temp_layer.seasonal_variation[y][x] =
                    self.parameters.seasonal_amplitude * (0.7 + distance_from_center * 0.3);
            }
        }

        // Apply spatial smoothing to eliminate banding artifacts
        self.apply_spatial_smoothing(&mut temp_layer);

        temp_layer
    }

    /// Optimized temperature layer generation using HeightMap directly
    /// Eliminates expensive Vec<Vec<f32>> conversion for better performance
    pub fn generate_temperature_layer_optimized(
        &self,
        heightmap: &super::super::core::heightmap::HeightMap,
    ) -> TemperatureLayer {
        let width = heightmap.width();
        let height = heightmap.height();

        let mut temp_layer = TemperatureLayer::new(width, height);

        // Optimized calculation using HeightMap's flat memory layout for better cache performance
        for y in 0..height {
            for x in 0..width {
                let elevation = heightmap.get(x, y);

                // Base temperature calculation
                let mut temperature = self.parameters.base_temperature_c;

                // Apply elevation-based cooling (higher = colder)
                temperature -= elevation.max(0.0) * self.parameters.elevation_lapse_rate * 1000.0;

                // Apply continental-scale north-south temperature gradient
                let north_south_position = (y as f32) / (height as f32).max(1.0);
                let distance_from_center = (north_south_position - 0.5).abs() * 2.0;
                temperature -= distance_from_center * self.parameters.latitude_gradient;

                // Clamp to reasonable limits
                temperature = temperature
                    .max(self.parameters.min_temperature)
                    .min(self.parameters.max_temperature);

                temp_layer.temperature[y][x] = temperature;

                // Seasonal variation scales with distance from center
                temp_layer.seasonal_variation[y][x] =
                    self.parameters.seasonal_amplitude * (0.7 + distance_from_center * 0.3);
            }
        }

        // Apply spatial smoothing to eliminate banding artifacts
        self.apply_spatial_smoothing(&mut temp_layer);

        temp_layer
    }

    /// Generate temperature layer with explicit scale context for debugging/analysis
    /// Useful for understanding how scale affects temperature patterns
    pub fn generate_temperature_layer_with_scale(
        &self,
        heightmap: &[Vec<f32>],
        scale: &WorldScale,
    ) -> TemperatureLayer {
        // Log scale-dependent parameters for debugging
        let domain_size = scale.physical_size_km;
        let expected_variation = (domain_size / 2.0) * 0.1; // 0.1°C/km * half-domain

        eprintln!("Generating temperature for {:.1}km domain:", domain_size);
        eprintln!(
            "  Expected N-S temperature variation: {:.1}°C",
            expected_variation
        );
        eprintln!(
            "  Actual latitude_gradient parameter: {:.1}°C",
            self.parameters.latitude_gradient
        );
        eprintln!(
            "  Resolution: {}x{} ({:.0}m/pixel)",
            scale.resolution.0,
            scale.resolution.1,
            scale.meters_per_pixel()
        );

        // Use the standard generation method
        self.generate_temperature_layer(heightmap)
    }

    /// Apply spatial smoothing to eliminate temperature banding artifacts
    /// Uses a simple 3x3 gaussian-like kernel for natural thermal diffusion
    fn apply_spatial_smoothing(&self, temp_layer: &mut TemperatureLayer) {
        let height = temp_layer.height();
        let width = temp_layer.width();

        if height < 3 || width < 3 {
            return; // Skip smoothing for very small maps
        }

        // Create a copy of original temperatures
        let original_temps: Vec<Vec<f32>> = temp_layer.temperature.clone();

        // Apply smoothing with thermal diffusion kernel
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                // 3x3 gaussian-like kernel for natural heat distribution
                // Center weight higher to preserve original values while smoothing
                let center_weight = 0.4;
                let adjacent_weight = 0.15; // orthogonal neighbors
                let diagonal_weight = 0.1; // diagonal neighbors

                let smoothed_temp = original_temps[y][x] * center_weight +
                    original_temps[y-1][x] * adjacent_weight +     // north
                    original_temps[y+1][x] * adjacent_weight +     // south
                    original_temps[y][x-1] * adjacent_weight +     // west
                    original_temps[y][x+1] * adjacent_weight +     // east
                    original_temps[y-1][x-1] * diagonal_weight +   // northwest
                    original_temps[y-1][x+1] * diagonal_weight +   // northeast
                    original_temps[y+1][x-1] * diagonal_weight +   // southwest
                    original_temps[y+1][x+1] * diagonal_weight; // southeast

                temp_layer.temperature[y][x] = smoothed_temp;
            }
        }

        // Also smooth seasonal variation to maintain consistency
        let original_seasonal: Vec<Vec<f32>> = temp_layer.seasonal_variation.clone();

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let center_weight = 0.6; // Higher weight for seasonal variation to preserve patterns
                let adjacent_weight = 0.1;

                let smoothed_seasonal = original_seasonal[y][x] * center_weight
                    + original_seasonal[y - 1][x] * adjacent_weight
                    + original_seasonal[y + 1][x] * adjacent_weight
                    + original_seasonal[y][x - 1] * adjacent_weight
                    + original_seasonal[y][x + 1] * adjacent_weight;

                temp_layer.seasonal_variation[y][x] = smoothed_seasonal;
            }
        }
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

    /// Generate atmospheric pressure layer from temperature field
    /// Pressure is coupled to temperature through the ideal gas law and hydrostatic balance
    pub fn generate_pressure_layer(
        &self,
        temperature_layer: &TemperatureLayer,
        heightmap: &[Vec<f32>],
        scale: &WorldScale,
    ) -> AtmosphericPressureLayer {
        let height = heightmap.len();
        let width = if height > 0 { heightmap[0].len() } else { 0 };

        let mut pressure_layer = AtmosphericPressureLayer::new(width, height);

        // Simple PRNG for reproducible weather patterns
        let mut rng_state = self.pressure_seed.wrapping_add(self.tick_count() as u64);

        // Calculate pressure for each cell
        for y in 0..height {
            for x in 0..width {
                let elevation = heightmap[y][x];
                let temperature_c =
                    temperature_layer.get_current_temperature(x, y, self.current_season);
                let _temperature_k = temperature_c + 273.15;

                // Base pressure calculation using barometric formula
                // P = P₀ × exp(-mgh/kT) where m = molar mass of air, g = gravity, h = height
                let mut pressure = self.parameters.base_pressure_pa;

                // Apply elevation-based pressure reduction (hydrostatic balance)
                // Using simplified barometric formula: P = P₀ × exp(-h/H) where H ≈ 8400m (scale height)
                let scale_height = 8400.0; // meters
                let elevation_meters = elevation.max(0.0) * 1000.0; // Convert to meters (assuming elevation is in km)
                pressure *= (-elevation_meters / scale_height).exp();

                // Apply temperature-pressure coupling (warmer air = lower pressure)
                // This creates thermal low/high pressure systems
                let temp_deviation = temperature_c - self.parameters.base_temperature_c;
                let thermal_pressure_change =
                    -temp_deviation * self.parameters.pressure_temperature_coupling / 10.0;
                pressure += thermal_pressure_change;

                // Apply seasonal pressure variation
                let seasonal_factor = (self.current_season * 2.0 * std::f32::consts::PI).sin();
                pressure += seasonal_factor * self.parameters.seasonal_pressure_amplitude;

                // Add weather noise for realistic pressure perturbations
                // Simple LCG for reproducible pseudo-random numbers
                rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
                let noise_factor = ((rng_state as f32) / (u64::MAX as f32)) * 2.0 - 1.0; // -1 to 1
                pressure += noise_factor * self.parameters.pressure_noise_amplitude;

                // Clamp to reasonable atmospheric pressure bounds (500-1100 hPa)
                pressure = pressure.max(50000.0).min(110000.0);

                pressure_layer.pressure[y][x] = pressure;
            }
        }

        // Calculate pressure gradients
        pressure_layer.calculate_pressure_gradients(scale.meters_per_pixel() as f32);

        pressure_layer
    }

    /// Optimized pressure layer generation using HeightMap directly
    /// Eliminates expensive Vec<Vec<f32>> conversion for better performance
    pub fn generate_pressure_layer_optimized(
        &self,
        temperature_layer: &TemperatureLayer,
        heightmap: &super::super::core::heightmap::HeightMap,
        scale: &WorldScale,
    ) -> AtmosphericPressureLayer {
        let width = heightmap.width();
        let height = heightmap.height();

        let mut pressure_layer = AtmosphericPressureLayer::new(width, height);

        // Optimized calculation using HeightMap's flat memory layout for better cache performance
        for y in 0..height {
            for x in 0..width {
                let elevation = heightmap.get(x, y);
                let temperature_c =
                    temperature_layer.get_current_temperature(x, y, self.current_season);
                let _temperature_k = temperature_c + 273.15;

                // Base pressure calculation using barometric formula
                let mut pressure = self.parameters.base_pressure_pa;

                // Apply elevation-based pressure reduction (hydrostatic balance)
                let scale_height = 8400.0; // meters
                let elevation_meters = elevation.max(0.0) * 1000.0; // Convert to meters
                pressure *= (-elevation_meters / scale_height).exp();

                // Apply thermal circulation physics (warm areas = low pressure, cool areas = high pressure)
                let temp_deviation = temperature_c - self.parameters.base_temperature_c;
                let thermal_pressure_change =
                    -temp_deviation * self.parameters.pressure_temperature_coupling / 10.0;
                pressure += thermal_pressure_change;

                // Clamp to reasonable atmospheric pressure range
                pressure = pressure.max(50000.0).min(110000.0);

                pressure_layer.pressure[y][x] = pressure;
            }
        }

        // Calculate pressure gradients
        pressure_layer.calculate_pressure_gradients(scale.meters_per_pixel() as f32);

        pressure_layer
    }

    /// SIMD-optimized temperature layer generation for continental-scale performance
    /// Uses vectorized operations to process multiple cells in parallel
    #[cfg(feature = "simd")]
    pub fn generate_temperature_layer_simd(
        &self,
        heightmap: &super::super::core::heightmap::HeightMap,
    ) -> TemperatureLayer {
        use rayon::prelude::*;

        let width = heightmap.width();
        let height = heightmap.height();

        // Create parallel row vectors for better cache performance
        let temperature_rows: Vec<Vec<f32>> = (0..height)
            .into_par_iter()
            .map(|y| {
                let row_start = y * width;
                let elevation_row = &heightmap.data()[row_start..row_start + width];

                // Pre-calculate common values for this row to avoid redundant computation
                let north_south_position = (y as f32) / (height as f32).max(1.0);
                let distance_from_center = (north_south_position - 0.5).abs() * 2.0;
                let latitude_temperature_offset =
                    distance_from_center * self.parameters.latitude_gradient;

                // Process entire row with vectorizable operations
                let mut row_temps = Vec::with_capacity(width);

                // Process cells in chunks for better compiler vectorization
                for elevation_chunk in elevation_row.chunks(8) {
                    for &elevation in elevation_chunk {
                        // Vectorizable calculations - compiler can optimize these
                        let mut temperature = self.parameters.base_temperature_c;
                        temperature -=
                            elevation.max(0.0) * self.parameters.elevation_lapse_rate * 1000.0;
                        temperature -= latitude_temperature_offset;

                        // Clamp to reasonable limits
                        temperature = temperature
                            .max(self.parameters.min_temperature)
                            .min(self.parameters.max_temperature);

                        row_temps.push(temperature);
                    }
                }

                row_temps
            })
            .collect();

        // Create seasonal variation in parallel
        let seasonal_rows: Vec<Vec<f32>> = (0..height)
            .into_par_iter()
            .map(|y| {
                let north_south_position = (y as f32) / (height as f32).max(1.0);
                let distance_from_center = (north_south_position - 0.5).abs() * 2.0;
                let seasonal_variation =
                    self.parameters.seasonal_amplitude * (0.7 + distance_from_center * 0.3);

                vec![seasonal_variation; width]
            })
            .collect();

        // Assemble the temperature layer
        let mut temp_layer = TemperatureLayer::new(width, height);
        temp_layer.temperature = temperature_rows;
        temp_layer.seasonal_variation = seasonal_rows;

        // Apply spatial smoothing to eliminate banding artifacts
        self.apply_spatial_smoothing(&mut temp_layer);

        temp_layer
    }

    /// SIMD-optimized pressure layer generation for better performance
    #[cfg(feature = "simd")]
    pub fn generate_pressure_layer_simd(
        &self,
        temperature_layer: &TemperatureLayer,
        heightmap: &super::super::core::heightmap::HeightMap,
        scale: &WorldScale,
    ) -> AtmosphericPressureLayer {
        use rayon::prelude::*;

        let width = heightmap.width();
        let height = heightmap.height();

        // Pre-calculate constants outside the parallel loop
        let base_pressure = self.parameters.base_pressure_pa;
        let base_temp_c = self.parameters.base_temperature_c;
        let noise_amplitude = self.parameters.pressure_noise_amplitude;
        let scale_height_inv = 1.0 / 8400.0; // Pre-calculate reciprocal for faster division
        let rng_base = self.pressure_seed.wrapping_add(self.tick_count() as u64);

        // Process rows in parallel and collect results
        let pressure_rows: Vec<Vec<f32>> = (0..height)
            .into_par_iter()
            .map(|y| {
                let mut rng_state = rng_base.wrapping_add(y as u64 * 1000);
                let row_start = y * width;
                let elevation_row = &heightmap.data()[row_start..row_start + width];
                let mut row_pressures = Vec::with_capacity(width);

                // Process multiple cells with vectorizable operations
                for (x, &elevation) in elevation_row.iter().enumerate() {
                    let temperature_c =
                        temperature_layer.get_current_temperature(x, y, self.current_season);

                    // Vectorizable pressure calculations
                    let mut pressure = base_pressure;

                    // Apply elevation-based pressure reduction (vectorizable exp operation)
                    let elevation_meters = elevation.max(0.0) * 1000.0;
                    pressure *= (-elevation_meters * scale_height_inv).exp();

                    // Apply temperature-induced pressure variation (vectorizable)
                    let temp_deviation = temperature_c - base_temp_c;
                    let thermal_pressure_factor = 1.0 - (temp_deviation * 0.002);
                    pressure *= thermal_pressure_factor;

                    // Add small-scale pressure variations (fast PRNG)
                    rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
                    let noise = ((rng_state as f32) / (u32::MAX as f32) - 0.5) * 2.0;
                    pressure += noise * noise_amplitude;

                    // Clamp to reasonable atmospheric pressure range
                    pressure = pressure.max(50000.0).min(110000.0);

                    row_pressures.push(pressure);
                }

                row_pressures
            })
            .collect();

        // Assemble the pressure layer
        let mut pressure_layer = AtmosphericPressureLayer::new(width, height);
        pressure_layer.pressure = pressure_rows;

        // Calculate pressure gradients
        pressure_layer.calculate_pressure_gradients(scale.meters_per_pixel() as f32);

        pressure_layer
    }

    /// Specialized continental-scale optimization for 240x120 grids
    /// This version is hand-optimized for the most common continental simulation size
    #[cfg(feature = "simd")]
    pub fn generate_temperature_layer_continental_240x120(
        &self,
        heightmap: &super::super::core::heightmap::HeightMap,
    ) -> TemperatureLayer {
        use rayon::prelude::*;

        // Compile-time validation for expected size
        assert_eq!(heightmap.width(), 240);
        assert_eq!(heightmap.height(), 120);

        // Constants optimized for continental scale
        const WIDTH: usize = 240;
        const HEIGHT: usize = 120;

        // Create parallel row vectors optimized for continental grid
        let temperature_rows: Vec<Vec<f32>> = (0..HEIGHT)
            .into_par_iter()
            .map(|y| {
                let row_start = y * WIDTH;
                let elevation_row = &heightmap.data()[row_start..row_start + WIDTH];

                // Continental-scale specific optimizations
                let north_south_position = (y as f32) / 120.0; // Hardcoded for compiler optimization
                let distance_from_center = (north_south_position - 0.5).abs() * 2.0;
                let latitude_temperature_offset =
                    distance_from_center * self.parameters.latitude_gradient;

                let mut row_temps = Vec::with_capacity(240); // Hardcoded capacity

                // Process in optimal chunks for 240-wide continental grids
                for elevation_chunk in elevation_row.chunks(16) {
                    // Larger chunks for continental scale
                    for &elevation in elevation_chunk {
                        let mut temperature = self.parameters.base_temperature_c;
                        temperature -=
                            elevation.max(0.0) * self.parameters.elevation_lapse_rate * 1000.0;
                        temperature -= latitude_temperature_offset;

                        temperature = temperature
                            .max(self.parameters.min_temperature)
                            .min(self.parameters.max_temperature);

                        row_temps.push(temperature);
                    }
                }

                row_temps
            })
            .collect();

        // Seasonal variation optimized for continental scale
        let seasonal_rows: Vec<Vec<f32>> = (0..HEIGHT)
            .into_par_iter()
            .map(|y| {
                let north_south_position = (y as f32) / 120.0; // Hardcoded for optimization
                let distance_from_center = (north_south_position - 0.5).abs() * 2.0;
                let seasonal_variation =
                    self.parameters.seasonal_amplitude * (0.7 + distance_from_center * 0.3);

                vec![seasonal_variation; 240] // Hardcoded size
            })
            .collect();

        // Assemble the temperature layer
        let mut temp_layer = TemperatureLayer::new(240, 120);
        temp_layer.temperature = temperature_rows;
        temp_layer.seasonal_variation = seasonal_rows;

        // Apply spatial smoothing
        self.apply_spatial_smoothing(&mut temp_layer);

        temp_layer
    }

    /// Get current tick count for pressure noise generation
    fn tick_count(&self) -> u32 {
        // Convert seasonal position back to approximate tick count
        (self.current_season / self.seasonal_rate) as u32
    }

    /// Evolve existing pressure layer over time with gradual changes
    /// This preserves atmospheric circulation patterns while allowing realistic temporal evolution
    pub fn evolve_pressure_layer(
        &self,
        current_pressure: &mut AtmosphericPressureLayer,
        temperature_layer: &TemperatureLayer,
        heightmap: &[Vec<f32>],
        scale: &WorldScale,
        evolution_rate: f32,
    ) {
        let height = heightmap.len();
        let width = if height > 0 { heightmap[0].len() } else { 0 };

        // Use a fixed seed for weather noise evolution to ensure temporal continuity
        // This evolves the weather patterns gradually rather than regenerating them
        let mut rng_state = self
            .pressure_seed
            .wrapping_add((self.tick_count() / 10) as u64);

        // Evolve pressure for each cell
        for y in 0..height {
            for x in 0..width {
                let elevation = heightmap[y][x];
                let temperature_c =
                    temperature_layer.get_current_temperature(x, y, self.current_season);

                // Calculate target pressure based on current conditions
                let mut target_pressure = self.parameters.base_pressure_pa;

                // Apply elevation-based pressure reduction (hydrostatic balance)
                let scale_height = 8400.0; // meters
                let elevation_meters = elevation.max(0.0) * 1000.0; // Convert to meters
                target_pressure *= (-elevation_meters / scale_height).exp();

                // Apply temperature-pressure coupling (warmer air = lower pressure)
                let temp_deviation = temperature_c - self.parameters.base_temperature_c;
                let thermal_pressure_change =
                    -temp_deviation * self.parameters.pressure_temperature_coupling / 10.0;
                target_pressure += thermal_pressure_change;

                // Apply seasonal pressure variation
                let seasonal_factor = (self.current_season * 2.0 * std::f32::consts::PI).sin();
                target_pressure += seasonal_factor * self.parameters.seasonal_pressure_amplitude;

                // Evolve weather noise gradually instead of regenerating
                rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
                let noise_factor = ((rng_state as f32) / (u64::MAX as f32)) * 2.0 - 1.0; // -1 to 1
                let noise_contribution =
                    noise_factor * self.parameters.pressure_noise_amplitude * 0.1; // Smaller noise evolution

                // Gradually evolve toward target pressure instead of jumping to it
                let current_pressure_val = current_pressure.pressure[y][x];
                let pressure_change =
                    (target_pressure - current_pressure_val) * evolution_rate + noise_contribution;
                let new_pressure = current_pressure_val + pressure_change;

                // Clamp to reasonable atmospheric pressure bounds (500-1100 hPa)
                current_pressure.pressure[y][x] = new_pressure.max(50000.0).min(110000.0);
            }
        }

        // Recalculate pressure gradients after evolution
        current_pressure.calculate_pressure_gradients(scale.meters_per_pixel() as f32);
    }

    /// SIMD-optimized pressure evolution for better performance
    #[cfg(feature = "simd")]
    pub fn evolve_pressure_layer_simd(
        &self,
        current_pressure: &mut AtmosphericPressureLayer,
        temperature_layer: &TemperatureLayer,
        heightmap: &super::super::core::heightmap::HeightMap,
        scale: &WorldScale,
        evolution_rate: f32,
    ) {
        use rayon::prelude::*;

        let width = heightmap.width();
        let height = heightmap.height();

        // Pre-calculate constants outside the parallel loop
        let base_pressure = self.parameters.base_pressure_pa;
        let base_temp_c = self.parameters.base_temperature_c;
        let noise_amplitude = self.parameters.pressure_noise_amplitude * 0.1; // Smaller noise evolution
        let scale_height_inv = 1.0 / 8400.0; // Pre-calculate reciprocal
        let rng_base = self
            .pressure_seed
            .wrapping_add((self.tick_count() / 10) as u64);
        let seasonal_factor = (self.current_season * 2.0 * std::f32::consts::PI).sin()
            * self.parameters.seasonal_pressure_amplitude;
        let thermal_coupling = self.parameters.pressure_temperature_coupling / 10.0;

        // Process rows in parallel
        current_pressure
            .pressure
            .par_iter_mut()
            .enumerate()
            .for_each(|(y, row)| {
                let mut rng_state = rng_base.wrapping_add(y as u64 * 1000);

                for x in 0..width {
                    let elevation = heightmap.get(x, y);
                    let temperature_c =
                        temperature_layer.get_current_temperature(x, y, self.current_season);

                    // Calculate target pressure
                    let elevation_meters = elevation.max(0.0) * 1000.0;
                    let elevation_factor = (-elevation_meters * scale_height_inv).exp();
                    let temp_deviation = temperature_c - base_temp_c;
                    let thermal_change = -temp_deviation * thermal_coupling;

                    let mut target_pressure =
                        base_pressure * elevation_factor + thermal_change + seasonal_factor;

                    // Evolve weather noise gradually
                    rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
                    let noise_factor = ((rng_state as f32) / (u64::MAX as f32)) * 2.0 - 1.0;
                    let noise_contribution = noise_factor * noise_amplitude;

                    // Gradually evolve toward target pressure
                    let current_pressure_val = row[x];
                    let pressure_change = (target_pressure - current_pressure_val) * evolution_rate
                        + noise_contribution;
                    let new_pressure = current_pressure_val + pressure_change;

                    // Clamp to reasonable bounds
                    row[x] = new_pressure.max(50000.0).min(110000.0);
                }
            });

        // Recalculate pressure gradients after evolution
        current_pressure.calculate_pressure_gradients(scale.meters_per_pixel() as f32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::core::scale::{DetailLevel, WorldScale};

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

    // Atmospheric pressure tests
    #[test]
    fn atmospheric_pressure_layer_basic_operations() {
        let pressure_layer = AtmosphericPressureLayer::new(5, 5);

        // Should initialize to standard sea level pressure
        assert_eq!(pressure_layer.get_pressure(2, 2), 101325.0);
        assert_eq!(pressure_layer.get_pressure(10, 10), 101325.0); // Out of bounds should return default

        // Should initialize gradients to zero
        let gradient = pressure_layer.get_pressure_gradient(2, 2);
        assert_eq!(gradient.x, 0.0);
        assert_eq!(gradient.y, 0.0);
    }

    #[test]
    fn pressure_gradient_calculation() {
        let mut pressure_layer = AtmosphericPressureLayer::new(3, 3);

        // Create a simple pressure gradient (high to low from left to right)
        pressure_layer.pressure[1][0] = 102000.0; // High pressure
        pressure_layer.pressure[1][1] = 101325.0; // Standard pressure
        pressure_layer.pressure[1][2] = 100650.0; // Low pressure

        let meters_per_pixel = 1000.0; // 1km per pixel
        pressure_layer.calculate_pressure_gradients(meters_per_pixel);

        // Center cell should have negative x gradient (pressure decreases to the right)
        let center_gradient = pressure_layer.get_pressure_gradient(1, 1);
        assert!(
            center_gradient.x < 0.0,
            "Pressure gradient should be negative (decreasing to right)"
        );

        // Gradient magnitude should be reasonable
        let expected_gradient = (100650.0 - 102000.0) / (2.0 * meters_per_pixel); // Central difference
        assert!(
            (center_gradient.x - expected_gradient).abs() < 0.1,
            "Gradient calculation should match expected value"
        );
    }

    #[test]
    fn pressure_generation_from_temperature() {
        let heightmap = vec![
            vec![0.0, 0.0, 0.0], // Flat terrain
            vec![0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ];
        let scale = WorldScale::new(10.0, (3, 3), DetailLevel::Standard);
        let climate = ClimateSystem::new_for_scale(&scale);
        let temp_layer = climate.generate_temperature_layer(&heightmap);
        let pressure_layer = climate.generate_pressure_layer(&temp_layer, &heightmap, &scale);

        // Pressure should be in reasonable atmospheric range
        let avg_pressure = pressure_layer.get_average_pressure();
        assert!(
            avg_pressure > 50000.0 && avg_pressure < 110000.0,
            "Average pressure should be in reasonable range, got: {}",
            avg_pressure
        );

        // Should have some pressure variation due to temperature coupling and noise
        let mut pressures = Vec::new();
        for y in 0..3 {
            for x in 0..3 {
                pressures.push(pressure_layer.get_pressure(x, y));
            }
        }
        let min_pressure = pressures.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_pressure = pressures.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        // Should have some variation (at least from noise)
        assert!(
            max_pressure > min_pressure,
            "Should have pressure variation"
        );
    }

    #[test]
    fn pressure_elevation_dependence() {
        let heightmap = vec![
            vec![0.0, 0.5, 1.0], // Sea level to mountain
            vec![0.0, 0.5, 1.0],
            vec![0.0, 0.5, 1.0],
        ];
        let scale = WorldScale::new(10.0, (3, 3), DetailLevel::Standard);
        let climate = ClimateSystem::new_for_scale(&scale);
        let temp_layer = climate.generate_temperature_layer(&heightmap);
        let pressure_layer = climate.generate_pressure_layer(&temp_layer, &heightmap, &scale);

        // Higher elevations should have lower pressure (barometric formula)
        let sea_level_pressure = pressure_layer.get_pressure(0, 0);
        let mountain_pressure = pressure_layer.get_pressure(2, 0);

        assert!(
            mountain_pressure < sea_level_pressure,
            "Mountain pressure ({:.0} Pa) should be lower than sea level ({:.0} Pa)",
            mountain_pressure,
            sea_level_pressure
        );
    }

    #[test]
    fn pressure_parameters_scaling() {
        let base_params = ClimateParameters::default();
        let small_scale = WorldScale::new(1.0, (50, 50), DetailLevel::Standard);
        let large_scale = WorldScale::new(1000.0, (500, 500), DetailLevel::Standard);

        let small_scaled = base_params.derive_parameters(&small_scale);
        let large_scaled = base_params.derive_parameters(&large_scale);

        // Base pressure should remain constant
        assert_eq!(small_scaled.base_pressure_pa, large_scaled.base_pressure_pa);

        // Larger maps should have stronger pressure variations
        assert!(
            large_scaled.pressure_temperature_coupling > small_scaled.pressure_temperature_coupling
        );
        assert!(
            large_scaled.seasonal_pressure_amplitude > small_scaled.seasonal_pressure_amplitude
        );
        assert!(large_scaled.pressure_noise_amplitude > small_scaled.pressure_noise_amplitude);
    }
}
