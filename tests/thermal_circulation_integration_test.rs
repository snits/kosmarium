// ABOUTME: Integration tests for thermal circulation coupling system
// ABOUTME: Validates temperature-driven atmospheric flow patterns and physics integration

use sim_protoype::engine::{
    core::{
        PhysicsGrid,
        heightmap::HeightMap,
        scale::{DetailLevel, WorldScale},
    },
    physics::{
        atmosphere::AtmosphericSystem,
        climate::{AtmosphericPressureLayer, ClimateSystem, TemperatureLayer},
        flow_engine::FlowEngine,
        thermal_circulation::{
            ThermalCirculationEffects, ThermalCirculationParameters, ThermalCirculationSystem,
        },
        water::Vec2,
    },
};

/// Test thermal circulation with simple temperature gradient
#[test]
fn test_thermal_circulation_with_temperature_gradient() {
    let width = 20usize;
    let height = 20usize;
    let scale = WorldScale::new(1000.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create temperature layer with east-west gradient (warm east, cool west)
    let mut temp_layer = TemperatureLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let temperature = 10.0 + (x as f32 / width as f32) * 20.0; // 10°C to 30°C
            temp_layer.temperature.set(x, y, temperature);
        }
    }

    // Create required systems
    let mut flow_engine = FlowEngine::for_climate(width, height, &scale);
    let mut pressure_layer = AtmosphericPressureLayer {
        pressure: PhysicsGrid::new(width, height, 101325.0),
        pressure_gradient: PhysicsGrid::new(width, height, Vec2::new(0.0, 0.0)),
    };
    let climate_system = ClimateSystem::new_for_scale(&scale);

    // Initialize thermal circulation system
    let mut thermal_system = ThermalCirculationSystem::new(ThermalCirculationParameters::default());

    // Run thermal circulation update
    thermal_system.update(
        &temp_layer,
        &mut flow_engine,
        &mut pressure_layer,
        &climate_system,
        &scale,
        1.0, // 1 second timestep
    );

    // Verify thermal effects were generated
    assert!(thermal_system.has_active_effects());
    let effects = thermal_system.get_effects().unwrap();

    // Check center region for thermal activity
    let center_x = width / 2;
    let center_y = height / 2;

    // Temperature gradient should be detected
    let gradient = effects.get_temperature_gradient(center_x, center_y);
    assert!(
        gradient > 0.0,
        "Temperature gradient should be positive: {}",
        gradient
    );

    // Thermal velocity should be generated
    let thermal_velocity = effects.get_thermal_velocity(center_x, center_y);
    assert!(
        thermal_velocity.magnitude() > 0.0,
        "Thermal velocity should be generated"
    );

    // Buoyancy forces should exist in temperature gradient
    let buoyancy = effects.get_buoyancy_force(center_x, center_y);
    assert!(
        buoyancy != 0.0,
        "Buoyancy force should be non-zero in temperature gradient"
    );

    // Pressure adjustments should be applied
    let pressure_change = effects.get_thermal_pressure(center_x, center_y);
    assert!(
        pressure_change != 0.0,
        "Thermal pressure adjustment should be applied"
    );
}

/// Test thermal circulation parameters validation
#[test]
fn test_thermal_circulation_parameters() {
    let params = ThermalCirculationParameters::default();

    // Verify physical constants are reasonable
    assert!(params.reference_temperature_difference > 0.0);
    assert!(params.reference_temperature_difference < 100.0);

    assert!(params.buoyancy_coefficient > 0.0);
    assert!(params.buoyancy_coefficient < 1.0);

    assert!(params.pressure_response_coefficient > 0.0);
    assert!(params.pressure_response_coefficient < 1000.0);

    assert!(params.max_thermal_enhancement > 1.0);
    assert!(params.max_thermal_enhancement < 10.0);

    assert!(params.thermal_diffusion_rate >= 0.0);
    assert!(params.thermal_diffusion_rate <= 1.0);
}

/// Test convection cell detection in thermal hotspots
#[test]
fn test_convection_cell_detection() {
    let width = 10usize;
    let height = 10usize;
    let scale = WorldScale::new(500.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create temperature layer with hotspot in center
    let mut temp_layer = TemperatureLayer::new(width, height);
    let base_temp = 15.0;

    for x in 0..width {
        for y in 0..height {
            let dx = (x as f32) - (width as f32 / 2.0);
            let dy = (y as f32) - (height as f32 / 2.0);
            let distance = (dx * dx + dy * dy).sqrt();

            // Create hotspot: warm center, cool edges
            let temperature = if distance < 2.0 {
                base_temp + 5.0 // Hot center
            } else if distance > 4.0 {
                base_temp - 5.0 // Cool edges
            } else {
                base_temp // Neutral zone
            };

            temp_layer.temperature.set(x, y, temperature);
        }
    }

    // Create required systems
    let mut flow_engine = FlowEngine::for_climate(width, height, &scale);
    let mut pressure_layer = AtmosphericPressureLayer {
        pressure: PhysicsGrid::new(width, height, 101325.0),
        pressure_gradient: PhysicsGrid::new(width, height, Vec2::new(0.0, 0.0)),
    };
    let climate_system = ClimateSystem::new_for_scale(&scale);

    // Initialize thermal circulation system
    let mut thermal_system = ThermalCirculationSystem::new(ThermalCirculationParameters::default());

    // Run thermal circulation update
    thermal_system.update(
        &temp_layer,
        &mut flow_engine,
        &mut pressure_layer,
        &climate_system,
        &scale,
        1.0,
    );

    let effects = thermal_system.get_effects().unwrap();

    // Check convection cell detection
    let center_x = width / 2;
    let center_y = height / 2;

    // Center should show rising air (positive convection)
    let center_convection = effects.get_convection_cell(center_x, center_y);
    assert!(
        center_convection > 0.0,
        "Center should have rising air convection"
    );

    // Edges should show sinking air (negative convection)
    let edge_convection = effects.get_convection_cell(1, 1);
    assert!(
        edge_convection < 0.0,
        "Edges should have sinking air convection"
    );
}

/// Test thermal circulation integration with flow engine
#[test]
fn test_thermal_flow_engine_integration() {
    let width = 15usize;
    let height = 15usize;
    let scale = WorldScale::new(750.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create strong temperature gradient
    let mut temp_layer = TemperatureLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            // Strong north-south temperature gradient
            let temperature = 5.0 + (y as f32 / height as f32) * 30.0; // 5°C to 35°C
            temp_layer.temperature.set(x, y, temperature);
        }
    }

    // Create systems
    let mut flow_engine = FlowEngine::for_climate(width, height, &scale);
    let mut pressure_layer = AtmosphericPressureLayer {
        pressure: PhysicsGrid::new(width, height, 101325.0),
        pressure_gradient: PhysicsGrid::new(width, height, Vec2::new(0.0, 0.0)),
    };
    let climate_system = ClimateSystem::new_for_scale(&scale);

    // Record initial velocity field state
    let initial_velocity = flow_engine
        .velocity_field
        .get_velocity(width / 2, height / 2);
    let initial_kinetic_energy = flow_engine.velocity_field.total_kinetic_energy();

    // Apply thermal circulation
    let mut thermal_system = ThermalCirculationSystem::new(ThermalCirculationParameters::default());
    thermal_system.update(
        &temp_layer,
        &mut flow_engine,
        &mut pressure_layer,
        &climate_system,
        &scale,
        1.0,
    );

    // Check that flow engine was modified
    let final_velocity = flow_engine
        .velocity_field
        .get_velocity(width / 2, height / 2);
    let final_kinetic_energy = flow_engine.velocity_field.total_kinetic_energy();

    // Thermal circulation should enhance velocity field
    assert!(
        final_velocity.magnitude() >= initial_velocity.magnitude(),
        "Thermal circulation should enhance or maintain velocity magnitude"
    );

    // Total kinetic energy should increase due to thermal driving
    assert!(
        final_kinetic_energy >= initial_kinetic_energy,
        "Thermal circulation should add energy to the system"
    );

    // Maximum velocity in field should be reasonable (not infinite)
    let max_velocity = flow_engine.velocity_field.max_velocity_magnitude();
    assert!(
        max_velocity < 100.0,
        "Maximum velocity should be physically reasonable: {}",
        max_velocity
    );
}

/// Test thermal pressure coupling with atmospheric system
#[test]
fn test_thermal_pressure_coupling() {
    let width = 12usize;
    let height = 12usize;
    let scale = WorldScale::new(600.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create temperature anomalies
    let mut temp_layer = TemperatureLayer::new(width, height);
    let base_temp = 20.0;

    for x in 0..width {
        for y in 0..height {
            let temperature = if x < width / 2 {
                base_temp + 10.0 // Warm western half
            } else {
                base_temp - 10.0 // Cool eastern half
            };
            temp_layer.temperature.set(x, y, temperature);
        }
    }

    // Initialize pressure layer with uniform pressure
    let standard_pressure = 101325.0;
    let mut pressure_layer = AtmosphericPressureLayer {
        pressure: PhysicsGrid::new(width, height, standard_pressure),
        pressure_gradient: PhysicsGrid::new(width, height, Vec2::new(0.0, 0.0)),
    };

    // Create systems
    let mut flow_engine = FlowEngine::for_climate(width, height, &scale);
    let climate_system = ClimateSystem::new_for_scale(&scale);

    // Apply thermal circulation
    let mut thermal_system = ThermalCirculationSystem::new(ThermalCirculationParameters::default());
    thermal_system.update(
        &temp_layer,
        &mut flow_engine,
        &mut pressure_layer,
        &climate_system,
        &scale,
        1.0,
    );

    // Check pressure modifications
    let warm_pressure = *pressure_layer.pressure.get(width / 4, height / 2);
    let cool_pressure = *pressure_layer.pressure.get(3 * width / 4, height / 2);

    // Warm areas should have lower pressure, cool areas higher pressure
    assert!(
        warm_pressure < standard_pressure,
        "Warm areas should have reduced pressure: {} < {}",
        warm_pressure,
        standard_pressure
    );

    assert!(
        cool_pressure > standard_pressure,
        "Cool areas should have increased pressure: {} > {}",
        cool_pressure,
        standard_pressure
    );

    // Pressure gradient should exist between warm and cool areas
    let pressure_diff = cool_pressure - warm_pressure;
    assert!(
        pressure_diff > 0.0,
        "Pressure gradient should exist: {}",
        pressure_diff
    );
}

/// Test thermal diffusion smoothing effects
#[test]
fn test_thermal_diffusion_smoothing() {
    let width = 8usize;
    let height = 8usize;
    let scale = WorldScale::new(400.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create sharp temperature spike
    let mut temp_layer = TemperatureLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let temperature = if x == width / 2 && y == height / 2 {
                50.0 // Hot spike
            } else {
                10.0 // Cool background
            };
            temp_layer.temperature.set(x, y, temperature);
        }
    }

    // Create systems
    let mut flow_engine = FlowEngine::for_climate(width, height, &scale);
    let mut pressure_layer = AtmosphericPressureLayer {
        pressure: PhysicsGrid::new(width, height, 101325.0),
        pressure_gradient: PhysicsGrid::new(width, height, Vec2::new(0.0, 0.0)),
    };
    let climate_system = ClimateSystem::new_for_scale(&scale);

    // Create thermal system with high diffusion
    let mut params = ThermalCirculationParameters::default();
    params.thermal_diffusion_rate = 0.5; // High diffusion
    let mut thermal_system = ThermalCirculationSystem::new(params);

    // Run multiple updates to see diffusion effect
    for _ in 0..5 {
        thermal_system.update(
            &temp_layer,
            &mut flow_engine,
            &mut pressure_layer,
            &climate_system,
            &scale,
            0.1,
        );
    }

    let effects = thermal_system.get_effects().unwrap();

    // Check that thermal velocities are smoothed (not extreme)
    let center_velocity = effects.get_thermal_velocity(width / 2, height / 2);
    let neighbor_velocity = effects.get_thermal_velocity(width / 2 + 1, height / 2);

    // Diffusion should prevent extreme velocity differences
    let velocity_diff = (center_velocity.magnitude() - neighbor_velocity.magnitude()).abs();
    assert!(
        velocity_diff < 50.0,
        "Diffusion should smooth velocity differences: {}",
        velocity_diff
    );
}

/// Test thermal circulation system memory and state management
#[test]
fn test_thermal_system_state_management() {
    let width = 6usize;
    let height = 6usize;
    let scale = WorldScale::new(300.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create simple temperature field
    let mut temp_layer = TemperatureLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            temp_layer.temperature.set(x, y, 15.0 + x as f32); // Simple gradient
        }
    }

    // Create systems
    let mut flow_engine = FlowEngine::for_climate(width, height, &scale);
    let mut pressure_layer = AtmosphericPressureLayer {
        pressure: PhysicsGrid::new(width, height, 101325.0),
        pressure_gradient: PhysicsGrid::new(width, height, Vec2::new(0.0, 0.0)),
    };
    let climate_system = ClimateSystem::new_for_scale(&scale);

    // Initialize thermal system
    let mut thermal_system = ThermalCirculationSystem::new(ThermalCirculationParameters::default());

    // Initially should have no effects
    assert!(!thermal_system.has_active_effects());
    assert!(thermal_system.get_effects().is_none());

    // After update should have effects
    thermal_system.update(
        &temp_layer,
        &mut flow_engine,
        &mut pressure_layer,
        &climate_system,
        &scale,
        1.0,
    );

    assert!(thermal_system.has_active_effects());
    assert!(thermal_system.get_effects().is_some());

    // Effects should persist until next update
    let first_effects = thermal_system.get_effects().unwrap();
    let first_gradient = first_effects.get_temperature_gradient(2, 2);

    // Run another update
    thermal_system.update(
        &temp_layer,
        &mut flow_engine,
        &mut pressure_layer,
        &climate_system,
        &scale,
        1.0,
    );

    // Effects should be updated (may be similar but recalculated)
    let second_effects = thermal_system.get_effects().unwrap();
    let second_gradient = second_effects.get_temperature_gradient(2, 2);

    // Gradients should be similar (same temperature field)
    let gradient_diff = (first_gradient - second_gradient).abs();
    assert!(
        gradient_diff < 0.01,
        "Gradient calculations should be consistent: {} vs {}",
        first_gradient,
        second_gradient
    );
}
