// ABOUTME: Orographic precipitation coupling - terrain-driven rainfall patterns via elevation-climate coupling
// ABOUTME: Implements physical orographic lift and adiabatic cooling for realistic mountain weather patterns

use super::atmospheric_moisture::AtmosphericMoistureSystem;
use super::climate::ClimateSystem;
use super::flow_engine::{FlowEngine, VelocityField};
use crate::engine::core::{heightmap::HeightMap, scale::WorldScale};

/// Orographic precipitation parameters controlling mountain weather effects
///
/// **Physical Foundation**: Orographic precipitation occurs when air masses are forced
/// upward by terrain, causing adiabatic cooling and moisture condensation. This is one
/// of the most important mechanisms for creating spatial precipitation patterns.
#[derive(Clone, Debug)]
pub struct OrographicParameters {
    /// Lifting condensation level (m) - height where clouds form
    /// Typical values: 1000-3000m depending on temperature and humidity
    pub lifting_condensation_level: f32,

    /// Dry adiabatic lapse rate (°C/100m)
    /// Physical constant: ~0.98°C/100m for dry air
    pub dry_lapse_rate: f32,

    /// Moist adiabatic lapse rate (°C/100m)
    /// Physical constant: ~0.6°C/100m for saturated air (varies with temperature)
    pub moist_lapse_rate: f32,

    /// Minimum wind speed for orographic effects (m/s)
    /// Below this threshold, air flows around rather than over mountains
    pub min_wind_speed: f32,

    /// Precipitation efficiency factor (0.0-1.0)
    /// Fraction of condensed moisture that falls as precipitation vs. carried away
    pub precipitation_efficiency: f32,

    /// Shadow zone multiplier (0.0-1.0)
    /// Reduction factor for precipitation on leeward (downwind) side of mountains
    pub rain_shadow_factor: f32,

    /// Maximum orographic enhancement ratio
    /// Limits how much precipitation can be increased by terrain (prevents unrealistic values)
    pub max_enhancement_ratio: f32,
}

impl Default for OrographicParameters {
    fn default() -> Self {
        Self {
            lifting_condensation_level: 1500.0, // 1.5km typical
            dry_lapse_rate: 0.0098,             // °C/m (9.8°C/km)
            moist_lapse_rate: 0.006,            // °C/m (6°C/km)
            min_wind_speed: 2.0,                // m/s minimum for orographic lifting
            precipitation_efficiency: 0.7,      // 70% of condensed moisture falls locally
            rain_shadow_factor: 0.3,            // 30% of normal precipitation in shadow zones
            max_enhancement_ratio: 5.0,         // Up to 5x precipitation enhancement
        }
    }
}

/// Orographic precipitation effects calculated from terrain and atmospheric conditions
///
/// **Educational Context**: This represents the spatial pattern of how mountains modify
/// precipitation. The windward side gets enhanced rainfall while the leeward side
/// experiences rain shadow effects, creating dramatic precipitation gradients.
#[derive(Debug, Clone)]
pub struct OrographicEffects {
    /// Precipitation enhancement factor at each cell (0.0+ multiplier)
    /// 1.0 = no enhancement, >1.0 = increased precipitation, <1.0 = rain shadow
    pub precipitation_multiplier: Vec<Vec<f32>>,

    /// Vertical air velocity due to orographic lifting (m/s)
    /// Positive = upward motion (windward), negative = downward (leeward)
    pub vertical_velocity: Vec<Vec<f32>>,

    /// Condensation rate from orographic cooling (kg/m²/s)
    /// Amount of moisture condensing due to adiabatic cooling during uplift
    pub condensation_rate: Vec<Vec<f32>>,

    /// Rain shadow intensity (0.0-1.0)
    /// 1.0 = full rain shadow effect, 0.0 = no shadow
    pub rain_shadow_intensity: Vec<Vec<f32>>,

    /// Effective lifting height (m)
    /// How much air is lifted above the lifting condensation level
    pub lifting_height: Vec<Vec<f32>>,

    /// Grid dimensions
    pub width: usize,
    pub height: usize,
}

impl OrographicEffects {
    /// Calculate orographic precipitation effects from terrain and atmospheric flow
    ///
    /// **Physical Process**:
    /// 1. Air masses encounter terrain and are forced upward (orographic lifting)
    /// 2. Rising air cools adiabatically (expansion cooling in lower pressure)
    /// 3. When air reaches lifting condensation level, water vapor condenses
    /// 4. Condensed moisture falls as enhanced precipitation on windward slopes
    /// 5. Descending air on leeward side creates dry "rain shadow" zones
    ///
    /// **Mathematical Foundation**:
    /// - Vertical velocity: w = u × dh/dx (wind speed × terrain slope)
    /// - Adiabatic cooling: ΔT = -Γ × Δz (lapse rate × height change)
    /// - Condensation: dq/dt = function of cooling rate and saturation deficit
    pub fn from_terrain_and_wind(
        heightmap: &HeightMap,
        velocity_field: &VelocityField,
        atmospheric_moisture: &AtmosphericMoistureSystem,
        parameters: &OrographicParameters,
        scale: &WorldScale,
    ) -> Self {
        let width = heightmap.width();
        let height = heightmap.height();
        let cell_size_m = scale.meters_per_pixel() as f32;

        // Initialize output arrays
        let mut precipitation_multiplier = vec![vec![1.0; height]; width];
        let mut vertical_velocity = vec![vec![0.0; height]; width];
        let mut condensation_rate = vec![vec![0.0; height]; width];
        let mut rain_shadow_intensity = vec![vec![0.0; height]; width];
        let mut lifting_height = vec![vec![0.0; height]; width];

        // Calculate orographic effects for each cell
        for x in 1..width - 1 {
            for y in 1..height - 1 {
                let elevation = heightmap.get(x, y);
                let wind_velocity = velocity_field.get_velocity(x, y);
                let wind_speed = wind_velocity.magnitude();

                // Skip if wind is too weak for orographic effects
                if wind_speed < parameters.min_wind_speed {
                    continue;
                }

                // Calculate terrain gradients (slopes) in wind direction
                let (slope_upwind, slope_downwind) =
                    calculate_terrain_slopes(heightmap, x, y, &wind_velocity, cell_size_m);

                // 1. Calculate vertical air velocity from orographic forcing
                // w = u × sin(θ) ≈ u × dh/dx for small slopes
                let vertical_vel = wind_speed * slope_upwind;
                vertical_velocity[x][y] = vertical_vel;

                // 2. Determine lifting height above condensation level
                let terrain_height_m = elevation * 3000.0; // Convert normalized to meters
                let lift_height = if terrain_height_m > parameters.lifting_condensation_level {
                    terrain_height_m - parameters.lifting_condensation_level
                } else {
                    0.0
                };
                lifting_height[x][y] = lift_height;

                // 3. Calculate adiabatic cooling and condensation
                if vertical_vel > 0.0 && lift_height > 0.0 {
                    // Rising air cools at dry rate until LCL, then moist rate
                    let cooling_rate = if terrain_height_m <= parameters.lifting_condensation_level
                    {
                        parameters.dry_lapse_rate
                    } else {
                        parameters.moist_lapse_rate
                    };

                    // Temperature drop due to lifting
                    let temperature_drop = cooling_rate * lift_height;

                    // Calculate condensation rate from cooling (simplified)
                    // More cooling → more condensation
                    let available_moisture =
                        atmospheric_moisture.surface_moisture.get_humidity(x, y);

                    let condensation = available_moisture
                        * (temperature_drop / 10.0).min(1.0) // Empirical scaling
                        * (vertical_vel / 5.0).min(1.0); // Stronger lifting → more condensation

                    condensation_rate[x][y] = condensation * parameters.precipitation_efficiency;

                    // Enhanced precipitation on windward slopes
                    let enhancement = 1.0 + (lift_height / 1000.0) * (vertical_vel / 2.0);
                    precipitation_multiplier[x][y] =
                        enhancement.min(parameters.max_enhancement_ratio);
                }

                // 4. Calculate rain shadow effects on leeward slopes
                if slope_downwind < -0.001 {
                    // Downwind slope (negative gradient)
                    // Descending air warms and dries
                    let shadow_strength = (-slope_downwind * wind_speed * 2.0).min(1.0);
                    rain_shadow_intensity[x][y] = shadow_strength;

                    // Reduced precipitation in rain shadow
                    let shadow_reduction =
                        1.0 - (shadow_strength * (1.0 - parameters.rain_shadow_factor));
                    precipitation_multiplier[x][y] = shadow_reduction;
                }

                // 5. Moderate effects based on local moisture availability
                let available_moisture = atmospheric_moisture.surface_moisture.get_humidity(x, y);
                let moisture_factor = (available_moisture / 50.0).min(1.0); // Scale by typical humidity
                precipitation_multiplier[x][y] *= 0.5 + 0.5 * moisture_factor; // 50-100% based on moisture
                condensation_rate[x][y] *= moisture_factor;
            }
        }

        Self {
            precipitation_multiplier,
            vertical_velocity,
            condensation_rate,
            rain_shadow_intensity,
            lifting_height,
            width,
            height,
        }
    }

    /// Get precipitation enhancement multiplier at coordinates
    pub fn get_precipitation_multiplier(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.precipitation_multiplier[x][y]
        } else {
            1.0
        }
    }

    /// Get vertical air velocity at coordinates (m/s)
    pub fn get_vertical_velocity(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.vertical_velocity[x][y]
        } else {
            0.0
        }
    }

    /// Get condensation rate at coordinates (kg/m²/s)
    pub fn get_condensation_rate(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.condensation_rate[x][y]
        } else {
            0.0
        }
    }

    /// Get rain shadow intensity at coordinates (0.0-1.0)
    pub fn get_rain_shadow_intensity(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.rain_shadow_intensity[x][y]
        } else {
            0.0
        }
    }

    /// Apply orographic precipitation to atmospheric moisture system
    ///
    /// **Integration Process**: This demonstrates the cross-system coupling enabled
    /// by unified FlowEngine architecture. Atmospheric flow patterns directly drive
    /// precipitation patterns through terrain interactions.
    pub fn apply_to_moisture_system(
        &self,
        atmospheric_moisture: &mut AtmosphericMoistureSystem,
        dt: f32, // Time step in hours
    ) {
        for x in 0..self.width {
            for y in 0..self.height {
                let condensation = self.get_condensation_rate(x, y);
                let dt_seconds = dt * 3600.0; // Convert hours to seconds

                // Convert condensation rate to precipitation amount (m)
                let precipitation_amount = condensation * dt_seconds / 1000.0; // kg/m²/s → m

                if precipitation_amount > 0.001 {
                    // Minimum threshold (0.1mm)
                    atmospheric_moisture.add_precipitation(x, y, precipitation_amount);
                }
            }
        }
    }
}

/// Calculate terrain slopes in upwind and downwind directions
///
/// **Mathematical Approach**: Uses centered finite differences to calculate slopes
/// in the direction of wind flow. This gives the component of terrain gradient
/// that air masses actually encounter.
fn calculate_terrain_slopes(
    heightmap: &HeightMap,
    x: usize,
    y: usize,
    wind_velocity: &crate::engine::core::math::Vec2,
    cell_size_m: f32,
) -> (f32, f32) {
    // Normalize wind direction
    let wind_speed = wind_velocity.magnitude();
    if wind_speed < 1e-6 {
        return (0.0, 0.0);
    }

    let wind_dir_x = wind_velocity.x / wind_speed;
    let wind_dir_y = wind_velocity.y / wind_speed;

    // Calculate terrain gradients using centered differences
    let dh_dx = (heightmap.get(x + 1, y) - heightmap.get(x - 1, y)) / (2.0 * cell_size_m);
    let dh_dy = (heightmap.get(x, y + 1) - heightmap.get(x, y - 1)) / (2.0 * cell_size_m);

    // Project terrain gradient onto wind direction
    // Positive = upslope in wind direction, negative = downslope
    let slope_in_wind_direction = dh_dx * wind_dir_x + dh_dy * wind_dir_y;

    // For orographic effects:
    // - Upwind slope (positive): air is forced upward
    // - Downwind slope (negative): air descends (rain shadow)
    let slope_upwind = slope_in_wind_direction.max(0.0);
    let slope_downwind = slope_in_wind_direction.min(0.0);

    (slope_upwind, slope_downwind)
}

/// Orographic precipitation system integrating terrain-driven weather patterns
///
/// **System Integration**: This represents the fifth cross-system physics coupling
/// enabled by Phase 2 architecture consolidation. It connects:
/// - Terrain elevation (geological system)
/// - Atmospheric flow (climate system)
/// - Moisture transport (atmospheric moisture system)
/// - Precipitation patterns (hydrological system)
#[derive(Clone, Debug)]
pub struct OrographicPrecipitationSystem {
    /// Orographic precipitation parameters
    pub parameters: OrographicParameters,

    /// Current orographic effects state
    pub effects: Option<OrographicEffects>,
}

impl OrographicPrecipitationSystem {
    /// Create new orographic precipitation system
    pub fn new(parameters: OrographicParameters) -> Self {
        Self {
            parameters,
            effects: None,
        }
    }

    /// Create with default parameters
    pub fn default() -> Self {
        Self::new(OrographicParameters::default())
    }

    /// Update orographic precipitation effects from current atmospheric state
    ///
    /// **Cross-System Integration**: This method exemplifies the unified physics
    /// approach - using velocity fields from FlowEngine to drive precipitation
    /// patterns through terrain interactions.
    pub fn update(
        &mut self,
        heightmap: &HeightMap,
        flow_engine: &FlowEngine,
        atmospheric_moisture: &mut AtmosphericMoistureSystem,
        _climate: &ClimateSystem,
        scale: &WorldScale,
        dt: f32, // Time step in hours
    ) {
        // Calculate current orographic effects from terrain and atmospheric flow
        let new_effects = OrographicEffects::from_terrain_and_wind(
            heightmap,
            &flow_engine.velocity_field,
            atmospheric_moisture,
            &self.parameters,
            scale,
        );

        // Apply orographic precipitation to moisture system
        new_effects.apply_to_moisture_system(atmospheric_moisture, dt);

        // Store effects for analysis and rendering
        self.effects = Some(new_effects);
    }

    /// Get current orographic effects (for analysis and visualization)
    pub fn get_effects(&self) -> Option<&OrographicEffects> {
        self.effects.as_ref()
    }

    /// Get precipitation enhancement multiplier at coordinates
    pub fn get_precipitation_multiplier(&self, x: usize, y: usize) -> f32 {
        if let Some(effects) = &self.effects {
            effects.get_precipitation_multiplier(x, y)
        } else {
            1.0
        }
    }

    /// Check if there are active orographic effects
    pub fn has_active_effects(&self) -> bool {
        self.effects.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::core::scale::{DetailLevel, WorldScale};
    use crate::engine::physics::flow_engine::{FlowAlgorithm, FlowEngine};
    use crate::engine::physics::water::WaterLayer;

    #[test]
    fn orographic_effects_calculation() {
        // Create test terrain with mountain ridge
        let scale = WorldScale::new(10.0, (5, 5), DetailLevel::Standard);

        let heightmap = HeightMap::from_nested(vec![
            vec![0.1, 0.2, 0.3, 0.2, 0.1], // Valley-mountain-valley profile
            vec![0.2, 0.4, 0.6, 0.4, 0.2], // Higher elevations
            vec![0.3, 0.6, 1.0, 0.6, 0.3], // Mountain ridge
            vec![0.2, 0.4, 0.6, 0.4, 0.2], // Descending
            vec![0.1, 0.2, 0.3, 0.2, 0.1], // Back to valley
        ]);

        // Create atmospheric moisture system
        let mut atmospheric_moisture = AtmosphericMoistureSystem::new_for_scale(&scale, 5, 5);
        let water_layer = WaterLayer::new(5, 5);
        atmospheric_moisture.initialize_from_terrain(&heightmap, &water_layer);

        // Set up moisture in atmosphere
        for x in 0..5 {
            for y in 0..5 {
                atmospheric_moisture
                    .surface_moisture
                    .set_humidity(x, y, 40.0); // Moderate humidity
            }
        }

        // Create flow engine with westerly wind (left to right)
        let mut flow_engine = FlowEngine::new(FlowAlgorithm::Gradient, 5, 5, &scale);
        for x in 0..5 {
            for y in 0..5 {
                flow_engine.velocity_field.set_velocity(
                    x,
                    y,
                    crate::engine::core::math::Vec2::new(3.0, 0.0), // 3 m/s eastward wind
                );
            }
        }

        // Calculate orographic effects
        let parameters = OrographicParameters::default();
        let effects = OrographicEffects::from_terrain_and_wind(
            &heightmap,
            &flow_engine.velocity_field,
            &atmospheric_moisture,
            &parameters,
            &scale,
        );

        // Test windward enhancement
        // Slopes facing into the wind (west-facing) should have enhanced precipitation
        let windward_multiplier = effects.get_precipitation_multiplier(1, 2); // Western slope of mountain
        assert!(
            windward_multiplier > 1.0,
            "Windward slopes should have enhanced precipitation"
        );

        // Test mountain peak effects
        let peak_multiplier = effects.get_precipitation_multiplier(2, 2); // Mountain peak
        assert!(
            peak_multiplier >= 1.0,
            "Mountain peaks should have at least normal precipitation"
        );

        // Test leeward rain shadow
        let leeward_multiplier = effects.get_precipitation_multiplier(3, 2); // Eastern slope (leeward)
        assert!(
            leeward_multiplier <= 1.0,
            "Leeward slopes should have reduced precipitation (rain shadow)"
        );

        // Test vertical velocities
        let windward_vertical = effects.get_vertical_velocity(1, 2);
        assert!(
            windward_vertical > 0.0,
            "Windward slopes should have upward air motion"
        );

        // Test condensation rates
        let windward_condensation = effects.get_condensation_rate(1, 2);
        assert!(
            windward_condensation >= 0.0,
            "Condensation rates should be non-negative"
        );

        // Test dimensional consistency
        assert_eq!(effects.width, 5);
        assert_eq!(effects.height, 5);
    }

    #[test]
    fn orographic_system_integration() {
        let scale = WorldScale::new(10.0, (4, 4), DetailLevel::Standard);
        let parameters = OrographicParameters::default();
        let mut orographic_system = OrographicPrecipitationSystem::new(parameters);

        // Create simple mountain terrain
        let heightmap = HeightMap::from_nested(vec![
            vec![0.2, 0.4, 0.3, 0.1],
            vec![0.3, 0.8, 0.6, 0.2],
            vec![0.4, 0.9, 0.7, 0.3],
            vec![0.2, 0.5, 0.4, 0.1],
        ]);

        // Create atmospheric systems
        let mut atmospheric_moisture = AtmosphericMoistureSystem::new_for_scale(&scale, 4, 4);
        let water_layer = WaterLayer::new(4, 4);
        atmospheric_moisture.initialize_from_terrain(&heightmap, &water_layer);

        let climate_system = crate::engine::physics::climate::ClimateSystem::new_for_scale(&scale);

        // Set up wind flow
        let mut flow_engine = FlowEngine::new(FlowAlgorithm::Gradient, 4, 4, &scale);
        for x in 0..4 {
            for y in 0..4 {
                flow_engine.velocity_field.set_velocity(
                    x,
                    y,
                    crate::engine::core::math::Vec2::new(2.5, 0.0), // Moderate eastward wind
                );
            }
        }

        // Test that system starts without effects
        assert!(!orographic_system.has_active_effects());

        // Update system
        orographic_system.update(
            &heightmap,
            &flow_engine,
            &mut atmospheric_moisture,
            &climate_system,
            &scale,
            0.1, // 0.1 hour time step
        );

        // Should now have effects
        assert!(orographic_system.has_active_effects());

        // Test that effects are accessible
        let effects = orographic_system.get_effects().unwrap();
        assert_eq!(effects.width, 4);
        assert_eq!(effects.height, 4);

        // Test precipitation multipliers are reasonable
        for x in 0..4 {
            for y in 0..4 {
                let multiplier = orographic_system.get_precipitation_multiplier(x, y);
                assert!(
                    multiplier >= 0.0 && multiplier <= 10.0,
                    "Precipitation multiplier should be reasonable at ({}, {}): {}",
                    x,
                    y,
                    multiplier
                );
            }
        }
    }

    #[test]
    fn terrain_slope_calculation() {
        let heightmap = HeightMap::from_nested(vec![
            vec![0.0, 0.1, 0.2],
            vec![0.1, 0.2, 0.3], // Gentle eastward slope
            vec![0.2, 0.3, 0.4],
        ]);

        // Test eastward wind encountering eastward slope (uphill)
        let wind_east = crate::engine::core::math::Vec2::new(1.0, 0.0);
        let (slope_up, slope_down) = calculate_terrain_slopes(&heightmap, 1, 1, &wind_east, 1000.0);

        assert!(
            slope_up > 0.0,
            "Eastward wind on eastward slope should have positive upslope"
        );
        assert_eq!(
            slope_down, 0.0,
            "Upslope case should have zero downslope component"
        );

        // Test westward wind encountering eastward slope (downhill)
        let wind_west = crate::engine::core::math::Vec2::new(-1.0, 0.0);
        let (slope_up2, slope_down2) =
            calculate_terrain_slopes(&heightmap, 1, 1, &wind_west, 1000.0);

        assert_eq!(
            slope_up2, 0.0,
            "Downslope case should have zero upslope component"
        );
        assert!(
            slope_down2 < 0.0,
            "Westward wind on eastward slope should have negative downslope"
        );
    }
}
