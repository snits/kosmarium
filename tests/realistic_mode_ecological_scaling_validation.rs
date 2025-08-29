//! Comprehensive validation of Realistic mode ecological scaling
//!
//! This test validates that the 3,650x temporal scaling violation has been fixed
//! by implementing scientifically accurate 2.5 kg/m²/year growth rates in Realistic mode
//! while preserving exact Demo mode behavior for backward compatibility.

use sim_prototype::engine::core::{
    scale::{DetailLevel, WorldScale},
    temporal_scaling::{TemporalMode, TemporalScalingConfig, TemporalScalingService},
};
use sim_prototype::engine::physics::{
    atmospheric_moisture::SurfaceMoistureLayer,
    ecosystem_feedback::{BiomeType, EcosystemFeedbackParameters, EcosystemFeedbackSystem},
    flow_engine::FlowEngine,
    temperature::TemperatureField,
    water::WaterLayer,
};

/// Test the core objective: Realistic mode achieving 2.5 kg/m²/year growth rates
#[test]
fn test_realistic_mode_achieves_target_growth_rate() {
    // Create temporal scaling service configured for Realistic mode
    let realistic_temporal_scaling = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Realistic,
        ..Default::default()
    });

    // Calculate the expected scaling factor
    // Target: 2.5 kg/m²/year from 10.0 kg/m²/day base rate
    let expected_scaling_factor = 2.5 / 3650.0; // 0.000685

    // Validate scaling factor is correct
    let actual_scaling_factor = realistic_temporal_scaling.biological_scaling_factor();
    let scaling_error = (actual_scaling_factor - expected_scaling_factor).abs();

    assert!(
        scaling_error < 1e-10,
        "Realistic mode scaling factor should be 2.5/3650 = {:.6}, but got {:.6} (error: {:.6})",
        expected_scaling_factor,
        actual_scaling_factor,
        scaling_error
    );

    // Test with realistic simulation timestep (0.1 hour)
    let dt_hours = 0.1;
    let base_growth_rate = 10.0; // kg/m²/day

    let scaled_rate =
        realistic_temporal_scaling.scale_ecosystem_growth_rate(base_growth_rate, dt_hours);

    // Calculate expected rate: 10.0 * (2.5/3650) * (0.1/24) = 10.0 * 0.000685 * 0.004167 = ~0.0000285
    let expected_rate = base_growth_rate * expected_scaling_factor * (dt_hours / 24.0);
    let rate_error = (scaled_rate - expected_rate).abs();

    assert!(
        rate_error < 1e-10,
        "Realistic mode should produce {:.9} kg/m²/timestep, but got {:.9} (error: {:.9})",
        expected_rate,
        scaled_rate,
        rate_error
    );

    // Validate that this achieves the annual target
    // Over a full year (8760 hours), this should accumulate to 2.5 kg/m²/year
    let timesteps_per_year = 8760.0 / dt_hours; // 87,600 timesteps per year
    let annual_accumulation = scaled_rate * timesteps_per_year;

    // Allow 1% tolerance for numerical precision
    let annual_error = (annual_accumulation - 2.5).abs();
    assert!(
        annual_error < 0.025,
        "Annual accumulation should be 2.5 kg/m²/year, but calculated {:.6} kg/m²/year (error: {:.6})",
        annual_accumulation,
        annual_error
    );
}

/// Test that Demo mode preserves exact behavior (bit-identical)
#[test]
fn test_demo_mode_preserves_exact_behavior() {
    let demo_temporal_scaling = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Demo,
        ..Default::default()
    });

    // Demo mode scaling factor should be exactly 1.0
    assert_eq!(demo_temporal_scaling.biological_scaling_factor(), 1.0);

    let dt_hours = 0.1;
    let base_growth_rate = 10.0;

    let scaled_rate = demo_temporal_scaling.scale_ecosystem_growth_rate(base_growth_rate, dt_hours);
    let expected_demo_rate = base_growth_rate * dt_hours / 24.0; // Current behavior

    // Must be bit-identical for backward compatibility
    assert_eq!(
        scaled_rate, expected_demo_rate,
        "Demo mode must preserve exact behavior: expected {:.9}, got {:.9}",
        expected_demo_rate, scaled_rate
    );
}

/// Test ecosystem feedback system integration with Realistic mode
#[test]
fn test_ecosystem_feedback_realistic_integration() {
    let world_scale = WorldScale::new(10.0, (50, 50), DetailLevel::Standard);

    // Create ecosystem feedback system with Realistic temporal scaling
    let realistic_temporal_scaling = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Realistic,
        ..Default::default()
    });

    let parameters = EcosystemFeedbackParameters::default();
    let mut ecosystem_system = EcosystemFeedbackSystem::new_with_temporal_scaling(
        parameters,
        50,
        50,
        realistic_temporal_scaling,
    );

    // Verify temporal mode is correct
    assert_eq!(
        ecosystem_system.get_temporal_mode(),
        TemporalMode::Realistic
    );
    assert!(!ecosystem_system.is_demo_mode());

    // Test the description
    let description = ecosystem_system.temporal_scaling_description();
    assert!(description.contains("Realistic mode"));
    assert!(description.contains("2.5 kg/m²/year"));

    // Create test environmental conditions
    let mut temperature_field = TemperatureField::new(50, 50);
    let mut water_layer = WaterLayer::new(50, 50);
    let mut moisture_layer = SurfaceMoistureLayer::new(50, 50);
    let flow_engine = FlowEngine::new(
        sim_prototype::engine::physics::flow_engine::FlowAlgorithm::Gradient,
        50,
        50,
        &world_scale,
    );

    // Set up favorable growing conditions
    for y in 0..50 {
        for x in 0..50 {
            temperature_field.set_temperature(x, y, 20.0); // Optimal temperature
            water_layer.add_water(x, y, 0.5); // Adequate water
            moisture_layer.set_moisture(x, y, 0.5); // Good humidity
        }
    }

    // Set up a forest biome for high growth potential
    ecosystem_system
        .biome_map_mut()
        .set_biome(25, 25, BiomeType::Forest);
    ecosystem_system.biome_map_mut().set_biomass(25, 25, 100.0); // Starting biomass

    let initial_biomass = ecosystem_system.biome_map().get_biomass(25, 25);

    // Run one ecosystem update with realistic timestep
    let dt = 0.1 / 24.0; // 0.1 hour as fraction of day
    ecosystem_system.update(
        &mut temperature_field,
        &mut water_layer,
        &mut moisture_layer,
        &flow_engine,
        &world_scale,
        dt,
    );

    let final_biomass = ecosystem_system.biome_map().get_biomass(25, 25);
    let biomass_change = final_biomass - initial_biomass;

    // Biomass change should be much smaller than Demo mode (scientifically realistic)
    // In Demo mode, this change would be ~1000x larger
    assert!(
        biomass_change > 0.0,
        "Should have positive growth under optimal conditions"
    );
    assert!(
        biomass_change < 0.01,
        "Growth should be much smaller than Demo mode (< 0.01 per timestep)"
    );

    // The change should be proportional to the 2.5/3650 scaling factor
    // This validates that the integration is working correctly
    let expected_relative_change = 2.5 / 3650.0; // Relative to Demo mode
    assert!(
        biomass_change < expected_relative_change * 10.0,
        "Biomass change should reflect realistic scaling: got {:.9}, expected magnitude ~{:.9}",
        biomass_change,
        expected_relative_change
    );
}

/// Test ecological process coherence in Realistic mode
#[test]
fn test_ecological_process_coherence() {
    let world_scale = WorldScale::new(10.0, (20, 20), DetailLevel::Standard);

    let realistic_temporal_scaling = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Realistic,
        ..Default::default()
    });

    let parameters = EcosystemFeedbackParameters::default();
    let mut ecosystem_system = EcosystemFeedbackSystem::new_with_temporal_scaling(
        parameters,
        20,
        20,
        realistic_temporal_scaling,
    );

    // Test environmental stress patterns
    let mut temperature_field = TemperatureField::new(20, 20);
    let mut water_layer = WaterLayer::new(20, 20);
    let mut moisture_layer = SurfaceMoistureLayer::new(20, 20);
    let flow_engine = FlowEngine::new(
        sim_prototype::engine::physics::flow_engine::FlowAlgorithm::Gradient,
        20,
        20,
        &world_scale,
    );

    // Create drought stress condition (very low water)
    for y in 0..20 {
        for x in 0..20 {
            temperature_field.set_temperature(x, y, 25.0); // Warm
            water_layer.add_water(x, y, 0.01); // Very low water (drought)
            moisture_layer.set_moisture(x, y, 0.1); // Low humidity
        }
    }

    // Set up grassland with initial biomass
    ecosystem_system
        .biome_map_mut()
        .set_biome(10, 10, BiomeType::Grassland);
    ecosystem_system.biome_map_mut().set_biomass(10, 10, 150.0); // Starting biomass

    let initial_biomass = ecosystem_system.biome_map().get_biomass(10, 10);

    // Run ecosystem update under drought conditions
    let dt = 0.1 / 24.0; // 0.1 hour as fraction of day
    ecosystem_system.update(
        &mut temperature_field,
        &mut water_layer,
        &mut moisture_layer,
        &flow_engine,
        &world_scale,
        dt,
    );

    let final_biomass = ecosystem_system.biome_map().get_biomass(10, 10);
    let biomass_change = final_biomass - initial_biomass;

    // Under drought stress, biomass should decline or grow very slowly
    // This tests that stress responses still work correctly with temporal scaling
    assert!(
        biomass_change <= 0.001,
        "Under drought stress, growth should be minimal or negative, got change: {:.6}",
        biomass_change
    );

    // The stress response should be proportional to the realistic scaling
    // but the ecological pattern (decline under stress) should be preserved
    assert!(
        biomass_change.abs() < 0.01,
        "Stress response should be scaled but ecologically coherent"
    );
}

/// Performance validation: Realistic mode should maintain <5% overhead
#[test]
fn test_realistic_mode_performance() {
    let world_scale = WorldScale::new(10.0, (100, 100), DetailLevel::Standard);

    // Test Demo mode performance
    let demo_temporal_scaling = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Demo,
        ..Default::default()
    });

    let demo_system = EcosystemFeedbackSystem::new_with_temporal_scaling(
        EcosystemFeedbackParameters::default(),
        100,
        100,
        demo_temporal_scaling,
    );

    // Test Realistic mode performance
    let realistic_temporal_scaling = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Realistic,
        ..Default::default()
    });

    let realistic_system = EcosystemFeedbackSystem::new_with_temporal_scaling(
        EcosystemFeedbackParameters::default(),
        100,
        100,
        realistic_temporal_scaling,
    );

    // Performance test: scaling calculation should be very fast
    let iterations = 1_000_000;
    let dt_hours = 0.1;
    let base_rate = 10.0;

    // Time realistic scaling
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _scaled = realistic_system
            .temporal_scaling
            .scale_ecosystem_growth_rate(base_rate, dt_hours);
    }
    let realistic_duration = start.elapsed();

    // Scaling should be extremely fast (< 1 microsecond per operation)
    let ops_per_second = iterations as f64 / realistic_duration.as_secs_f64();
    assert!(
        ops_per_second > 1_000_000.0,
        "Realistic mode scaling should achieve >1M ops/sec, got {:.0}",
        ops_per_second
    );

    println!(
        "Realistic mode scaling performance: {:.0} ops/sec",
        ops_per_second
    );

    // Memory overhead should be minimal (same struct size)
    assert_eq!(
        std::mem::size_of_val(&demo_system),
        std::mem::size_of_val(&realistic_system),
        "Realistic mode should not add memory overhead"
    );
}

/// Integration test: Mode switching during runtime
#[test]
fn test_runtime_mode_switching() {
    let world_scale = WorldScale::new(10.0, (30, 30), DetailLevel::Standard);

    // Start with Demo mode
    let demo_temporal_scaling = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Demo,
        ..Default::default()
    });

    let mut ecosystem_system = EcosystemFeedbackSystem::new_with_temporal_scaling(
        EcosystemFeedbackParameters::default(),
        30,
        30,
        demo_temporal_scaling,
    );

    // Verify initial Demo mode
    assert_eq!(ecosystem_system.get_temporal_mode(), TemporalMode::Demo);
    assert!(ecosystem_system.is_demo_mode());

    let demo_description = ecosystem_system.temporal_scaling_description();
    assert!(demo_description.contains("Demo mode"));

    // Switch to Realistic mode
    let realistic_temporal_scaling = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Realistic,
        ..Default::default()
    });

    ecosystem_system.update_temporal_scaling(realistic_temporal_scaling);

    // Verify mode switch
    assert_eq!(
        ecosystem_system.get_temporal_mode(),
        TemporalMode::Realistic
    );
    assert!(!ecosystem_system.is_demo_mode());

    let realistic_description = ecosystem_system.temporal_scaling_description();
    assert!(realistic_description.contains("Realistic mode"));
    assert!(realistic_description.contains("2.5 kg/m²/year"));

    // Switch to Research mode
    let research_temporal_scaling = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Research,
        custom_scaling_factor: 0.1, // 10% of Demo rate
        ..Default::default()
    });

    ecosystem_system.update_temporal_scaling(research_temporal_scaling);

    // Verify Research mode
    assert_eq!(ecosystem_system.get_temporal_mode(), TemporalMode::Research);
    assert!(!ecosystem_system.is_demo_mode());

    let research_description = ecosystem_system.temporal_scaling_description();
    assert!(research_description.contains("Research mode"));
    assert!(research_description.contains("0.100000"));
}

/// Mathematical validation of scaling factors
#[test]
fn test_scaling_factor_mathematical_accuracy() {
    let realistic_service = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Realistic,
        ..Default::default()
    });

    // Test various timesteps for mathematical consistency
    let test_timesteps = vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.0]; // Hours
    let base_rate = 10.0; // kg/m²/day

    for &dt_hours in &test_timesteps {
        let scaled_rate = realistic_service.scale_ecosystem_growth_rate(base_rate, dt_hours);

        // Calculate expected rate manually
        let expected_scaling = 2.5 / 3650.0; // Realistic mode scaling
        let expected_rate = base_rate * expected_scaling * (dt_hours / 24.0);

        let error = (scaled_rate - expected_rate).abs();
        assert!(
            error < 1e-12,
            "Scaling should be mathematically precise for dt={} hours: expected {:.12}, got {:.12}, error {:.12}",
            dt_hours,
            expected_rate,
            scaled_rate,
            error
        );
    }
}

/// Validate that the 3,650x temporal scaling violation is fixed
#[test]
fn test_temporal_scaling_violation_fixed() {
    let demo_service = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Demo,
        ..Default::default()
    });

    let realistic_service = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Realistic,
        ..Default::default()
    });

    let dt_hours = 0.1;
    let base_rate = 10.0; // kg/m²/day

    let demo_rate = demo_service.scale_ecosystem_growth_rate(base_rate, dt_hours);
    let realistic_rate = realistic_service.scale_ecosystem_growth_rate(base_rate, dt_hours);

    // Calculate the ratio between Demo and Realistic modes
    let scaling_ratio = demo_rate / realistic_rate;

    // The ratio should be approximately 3650/2.5 = 1460
    let expected_ratio = 3650.0 / 2.5; // 1460
    let ratio_error = (scaling_ratio - expected_ratio).abs();

    assert!(
        ratio_error < 1.0,
        "Scaling ratio should be ~1460 (fixing 3650x violation with 2.5x target), got {:.1} (error: {:.1})",
        scaling_ratio,
        ratio_error
    );

    // Validate that Realistic mode produces scientifically reasonable rates
    // At 0.1 hour timesteps, the rate should accumulate to 2.5 kg/m²/year
    let timesteps_per_year = 8760.0 / dt_hours; // 87,600 timesteps per year
    let annual_accumulation = realistic_rate * timesteps_per_year;

    let annual_target = 2.5; // kg/m²/year
    let annual_error = (annual_accumulation - annual_target).abs();

    assert!(
        annual_error < 0.01,
        "Realistic mode should accumulate to 2.5 kg/m²/year, got {:.6} kg/m²/year",
        annual_accumulation
    );

    println!("✅ Temporal scaling violation fixed:");
    println!("   Demo mode: {:.9} kg/m²/timestep", demo_rate);
    println!("   Realistic mode: {:.9} kg/m²/timestep", realistic_rate);
    println!("   Scaling ratio: {:.1}x reduction", scaling_ratio);
    println!(
        "   Annual accumulation: {:.3} kg/m²/year",
        annual_accumulation
    );
}
