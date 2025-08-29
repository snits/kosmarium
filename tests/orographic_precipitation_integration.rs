// ABOUTME: Integration test for orographic precipitation coupling - validates terrain-driven rainfall patterns
// ABOUTME: Tests cross-system physics coupling between terrain, atmospheric flow, and precipitation

use kosmarium::engine::core::{
    heightmap::HeightMap,
    scale::{DetailLevel, WorldScale},
};
use kosmarium::engine::physics::{
    atmospheric_moisture::AtmosphericMoistureSystem,
    climate::ClimateSystem,
    flow_engine::{FlowAlgorithm, FlowEngine},
    orographic_precipitation::{OrographicParameters, OrographicPrecipitationSystem},
    water::WaterLayer,
};

#[test]
fn test_orographic_precipitation_mountain_ridge_scenario() {
    // Create a test scenario with a mountain ridge and westerly winds
    // This should produce enhanced precipitation on windward slopes and rain shadows on leeward sides

    let scale = WorldScale::new(50.0, (10, 10), DetailLevel::Standard);

    // Create mountain ridge terrain: west-to-east elevation profile
    // Low elevation (west) -> mountain ridge (center) -> low elevation (east)
    let heightmap = HeightMap::from_nested(vec![
        vec![0.1, 0.2, 0.4, 0.6, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1], // N
        vec![0.1, 0.2, 0.4, 0.6, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1],
        vec![0.1, 0.2, 0.4, 0.6, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1],
        vec![0.1, 0.2, 0.4, 0.6, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1], // Mountain ridge
        vec![0.1, 0.2, 0.4, 0.6, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1], // (East-west profile)
        vec![0.1, 0.2, 0.4, 0.6, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1],
        vec![0.1, 0.2, 0.4, 0.6, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1],
        vec![0.1, 0.2, 0.4, 0.6, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1],
        vec![0.1, 0.2, 0.4, 0.6, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1],
        vec![0.1, 0.2, 0.4, 0.6, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1], // S
    ]);

    // Initialize atmospheric moisture system
    let mut atmospheric_moisture = AtmosphericMoistureSystem::new_for_scale(&scale, 10, 10);
    let water_layer = WaterLayer::new(10, 10);
    atmospheric_moisture.initialize_from_terrain(&heightmap, &water_layer);

    // Set uniform moderate humidity across the domain
    for x in 0..10 {
        for y in 0..10 {
            atmospheric_moisture
                .surface_moisture
                .set_humidity(x, y, 45.0); // Good moisture content
        }
    }

    // Create flow engine with steady westerly wind (west to east)
    let mut flow_engine = FlowEngine::new(FlowAlgorithm::Gradient, 10, 10, &scale);
    for x in 0..10 {
        for y in 0..10 {
            // Strong westerly wind: adequate for orographic effects
            flow_engine.velocity_field.set_velocity(
                x,
                y,
                kosmarium::engine::core::math::Vec2::new(4.0, 0.0), // 4 m/s eastward
            );
        }
    }

    // Create climate system (needed for orographic system interface)
    let climate_system = ClimateSystem::new_for_scale(&scale);

    // Initialize orographic precipitation system
    let parameters = OrographicParameters::default();
    let mut orographic_system = OrographicPrecipitationSystem::new(parameters);

    // Record initial moisture for conservation check
    let initial_total_moisture = atmospheric_moisture.get_total_moisture();

    // Update orographic system (this applies the terrain-driven precipitation effects)
    orographic_system.update(
        &heightmap,
        &flow_engine,
        &mut atmospheric_moisture,
        &climate_system,
        &scale,
        0.25, // 15-minute time step
    );

    // Verify orographic effects were calculated
    assert!(
        orographic_system.has_active_effects(),
        "Orographic system should have calculated effects"
    );

    let effects = orographic_system.get_effects().unwrap();

    // Test 1: Windward slopes should have enhanced precipitation
    // Western slopes (x=2,3) facing into westerly wind should have multipliers > 1.0
    let windward_enhancement_x2 = effects.get_precipitation_multiplier(2, 5);
    let windward_enhancement_x3 = effects.get_precipitation_multiplier(3, 5);

    println!(
        "Windward enhancement at x=2: {:.3}",
        windward_enhancement_x2
    );
    println!(
        "Windward enhancement at x=3: {:.3}",
        windward_enhancement_x3
    );

    // At least one windward slope should show enhancement
    assert!(
        windward_enhancement_x2 > 1.0 || windward_enhancement_x3 > 1.0,
        "Windward slopes should show precipitation enhancement"
    );

    // Test 2: Mountain peaks should have significant effects
    let peak_effects = effects.get_precipitation_multiplier(4, 5); // Mountain peak
    println!("Peak precipitation effects: {:.3}", peak_effects);
    assert!(
        peak_effects >= 1.0,
        "Mountain peaks should maintain or enhance precipitation"
    );

    // Test 3: Leeward slopes should show rain shadow (reduced precipitation)
    let leeward_shadow_x6 = effects.get_precipitation_multiplier(6, 5);
    let leeward_shadow_x7 = effects.get_precipitation_multiplier(7, 5);

    println!("Leeward effects at x=6: {:.3}", leeward_shadow_x6);
    println!("Leeward effects at x=7: {:.3}", leeward_shadow_x7);

    // At least one leeward location should show rain shadow
    assert!(
        leeward_shadow_x6 < 1.0 || leeward_shadow_x7 < 1.0,
        "Leeward slopes should show rain shadow effects"
    );

    // Test 4: Vertical velocities should show upward motion on windward slopes
    let windward_vertical = effects.get_vertical_velocity(3, 5);
    println!("Windward vertical velocity: {:.3} m/s", windward_vertical);
    assert!(
        windward_vertical >= 0.0,
        "Windward vertical velocity should be non-negative"
    );

    // Test 5: System should show spatial variation in effects
    let mut min_multiplier = f32::INFINITY;
    let mut max_multiplier: f32 = 0.0;

    for x in 0..10 {
        for y in 0..10 {
            let multiplier = effects.get_precipitation_multiplier(x, y);
            min_multiplier = min_multiplier.min(multiplier);
            max_multiplier = max_multiplier.max(multiplier);
        }
    }

    println!(
        "Precipitation multiplier range: {:.3} to {:.3}",
        min_multiplier, max_multiplier
    );
    assert!(
        max_multiplier > min_multiplier,
        "Should have spatial variation in precipitation effects"
    );

    // Test 6: Effects should be physically reasonable
    assert!(
        min_multiplier >= 0.0,
        "Precipitation multipliers should be non-negative"
    );
    assert!(
        max_multiplier <= 10.0,
        "Precipitation multipliers should be within reasonable bounds"
    );

    // Test 7: Moisture conservation check (orographic effects redistribute, don't create)
    let final_total_moisture = atmospheric_moisture.get_total_moisture();
    let moisture_change_percent =
        ((final_total_moisture - initial_total_moisture) / initial_total_moisture * 100.0).abs();

    println!(
        "Moisture conservation: {:.2}% change",
        moisture_change_percent
    );

    // Allow some change due to redistribution but should be conservative
    assert!(
        moisture_change_percent < 50.0,
        "Orographic effects should not dramatically change total moisture"
    );

    println!("✓ Orographic precipitation integration test passed");
    println!(
        "  - Windward enhancement: {:.1}x",
        windward_enhancement_x3.max(windward_enhancement_x2)
    );
    println!(
        "  - Leeward reduction: {:.1}x",
        leeward_shadow_x6.min(leeward_shadow_x7)
    );
    println!("  - Vertical motion: {:.2} m/s", windward_vertical);
    println!(
        "  - Spatial contrast: {:.1}x range",
        max_multiplier / min_multiplier.max(0.1)
    );
}

#[test]
fn test_orographic_effects_no_wind_scenario() {
    // Test that orographic effects are minimal when wind speed is below threshold

    let scale = WorldScale::new(10.0, (5, 5), DetailLevel::Standard);

    // Same mountain terrain as before
    let heightmap = HeightMap::from_nested(vec![
        vec![0.1, 0.3, 0.8, 0.4, 0.1],
        vec![0.1, 0.3, 0.8, 0.4, 0.1],
        vec![0.1, 0.3, 0.8, 0.4, 0.1],
        vec![0.1, 0.3, 0.8, 0.4, 0.1],
        vec![0.1, 0.3, 0.8, 0.4, 0.1],
    ]);

    let mut atmospheric_moisture = AtmosphericMoistureSystem::new_for_scale(&scale, 5, 5);
    let water_layer = WaterLayer::new(5, 5);
    atmospheric_moisture.initialize_from_terrain(&heightmap, &water_layer);

    // Set low wind speeds (below orographic threshold)
    let mut flow_engine = FlowEngine::new(FlowAlgorithm::Gradient, 5, 5, &scale);
    for x in 0..5 {
        for y in 0..5 {
            flow_engine.velocity_field.set_velocity(
                x,
                y,
                kosmarium::engine::core::math::Vec2::new(1.0, 0.0), // 1 m/s (below 2 m/s threshold)
            );
        }
    }

    let climate_system = ClimateSystem::new_for_scale(&scale);
    let parameters = OrographicParameters::default();
    let mut orographic_system = OrographicPrecipitationSystem::new(parameters);

    orographic_system.update(
        &heightmap,
        &flow_engine,
        &mut atmospheric_moisture,
        &climate_system,
        &scale,
        0.1,
    );

    // With low wind, orographic effects should be minimal
    let effects = orographic_system.get_effects().unwrap();

    let mut total_enhancement = 0.0;
    let mut cell_count = 0;

    for x in 0..5 {
        for y in 0..5 {
            let multiplier = effects.get_precipitation_multiplier(x, y);
            total_enhancement += (multiplier - 1.0).abs();
            cell_count += 1;
        }
    }

    let average_deviation = total_enhancement / cell_count as f32;
    println!(
        "Low wind orographic effects - average deviation from 1.0: {:.3}",
        average_deviation
    );

    // Effects should be minimal with low wind
    assert!(
        average_deviation < 0.5,
        "Low wind speeds should produce minimal orographic effects"
    );
}

#[test]
fn test_orographic_precipitation_parameters() {
    // Test that orographic parameters produce reasonable physical behavior

    let parameters = OrographicParameters::default();

    // Test physical constants are reasonable
    assert!(
        parameters.lifting_condensation_level > 0.0
            && parameters.lifting_condensation_level < 5000.0,
        "Lifting condensation level should be reasonable (0-5km)"
    );

    assert!(
        parameters.dry_lapse_rate > 0.0 && parameters.dry_lapse_rate < 0.02,
        "Dry lapse rate should be physically reasonable (~0.01 °C/m)"
    );

    assert!(
        parameters.moist_lapse_rate > 0.0
            && parameters.moist_lapse_rate < parameters.dry_lapse_rate,
        "Moist lapse rate should be less than dry lapse rate"
    );

    assert!(
        parameters.precipitation_efficiency > 0.0 && parameters.precipitation_efficiency <= 1.0,
        "Precipitation efficiency should be between 0 and 1"
    );

    assert!(
        parameters.rain_shadow_factor >= 0.0 && parameters.rain_shadow_factor <= 1.0,
        "Rain shadow factor should be between 0 and 1"
    );

    println!("✓ Orographic parameters validation passed");
}
