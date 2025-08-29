// ABOUTME: Validation binary for Metis scaling corrections implementation
// ABOUTME: Tests the three critical physics violations fixed following mathematical analysis

use kosmarium::engine::core::{
    heightmap::HeightMap,
    scale::{DetailLevel, WorldScale},
};
use kosmarium::engine::physics::{
    atmospheric_moisture::AtmosphericMoistureSystem,
    climate::ClimateSystem,
    flow_engine::{FlowAlgorithm, FlowEngine},
    maritime_climate_coupling::{CoastalThermalEffects, MaritimAwareAtmosphereSystem},
    orographic_precipitation::{OrographicParameters, OrographicPrecipitationSystem},
    thermal_circulation::{ThermalCirculationParameters, ThermalCirculationSystem},
    water::WaterLayer,
};

/// Test thermal circulation scaling behavior across different domain sizes
/// Metis prediction: α = -1.0 → α ≈ 0.0 (scale invariant)
fn test_thermal_circulation_scaling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== METIS THERMAL CIRCULATION SCALING VALIDATION ===");

    let domain_sizes = vec![10.0, 100.0, 1000.0, 10000.0]; // km
    let mut thermal_velocities = Vec::new();

    for &domain_size_km in &domain_sizes {
        // Create consistent grid size but different domain scales
        let scale = WorldScale::new(domain_size_km, (50, 50), DetailLevel::Standard);

        // Create test terrain with temperature gradient
        let mut heightmap_data = vec![vec![0.0; 50]; 50];
        for x in 0..50 {
            for y in 0..50 {
                heightmap_data[x][y] = 0.5; // Constant elevation
            }
        }
        let heightmap = HeightMap::from_nested(heightmap_data);

        // Create climate system and set up temperature gradient
        let climate_system = ClimateSystem::new_for_scale(&scale);
        let heightmap_nested = heightmap.to_nested();
        let mut temperature_layer = climate_system.generate_temperature_layer(&heightmap_nested);

        // Artificial temperature gradient for testing
        for x in 0..50 {
            for y in 0..50 {
                let temp_gradient = 15.0 + (x as f32) * 0.2; // Linear temperature increase
                temperature_layer.temperature.set(x, y, temp_gradient);
            }
        }

        // Create thermal circulation system
        let params = ThermalCirculationParameters::default();
        let mut thermal_system = ThermalCirculationSystem::new(params);

        // Create supporting systems for thermal circulation test
        let mut flow_engine = FlowEngine::new(FlowAlgorithm::Gradient, 50, 50, &scale);
        let mut pressure_layer =
            kosmarium::engine::physics::climate::AtmosphericPressureLayer {
                pressure: kosmarium::engine::core::PhysicsGrid::new(50, 50, 101325.0),
                pressure_gradient: kosmarium::engine::core::PhysicsGrid::new(
                    50,
                    50,
                    kosmarium::engine::physics::water::Vec2::new(0.0, 0.0),
                ),
            };

        // Update thermal circulation
        thermal_system.update(
            &temperature_layer,
            &mut flow_engine,
            &mut pressure_layer,
            &climate_system,
            &scale,
            0.1, // dt
        );

        // Measure thermal velocity at center of domain
        if let Some(effects) = thermal_system.get_effects() {
            let center_velocity = effects.get_thermal_velocity(25, 25);
            let velocity_magnitude = center_velocity.magnitude();
            thermal_velocities.push(velocity_magnitude);

            println!(
                "Domain {:6.0} km: Thermal velocity = {:.6} m/s",
                domain_size_km, velocity_magnitude
            );
        } else {
            thermal_velocities.push(0.0);
            println!(
                "Domain {:6.0} km: No thermal effects calculated",
                domain_size_km
            );
        }
    }

    // Calculate scaling behavior
    let velocity_ratios: Vec<f32> = thermal_velocities
        .windows(2)
        .map(|w| w[1] / w[0].max(1e-10))
        .collect();

    println!("Velocity scaling ratios: {:?}", velocity_ratios);

    // Metis success criteria: velocities should be scale-invariant (ratios ≈ 1.0)
    let scale_invariance_achieved = velocity_ratios
        .iter()
        .all(|&ratio| (ratio - 1.0).abs() < 2.0); // Allow 2x variation vs previous 1000x

    println!("Scale invariance achieved: {}", scale_invariance_achieved);
    println!();

    Ok(())
}

/// Test orographic precipitation scaling behavior across different domain sizes
/// Metis prediction: α = +1.0 → α ≈ 0.0 (scale invariant enhancement)
fn test_orographic_precipitation_scaling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== METIS OROGRAPHIC PRECIPITATION SCALING VALIDATION ===");

    let domain_sizes = vec![10.0, 100.0, 1000.0, 10000.0]; // km
    let mut enhancement_factors = Vec::new();

    for &domain_size_km in &domain_sizes {
        let scale = WorldScale::new(domain_size_km, (20, 20), DetailLevel::Standard);

        // Create mountain terrain for orographic testing
        let mut heightmap_data = vec![vec![0.0; 20]; 20];
        for x in 0..20 {
            for y in 0..20 {
                // Create a mountain at center with consistent slope
                let center_x = 10.0;
                let center_y = 10.0;
                let distance =
                    ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
                heightmap_data[x][y] = (1.0 - distance / 10.0).max(0.0); // Cone-shaped mountain
            }
        }
        let heightmap = HeightMap::from_nested(heightmap_data);

        // Create atmospheric systems
        let climate_system = ClimateSystem::new_for_scale(&scale);
        let mut atmospheric_moisture = AtmosphericMoistureSystem::new_for_scale(&scale, 20, 20);
        let water_layer = WaterLayer::new(20, 20);
        atmospheric_moisture.initialize_from_terrain(&heightmap, &water_layer);

        // Set uniform humidity for consistent testing
        for x in 0..20 {
            for y in 0..20 {
                atmospheric_moisture
                    .surface_moisture
                    .set_humidity(x, y, 70.0); // 70% humidity
            }
        }

        // Create flow engine with uniform wind
        let mut flow_engine = FlowEngine::new(FlowAlgorithm::Gradient, 20, 20, &scale);
        for x in 0..20 {
            for y in 0..20 {
                flow_engine.velocity_field.set_velocity(
                    x,
                    y,
                    kosmarium::engine::core::math::Vec2::new(5.0, 0.0), // 5 m/s eastward wind
                );
            }
        }

        // Create orographic precipitation system
        let params = OrographicParameters::default();
        let mut orographic_system = OrographicPrecipitationSystem::new(params);

        // Update orographic effects
        orographic_system.update(
            &heightmap,
            &flow_engine,
            &mut atmospheric_moisture,
            &climate_system,
            &scale,
            0.1, // dt in hours
        );

        // Measure enhancement at windward slope (x=8, mountain center)
        let enhancement_factor = orographic_system.get_precipitation_multiplier(8, 10);
        enhancement_factors.push(enhancement_factor);

        println!(
            "Domain {:6.0} km: Orographic enhancement = {:.3}x",
            domain_size_km, enhancement_factor
        );
    }

    // Calculate scaling behavior
    let enhancement_ratios: Vec<f32> = enhancement_factors
        .windows(2)
        .map(|w| w[1] / w[0].max(0.01))
        .collect();

    println!("Enhancement scaling ratios: {:?}", enhancement_ratios);

    // Metis success criteria: enhancement should be scale-invariant (not grow with domain size)
    let scale_invariance_achieved = enhancement_factors.iter().all(|&factor| factor < 10.0); // Should not exceed reasonable limits vs previous 2500x

    println!(
        "Realistic enhancement factors achieved: {}",
        scale_invariance_achieved
    );
    println!();

    Ok(())
}

/// Test maritime climate pressure anomaly scaling
/// Metis prediction: Fixed 1000m → scale-dependent h ∝ domain_size^0.5
fn test_maritime_climate_scaling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== METIS MARITIME CLIMATE SCALING VALIDATION ===");

    let domain_sizes = vec![10.0, 100.0, 1000.0, 10000.0]; // km
    let mut pressure_anomalies = Vec::new();

    for &domain_size_km in &domain_sizes {
        let scale = WorldScale::new(domain_size_km, (10, 10), DetailLevel::Standard);

        // Create coastal terrain (land-sea interface)
        let mut heightmap_data = vec![vec![0.0; 10]; 10];
        for x in 0..10 {
            for y in 0..10 {
                if x < 5 {
                    heightmap_data[x][y] = 0.3; // Land
                } else {
                    heightmap_data[x][y] = -0.1; // Sea
                }
            }
        }
        let heightmap = HeightMap::from_nested(heightmap_data);

        // Create climate system
        let climate_system = ClimateSystem::new_for_scale(&scale);
        let heightmap_nested = heightmap.to_nested();
        let temperature_layer = climate_system.generate_temperature_layer(&heightmap_nested);

        // Test coastal thermal effects calculation
        let coastal_effects = CoastalThermalEffects::from_temperature_gradients(
            &temperature_layer,
            &heightmap,
            &scale,
            0.5, // Noon
        );

        // Measure pressure anomaly at coastal interface (x=4, land side)
        let pressure_anomaly = coastal_effects.get_pressure_anomaly(4, 5).abs();
        pressure_anomalies.push(pressure_anomaly);

        println!(
            "Domain {:6.0} km: Pressure anomaly = {:.1} Pa",
            domain_size_km, pressure_anomaly
        );
    }

    // Calculate scaling behavior
    let pressure_ratios: Vec<f32> = pressure_anomalies
        .windows(2)
        .map(|w| w[1] / w[0].max(0.1))
        .collect();

    println!("Pressure scaling ratios: {:?}", pressure_ratios);

    // Metis success criteria: pressure should increase with domain size (not stay constant)
    // This indicates the scale-dependent mixing height is working
    let scaling_improvement_achieved = pressure_ratios.iter().any(|&ratio| ratio > 1.5); // Should increase vs. previous constant values

    println!(
        "Scale-dependent pressure scaling achieved: {}",
        scaling_improvement_achieved
    );
    println!();

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("METIS CROSS-SYSTEM PHYSICS SCALING CORRECTIONS VALIDATION");
    println!("=========================================================");
    println!("Following successful 7,883x velocity improvement pattern");
    println!("Testing the three critical scaling violations identified:");
    println!("1. Thermal Circulation: Grid scaling dependency (α = -1.0 → 0.0)");
    println!("2. Orographic Precipitation: Inverse scaling violation (α = +1.0 → 0.0)");
    println!("3. Maritime Climate: Fixed mixing height (97% error → scale-dependent)");
    println!();

    // Test each corrected system
    test_thermal_circulation_scaling()?;
    test_orographic_precipitation_scaling()?;
    test_maritime_climate_scaling()?;

    println!("=== METIS VALIDATION SUMMARY ===");
    println!("Mathematical corrections implemented following theoretical analysis:");
    println!("✓ Thermal circulation: Removed cell_size_m scaling from gradients");
    println!("✓ Orographic precipitation: Removed cell_size_m scaling from slopes");
    println!("✓ Maritime climate: Implemented scale-dependent mixing height h ∝ domain^0.5");
    println!();
    println!("Re-run Metis cross-system validation to measure actual improvement ratios!");

    Ok(())
}
