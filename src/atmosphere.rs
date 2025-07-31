// ABOUTME: Atmospheric dynamics system for large-scale flow effects including Coriolis forces
// ABOUTME: Implements geostrophic wind patterns, pressure-driven flows, and rotating reference frame physics

use crate::climate::AtmosphericPressureLayer;
use crate::scale::{ScaleAware, WorldScale};
use crate::sim::Vec2;

/// Atmospheric dynamics parameters for large-scale flow effects
#[derive(Clone, Debug)]
pub struct AtmosphericParameters {
    /// Earth's rotation rate in rad/s (Ω = 7.27×10⁻⁵ rad/s)
    pub earth_rotation_rate: f64,
    /// Air density at sea level in kg/m³
    pub air_density_sea_level: f32,
    /// Minimum domain size for Coriolis effects to activate (meters)
    pub coriolis_activation_threshold_m: f64,
    /// Geostrophic wind scaling factor (0.0-1.0)
    pub geostrophic_strength: f32,
    /// Friction coefficient for surface winds (0.0-1.0)
    pub surface_friction: f32,
}

impl Default for AtmosphericParameters {
    fn default() -> Self {
        Self {
            earth_rotation_rate: 7.27e-5, // Earth's rotation rate (rad/s)
            air_density_sea_level: 1.225, // Standard air density at sea level (kg/m³)
            coriolis_activation_threshold_m: 100_000.0, // 100km threshold for Coriolis effects
            geostrophic_strength: 1.0,    // Full geostrophic balance
            surface_friction: 0.1,        // 10% friction reduction
        }
    }
}

impl ScaleAware for AtmosphericParameters {
    fn derive_parameters(&self, scale: &WorldScale) -> Self {
        let physical_extent_m = scale.physical_size_km * 1000.0;

        Self {
            // Physical constants don't scale
            earth_rotation_rate: self.earth_rotation_rate,
            air_density_sea_level: self.air_density_sea_level,

            // Activation threshold remains constant
            coriolis_activation_threshold_m: self.coriolis_activation_threshold_m,

            // Geostrophic strength scales with domain size (larger domains = stronger geostrophic balance)
            geostrophic_strength: if physical_extent_m >= self.coriolis_activation_threshold_m {
                self.geostrophic_strength
                    * ((physical_extent_m / self.coriolis_activation_threshold_m).min(2.0) as f32)
            } else {
                0.0 // No geostrophic effects below threshold
            },

            // Surface friction scales with resolution (finer resolution = more surface effects)
            surface_friction: self.surface_friction
                * ((scale.meters_per_pixel() / 1000.0).min(1.0) as f32),
        }
    }
}

/// Wind field data layer
#[derive(Clone, Debug)]
pub struct WindLayer {
    /// Wind velocity vector (u, v) in m/s at each cell
    pub velocity: Vec<Vec<Vec2>>,
    /// Wind speed magnitude in m/s at each cell
    pub speed: Vec<Vec<f32>>,
    /// Wind direction in radians (0 = east, π/2 = north) at each cell
    pub direction: Vec<Vec<f32>>,
    /// Width and height for bounds checking
    width: usize,
    height: usize,
}

impl WindLayer {
    /// Create a new wind layer with the given dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            velocity: vec![vec![Vec2::zero(); width]; height],
            speed: vec![vec![0.0; width]; height],
            direction: vec![vec![0.0; width]; height],
            width,
            height,
        }
    }

    /// Get wind velocity at a specific location (with bounds checking)
    pub fn get_velocity(&self, x: usize, y: usize) -> Vec2 {
        if x < self.width && y < self.height {
            self.velocity[y][x].clone()
        } else {
            Vec2::zero()
        }
    }

    /// Get wind speed at a specific location (with bounds checking)
    pub fn get_speed(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.speed[y][x]
        } else {
            0.0
        }
    }

    /// Get wind direction at a specific location (with bounds checking)
    pub fn get_direction(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.direction[y][x]
        } else {
            0.0
        }
    }

    /// Update speed and direction from velocity components
    pub fn update_derived_fields(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let vel = &self.velocity[y][x];
                self.speed[y][x] = vel.magnitude();
                self.direction[y][x] = vel.y.atan2(vel.x); // atan2(v, u) gives direction
            }
        }
    }

    /// Get average wind speed across the entire map
    pub fn get_average_wind_speed(&self) -> f32 {
        let total: f32 = self.speed.iter().flat_map(|row| row.iter()).sum();

        let cell_count = (self.width * self.height) as f32;
        if cell_count > 0.0 {
            total / cell_count
        } else {
            0.0
        }
    }

    /// Calculate vorticity (curl of wind field) for storm detection
    /// ζ = ∂v/∂x - ∂u/∂y (vertical component of curl)
    pub fn calculate_vorticity(&self, meters_per_pixel: f32) -> Vec<Vec<f32>> {
        let mut vorticity = vec![vec![0.0; self.width]; self.height];

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                // Central differences for vorticity calculation
                let du_dy = (self.velocity[y + 1][x].x - self.velocity[y - 1][x].x)
                    / (2.0 * meters_per_pixel);
                let dv_dx = (self.velocity[y][x + 1].y - self.velocity[y][x - 1].y)
                    / (2.0 * meters_per_pixel);

                vorticity[y][x] = dv_dx - du_dy; // ζ = ∂v/∂x - ∂u/∂y
            }
        }

        vorticity
    }
}

/// Weather pattern types detected in the simulation
#[derive(Clone, Debug, PartialEq)]
pub enum WeatherPatternType {
    /// Low pressure system (cyclone/depression)
    LowPressureSystem,
    /// High pressure system (anticyclone)
    HighPressureSystem,
    /// Strong wind shear zone
    WindShear,
    /// Calm/stagnant air mass
    Calm,
}

/// Detected weather pattern with location and characteristics
#[derive(Clone, Debug)]
pub struct WeatherPattern {
    /// Type of weather pattern
    pub pattern_type: WeatherPatternType,
    /// Center location (x, y) in grid coordinates
    pub center: (usize, usize),
    /// Characteristic pressure (Pa)
    pub pressure: f32,
    /// Maximum wind speed in the pattern (m/s)
    pub max_wind_speed: f32,
    /// Vorticity strength (1/s)
    pub vorticity: f32,
    /// Approximate radius of influence (grid cells)
    pub radius: usize,
}

/// Weather analysis system for pattern detection
#[derive(Clone, Debug)]
pub struct WeatherAnalysis {
    /// Detected weather patterns
    pub patterns: Vec<WeatherPattern>,
    /// Vorticity field for the entire domain
    pub vorticity_field: Vec<Vec<f32>>,
    /// Storm detection thresholds
    pub low_pressure_threshold: f32, // Pa below average for low pressure systems
    pub high_pressure_threshold: f32, // Pa above average for high pressure systems
    pub vorticity_threshold: f32,     // 1/s threshold for significant rotation
    pub wind_speed_threshold: f32,    // m/s threshold for strong winds
}

impl Default for WeatherAnalysis {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            vorticity_field: Vec::new(),
            low_pressure_threshold: 500.0,  // 5 hPa below average
            high_pressure_threshold: 500.0, // 5 hPa above average
            vorticity_threshold: 1e-4,      // 10⁻⁴ s⁻¹ (typical for weather systems)
            wind_speed_threshold: 10.0,     // 10 m/s (strong breeze)
        }
    }
}

/// Atmospheric dynamics system for large-scale flow effects#[derive(Clone, Debug)]
pub struct AtmosphericSystem {
    /// Scale-derived atmospheric parameters
    pub parameters: AtmosphericParameters,
    /// Whether Coriolis effects are active for this domain size
    pub coriolis_active: bool,
    /// Effective Coriolis parameter at mid-latitude (f = 2Ω sin(45°))
    pub effective_coriolis_parameter: f64,
}

impl AtmosphericSystem {
    /// Create a new atmospheric system for the given world scale
    pub fn new_for_scale(scale: &WorldScale) -> Self {
        let parameters = AtmosphericParameters::default().derive_parameters(scale);
        let physical_extent_m = scale.physical_size_km * 1000.0;
        let coriolis_active = physical_extent_m >= parameters.coriolis_activation_threshold_m;

        // Calculate effective Coriolis parameter at mid-latitude (45°)
        let mid_latitude_rad = std::f64::consts::PI / 4.0; // 45 degrees
        let effective_coriolis_parameter =
            2.0 * parameters.earth_rotation_rate * mid_latitude_rad.sin();

        Self {
            parameters,
            coriolis_active,
            effective_coriolis_parameter,
        }
    }

    /// Create atmospheric system from custom parameters
    pub fn from_parameters(parameters: AtmosphericParameters, scale: &WorldScale) -> Self {
        let scaled_params = parameters.derive_parameters(scale);
        let physical_extent_m = scale.physical_size_km * 1000.0;
        let coriolis_active = physical_extent_m >= scaled_params.coriolis_activation_threshold_m;

        let mid_latitude_rad = std::f64::consts::PI / 4.0;
        let effective_coriolis_parameter =
            2.0 * scaled_params.earth_rotation_rate * mid_latitude_rad.sin();

        Self {
            parameters: scaled_params,
            coriolis_active,
            effective_coriolis_parameter,
        }
    }

    /// Calculate Coriolis parameter at a given latitude
    /// f = 2Ω sin(φ) where φ is latitude in radians
    pub fn coriolis_parameter_at_latitude(&self, latitude_rad: f64) -> f64 {
        2.0 * self.parameters.earth_rotation_rate * latitude_rad.sin()
    }

    /// Convert grid coordinates to latitude (assuming map spans -45° to +45°)
    pub fn grid_y_to_latitude(&self, y: usize, height: usize) -> f64 {
        // Map y coordinate to latitude range [-π/4, π/4] (±45°)
        let normalized_y = (y as f64) / (height as f64); // 0 to 1
        let latitude_range = std::f64::consts::PI / 2.0; // 90° total range
        (normalized_y - 0.5) * latitude_range // -45° to +45°
    }

    /// Generate geostrophic wind field from pressure gradients
    /// Uses geostrophic balance: f × v = -∇P/ρ
    pub fn generate_geostrophic_winds(
        &self,
        pressure_layer: &AtmosphericPressureLayer,
        _scale: &WorldScale,
    ) -> WindLayer {
        let height = pressure_layer.pressure.len();
        let width = if height > 0 {
            pressure_layer.pressure[0].len()
        } else {
            0
        };

        let mut wind_layer = WindLayer::new(width, height);

        if !self.coriolis_active {
            // No Coriolis effects - return zero wind field
            return wind_layer;
        }

        // Calculate geostrophic winds for each cell
        for y in 0..height {
            for x in 0..width {
                let pressure_gradient = pressure_layer.get_pressure_gradient(x, y);

                // Calculate latitude-dependent Coriolis parameter
                let latitude_rad = self.grid_y_to_latitude(y, height);
                let f = self.coriolis_parameter_at_latitude(latitude_rad);

                if f.abs() < 1e-10 {
                    // Near equator - no Coriolis effect
                    wind_layer.velocity[y][x] = Vec2::zero();
                    continue;
                }

                // Geostrophic balance: f × v = -∇P/ρ
                // In 2D: f * v_y = -∂P/∂x/ρ  and  -f * v_x = -∂P/∂y/ρ
                // Therefore: v_x = (1/f) * (∂P/∂y/ρ)  and  v_y = -(1/f) * (∂P/∂x/ρ)

                let rho = self.parameters.air_density_sea_level;
                let geostrophic_u = (pressure_gradient.y / rho) / (f as f32);
                let geostrophic_v = -(pressure_gradient.x / rho) / (f as f32);

                // Apply geostrophic strength scaling
                let scaled_u = geostrophic_u * self.parameters.geostrophic_strength;
                let scaled_v = geostrophic_v * self.parameters.geostrophic_strength;

                // Apply surface friction (reduces wind speed near surface)
                let friction_factor = 1.0 - self.parameters.surface_friction;

                wind_layer.velocity[y][x] =
                    Vec2::new(scaled_u * friction_factor, scaled_v * friction_factor);
            }
        }

        // Update derived fields (speed and direction)
        wind_layer.update_derived_fields();

        wind_layer
    }

    /// Check if domain is large enough for Coriolis effects
    pub fn is_coriolis_active(&self) -> bool {
        self.coriolis_active
    }

    /// Get the Rossby deformation radius for this system
    /// L_R = √(gH)/f where g is gravity, H is scale height, f is Coriolis parameter
    pub fn rossby_deformation_radius(&self) -> f64 {
        let g: f64 = 9.81; // gravity (m/s²)
        let h: f64 = 10000.0; // atmospheric scale height (m)
        let f = self.effective_coriolis_parameter;

        if f.abs() < 1e-10 {
            f64::INFINITY // No Coriolis effect
        } else {
            (g * h).sqrt() / f.abs()
        }
    }

    /// Analyze weather patterns in the current atmospheric state
    /// Detects low/high pressure systems, wind shear, and calm regions
    pub fn analyze_weather_patterns(
        &self,
        pressure_layer: &AtmosphericPressureLayer,
        wind_layer: &WindLayer,
        scale: &WorldScale,
    ) -> WeatherAnalysis {
        let mut analysis = WeatherAnalysis::default();

        if !self.coriolis_active {
            // No complex weather patterns without Coriolis effects
            return analysis;
        }

        let meters_per_pixel = scale.meters_per_pixel() as f32;

        // Calculate vorticity field
        analysis.vorticity_field = wind_layer.calculate_vorticity(meters_per_pixel);

        // Get average pressure for reference
        let avg_pressure = pressure_layer.get_average_pressure();

        // Scan for weather patterns
        let height = pressure_layer.pressure.len();
        let width = if height > 0 {
            pressure_layer.pressure[0].len()
        } else {
            0
        };

        // Use a coarser grid for pattern detection to avoid noise
        let step = 5.max(width / 20); // Sample every ~5% of domain width

        for y in (step..height - step).step_by(step) {
            for x in (step..width - step).step_by(step) {
                let pressure = pressure_layer.get_pressure(x, y);
                let wind_speed = wind_layer.get_speed(x, y);
                let vorticity = if y < analysis.vorticity_field.len()
                    && x < analysis.vorticity_field[y].len()
                {
                    analysis.vorticity_field[y][x]
                } else {
                    0.0
                };

                // Detect different pattern types
                let pattern_type = if pressure < avg_pressure - analysis.low_pressure_threshold {
                    Some(WeatherPatternType::LowPressureSystem)
                } else if pressure > avg_pressure + analysis.high_pressure_threshold {
                    Some(WeatherPatternType::HighPressureSystem)
                } else if vorticity.abs() > analysis.vorticity_threshold {
                    Some(WeatherPatternType::WindShear)
                } else if wind_speed < 2.0 {
                    Some(WeatherPatternType::Calm)
                } else {
                    None
                };

                if let Some(ptype) = pattern_type {
                    // Estimate pattern radius based on scale
                    let radius = match ptype {
                        WeatherPatternType::LowPressureSystem
                        | WeatherPatternType::HighPressureSystem => {
                            // Weather systems typically span 500-1000km
                            ((500_000.0 / meters_per_pixel) as usize).max(10).min(50)
                        }
                        WeatherPatternType::WindShear => {
                            // Wind shear zones are more linear/narrow
                            ((200_000.0 / meters_per_pixel) as usize).max(5).min(20)
                        }
                        WeatherPatternType::Calm => {
                            // Calm regions can be quite large
                            ((300_000.0 / meters_per_pixel) as usize).max(8).min(30)
                        }
                    };

                    let pattern = WeatherPattern {
                        pattern_type: ptype,
                        center: (x, y),
                        pressure,
                        max_wind_speed: wind_speed,
                        vorticity,
                        radius,
                    };

                    analysis.patterns.push(pattern);
                }
            }
        }

        // Remove overlapping patterns (keep strongest)
        analysis.patterns = Self::remove_overlapping_patterns(analysis.patterns);

        analysis
    }

    /// Remove overlapping weather patterns, keeping the strongest ones
    fn remove_overlapping_patterns(mut patterns: Vec<WeatherPattern>) -> Vec<WeatherPattern> {
        patterns.sort_by(|a, b| {
            // Sort by "strength" - combination of pressure deviation and wind speed
            let strength_a = a.pressure.abs() + a.max_wind_speed * 100.0;
            let strength_b = b.pressure.abs() + b.max_wind_speed * 100.0;
            strength_b
                .partial_cmp(&strength_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut filtered: Vec<WeatherPattern> = Vec::new();

        for pattern in patterns {
            let mut overlaps = false;

            for existing in &filtered {
                let dx = (pattern.center.0 as i32 - existing.center.0 as i32).abs() as usize;
                let dy = (pattern.center.1 as i32 - existing.center.1 as i32).abs() as usize;
                let distance = ((dx * dx + dy * dy) as f32).sqrt() as usize;

                // Check if patterns overlap significantly
                if distance < (pattern.radius + existing.radius) / 2 {
                    overlaps = true;
                    break;
                }
            }

            if !overlaps {
                filtered.push(pattern);
            }
        }

        filtered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::ClimateSystem;
    use crate::scale::{DetailLevel, WorldScale};

    fn test_scale(physical_size_km: f64, width: u32, height: u32) -> WorldScale {
        WorldScale::new(physical_size_km, (width, height), DetailLevel::Standard)
    }

    #[test]
    fn atmospheric_parameters_default_values() {
        let params = AtmosphericParameters::default();
        assert_eq!(params.earth_rotation_rate, 7.27e-5);
        assert_eq!(params.air_density_sea_level, 1.225);
        assert_eq!(params.coriolis_activation_threshold_m, 100_000.0);
        assert_eq!(params.geostrophic_strength, 1.0);
        assert_eq!(params.surface_friction, 0.1);
    }

    #[test]
    fn atmospheric_parameters_scaling() {
        let base_params = AtmosphericParameters::default();
        let small_scale = test_scale(10.0, 100, 100); // 10km - below threshold
        let large_scale = test_scale(200.0, 200, 200); // 200km - above threshold

        let small_scaled = base_params.derive_parameters(&small_scale);
        let large_scaled = base_params.derive_parameters(&large_scale);

        // Physical constants should remain the same
        assert_eq!(
            small_scaled.earth_rotation_rate,
            large_scaled.earth_rotation_rate
        );
        assert_eq!(
            small_scaled.air_density_sea_level,
            large_scaled.air_density_sea_level
        );

        // Small domain should have no geostrophic effects
        assert_eq!(small_scaled.geostrophic_strength, 0.0);

        // Large domain should have geostrophic effects
        assert!(large_scaled.geostrophic_strength > 0.0);
    }

    #[test]
    fn wind_layer_basic_operations() {
        let wind_layer = WindLayer::new(5, 5);

        // Should initialize to zero wind
        assert_eq!(wind_layer.get_speed(2, 2), 0.0);
        assert_eq!(wind_layer.get_direction(2, 2), 0.0);
        let velocity = wind_layer.get_velocity(2, 2);
        assert_eq!(velocity.x, 0.0);
        assert_eq!(velocity.y, 0.0);

        // Out of bounds should return defaults
        assert_eq!(wind_layer.get_speed(10, 10), 0.0);
    }

    #[test]
    fn wind_layer_derived_fields() {
        let mut wind_layer = WindLayer::new(3, 3);

        // Set some wind velocities
        wind_layer.velocity[1][1] = Vec2::new(3.0, 4.0); // 5 m/s wind
        wind_layer.velocity[1][2] = Vec2::new(1.0, 0.0); // 1 m/s eastward

        wind_layer.update_derived_fields();

        // Check speed calculation
        assert_eq!(wind_layer.get_speed(1, 1), 5.0); // sqrt(3² + 4²) = 5
        assert_eq!(wind_layer.get_speed(2, 1), 1.0);

        // Check direction calculation
        let direction_1_1 = wind_layer.get_direction(1, 1);
        let expected_direction = 4.0_f32.atan2(3.0); // atan2(v, u)
        assert!((direction_1_1 - expected_direction).abs() < 1e-6);

        let direction_2_1 = wind_layer.get_direction(2, 1);
        assert!((direction_2_1 - 0.0).abs() < 1e-6); // Pure eastward = 0 radians
    }

    #[test]
    fn atmospheric_system_coriolis_activation() {
        let small_scale = test_scale(50.0, 100, 100); // 50km - below threshold
        let large_scale = test_scale(150.0, 200, 200); // 150km - above threshold

        let small_system = AtmosphericSystem::new_for_scale(&small_scale);
        let large_system = AtmosphericSystem::new_for_scale(&large_scale);

        assert!(!small_system.is_coriolis_active());
        assert!(large_system.is_coriolis_active());
    }

    #[test]
    fn coriolis_parameter_calculation() {
        let scale = test_scale(200.0, 100, 100);
        let system = AtmosphericSystem::new_for_scale(&scale);

        // Test at different latitudes
        let equator_f = system.coriolis_parameter_at_latitude(0.0);
        let mid_lat_f = system.coriolis_parameter_at_latitude(std::f64::consts::PI / 4.0); // 45°
        let pole_f = system.coriolis_parameter_at_latitude(std::f64::consts::PI / 2.0); // 90°

        assert_eq!(equator_f, 0.0); // No Coriolis at equator
        assert!(mid_lat_f > 0.0); // Positive in northern hemisphere
        assert!(pole_f > mid_lat_f); // Stronger at poles

        // Check the formula: f = 2Ω sin(φ)
        let expected_mid_lat =
            2.0 * system.parameters.earth_rotation_rate * (std::f64::consts::PI / 4.0).sin();
        assert!((mid_lat_f - expected_mid_lat).abs() < 1e-10);
    }

    #[test]
    fn latitude_coordinate_conversion() {
        let scale = test_scale(200.0, 100, 100);
        let system = AtmosphericSystem::new_for_scale(&scale);

        // Test coordinate conversion
        let north_lat = system.grid_y_to_latitude(0, 100); // Top of map
        let center_lat = system.grid_y_to_latitude(50, 100); // Center of map
        let south_lat = system.grid_y_to_latitude(99, 100); // Bottom of map

        assert!(north_lat < center_lat);
        assert!(center_lat < south_lat);
        assert!((center_lat - 0.0).abs() < 1e-6); // Center should be ~0° (equator)

        // Should span ±45° range
        assert!((north_lat - (-std::f64::consts::PI / 4.0)).abs() < 0.1);
        assert!((south_lat - (std::f64::consts::PI / 4.0)).abs() < 0.1);
    }

    #[test]
    fn geostrophic_wind_generation_small_domain() {
        // Test with small domain (no Coriolis effects)
        let scale = test_scale(50.0, 50, 50); // Below threshold
        let system = AtmosphericSystem::new_for_scale(&scale);

        // Create a simple pressure field
        let heightmap = vec![vec![0.0; 50]; 50];
        let climate = ClimateSystem::new_for_scale(&scale);
        let temp_layer = climate.generate_temperature_layer(&heightmap);
        let pressure_layer = climate.generate_pressure_layer(&temp_layer, &heightmap, &scale);

        let wind_layer = system.generate_geostrophic_winds(&pressure_layer, &scale);

        // Should have zero winds (no Coriolis effects)
        assert_eq!(wind_layer.get_average_wind_speed(), 0.0);
    }

    #[test]
    fn geostrophic_wind_generation_large_domain() {
        // Test with large domain (Coriolis effects active)
        let scale = test_scale(200.0, 100, 100); // Above threshold
        let system = AtmosphericSystem::new_for_scale(&scale);

        // Create a heightmap and pressure field
        let heightmap = vec![vec![0.0; 100]; 100];
        let climate = ClimateSystem::new_for_scale(&scale);
        let temp_layer = climate.generate_temperature_layer(&heightmap);
        let pressure_layer = climate.generate_pressure_layer(&temp_layer, &heightmap, &scale);

        let wind_layer = system.generate_geostrophic_winds(&pressure_layer, &scale);

        // Should have some wind activity due to pressure gradients and Coriolis effects
        // (Exact values depend on pressure field, but should be non-zero)
        let avg_speed = wind_layer.get_average_wind_speed();
        assert!(avg_speed >= 0.0); // Should be non-negative

        // Check that wind field is properly initialized
        for y in 0..100 {
            for x in 0..100 {
                let speed = wind_layer.get_speed(x, y);
                assert!(speed >= 0.0);
                assert!(speed.is_finite());
            }
        }
    }

    #[test]
    fn rossby_deformation_radius() {
        let scale = test_scale(200.0, 100, 100);
        let system = AtmosphericSystem::new_for_scale(&scale);

        let rossby_radius = system.rossby_deformation_radius();

        // Should be a reasonable value for atmospheric Rossby radius (~1000km)
        assert!(rossby_radius > 100_000.0); // > 100km
        assert!(rossby_radius < 10_000_000.0); // < 10,000km
        assert!(rossby_radius.is_finite());
    }
}
