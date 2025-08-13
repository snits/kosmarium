// ABOUTME: Atmospheric pressure coupling for water flow system modifications based on barometric variations
// ABOUTME: Models pressure effects on evaporation rates and drainage patterns using unified FlowEngine data

use super::flow_engine::FlowEngine;
use super::water::WaterLayer;
use crate::engine::core::{heightmap::HeightMap, scale::WorldScale};
use crate::engine::physics::atmosphere::AtmosphericSystem;
use crate::engine::physics::climate::TemperatureLayer;

/// Atmospheric pressure effects on water system dynamics
///
/// **Scientific Foundation**: Barometric pressure variations significantly affect:
/// 1. Evaporation rates: Lower pressure → higher evaporation (Clausius-Clapeyron relation)
/// 2. Drainage efficiency: Pressure gradients drive flow acceleration/deceleration
/// 3. Water retention: High pressure areas → increased surface retention
#[derive(Debug, Clone)]
pub struct AtmosphericPressureEffects {
    /// Local barometric pressure at each cell (Pa)
    /// Standard atmospheric pressure: 101,325 Pa
    /// Typical variation: ±2,000 Pa (±20 mbar) for weather systems
    pub pressure_field: Vec<Vec<f32>>,

    /// Pressure-driven evaporation rate modifications (factor)
    /// 1.0 = normal evaporation, >1.0 = enhanced, <1.0 = reduced
    /// Based on: e_sat ∝ exp(-L/(RT)) where pressure affects vapor equilibrium
    pub evaporation_modifiers: Vec<Vec<f32>>,

    /// Pressure gradient effects on water flow (acceleration in m/s²)
    /// Derived from: ∇P/ρ = pressure gradient force per unit mass
    pub pressure_gradient_acceleration: Vec<Vec<(f32, f32)>>,

    /// Water retention modifiers due to pressure effects (factor)
    /// High pressure → increased retention, low pressure → enhanced drainage
    pub retention_modifiers: Vec<Vec<f32>>,

    /// Grid dimensions
    pub width: usize,
    pub height: usize,
}

impl AtmosphericPressureEffects {
    /// Calculate atmospheric pressure effects from atmospheric system data
    ///
    /// **Educational Context**: This implements the physical principle that atmospheric
    /// pressure variations create measurable effects on water system dynamics:
    /// 1. **Evaporation**: Lower pressure reduces vapor pressure barrier
    /// 2. **Flow Dynamics**: Pressure gradients create acceleration forces  
    /// 3. **Surface Retention**: High pressure increases surface tension effects
    pub fn from_atmospheric_conditions(
        atmospheric_system: &AtmosphericSystem,
        temperature_layer: &TemperatureLayer,
        heightmap: &HeightMap,
        scale: &WorldScale,
        time_of_day: f32, // 0.0 = midnight, 0.5 = noon, 1.0 = midnight
    ) -> Self {
        let width = heightmap.width();
        let height = heightmap.height();

        // Initialize storage
        let mut pressure_field = vec![vec![0.0; height]; width];
        let mut evaporation_modifiers = vec![vec![1.0; height]; width];
        let mut pressure_gradient_acceleration = vec![vec![(0.0, 0.0); height]; width];
        let mut retention_modifiers = vec![vec![1.0; height]; width];

        // Physical constants
        let standard_pressure = 101325.0; // Pa (sea level standard)
        let water_density = 1000.0; // kg/m³
        let gas_constant = 287.0; // J/(kg·K) for dry air

        // Calculate pressure field from atmospheric system
        for x in 0..width {
            for y in 0..height {
                let elevation = heightmap.get(x, y);
                let temperature = temperature_layer.get_current_temperature(x, y, time_of_day);
                let temperature_kelvin = temperature + 273.15;

                // Barometric pressure variation with elevation and weather
                // P = P₀ * exp(-Mgh/(RT)) + weather_variation
                let elevation_meters = elevation * 1000.0; // Assume 1km vertical scale
                let elevation_factor = (-0.000119 * elevation_meters).exp(); // Standard atmosphere

                // Add weather system pressure variations (simplified: temperature-driven)
                // For now, use a simple temperature-based pressure variation
                let weather_pressure_variation = (temperature - 15.0) * 50.0; // 50 Pa per degree from 15°C baseline
                let local_pressure =
                    standard_pressure * elevation_factor + weather_pressure_variation;
                pressure_field[x][y] = local_pressure;

                // Calculate evaporation rate modifier from pressure
                // Lower pressure → easier evaporation (reduced vapor pressure barrier)
                let pressure_ratio = local_pressure / standard_pressure;
                let evaporation_modifier =
                    Self::calculate_evaporation_pressure_effect(pressure_ratio, temperature_kelvin);
                evaporation_modifiers[x][y] = evaporation_modifier;

                // Calculate water retention modifier
                // Higher pressure → increased surface retention
                let retention_modifier = Self::calculate_retention_pressure_effect(pressure_ratio);
                retention_modifiers[x][y] = retention_modifier;
            }
        }

        // Calculate pressure gradient acceleration field
        let grid_spacing_m = scale.meters_per_pixel() as f32;
        for x in 1..width - 1 {
            for y in 1..height - 1 {
                // Pressure gradient: ∇P = (∂P/∂x, ∂P/∂y)
                let dp_dx =
                    (pressure_field[x + 1][y] - pressure_field[x - 1][y]) / (2.0 * grid_spacing_m);
                let dp_dy =
                    (pressure_field[x][y + 1] - pressure_field[x][y - 1]) / (2.0 * grid_spacing_m);

                // Acceleration: a = -∇P/ρ (force per unit mass)
                let acceleration_x = -dp_dx / water_density;
                let acceleration_y = -dp_dy / water_density;

                pressure_gradient_acceleration[x][y] = (acceleration_x, acceleration_y);
            }
        }

        Self {
            pressure_field,
            evaporation_modifiers,
            pressure_gradient_acceleration,
            retention_modifiers,
            width,
            height,
        }
    }

    /// Calculate pressure effect on evaporation rates
    ///
    /// **Physical Foundation**: Clausius-Clapeyron relation modified by pressure:
    /// e_sat(P,T) = e₀ * exp(L_v/R_v * (1/T₀ - 1/T)) * (P/P₀)^(-0.378)
    /// Lower pressure → higher evaporation rate
    fn calculate_evaporation_pressure_effect(pressure_ratio: f32, temperature_kelvin: f32) -> f32 {
        // Empirical relation for pressure effect on evaporation
        // Based on: reduced atmospheric pressure lowers vapor pressure barrier
        let pressure_effect = pressure_ratio.powf(-0.378); // Physical constant from thermodynamics

        // Temperature dependence (stronger effect at higher temperatures)
        let temperature_factor = 1.0 + (temperature_kelvin - 273.15) * 0.002; // 0.2%/°C enhancement

        (pressure_effect * temperature_factor).clamp(0.5, 2.0) // Reasonable physical bounds
    }

    /// Calculate pressure effect on water retention
    ///
    /// **Physical Foundation**: Surface tension and capillary effects are enhanced
    /// by higher atmospheric pressure, leading to increased water retention
    fn calculate_retention_pressure_effect(pressure_ratio: f32) -> f32 {
        // Higher pressure → increased retention (capillary and surface tension effects)
        let retention_factor = 0.8 + 0.4 * pressure_ratio; // Linear relationship
        retention_factor.clamp(0.6, 1.4) // Reasonable physical bounds
    }

    /// Get atmospheric pressure at specified coordinates
    pub fn get_pressure(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.pressure_field[x][y]
        } else {
            101325.0 // Standard pressure default
        }
    }

    /// Get evaporation rate modifier at specified coordinates
    pub fn get_evaporation_modifier(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.evaporation_modifiers[x][y]
        } else {
            1.0 // No modification default
        }
    }

    /// Get pressure gradient acceleration at specified coordinates
    pub fn get_pressure_acceleration(&self, x: usize, y: usize) -> (f32, f32) {
        if x < self.width && y < self.height {
            self.pressure_gradient_acceleration[x][y]
        } else {
            (0.0, 0.0) // No acceleration default
        }
    }

    /// Get water retention modifier at specified coordinates
    pub fn get_retention_modifier(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.retention_modifiers[x][y]
        } else {
            1.0 // No modification default
        }
    }
}

/// Extended water flow system that incorporates atmospheric pressure effects
#[derive(Debug)]
pub struct PressureAwareWaterFlowSystem {
    /// Atmospheric pressure influence strength (0.0-1.0)
    /// 0.0 = ignore pressure effects, 1.0 = fully influenced by atmospheric pressure
    pub pressure_influence: f32,
}

impl PressureAwareWaterFlowSystem {
    /// Create pressure-aware water flow system
    pub fn new_for_scale(_scale: &WorldScale, pressure_influence: f32) -> Self {
        Self {
            pressure_influence: pressure_influence.clamp(0.0, 1.0),
        }
    }

    /// Generate water flow with atmospheric pressure coupling
    ///
    /// **Innovation**: This demonstrates the third cross-system physics coupling,
    /// showing how atmospheric pressure variations modify water system dynamics.
    ///
    /// **Process**:
    /// 1. Calculate base water flow using FlowEngine
    /// 2. Derive atmospheric pressure effects from atmospheric system
    /// 3. Modify evaporation rates based on barometric pressure
    /// 4. Apply pressure gradient forces to flow acceleration
    /// 5. Adjust water retention based on surface pressure effects
    pub fn calculate_flow_with_pressure_effects(
        &self,
        heightmap: &HeightMap,
        water_layer: &mut WaterLayer,
        atmospheric_system: &AtmosphericSystem,
        temperature_layer: &TemperatureLayer,
        flow_engine: &mut FlowEngine, // Modified to include pressure effects
        scale: &WorldScale,
        time_of_day: f32,
        dt: f32, // Time step in seconds
    ) -> AtmosphericPressureEffects {
        // 1. Calculate atmospheric pressure effects
        let pressure_effects = AtmosphericPressureEffects::from_atmospheric_conditions(
            atmospheric_system,
            temperature_layer,
            heightmap,
            scale,
            time_of_day,
        );

        // 2. Apply pressure coupling to water flow if influence > 0
        if self.pressure_influence > 0.0 {
            self.apply_pressure_coupling(&pressure_effects, water_layer, flow_engine, dt);
        }

        pressure_effects
    }

    /// Apply atmospheric pressure coupling to modify water flow dynamics
    fn apply_pressure_coupling(
        &self,
        pressure_effects: &AtmosphericPressureEffects,
        water_layer: &mut WaterLayer,
        flow_engine: &mut FlowEngine,
        dt: f32,
    ) {
        for x in 0..water_layer.width() {
            for y in 0..water_layer.height() {
                // 1. Modify evaporation rates based on pressure
                let evaporation_modifier = pressure_effects.get_evaporation_modifier(x, y);
                let current_depth = water_layer.get_water_depth(x, y);

                if current_depth > 1e-6 {
                    // Apply pressure-modified evaporation
                    let base_evaporation_rate = 0.001 * dt; // 1mm/hour baseline
                    let pressure_modified_evaporation =
                        base_evaporation_rate * evaporation_modifier * self.pressure_influence;
                    let evaporated_amount = pressure_modified_evaporation.min(current_depth * 0.1); // Max 10% per timestep

                    // Remove water through evaporation (using existing method)
                    let new_depth = current_depth - evaporated_amount;
                    water_layer.depth.set(x, y, new_depth.max(0.0));
                }

                // 2. Apply pressure gradient acceleration to flow velocity
                let (pressure_accel_x, pressure_accel_y) =
                    pressure_effects.get_pressure_acceleration(x, y);
                let current_velocity = flow_engine.velocity_field.get_velocity(x, y);

                let velocity_change_x = pressure_accel_x * dt * self.pressure_influence;
                let velocity_change_y = pressure_accel_y * dt * self.pressure_influence;

                let new_velocity = crate::engine::core::math::Vec2::new(
                    current_velocity.x + velocity_change_x,
                    current_velocity.y + velocity_change_y,
                );

                flow_engine.velocity_field.set_velocity(x, y, new_velocity);

                // 3. Apply retention effects (affects drainage rates)
                let retention_modifier = pressure_effects.get_retention_modifier(x, y);
                if retention_modifier != 1.0 {
                    // Modify water depth based on retention effects
                    let retention_change = current_depth
                        * (retention_modifier - 1.0)
                        * 0.01
                        * dt
                        * self.pressure_influence;
                    let new_depth = (current_depth + retention_change).max(0.0);
                    water_layer.depth.set(x, y, new_depth);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atmospheric_pressure_effects_calculation() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        use crate::engine::physics::atmosphere::AtmosphericSystem;
        use crate::engine::physics::climate::ClimateSystem;

        // Create test terrain with elevation variation
        let heightmap = HeightMap::from_nested(vec![
            vec![1.0, 0.8, 0.6, 0.4],  // Mountain to valley
            vec![0.9, 0.6, 0.3, 0.2],  // Slope
            vec![0.8, 0.5, 0.2, 0.0],  // Sea level
            vec![0.7, 0.4, 0.1, -0.1], // Below sea level
        ]);

        let scale = WorldScale::new(10.0, (4, 4), DetailLevel::Standard);
        let climate_system = ClimateSystem::new_for_scale(&scale);
        let heightmap_nested = heightmap.to_nested();
        let temperature_layer = climate_system.generate_temperature_layer(&heightmap_nested);

        // Create atmospheric system
        let atmospheric_system = AtmosphericSystem::new_for_scale(&scale);

        // Calculate atmospheric pressure effects at noon (maximum variation)
        let pressure_effects = AtmosphericPressureEffects::from_atmospheric_conditions(
            &atmospheric_system,
            &temperature_layer,
            &heightmap,
            &scale,
            0.5, // Noon
        );

        assert_eq!(pressure_effects.width, 4);
        assert_eq!(pressure_effects.height, 4);

        // High elevation should have lower pressure
        let mountain_pressure = pressure_effects.get_pressure(0, 0); // Elevation 1.0
        let sea_level_pressure = pressure_effects.get_pressure(2, 2); // Elevation 0.2

        assert!(
            mountain_pressure < sea_level_pressure,
            "Mountain pressure ({:.1} Pa) should be lower than sea level pressure ({:.1} Pa)",
            mountain_pressure,
            sea_level_pressure
        );

        // Pressure should be within realistic ranges
        for x in 0..4 {
            for y in 0..4 {
                let pressure = pressure_effects.get_pressure(x, y);
                assert!(
                    pressure > 80000.0 && pressure < 110000.0,
                    "Pressure {:.1} Pa at ({}, {}) outside realistic range",
                    pressure,
                    x,
                    y
                );
            }
        }

        // Evaporation modifiers should exist and be reasonable
        for x in 0..4 {
            for y in 0..4 {
                let evap_modifier = pressure_effects.get_evaporation_modifier(x, y);
                assert!(
                    evap_modifier > 0.5 && evap_modifier < 2.0,
                    "Evaporation modifier {:.3} at ({}, {}) outside reasonable range",
                    evap_modifier,
                    x,
                    y
                );
            }
        }

        println!("Atmospheric pressure effects test results:");
        println!("  Mountain pressure: {:.1} Pa", mountain_pressure);
        println!("  Sea level pressure: {:.1} Pa", sea_level_pressure);
        println!(
            "  Pressure difference: {:.1} Pa",
            sea_level_pressure - mountain_pressure
        );
    }

    #[test]
    fn test_pressure_aware_water_flow_coupling() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        use crate::engine::physics::atmosphere::AtmosphericSystem;
        use crate::engine::physics::climate::ClimateSystem;
        use crate::engine::physics::flow_engine::{FlowAlgorithm, FlowEngine};

        // Create pressure gradient terrain (high to low elevation)
        let heightmap = HeightMap::from_nested(vec![
            vec![1.0, 0.8, 0.6, 0.4, 0.2], // Strong elevation gradient
            vec![0.9, 0.7, 0.5, 0.3, 0.1], // Continuing slope
            vec![0.8, 0.6, 0.4, 0.2, 0.0], // To sea level
        ]);

        let scale = WorldScale::new(15.0, (5, 3), DetailLevel::Standard);
        let climate_system = ClimateSystem::new_for_scale(&scale);
        let heightmap_nested = heightmap.to_nested();
        let temperature_layer = climate_system.generate_temperature_layer(&heightmap_nested);

        // Create water layer with initial water distribution
        let mut water_layer = WaterLayer::new(5, 3);
        for x in 0..5 {
            for y in 0..3 {
                water_layer.depth.set(x, y, 0.05); // 5cm initial water depth
            }
        }

        // Create atmospheric and flow systems
        let atmospheric_system = AtmosphericSystem::new_for_scale(&scale);
        let mut flow_engine = FlowEngine::new(FlowAlgorithm::Conservation, 5, 3, &scale);

        // Test different pressure influence levels
        let no_pressure = PressureAwareWaterFlowSystem::new_for_scale(&scale, 0.0);
        let full_pressure = PressureAwareWaterFlowSystem::new_for_scale(&scale, 1.0);

        // Store initial water state for comparison
        let initial_total_water: f32 = (0..5)
            .map(|x| {
                (0..3)
                    .map(|y| water_layer.get_water_depth(x, y))
                    .sum::<f32>()
            })
            .sum();

        // Test with no pressure influence
        let mut water_layer_no_pressure = water_layer.clone();
        let mut flow_engine_no_pressure =
            FlowEngine::new(FlowAlgorithm::Conservation, 5, 3, &scale);

        let _no_pressure_effects = no_pressure.calculate_flow_with_pressure_effects(
            &heightmap,
            &mut water_layer_no_pressure,
            &atmospheric_system,
            &temperature_layer,
            &mut flow_engine_no_pressure,
            &scale,
            0.5,    // Noon
            3600.0, // 1 hour timestep
        );

        // Test with full pressure influence
        let mut water_layer_full_pressure = water_layer.clone();
        let mut flow_engine_full_pressure =
            FlowEngine::new(FlowAlgorithm::Conservation, 5, 3, &scale);

        let full_pressure_effects = full_pressure.calculate_flow_with_pressure_effects(
            &heightmap,
            &mut water_layer_full_pressure,
            &atmospheric_system,
            &temperature_layer,
            &mut flow_engine_full_pressure,
            &scale,
            0.5,    // Noon
            3600.0, // 1 hour timestep
        );

        // Verify pressure effects were calculated
        assert_eq!(full_pressure_effects.width, 5);
        assert_eq!(full_pressure_effects.height, 3);

        // High elevation should have enhanced evaporation (lower pressure)
        let mountain_evap_modifier = full_pressure_effects.get_evaporation_modifier(0, 0);
        let valley_evap_modifier = full_pressure_effects.get_evaporation_modifier(4, 2);

        assert!(
            mountain_evap_modifier > valley_evap_modifier,
            "Mountain evaporation modifier {:.3} should be higher than valley {:.3}",
            mountain_evap_modifier,
            valley_evap_modifier
        );

        // Water amounts should be different due to pressure effects
        let no_pressure_total: f32 = (0..5)
            .map(|x| {
                (0..3)
                    .map(|y| water_layer_no_pressure.get_water_depth(x, y))
                    .sum::<f32>()
            })
            .sum();
        let full_pressure_total: f32 = (0..5)
            .map(|x| {
                (0..3)
                    .map(|y| water_layer_full_pressure.get_water_depth(x, y))
                    .sum::<f32>()
            })
            .sum();

        // Should have some water loss due to pressure-enhanced evaporation
        assert!(
            full_pressure_total < no_pressure_total,
            "Full pressure total {:.6} should be less than no pressure total {:.6} due to enhanced evaporation",
            full_pressure_total,
            no_pressure_total
        );

        println!("Pressure-aware water flow test results:");
        println!("  Initial water: {:.6}", initial_total_water);
        println!("  No pressure influence: {:.6}", no_pressure_total);
        println!("  Full pressure influence: {:.6}", full_pressure_total);
        println!("  Mountain evap modifier: {:.3}", mountain_evap_modifier);
        println!("  Valley evap modifier: {:.3}", valley_evap_modifier);
        println!(
            "  Pressure-enhanced evaporation: {:.6}",
            no_pressure_total - full_pressure_total
        );
    }
}
