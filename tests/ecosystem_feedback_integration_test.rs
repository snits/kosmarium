// ABOUTME: Integration tests for ecosystem feedback coupling system
// ABOUTME: Validates biome effects on climate and hydrology integration

use sim_prototype::engine::{
    core::scale::{DetailLevel, WorldScale},
    physics::{
        atmospheric_moisture::SurfaceMoistureLayer,
        ecosystem_feedback::{
            BiomeType, EcosystemFeedbackEffects, EcosystemFeedbackParameters,
            EcosystemFeedbackSystem, calculate_leaf_area_index, classify_biome_from_environment,
        },
        flow_engine::FlowEngine,
        temperature::TemperatureField,
        water::WaterLayer,
    },
};

/// Test ecosystem feedback with forest biome cooling
#[test]
fn test_forest_biome_cooling_effect() {
    let width = 15usize;
    let height = 15usize;
    let scale = WorldScale::new(800.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create temperature field with warm conditions
    let mut temperature_field = TemperatureField::new(width, height, 30.0);

    // Create water layer with adequate moisture
    let mut water_layer = WaterLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            water_layer.add_water(x, y, 1.5); // Adequate water for transpiration
        }
    }

    // Create atmospheric moisture layer
    let mut moisture_layer = SurfaceMoistureLayer::new(width, height);

    // Create flow engine
    let flow_engine = FlowEngine::for_climate(width, height, &scale);

    // Initialize ecosystem feedback system
    let mut params = EcosystemFeedbackParameters::default();
    params.temperature_moderation = 3.0; // Enhanced cooling for testing
    let mut ecosystem_system = EcosystemFeedbackSystem::new(params, width, height);

    // Set forest biome in center region
    for x in 5..10 {
        for y in 5..10 {
            ecosystem_system
                .biome_map_mut()
                .set_biome(x, y, BiomeType::Forest);
            ecosystem_system
                .biome_map_mut()
                .set_vegetation_density(x, y, 0.9);
            ecosystem_system.biome_map_mut().set_biomass(x, y, 350.0);
        }
    }

    // Record initial temperature
    let initial_temp = temperature_field.get_temperature(7, 7);

    // Run ecosystem feedback update
    ecosystem_system.update(
        &mut temperature_field,
        &mut water_layer,
        &mut moisture_layer,
        &flow_engine,
        &scale,
        3600.0, // 1 hour timestep
    );

    // Verify forest created cooling effect
    let final_temp = temperature_field.get_temperature(7, 7);
    assert!(
        final_temp < initial_temp,
        "Forest should cool temperature: {} -> {}",
        initial_temp,
        final_temp
    );

    // Verify effects were generated
    assert!(ecosystem_system.has_active_effects());
    let effects = ecosystem_system.get_effects().unwrap();

    // Check temperature modification is negative (cooling)
    let temp_mod = effects.get_temperature_modification(7, 7);
    assert!(
        temp_mod < 0.0,
        "Forest should provide cooling effect: {}",
        temp_mod
    );

    // Check evapotranspiration is significant
    let evapotranspiration = effects.get_evapotranspiration_rate(7, 7);
    assert!(
        evapotranspiration > 2.0,
        "Forest should have high evapotranspiration: {}",
        evapotranspiration
    );

    // Check humidity generation
    let humidity_gen = effects.get_humidity_generation(7, 7);
    assert!(
        humidity_gen > 0.0,
        "Forest should generate humidity: {}",
        humidity_gen
    );
}

/// Test desert biome characteristics
#[test]
fn test_desert_biome_characteristics() {
    let width = 10usize;
    let height = 10usize;
    let scale = WorldScale::new(500.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create systems
    let mut temperature_field = TemperatureField::new(width, height, 35.0);
    let mut water_layer = WaterLayer::new(width, height);
    let mut moisture_layer = SurfaceMoistureLayer::new(width, height);
    let flow_engine = FlowEngine::for_climate(width, height, &scale);

    // Add minimal water (desert conditions)
    for x in 0..width {
        for y in 0..height {
            water_layer.add_water(x, y, 0.1); // Very little water
        }
    }

    // Initialize ecosystem feedback system
    let mut ecosystem_system =
        EcosystemFeedbackSystem::new(EcosystemFeedbackParameters::default(), width, height);

    // Set desert biome
    for x in 0..width {
        for y in 0..height {
            ecosystem_system
                .biome_map_mut()
                .set_biome(x, y, BiomeType::Desert);
            ecosystem_system
                .biome_map_mut()
                .set_vegetation_density(x, y, 0.1);
            ecosystem_system.biome_map_mut().set_biomass(x, y, 15.0);
        }
    }

    // Run ecosystem feedback update
    ecosystem_system.update(
        &mut temperature_field,
        &mut water_layer,
        &mut moisture_layer,
        &flow_engine,
        &scale,
        3600.0,
    );

    let effects = ecosystem_system.get_effects().unwrap();

    // Check desert characteristics
    let center_x = width / 2;
    let center_y = height / 2;

    // Low evapotranspiration
    let evapotranspiration = effects.get_evapotranspiration_rate(center_x, center_y);
    assert!(
        evapotranspiration < 1.0,
        "Desert should have low evapotranspiration: {}",
        evapotranspiration
    );

    // Minimal humidity generation
    let humidity_gen = effects.get_humidity_generation(center_x, center_y);
    assert!(
        humidity_gen < 0.1,
        "Desert should generate minimal humidity: {}",
        humidity_gen
    );

    // High albedo modification
    let albedo = effects.get_albedo_modification(center_x, center_y);
    assert!(albedo > 0.25, "Desert should have high albedo: {}", albedo);

    // Low water retention
    let retention = effects.get_water_retention_enhancement(center_x, center_y);
    assert!(
        retention < 1.5,
        "Desert should have low water retention: {}",
        retention
    );
}

/// Test tropical biome high transpiration
#[test]
fn test_tropical_biome_transpiration() {
    let width = 12usize;
    let height = 12usize;
    let scale = WorldScale::new(600.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create systems with tropical conditions
    let mut temperature_field = TemperatureField::new(width, height, 26.0);
    let mut water_layer = WaterLayer::new(width, height);
    let mut moisture_layer = SurfaceMoistureLayer::new(width, height);
    let flow_engine = FlowEngine::for_climate(width, height, &scale);

    // Abundant water for tropical conditions
    for x in 0..width {
        for y in 0..height {
            water_layer.add_water(x, y, 3.0);
        }
    }

    // Initialize ecosystem feedback system
    let mut ecosystem_system =
        EcosystemFeedbackSystem::new(EcosystemFeedbackParameters::default(), width, height);

    // Set tropical biome
    for x in 0..width {
        for y in 0..height {
            ecosystem_system
                .biome_map_mut()
                .set_biome(x, y, BiomeType::Tropical);
            ecosystem_system
                .biome_map_mut()
                .set_vegetation_density(x, y, 1.0);
            ecosystem_system.biome_map_mut().set_biomass(x, y, 450.0);
        }
    }

    // Run ecosystem feedback update
    ecosystem_system.update(
        &mut temperature_field,
        &mut water_layer,
        &mut moisture_layer,
        &flow_engine,
        &scale,
        1800.0, // 30 minutes
    );

    let effects = ecosystem_system.get_effects().unwrap();
    let center_x = width / 2;
    let center_y = height / 2;

    // Check tropical characteristics
    let evapotranspiration = effects.get_evapotranspiration_rate(center_x, center_y);
    assert!(
        evapotranspiration > 4.0,
        "Tropical should have high evapotranspiration: {}",
        evapotranspiration
    );

    let humidity_gen = effects.get_humidity_generation(center_x, center_y);
    assert!(
        humidity_gen > 0.05,
        "Tropical should generate significant humidity: {}",
        humidity_gen
    );

    let temp_mod = effects.get_temperature_modification(center_x, center_y);
    assert!(
        temp_mod < -1.0,
        "Tropical should provide strong cooling: {}",
        temp_mod
    );
}

/// Test vegetation growth and decline dynamics
#[test]
fn test_vegetation_dynamics() {
    let width = 8usize;
    let height = 8usize;
    let scale = WorldScale::new(400.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create systems
    let mut temperature_field = TemperatureField::new(width, height, 20.0);
    let mut water_layer = WaterLayer::new(width, height);
    let mut moisture_layer = SurfaceMoistureLayer::new(width, height);
    let flow_engine = FlowEngine::for_climate(width, height, &scale);

    // Add water for growth
    for x in 0..width {
        for y in 0..height {
            water_layer.add_water(x, y, 1.0);
        }
    }

    // Initialize ecosystem feedback system with enhanced growth
    let mut params = EcosystemFeedbackParameters::default();
    params.growth_rate = 50.0; // Accelerated growth for testing
    let mut ecosystem_system = EcosystemFeedbackSystem::new(params, width, height);

    // Set forest biome with low initial biomass
    for x in 0..width {
        for y in 0..height {
            ecosystem_system
                .biome_map_mut()
                .set_biome(x, y, BiomeType::Forest);
            ecosystem_system
                .biome_map_mut()
                .set_vegetation_density(x, y, 0.3);
            ecosystem_system.biome_map_mut().set_biomass(x, y, 100.0); // Below optimal
        }
    }

    // Record initial state
    let initial_biomass = ecosystem_system.biome_map_mut().get_biomass(4, 4);
    let initial_density = ecosystem_system
        .biome_map_mut()
        .get_vegetation_density(4, 4);

    // Run multiple updates to see growth
    for _ in 0..5 {
        ecosystem_system.update(
            &mut temperature_field,
            &mut water_layer,
            &mut moisture_layer,
            &flow_engine,
            &scale,
            86400.0, // 1 day timestep
        );
    }

    // Check vegetation growth
    let final_biomass = ecosystem_system.biome_map_mut().get_biomass(4, 4);
    let final_density = ecosystem_system
        .biome_map_mut()
        .get_vegetation_density(4, 4);

    assert!(
        final_biomass > initial_biomass,
        "Vegetation should grow under favorable conditions: {} -> {}",
        initial_biomass,
        final_biomass
    );

    assert!(
        final_density > initial_density,
        "Vegetation density should increase: {} -> {}",
        initial_density,
        final_density
    );
}

/// Test water stress effects on vegetation
#[test]
fn test_water_stress_effects() {
    let width = 6usize;
    let height = 6usize;
    let scale = WorldScale::new(300.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create systems
    let mut temperature_field = TemperatureField::new(width, height, 22.0);
    let mut water_layer = WaterLayer::new(width, height);
    let mut moisture_layer = SurfaceMoistureLayer::new(width, height);
    let flow_engine = FlowEngine::for_climate(width, height, &scale);

    // Add very little water (stress conditions)
    for x in 0..width {
        for y in 0..height {
            water_layer.add_water(x, y, 0.05); // Below stress threshold
        }
    }

    // Initialize ecosystem feedback system
    let mut params = EcosystemFeedbackParameters::default();
    params.water_stress_threshold = 0.2; // Higher threshold for testing
    let mut ecosystem_system = EcosystemFeedbackSystem::new(params, width, height);

    // Set forest biome
    for x in 0..width {
        for y in 0..height {
            ecosystem_system
                .biome_map_mut()
                .set_biome(x, y, BiomeType::Forest);
            ecosystem_system
                .biome_map_mut()
                .set_vegetation_density(x, y, 0.8);
            ecosystem_system.biome_map_mut().set_biomass(x, y, 300.0);
        }
    }

    // Run ecosystem feedback update
    ecosystem_system.update(
        &mut temperature_field,
        &mut water_layer,
        &mut moisture_layer,
        &flow_engine,
        &scale,
        3600.0,
    );

    let effects = ecosystem_system.get_effects().unwrap();
    let center_x = width / 2;
    let center_y = height / 2;

    // Check reduced evapotranspiration due to water stress
    let evapotranspiration = effects.get_evapotranspiration_rate(center_x, center_y);
    assert!(
        evapotranspiration < 2.0,
        "Water stress should reduce evapotranspiration: {}",
        evapotranspiration
    );

    // Check reduced humidity generation
    let humidity_gen = effects.get_humidity_generation(center_x, center_y);
    assert!(
        humidity_gen < 0.02,
        "Water stress should reduce humidity generation: {}",
        humidity_gen
    );
}

/// Test biome classification from environmental conditions
#[test]
fn test_biome_classification_logic() {
    // Test desert classification
    assert_eq!(
        classify_biome_from_environment(35.0, 0.1, 200.0),
        BiomeType::Desert
    );

    // Test tundra classification
    assert_eq!(
        classify_biome_from_environment(2.0, 0.4, 800.0),
        BiomeType::Tundra
    );

    // Test wetland classification
    assert_eq!(
        classify_biome_from_environment(18.0, 0.9, 50.0),
        BiomeType::Wetland
    );

    // Test tropical classification
    assert_eq!(
        classify_biome_from_environment(26.0, 0.7, 100.0),
        BiomeType::Tropical
    );

    // Test forest classification
    assert_eq!(
        classify_biome_from_environment(16.0, 0.5, 300.0),
        BiomeType::Forest
    );

    // Test grassland classification
    assert_eq!(
        classify_biome_from_environment(12.0, 0.3, 400.0),
        BiomeType::Grassland
    );
}

/// Test leaf area index calculations
#[test]
fn test_leaf_area_index_calculations() {
    // Test LAI increases with vegetation density
    let forest_lai_low = calculate_leaf_area_index(0.3, BiomeType::Forest);
    let forest_lai_high = calculate_leaf_area_index(0.9, BiomeType::Forest);
    assert!(forest_lai_high > forest_lai_low);

    // Test LAI varies by biome type
    let tropical_lai = calculate_leaf_area_index(1.0, BiomeType::Tropical);
    let desert_lai = calculate_leaf_area_index(1.0, BiomeType::Desert);
    assert!(tropical_lai > desert_lai);

    // Test realistic LAI ranges
    assert!(tropical_lai <= 8.0);
    assert!(desert_lai <= 0.5);
    assert!(forest_lai_high <= 6.0);
}

/// Test ecosystem feedback parameters validation
#[test]
fn test_ecosystem_feedback_parameters() {
    let params = EcosystemFeedbackParameters::default();

    // Verify parameters are in reasonable ranges
    assert!(params.base_evapotranspiration > 0.0 && params.base_evapotranspiration < 20.0);
    assert!(params.temperature_moderation > 0.0 && params.temperature_moderation < 10.0);
    assert!(params.humidity_coefficient > 0.0 && params.humidity_coefficient < 1.0);
    assert!(params.albedo_variation > 0.0 && params.albedo_variation < 1.0);
    assert!(params.moisture_enhancement > 1.0 && params.moisture_enhancement < 5.0);
    assert!(params.growth_rate > 0.0 && params.growth_rate < 100.0);
    assert!(params.water_stress_threshold > 0.0 && params.water_stress_threshold < 1.0);
    assert!(params.temperature_stress_range > 0.0 && params.temperature_stress_range < 50.0);
}

/// Test ecosystem system state management
#[test]
fn test_ecosystem_system_state_management() {
    let width = 5usize;
    let height = 5usize;
    let scale = WorldScale::new(250.0, (width as u32, height as u32), DetailLevel::Standard);

    // Create simple test systems
    let mut temperature_field = TemperatureField::new(width, height, 20.0);
    let mut water_layer = WaterLayer::new(width, height);
    let mut moisture_layer = SurfaceMoistureLayer::new(width, height);
    let flow_engine = FlowEngine::for_climate(width, height, &scale);

    // Add water and set up biome
    for x in 0..width {
        for y in 0..height {
            water_layer.add_water(x, y, 1.0);
        }
    }

    // Initialize ecosystem feedback system
    let mut ecosystem_system =
        EcosystemFeedbackSystem::new(EcosystemFeedbackParameters::default(), width, height);

    // Initially should have no effects
    assert!(!ecosystem_system.has_active_effects());
    assert!(ecosystem_system.get_effects().is_none());

    // After update should have effects
    ecosystem_system.update(
        &mut temperature_field,
        &mut water_layer,
        &mut moisture_layer,
        &flow_engine,
        &scale,
        1.0,
    );

    assert!(ecosystem_system.has_active_effects());
    assert!(ecosystem_system.get_effects().is_some());

    // Effects should be reasonable
    let effects = ecosystem_system.get_effects().unwrap();
    let temp_mod = effects.get_temperature_modification(2, 2);
    let humidity_gen = effects.get_humidity_generation(2, 2);
    let evapotranspiration = effects.get_evapotranspiration_rate(2, 2);

    assert!(temp_mod >= -10.0 && temp_mod <= 10.0);
    assert!(humidity_gen >= 0.0 && humidity_gen <= 1.0);
    assert!(evapotranspiration >= 0.0 && evapotranspiration <= 20.0);
}

/// Test biome type property consistency
#[test]
fn test_biome_type_property_consistency() {
    for &biome in &[
        BiomeType::Desert,
        BiomeType::Grassland,
        BiomeType::Forest,
        BiomeType::Wetland,
        BiomeType::Tundra,
        BiomeType::Tropical,
    ] {
        // All properties should be in valid ranges
        assert!(biome.albedo() >= 0.0 && biome.albedo() <= 1.0);
        assert!(
            biome.evapotranspiration_coefficient() >= 0.0
                && biome.evapotranspiration_coefficient() <= 1.0
        );
        assert!(biome.thermal_regulation() >= 0.0 && biome.thermal_regulation() <= 1.0);
        assert!(biome.moisture_retention() >= 0.0 && biome.moisture_retention() <= 1.0);
    }

    // Verify logical relationships
    assert!(
        BiomeType::Tropical.evapotranspiration_coefficient()
            >= BiomeType::Desert.evapotranspiration_coefficient()
    );
    assert!(BiomeType::Forest.thermal_regulation() >= BiomeType::Desert.thermal_regulation());
    assert!(BiomeType::Wetland.moisture_retention() >= BiomeType::Desert.moisture_retention());
    assert!(BiomeType::Desert.albedo() >= BiomeType::Forest.albedo());
}
