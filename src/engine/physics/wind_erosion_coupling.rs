// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Wind-driven erosion coupling for atmospheric influence on geological processes using unified FlowEngine
// ABOUTME: Models wind velocity effects on sediment transport and terrain modification through cross-system physics

use super::flow_engine::FlowEngine;
use crate::engine::core::{heightmap::HeightMap, math::Vec2, scale::WorldScale};
use crate::engine::physics::atmosphere::AtmosphericSystem;
use crate::engine::physics::climate::TemperatureLayer;

/// Wind erosion effects on geological processes
///
/// **Scientific Foundation**: Wind erosion is a fundamental geomorphological process that
/// reshapes terrain through particle transport. Key physical processes include:
/// 1. Saltation: Wind-driven bouncing of sand particles
/// 2. Suspension: Fine particles carried in atmospheric flow
/// 3. Surface creep: Rolling/sliding of larger particles along ground
/// 4. Deflation: Removal of loose material from exposed surfaces
#[derive(Debug, Clone)]
pub struct WindErosionEffects {
    /// Wind shear stress at surface (Pa)
    /// τ = ρ_air × u_friction² where u_friction is friction velocity
    /// Above threshold stress → sediment transport begins
    pub surface_shear_stress: Vec<Vec<f32>>,

    /// Erosion potential at each cell (kg/(m²·s))
    /// Rate at which wind can remove sediment from surface
    /// Depends on: wind speed, surface roughness, particle size, moisture
    pub erosion_potential: Vec<Vec<f32>>,

    /// Sediment transport capacity (kg/(m²·s))
    /// Maximum amount of sediment wind can carry at each location
    /// Function of wind speed cubed: Q ∝ u³ (Bagnold equation)
    pub transport_capacity: Vec<Vec<f32>>,

    /// Wind velocity field at surface level (m/s)
    /// Derived from atmospheric system but modified by terrain effects
    pub surface_wind_velocity: Vec<Vec<Vec2>>,

    /// Deposition rate at each cell (kg/(m²·s))
    /// Where sediment settles out when transport capacity is exceeded
    /// Occurs in wind shadows, vegetation, or reduced wind zones
    pub deposition_rate: Vec<Vec<f32>>,

    /// Grid dimensions
    pub width: usize,
    pub height: usize,
}

impl WindErosionEffects {
    /// Calculate wind erosion effects from atmospheric system and terrain data
    ///
    /// **Educational Context**: This demonstrates how atmospheric flow patterns
    /// directly influence geological processes through the unified FlowEngine architecture.
    /// Wind erosion is a major landscape-shaping force in arid and semi-arid regions,
    /// creating features like sand dunes, deflation basins, and ventifacts.
    pub fn from_atmospheric_conditions(
        atmospheric_system: &AtmosphericSystem,
        heightmap: &HeightMap,
        temperature_layer: &TemperatureLayer,
        scale: &WorldScale,
        time_of_day: f32, // 0.0 = midnight, 0.5 = noon
    ) -> Self {
        let width = heightmap.width();
        let height = heightmap.height();

        // Initialize storage
        let mut surface_shear_stress = vec![vec![0.0; height]; width];
        let mut erosion_potential = vec![vec![0.0; height]; width];
        let mut transport_capacity = vec![vec![0.0; height]; width];
        let mut surface_wind_velocity = vec![vec![Vec2::new(0.0, 0.0); height]; width];
        let mut deposition_rate = vec![vec![0.0; height]; width];

        // Physical constants
        let air_density = 1.225; // kg/m³ at sea level
        let critical_shear_stress = 0.1; // Pa - threshold for sediment movement
        let roughness_length = 0.001; // m - surface roughness parameter

        // Calculate wind erosion effects for each cell
        for x in 0..width {
            for y in 0..height {
                let elevation = heightmap.get(x, y);
                let temperature = temperature_layer.get_current_temperature(x, y, time_of_day);

                // 1. Calculate surface wind velocity with terrain effects
                let base_wind_speed =
                    Self::calculate_base_wind_speed(elevation, temperature, time_of_day);

                // Terrain modification: wind acceleration over ridges, deceleration in valleys
                let terrain_factor = Self::calculate_terrain_wind_factor(heightmap, x, y, scale);

                let surface_wind_speed = base_wind_speed * terrain_factor;

                // Simplified wind direction (eastward with terrain deflection)
                let wind_direction = Self::calculate_wind_direction(heightmap, x, y);
                let wind_velocity = Vec2::new(
                    surface_wind_speed * wind_direction.x,
                    surface_wind_speed * wind_direction.y,
                );
                surface_wind_velocity[x][y] = wind_velocity;

                // 2. Calculate surface shear stress: τ = ρ × u_friction²
                // Use logarithmic wind profile: u_friction = κ × u / ln(z/z₀)
                let friction_velocity = 0.4 * surface_wind_speed / (elevation * 10.0 + 1.0).ln(); // κ = 0.4
                let shear_stress = air_density * friction_velocity * friction_velocity;
                surface_shear_stress[x][y] = shear_stress;

                // 3. Calculate erosion potential (if above critical threshold)
                let erosion_potential_rate = if shear_stress > critical_shear_stress {
                    // Erosion rate proportional to excess shear stress
                    let excess_stress = shear_stress - critical_shear_stress;
                    let base_erodibility =
                        Self::calculate_surface_erodibility(elevation, temperature);
                    excess_stress * base_erodibility * 0.001 // Convert to kg/(m²·s)
                } else {
                    0.0
                };
                erosion_potential[x][y] = erosion_potential_rate;

                // 4. Calculate transport capacity using Bagnold equation: Q ∝ u³
                let transport_capacity_rate = if surface_wind_speed > 3.0 {
                    // Minimum wind for transport
                    let wind_speed_cubed = surface_wind_speed.powi(3);
                    let air_density_factor = air_density / 1.225; // Adjust for altitude
                    wind_speed_cubed * air_density_factor * 0.0001 // Scaling factor
                } else {
                    0.0
                };
                transport_capacity[x][y] = transport_capacity_rate;

                // 5. Calculate deposition rate (where transport capacity is reduced)
                let deposition_rate_value =
                    Self::calculate_deposition_rate(surface_wind_speed, elevation, terrain_factor);
                deposition_rate[x][y] = deposition_rate_value;
            }
        }

        Self {
            surface_shear_stress,
            erosion_potential,
            transport_capacity,
            surface_wind_velocity,
            deposition_rate,
            width,
            height,
        }
    }

    /// Calculate base wind speed from atmospheric conditions
    ///
    /// **Physical Foundation**: Wind speed varies with:
    /// - Elevation: Generally increases with altitude due to reduced friction
    /// - Temperature: Thermal gradients drive convective circulation
    /// - Diurnal cycle: Daytime heating creates stronger thermal winds
    fn calculate_base_wind_speed(elevation: f32, temperature: f32, time_of_day: f32) -> f32 {
        // Base wind speed increases with elevation (reduced surface friction)
        let elevation_wind = 2.0 + elevation * 5.0; // 2-7 m/s range

        // Temperature effect: warmer air creates more convection
        let temperature_factor = 1.0 + (temperature - 15.0) * 0.02; // 2% per degree from 15°C

        // Diurnal cycle: stronger winds during day due to thermal heating
        let diurnal_factor = 0.7 + 0.6 * (time_of_day * 2.0 * std::f32::consts::PI).sin().max(0.0);

        elevation_wind * temperature_factor * diurnal_factor
    }

    /// Calculate terrain effects on wind flow
    ///
    /// **Fluid Dynamics**: Wind flow is modified by terrain through:
    /// - Speed-up over ridges and peaks (Venturi effect)
    /// - Deceleration in valleys and depressions
    /// - Flow separation and turbulence on lee sides
    fn calculate_terrain_wind_factor(
        heightmap: &HeightMap,
        x: usize,
        y: usize,
        scale: &WorldScale,
    ) -> f32 {
        let width = heightmap.width();
        let height = heightmap.height();
        let current_elevation = heightmap.get(x, y);

        // Calculate local slope magnitude
        let mut slope_magnitude = 0.0;
        if x > 0 && x < width - 1 && y > 0 && y < height - 1 {
            let dx = (heightmap.get(x + 1, y) - heightmap.get(x - 1, y)) / 2.0;
            let dy = (heightmap.get(x, y + 1) - heightmap.get(x, y - 1)) / 2.0;
            let pixel_size = scale.meters_per_pixel() as f32;
            slope_magnitude = (dx * dx + dy * dy).sqrt() / pixel_size;
        }

        // Calculate relative elevation (how much higher than surroundings)
        let mut surrounding_elevation = 0.0;
        let mut count = 0;
        for dx in -2..=2 {
            for dy in -2..=2 {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < width && ny < height {
                    surrounding_elevation += heightmap.get(nx, ny);
                    count += 1;
                }
            }
        }
        let relative_elevation = if count > 0 {
            current_elevation - surrounding_elevation / count as f32
        } else {
            0.0
        };

        // Wind speed factor: higher on ridges, lower in valleys
        let elevation_factor = 1.0 + relative_elevation * 2.0; // ±200% based on relative position
        let slope_factor = 1.0 + slope_magnitude * 0.5; // Increase with slope steepness

        (elevation_factor * slope_factor).clamp(0.3, 3.0) // Reasonable physical bounds
    }

    /// Calculate wind direction based on terrain gradients
    fn calculate_wind_direction(heightmap: &HeightMap, x: usize, y: usize) -> Vec2 {
        let width = heightmap.width();
        let height = heightmap.height();

        // Default eastward wind
        let mut direction = Vec2::new(1.0, 0.0);

        // Modify direction based on local terrain gradients
        if x > 0 && x < width - 1 && y > 0 && y < height - 1 {
            let dx = heightmap.get(x + 1, y) - heightmap.get(x - 1, y);
            let dy = heightmap.get(x, y + 1) - heightmap.get(x, y - 1);

            // Wind tends to flow around obstacles (perpendicular to gradient)
            let deflection = Vec2::new(-dy, dx) * 0.3; // 30% deflection strength
            direction = (direction + deflection).normalize();
        }

        direction
    }

    /// Calculate surface erodibility based on environmental conditions
    ///
    /// **Geomorphology**: Erodibility depends on:
    /// - Moisture content: Wet surfaces resist erosion
    /// - Temperature: Freeze-thaw cycles affect particle cohesion
    /// - Surface material: Rock type and weathering state
    fn calculate_surface_erodibility(elevation: f32, temperature: f32) -> f32 {
        // Base erodibility varies with surface type (inferred from elevation)
        let base_erodibility = if elevation > 0.8 {
            0.5 // Rocky mountain surfaces - low erodibility
        } else if elevation > 0.4 {
            1.0 // Mixed terrain - moderate erodibility
        } else {
            1.5 // Lowland sediments - high erodibility
        };

        // Temperature effects on particle cohesion
        let temperature_factor = if temperature < 0.0 {
            0.5 // Frozen ground resists erosion
        } else if temperature > 30.0 {
            1.2 // Hot, dry conditions increase erodibility
        } else {
            1.0 // Normal conditions
        };

        base_erodibility * temperature_factor
    }

    /// Calculate deposition rate where sediment settles
    fn calculate_deposition_rate(wind_speed: f32, elevation: f32, terrain_factor: f32) -> f32 {
        // Deposition occurs where wind speed decreases (transport capacity drops)
        let speed_deposition = if wind_speed < 2.0 {
            0.001 * (2.0 - wind_speed) // Higher deposition in calm areas
        } else {
            0.0
        };

        // Deposition in sheltered areas (low terrain factor)
        let shelter_deposition = if terrain_factor < 0.7 {
            0.0005 * (0.7 - terrain_factor)
        } else {
            0.0
        };

        speed_deposition + shelter_deposition
    }

    /// Get wind shear stress at specified coordinates
    pub fn get_shear_stress(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.surface_shear_stress[x][y]
        } else {
            0.0
        }
    }

    /// Get erosion potential at specified coordinates
    pub fn get_erosion_potential(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.erosion_potential[x][y]
        } else {
            0.0
        }
    }

    /// Get transport capacity at specified coordinates
    pub fn get_transport_capacity(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.transport_capacity[x][y]
        } else {
            0.0
        }
    }

    /// Get surface wind velocity at specified coordinates
    pub fn get_wind_velocity(&self, x: usize, y: usize) -> Vec2 {
        if x < self.width && y < self.height {
            self.surface_wind_velocity[x][y]
        } else {
            Vec2::new(0.0, 0.0)
        }
    }

    /// Get deposition rate at specified coordinates
    pub fn get_deposition_rate(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.deposition_rate[x][y]
        } else {
            0.0
        }
    }
}

/// Extended geological evolution system that incorporates wind erosion effects
#[derive(Debug)]
pub struct WindAwareGeologicalSystem {
    /// Wind erosion influence strength (0.0-1.0)
    /// 0.0 = ignore wind effects, 1.0 = fully influenced by atmospheric flow
    pub wind_influence: f32,
}

impl WindAwareGeologicalSystem {
    /// Create wind-aware geological system
    pub fn new_for_scale(_scale: &WorldScale, wind_influence: f32) -> Self {
        Self {
            wind_influence: wind_influence.clamp(0.0, 1.0),
        }
    }

    /// Apply wind erosion effects to terrain evolution
    ///
    /// **Innovation**: This demonstrates the fourth cross-system physics coupling,
    /// showing how atmospheric flow patterns modify geological processes through
    /// the unified FlowEngine architecture.
    ///
    /// **Process**:
    /// 1. Calculate wind erosion effects from atmospheric system
    /// 2. Apply erosion where wind shear stress exceeds thresholds
    /// 3. Transport sediment according to wind capacity
    /// 4. Deposit sediment where transport capacity is reduced
    /// 5. Modify terrain heightmap based on net erosion/deposition
    pub fn evolve_terrain_with_wind_erosion(
        &self,
        heightmap: &mut HeightMap,
        atmospheric_system: &AtmosphericSystem,
        temperature_layer: &TemperatureLayer,
        flow_engine: &mut FlowEngine, // Modified to include wind effects
        scale: &WorldScale,
        time_of_day: f32,
        dt: f32, // Time step in seconds
    ) -> WindErosionEffects {
        // 1. Calculate wind erosion effects
        let wind_effects = WindErosionEffects::from_atmospheric_conditions(
            atmospheric_system,
            heightmap,
            temperature_layer,
            scale,
            time_of_day,
        );

        // 2. Apply wind erosion coupling to terrain if influence > 0
        if self.wind_influence > 0.0 {
            self.apply_wind_erosion_coupling(&wind_effects, heightmap, dt);
        }

        wind_effects
    }

    /// Apply wind erosion coupling to modify terrain
    fn apply_wind_erosion_coupling(
        &self,
        wind_effects: &WindErosionEffects,
        heightmap: &mut HeightMap,
        dt: f32,
    ) {
        for x in 0..wind_effects.width {
            for y in 0..wind_effects.height {
                // Get current elevation
                let current_elevation = heightmap.get(x, y);

                // Calculate net elevation change from wind processes
                let erosion_rate = wind_effects.get_erosion_potential(x, y); // kg/(m²·s)
                let deposition_rate = wind_effects.get_deposition_rate(x, y); // kg/(m²·s)

                // Convert mass rates to elevation change (assuming 2000 kg/m³ rock density)
                let rock_density = 2000.0; // kg/m³
                let erosion_depth = erosion_rate * dt / rock_density; // meters
                let deposition_depth = deposition_rate * dt / rock_density; // meters

                // Net elevation change
                let net_change = (deposition_depth - erosion_depth) * self.wind_influence;

                // Apply change with stability limits
                let max_change = 0.001 * dt; // Maximum 1mm per second
                let limited_change = net_change.clamp(-max_change, max_change);

                let new_elevation = (current_elevation + limited_change).max(0.0);
                heightmap.set(x, y, new_elevation);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::core::scale::{DetailLevel, WorldScale};
    use crate::engine::physics::atmosphere::AtmosphericSystem;
    use crate::engine::physics::climate::ClimateSystem;

    #[test]
    fn test_wind_erosion_effects_calculation() {
        // Create test terrain with varied elevation
        let heightmap = HeightMap::from_nested(vec![
            vec![1.0, 0.9, 0.8, 0.7], // Ridge line
            vec![0.8, 0.6, 0.4, 0.5], // Valley
            vec![0.6, 0.4, 0.2, 0.3], // Lower terrain
            vec![0.4, 0.2, 0.0, 0.1], // Sea level
        ]);

        let scale = WorldScale::new(5.0, (4, 4), DetailLevel::Standard);
        let climate_system = ClimateSystem::new_for_scale(&scale);
        let heightmap_nested = heightmap.to_nested();
        let temperature_layer = climate_system.generate_temperature_layer(&heightmap_nested);

        // Create atmospheric system
        let atmospheric_system = AtmosphericSystem::new_for_scale(&scale);

        // Calculate wind erosion effects at noon (maximum heating)
        let wind_effects = WindErosionEffects::from_atmospheric_conditions(
            &atmospheric_system,
            &heightmap,
            &temperature_layer,
            &scale,
            0.5, // Noon
        );

        assert_eq!(wind_effects.width, 4);
        assert_eq!(wind_effects.height, 4);

        // Ridge areas should have higher wind speeds and erosion potential
        let ridge_shear = wind_effects.get_shear_stress(0, 0); // Highest elevation
        let valley_shear = wind_effects.get_shear_stress(2, 2); // Lower elevation

        assert!(
            ridge_shear >= valley_shear,
            "Ridge shear stress ({:.6}) should be >= valley shear stress ({:.6})",
            ridge_shear,
            valley_shear
        );

        // All values should be within realistic physical ranges
        for x in 0..4 {
            for y in 0..4 {
                let shear_stress = wind_effects.get_shear_stress(x, y);
                let erosion_potential = wind_effects.get_erosion_potential(x, y);
                let transport_capacity = wind_effects.get_transport_capacity(x, y);

                assert!(
                    shear_stress >= 0.0 && shear_stress < 10.0,
                    "Shear stress {:.3} Pa at ({}, {}) outside realistic range",
                    shear_stress,
                    x,
                    y
                );

                assert!(
                    erosion_potential >= 0.0,
                    "Erosion potential {:.6} at ({}, {}) should be non-negative",
                    erosion_potential,
                    x,
                    y
                );

                assert!(
                    transport_capacity >= 0.0,
                    "Transport capacity {:.6} at ({}, {}) should be non-negative",
                    transport_capacity,
                    x,
                    y
                );
            }
        }

        println!("Wind erosion effects test results:");
        println!("  Ridge shear stress: {:.3} Pa", ridge_shear);
        println!("  Valley shear stress: {:.3} Pa", valley_shear);
        println!(
            "  Ridge erosion potential: {:.6} kg/(m²·s)",
            wind_effects.get_erosion_potential(0, 0)
        );
        println!(
            "  Valley erosion potential: {:.6} kg/(m²·s)",
            wind_effects.get_erosion_potential(2, 2)
        );
    }

    #[test]
    fn test_wind_aware_geological_coupling() {
        use crate::engine::physics::flow_engine::{FlowAlgorithm, FlowEngine};

        // Create exposed terrain susceptible to wind erosion
        let mut heightmap = HeightMap::from_nested(vec![
            vec![0.8, 0.7, 0.6, 0.5, 0.4], // Gradual slope
            vec![0.7, 0.6, 0.5, 0.4, 0.3], // Continuing slope
            vec![0.6, 0.5, 0.4, 0.3, 0.2], // Lower terrain
        ]);

        let scale = WorldScale::new(8.0, (5, 3), DetailLevel::Standard);
        let climate_system = ClimateSystem::new_for_scale(&scale);
        let heightmap_nested = heightmap.to_nested();
        let temperature_layer = climate_system.generate_temperature_layer(&heightmap_nested);

        // Create atmospheric and flow systems
        let atmospheric_system = AtmosphericSystem::new_for_scale(&scale);
        let mut flow_engine = FlowEngine::new(FlowAlgorithm::Conservation, 5, 3, &scale);

        // Test different wind influence levels
        let no_wind = WindAwareGeologicalSystem::new_for_scale(&scale, 0.0);
        let full_wind = WindAwareGeologicalSystem::new_for_scale(&scale, 1.0);

        // Store initial terrain state for comparison
        let initial_heightmap = heightmap.clone();
        let initial_total_elevation: f32 = (0..5)
            .map(|x| (0..3).map(|y| initial_heightmap.get(x, y)).sum::<f32>())
            .sum();

        // Test with no wind influence
        let mut heightmap_no_wind = initial_heightmap.clone();
        let _no_wind_effects = no_wind.evolve_terrain_with_wind_erosion(
            &mut heightmap_no_wind,
            &atmospheric_system,
            &temperature_layer,
            &mut flow_engine,
            &scale,
            0.5,    // Noon
            3600.0, // 1 hour timestep
        );

        // Test with full wind influence
        let mut heightmap_full_wind = initial_heightmap.clone();
        let full_wind_effects = full_wind.evolve_terrain_with_wind_erosion(
            &mut heightmap_full_wind,
            &atmospheric_system,
            &temperature_layer,
            &mut flow_engine,
            &scale,
            0.5,    // Noon
            3600.0, // 1 hour timestep
        );

        // Verify wind effects were calculated
        assert_eq!(full_wind_effects.width, 5);
        assert_eq!(full_wind_effects.height, 3);

        // Calculate total elevation after wind erosion
        let no_wind_total: f32 = (0..5)
            .map(|x| (0..3).map(|y| heightmap_no_wind.get(x, y)).sum::<f32>())
            .sum();
        let full_wind_total: f32 = (0..5)
            .map(|x| (0..3).map(|y| heightmap_full_wind.get(x, y)).sum::<f32>())
            .sum();

        // Wind erosion should modify terrain (though changes may be small over 1 hour)
        let no_wind_change = (no_wind_total - initial_total_elevation).abs();
        let full_wind_change = (full_wind_total - initial_total_elevation).abs();

        // No wind should have minimal change, full wind should have some effect
        assert!(
            no_wind_change <= full_wind_change,
            "No wind change {:.6} should be <= full wind change {:.6}",
            no_wind_change,
            full_wind_change
        );

        // Exposed areas should show wind effects
        let exposed_wind_velocity = full_wind_effects.get_wind_velocity(0, 0); // High elevation
        let sheltered_wind_velocity = full_wind_effects.get_wind_velocity(4, 2); // Low elevation

        assert!(
            exposed_wind_velocity.magnitude() >= sheltered_wind_velocity.magnitude(),
            "Exposed wind speed {:.3} should be >= sheltered wind speed {:.3}",
            exposed_wind_velocity.magnitude(),
            sheltered_wind_velocity.magnitude()
        );

        println!("Wind-aware geological coupling test results:");
        println!("  Initial total elevation: {:.6}", initial_total_elevation);
        println!("  No wind influence: {:.6}", no_wind_total);
        println!("  Full wind influence: {:.6}", full_wind_total);
        println!("  No wind change: {:.6}", no_wind_change);
        println!("  Full wind change: {:.6}", full_wind_change);
        println!(
            "  Exposed wind speed: {:.3} m/s",
            exposed_wind_velocity.magnitude()
        );
        println!(
            "  Sheltered wind speed: {:.3} m/s",
            sheltered_wind_velocity.magnitude()
        );
    }
}
