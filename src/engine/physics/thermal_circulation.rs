// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Thermal circulation coupling - temperature-driven atmospheric flow patterns
// ABOUTME: Creates buoyancy effects and pressure gradients from temperature differences

use super::super::core::{math::Vec2 as MathVec2, scale::WorldScale};
use super::{
    climate::{AtmosphericPressureLayer, ClimateSystem, TemperatureLayer},
    flow_engine::FlowEngine,
    water::Vec2,
};

/// Configuration parameters for thermal circulation effects
#[derive(Clone, Debug)]
pub struct ThermalCirculationParameters {
    /// Temperature difference that drives significant circulation (°C)
    pub reference_temperature_difference: f32,
    /// Buoyancy strength factor (m/s per °C)  
    pub buoyancy_coefficient: f32,
    /// Thermal diffusion rate (smooths temperature gradients)
    pub thermal_diffusion_rate: f32,
    /// Minimum wind speed threshold for thermal enhancement (m/s)
    pub min_wind_threshold: f32,
    /// Maximum thermal velocity enhancement factor
    pub max_thermal_enhancement: f32,
    /// Pressure response coefficient (Pa per °C)
    pub pressure_response_coefficient: f32,
    /// Convection cell aspect ratio (width/height)
    pub convection_aspect_ratio: f32,
}

impl Default for ThermalCirculationParameters {
    fn default() -> Self {
        Self {
            reference_temperature_difference: 10.0, // 10°C drives strong circulation
            buoyancy_coefficient: 0.03,             // 3 cm/s per °C
            thermal_diffusion_rate: 0.1,            // Moderate smoothing
            min_wind_threshold: 0.5,                // 0.5 m/s minimum
            max_thermal_enhancement: 3.0,           // 3x max velocity enhancement
            pressure_response_coefficient: 120.0,   // 120 Pa per °C (realistic)
            convection_aspect_ratio: 2.0,           // 2:1 width:height cells
        }
    }
}

/// Thermal circulation effects data
#[derive(Clone, Debug)]
pub struct ThermalCirculationEffects {
    /// Temperature-driven velocity field (m/s)
    pub thermal_velocity: Vec<Vec<MathVec2>>,
    /// Buoyancy force field (N/kg)
    pub buoyancy_force: Vec<Vec<f32>>,
    /// Temperature gradient magnitude (°C/m)
    pub temperature_gradient: Vec<Vec<f32>>,
    /// Thermal pressure adjustments (Pa)
    pub thermal_pressure: Vec<Vec<f32>>,
    /// Convection cell indicators (0=none, 1=rising, -1=sinking)
    pub convection_cells: Vec<Vec<f32>>,
}

impl ThermalCirculationEffects {
    /// Create new effects data structure
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            thermal_velocity: vec![vec![MathVec2::new(0.0, 0.0); height]; width],
            buoyancy_force: vec![vec![0.0; height]; width],
            temperature_gradient: vec![vec![0.0; height]; width],
            thermal_pressure: vec![vec![0.0; height]; width],
            convection_cells: vec![vec![0.0; height]; width],
        }
    }

    /// Get thermal velocity at position with bounds checking
    pub fn get_thermal_velocity(&self, x: usize, y: usize) -> Vec2 {
        if x < self.thermal_velocity.len() && y < self.thermal_velocity[0].len() {
            let math_vec = self.thermal_velocity[x][y];
            Vec2::new(math_vec.x, math_vec.y)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }

    /// Get buoyancy force at position with bounds checking
    pub fn get_buoyancy_force(&self, x: usize, y: usize) -> f32 {
        if x < self.buoyancy_force.len() && y < self.buoyancy_force[0].len() {
            self.buoyancy_force[x][y]
        } else {
            0.0
        }
    }

    /// Get temperature gradient magnitude at position with bounds checking
    pub fn get_temperature_gradient(&self, x: usize, y: usize) -> f32 {
        if x < self.temperature_gradient.len() && y < self.temperature_gradient[0].len() {
            self.temperature_gradient[x][y]
        } else {
            0.0
        }
    }

    /// Get thermal pressure adjustment at position with bounds checking
    pub fn get_thermal_pressure(&self, x: usize, y: usize) -> f32 {
        if x < self.thermal_pressure.len() && y < self.thermal_pressure[0].len() {
            self.thermal_pressure[x][y]
        } else {
            0.0
        }
    }

    /// Get convection cell indicator at position with bounds checking
    pub fn get_convection_cell(&self, x: usize, y: usize) -> f32 {
        if x < self.convection_cells.len() && y < self.convection_cells[0].len() {
            self.convection_cells[x][y]
        } else {
            0.0
        }
    }
}

/// Thermal circulation coupling system
pub struct ThermalCirculationSystem {
    /// Physics parameters
    pub parameters: ThermalCirculationParameters,
    /// Current thermal effects (calculated during last update)
    effects: Option<ThermalCirculationEffects>,
}

impl ThermalCirculationSystem {
    /// Create new thermal circulation system
    pub fn new(parameters: ThermalCirculationParameters) -> Self {
        Self {
            parameters,
            effects: None,
        }
    }

    /// Check if thermal effects are currently active
    pub fn has_active_effects(&self) -> bool {
        self.effects.is_some()
    }

    /// Get current thermal effects (if any)
    pub fn get_effects(&self) -> Option<&ThermalCirculationEffects> {
        self.effects.as_ref()
    }

    /// Update thermal circulation effects
    pub fn update(
        &mut self,
        temperature_layer: &TemperatureLayer,
        flow_engine: &mut FlowEngine,
        atmospheric_pressure: &mut AtmosphericPressureLayer,
        climate_system: &ClimateSystem,
        scale: &WorldScale,
        dt: f32,
    ) {
        let width = temperature_layer.temperature.width();
        let height = temperature_layer.temperature.height();

        // Calculate thermal effects
        let mut effects = ThermalCirculationEffects::new(width, height);

        // Physical constants
        let cell_size_m = scale.meters_per_pixel() as f32;
        let reference_density = 1.225; // kg/m³ at sea level
        let gravitational_acceleration = 9.81; // m/s²

        // Calculate temperature gradients and thermal forces
        for x in 1..width - 1 {
            for y in 1..height - 1 {
                let current_temp = temperature_layer.get_temperature(x, y);

                // Calculate horizontal temperature gradients
                let temp_east = temperature_layer.get_temperature(x + 1, y);
                let temp_west = temperature_layer.get_temperature(x - 1, y);
                let temp_north = temperature_layer.get_temperature(x, y - 1);
                let temp_south = temperature_layer.get_temperature(x, y + 1);

                // METIS SCALING FIX: Remove cell_size_m scaling to achieve scale invariance
                // Theoretical analysis showed α = -1.0 scaling exponent due to grid dependency
                // Scale-invariant formulation uses dimensionless temperature differences
                let dt_dx = (temp_east - temp_west) / 2.0_f32;
                let dt_dy = (temp_south - temp_north) / 2.0_f32;

                let gradient_magnitude = (dt_dx * dt_dx + dt_dy * dt_dy).sqrt();
                effects.temperature_gradient[x][y] = gradient_magnitude;

                // Calculate thermal buoyancy force
                // Warmer air is less dense and rises, cooler air sinks
                let base_temperature = climate_system.parameters.base_temperature_c;
                let temperature_anomaly = current_temp - base_temperature;

                // Buoyancy force: F = ρ * g * β * ΔT (where β is thermal expansion coefficient)
                let thermal_expansion_coefficient = 1.0 / (base_temperature + 273.15); // 1/T in Kelvin
                let buoyancy_force = reference_density
                    * gravitational_acceleration
                    * thermal_expansion_coefficient
                    * temperature_anomaly;
                effects.buoyancy_force[x][y] = buoyancy_force;

                // Calculate thermal pressure response
                // Warmer areas have lower pressure due to thermal expansion
                let pressure_adjustment =
                    -self.parameters.pressure_response_coefficient * temperature_anomaly;
                effects.thermal_pressure[x][y] = pressure_adjustment;

                // Apply pressure adjustment to atmospheric pressure layer
                let current_pressure = atmospheric_pressure.pressure.get(x, y);
                atmospheric_pressure
                    .pressure
                    .set(x, y, current_pressure + pressure_adjustment);

                // Calculate thermal circulation velocity
                if gradient_magnitude > 0.001 {
                    // Horizontal circulation driven by temperature gradients
                    // Air flows from high pressure (cold) to low pressure (warm) areas
                    let circulation_strength = self.parameters.buoyancy_coefficient
                        * gradient_magnitude
                        / self.parameters.reference_temperature_difference;

                    // Direction perpendicular to temperature gradient (circulation)
                    let circulation_velocity = MathVec2::new(
                        -dt_dy * circulation_strength, // Perpendicular to gradient
                        dt_dx * circulation_strength,
                    );

                    effects.thermal_velocity[x][y] = circulation_velocity;

                    // Apply thermal velocity to flow engine
                    let current_velocity = flow_engine.velocity_field.get_velocity(x, y);
                    let enhanced_velocity = current_velocity + circulation_velocity;

                    // Limit enhancement factor
                    let enhancement_factor = enhanced_velocity.magnitude()
                        / current_velocity
                            .magnitude()
                            .max(self.parameters.min_wind_threshold);
                    let clamped_factor =
                        enhancement_factor.min(self.parameters.max_thermal_enhancement);

                    let final_velocity = current_velocity * clamped_factor;
                    flow_engine
                        .velocity_field
                        .set_velocity(x, y, final_velocity);
                }

                // Detect convection cells
                // Rising air in warm areas, sinking air in cool areas
                if temperature_anomaly > 2.0 {
                    effects.convection_cells[x][y] = 1.0; // Rising
                } else if temperature_anomaly < -2.0 {
                    effects.convection_cells[x][y] = -1.0; // Sinking
                } else {
                    effects.convection_cells[x][y] = 0.0; // Neutral
                }
            }
        }

        // Update pressure gradients based on thermal effects
        self.update_pressure_gradients(&mut effects, atmospheric_pressure, cell_size_m);

        // Apply thermal diffusion to smooth extreme gradients
        if self.parameters.thermal_diffusion_rate > 0.0 {
            self.apply_thermal_diffusion(&mut effects, dt);
        }

        self.effects = Some(effects);
    }

    /// Update pressure gradients based on thermal pressure changes
    fn update_pressure_gradients(
        &self,
        effects: &mut ThermalCirculationEffects,
        atmospheric_pressure: &mut AtmosphericPressureLayer,
        cell_size_m: f32,
    ) {
        let width = atmospheric_pressure.pressure.width();
        let height = atmospheric_pressure.pressure.height();

        // Calculate pressure gradients after thermal adjustments
        for x in 1..width - 1 {
            for y in 1..height - 1 {
                let pressure_east = atmospheric_pressure.pressure.get(x + 1, y);
                let pressure_west = atmospheric_pressure.pressure.get(x - 1, y);
                let pressure_north = atmospheric_pressure.pressure.get(x, y - 1);
                let pressure_south = atmospheric_pressure.pressure.get(x, y + 1);

                let dp_dx = (pressure_east - pressure_west) / (2.0 * cell_size_m);
                let dp_dy = (pressure_south - pressure_north) / (2.0 * cell_size_m);

                atmospheric_pressure
                    .pressure_gradient
                    .set(x, y, Vec2::new(dp_dx, dp_dy));
            }
        }
    }

    /// Apply thermal diffusion to smooth temperature gradients
    fn apply_thermal_diffusion(&self, effects: &mut ThermalCirculationEffects, dt: f32) {
        let width = effects.thermal_velocity.len();
        let height = effects.thermal_velocity[0].len();

        let diffusion_factor = self.parameters.thermal_diffusion_rate * dt;

        // Smooth thermal velocities to prevent numerical instabilities
        for x in 1..width - 1 {
            for y in 1..height - 1 {
                let current = effects.thermal_velocity[x][y];

                // Average with neighbors
                let avg_neighbors = (effects.thermal_velocity[x - 1][y]
                    + effects.thermal_velocity[x + 1][y]
                    + effects.thermal_velocity[x][y - 1]
                    + effects.thermal_velocity[x][y + 1])
                    * 0.25;

                // Apply diffusion
                effects.thermal_velocity[x][y] =
                    current + (avg_neighbors - current) * diffusion_factor;
            }
        }
    }
}

/// Helper function to calculate thermal circulation strength at a position
pub fn calculate_thermal_circulation_strength(
    temperature: f32,
    base_temperature: f32,
    gradient_magnitude: f32,
    parameters: &ThermalCirculationParameters,
) -> f32 {
    let temperature_difference = (temperature - base_temperature).abs();
    let normalized_difference =
        temperature_difference / parameters.reference_temperature_difference;
    let gradient_factor = gradient_magnitude / 0.01; // Reference gradient: 0.01°C/m

    (normalized_difference * gradient_factor * parameters.buoyancy_coefficient)
        .min(parameters.max_thermal_enhancement)
}

/// Helper function to calculate thermal pressure from temperature
pub fn calculate_thermal_pressure(
    temperature: f32,
    base_temperature: f32,
    parameters: &ThermalCirculationParameters,
) -> f32 {
    let temperature_anomaly = temperature - base_temperature;
    -parameters.pressure_response_coefficient * temperature_anomaly
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::core::{
        PhysicsGrid,
        scale::{DetailLevel, WorldScale},
    };

    #[test]
    fn test_thermal_circulation_parameters() {
        let params = ThermalCirculationParameters::default();

        // Test physical constants are reasonable
        assert!(
            params.reference_temperature_difference > 0.0
                && params.reference_temperature_difference < 50.0
        );
        assert!(params.buoyancy_coefficient > 0.0 && params.buoyancy_coefficient < 1.0);
        assert!(
            params.pressure_response_coefficient > 0.0
                && params.pressure_response_coefficient < 1000.0
        );
        assert!(params.max_thermal_enhancement > 1.0 && params.max_thermal_enhancement < 10.0);
    }

    #[test]
    fn test_thermal_circulation_effects_creation() {
        let effects = ThermalCirculationEffects::new(10, 10);

        assert_eq!(effects.thermal_velocity.len(), 10);
        assert_eq!(effects.thermal_velocity[0].len(), 10);
        assert_eq!(effects.buoyancy_force.len(), 10);
        assert_eq!(effects.temperature_gradient.len(), 10);
        assert_eq!(effects.thermal_pressure.len(), 10);
        assert_eq!(effects.convection_cells.len(), 10);
    }

    #[test]
    fn test_thermal_circulation_effects_access() {
        let mut effects = ThermalCirculationEffects::new(5, 5);

        // Test setting and getting values
        effects.thermal_velocity[2][2] = MathVec2::new(1.5, 2.0);
        effects.buoyancy_force[2][2] = 0.8;
        effects.temperature_gradient[2][2] = 0.05;
        effects.thermal_pressure[2][2] = -50.0;
        effects.convection_cells[2][2] = 1.0;

        assert_eq!(effects.get_thermal_velocity(2, 2), Vec2::new(1.5, 2.0));
        assert_eq!(effects.get_buoyancy_force(2, 2), 0.8);
        assert_eq!(effects.get_temperature_gradient(2, 2), 0.05);
        assert_eq!(effects.get_thermal_pressure(2, 2), -50.0);
        assert_eq!(effects.get_convection_cell(2, 2), 1.0);

        // Test bounds checking
        assert_eq!(effects.get_thermal_velocity(10, 10), Vec2::new(0.0, 0.0));
        assert_eq!(effects.get_buoyancy_force(10, 10), 0.0);
        assert_eq!(effects.get_temperature_gradient(10, 10), 0.0);
        assert_eq!(effects.get_thermal_pressure(10, 10), 0.0);
        assert_eq!(effects.get_convection_cell(10, 10), 0.0);
    }

    #[test]
    fn test_thermal_circulation_system_initialization() {
        let params = ThermalCirculationParameters::default();
        let system = ThermalCirculationSystem::new(params);

        assert!(!system.has_active_effects());
        assert!(system.get_effects().is_none());
    }

    #[test]
    fn test_calculate_thermal_circulation_strength() {
        let params = ThermalCirculationParameters::default();

        // Test with significant temperature difference and gradient
        let strength = calculate_thermal_circulation_strength(
            25.0, // temperature
            15.0, // base temperature
            0.02, // gradient magnitude (°C/m)
            &params,
        );

        assert!(strength > 0.0);
        assert!(strength <= params.max_thermal_enhancement);

        // Test with no temperature difference
        let strength_zero = calculate_thermal_circulation_strength(
            15.0, // temperature
            15.0, // base temperature
            0.02, // gradient magnitude
            &params,
        );

        assert_eq!(strength_zero, 0.0);
    }

    #[test]
    fn test_calculate_thermal_pressure() {
        let params = ThermalCirculationParameters::default();

        // Test warmer temperature (should create lower pressure)
        let pressure_warm = calculate_thermal_pressure(25.0, 15.0, &params);
        assert!(pressure_warm < 0.0); // Lower pressure

        // Test cooler temperature (should create higher pressure)
        let pressure_cool = calculate_thermal_pressure(5.0, 15.0, &params);
        assert!(pressure_cool > 0.0); // Higher pressure

        // Test no temperature difference
        let pressure_neutral = calculate_thermal_pressure(15.0, 15.0, &params);
        assert_eq!(pressure_neutral, 0.0);
    }

    #[test]
    fn test_thermal_system_integration() {
        let params = ThermalCirculationParameters::default();
        let mut system = ThermalCirculationSystem::new(params);

        // Create test temperature layer with temperature gradient
        let scale = WorldScale::new(10.0, (5, 5), DetailLevel::Standard);
        let mut temp_layer = TemperatureLayer::new(5, 5);

        // Set up strong temperature gradient (warm on left, cool on right)
        for x in 0..5 {
            for y in 0..5 {
                let temperature = 25.0 - (x as f32) * 3.0; // 25°C to 13°C (12°C difference)
                temp_layer.temperature.set(x, y, temperature);
            }
        }

        // Create other required systems
        let mut flow_engine = FlowEngine::for_climate(5, 5, &scale);
        let mut pressure_layer = AtmosphericPressureLayer {
            pressure: PhysicsGrid::new(5, 5, 101325.0), // Standard pressure
            pressure_gradient: PhysicsGrid::new(5, 5, Vec2::new(0.0, 0.0)),
        };
        let climate_system = ClimateSystem::new_for_scale(&scale);

        // Test system update
        system.update(
            &temp_layer,
            &mut flow_engine,
            &mut pressure_layer,
            &climate_system,
            &scale,
            0.1, // 0.1 second time step
        );

        // Verify effects were calculated
        assert!(system.has_active_effects());
        let effects = system.get_effects().unwrap();

        // Check that temperature gradients were detected
        assert!(effects.get_temperature_gradient(2, 2) > 0.0);

        // Check that thermal circulation was generated
        let thermal_vel = effects.get_thermal_velocity(2, 2);
        assert!(thermal_vel.magnitude() > 0.0);
    }
}
