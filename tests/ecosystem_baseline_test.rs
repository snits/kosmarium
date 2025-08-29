// ABOUTME: Simple ecosystem baseline test for temporal scaling integration validation
// ABOUTME: Captures current ecosystem behavior before temporal scaling implementation

use sim_prototype::engine::{
    core::scale::{DetailLevel, WorldScale},
    physics::{
        atmospheric_moisture::SurfaceMoistureLayer,
        ecosystem_feedback::{EcosystemFeedbackParameters, EcosystemFeedbackSystem},
        flow_engine::FlowEngine,
        temperature::TemperatureField,
        water::WaterLayer,
    },
};

/// Simple test that captures baseline ecosystem behavior
#[test]
fn test_capture_baseline_ecosystem_behavior() {
    let width = 10;
    let height = 10;

    // Create deterministic world scale
    let world_scale = WorldScale::new(
        10.0, // 10km domain
        (width as u32, height as u32),
        DetailLevel::Standard,
    );

    // Create ecosystem system with default parameters (including hardcoded growth_rate: 10.0)
    let mut ecosystem_system =
        EcosystemFeedbackSystem::new(EcosystemFeedbackParameters::default(), width, height);

    // Create temperature field with uniform temperature
    let mut temperature_field = TemperatureField::new(width, height, 20.0);

    // Create water layer with some water
    let mut water_layer = WaterLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            water_layer.add_water(x, y, 0.5);
        }
    }

    // Create surface moisture layer
    let mut moisture_layer = SurfaceMoistureLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            moisture_layer.set_moisture(x, y, 0.3);
        }
    }

    // Create flow engine
    let flow_engine = FlowEngine::new(
        sim_prototype::engine::physics::flow_engine::FlowAlgorithm::Gradient,
        width,
        height,
        &world_scale,
    );

    // Capture initial biomass
    let initial_biomass = ecosystem_system.biome_map().get_biomass(5, 5);

    // Run ecosystem update cycle (current behavior without temporal scaling)
    let dt = 0.1; // Standard timestep
    ecosystem_system.update(
        &mut temperature_field,
        &mut water_layer,
        &mut moisture_layer,
        &flow_engine,
        &world_scale,
        dt,
    );

    // Capture biomass after update
    let post_update_biomass = ecosystem_system.biome_map().get_biomass(5, 5);

    // Verify that ecosystem state changes (growth occurs)
    assert!(
        post_update_biomass >= initial_biomass,
        "Expected ecosystem growth or stability in favorable conditions"
    );

    // Calculate growth rate for reference
    let biomass_change = post_update_biomass - initial_biomass;

    println!("Baseline ecosystem behavior captured:");
    println!("  Initial biomass (5,5): {:.6}", initial_biomass);
    println!("  Post-update biomass (5,5): {:.6}", post_update_biomass);
    println!("  Biomass change: {:.6}", biomass_change);

    // Store as baseline for future comparison
    // This test establishes the reference behavior that Demo mode must preserve
    assert!(
        post_update_biomass > 0.0,
        "Ecosystem should maintain positive biomass"
    );
}

/// Test ecosystem parameter defaults to verify current growth_rate value
#[test]
fn test_ecosystem_parameters_baseline() {
    let params = EcosystemFeedbackParameters::default();

    // Verify the current hardcoded growth_rate value
    assert_eq!(
        params.growth_rate, 10.0,
        "Growth rate should be 10.0 kg/m²/day (current hardcoded value)"
    );

    println!("Baseline ecosystem parameters:");
    println!("  Growth rate: {} kg/m²/day", params.growth_rate);
    println!(
        "  Base evapotranspiration: {} mm/day",
        params.base_evapotranspiration
    );
    println!(
        "  Temperature moderation: {}°C per unit vegetation",
        params.temperature_moderation
    );
}
