// ABOUTME: Validation test for temporal scaling Demo mode bit-identical behavior preservation
// ABOUTME: Ensures ecosystem dynamics remain exactly the same after temporal scaling integration

use kosmarium::engine::{
    core::{
        scale::{DetailLevel, WorldScale},
        temporal_scaling::{TemporalMode, TemporalScalingConfig, TemporalScalingService},
    },
    physics::{
        atmospheric_moisture::SurfaceMoistureLayer,
        ecosystem_feedback::{
            BiomeMap, BiomeType, EcosystemFeedbackParameters, EcosystemFeedbackSystem,
        },
        flow_engine::FlowEngine,
        temperature::TemperatureField,
        water::WaterLayer,
    },
};

/// Test structure to capture ecosystem state for bit-identical comparison
#[derive(Debug, Clone)]
struct EcosystemState {
    biomass_values: Vec<Vec<f32>>,
    vegetation_density: Vec<Vec<f32>>,
    total_biomass: f64,
    max_biomass: f32,
    min_biomass: f32,
    avg_vegetation_density: f64,
}

impl EcosystemState {
    /// Capture current ecosystem state from biome map
    fn capture(biome_map: &BiomeMap) -> Self {
        let (width, height) = biome_map.dimensions();
        let mut biomass_values = vec![vec![0.0; height]; width];
        let mut vegetation_density = vec![vec![0.0; height]; width];

        let mut total_biomass = 0.0;
        let mut max_biomass = f32::NEG_INFINITY;
        let mut min_biomass = f32::INFINITY;
        let mut total_vegetation = 0.0;

        for x in 0..width {
            for y in 0..height {
                let biomass = biome_map.get_biomass(x, y);
                let vegetation = biome_map.get_vegetation_density(x, y);

                biomass_values[x][y] = biomass;
                vegetation_density[x][y] = vegetation;

                total_biomass += biomass as f64;
                max_biomass = max_biomass.max(biomass);
                min_biomass = min_biomass.min(biomass);
                total_vegetation += vegetation as f64;
            }
        }

        let total_cells = (width * height) as f64;
        let avg_vegetation_density = total_vegetation / total_cells;

        Self {
            biomass_values,
            vegetation_density,
            total_biomass,
            max_biomass,
            min_biomass,
            avg_vegetation_density,
        }
    }

    /// Compare two ecosystem states for bit-identical equality
    fn is_identical(&self, other: &EcosystemState, tolerance: f64) -> bool {
        // Check dimensions first
        if self.biomass_values.len() != other.biomass_values.len() {
            return false;
        }

        // Check biomass values cell by cell
        for x in 0..self.biomass_values.len() {
            if self.biomass_values[x].len() != other.biomass_values[x].len() {
                return false;
            }
            for y in 0..self.biomass_values[x].len() {
                let diff = (self.biomass_values[x][y] - other.biomass_values[x][y]).abs() as f64;
                if diff > tolerance {
                    eprintln!(
                        "Biomass mismatch at ({}, {}): {} vs {} (diff: {})",
                        x, y, self.biomass_values[x][y], other.biomass_values[x][y], diff
                    );
                    return false;
                }
            }
        }

        // Check vegetation density values cell by cell
        for x in 0..self.vegetation_density.len() {
            for y in 0..self.vegetation_density[x].len() {
                let diff =
                    (self.vegetation_density[x][y] - other.vegetation_density[x][y]).abs() as f64;
                if diff > tolerance {
                    eprintln!(
                        "Vegetation density mismatch at ({}, {}): {} vs {} (diff: {})",
                        x, y, self.vegetation_density[x][y], other.vegetation_density[x][y], diff
                    );
                    return false;
                }
            }
        }

        // Check aggregate statistics
        let total_biomass_diff = (self.total_biomass - other.total_biomass).abs();
        if total_biomass_diff > tolerance {
            eprintln!(
                "Total biomass mismatch: {} vs {} (diff: {})",
                self.total_biomass, other.total_biomass, total_biomass_diff
            );
            return false;
        }

        let avg_vegetation_diff =
            (self.avg_vegetation_density - other.avg_vegetation_density).abs();
        if avg_vegetation_diff > tolerance {
            eprintln!(
                "Average vegetation density mismatch: {} vs {} (diff: {})",
                self.avg_vegetation_density, other.avg_vegetation_density, avg_vegetation_diff
            );
            return false;
        }

        true
    }
}

/// Create a test setup with controlled parameters
fn create_test_setup() -> (
    EcosystemFeedbackSystem,
    TemperatureField,
    WaterLayer,
    SurfaceMoistureLayer,
    FlowEngine,
    WorldScale,
) {
    let width = 20;
    let height = 15;

    // Create deterministic world scale
    let world_scale = WorldScale::new(
        10.0, // 10km domain
        (width as u32, height as u32),
        DetailLevel::Standard,
    );

    // Create ecosystem system with default parameters (including hardcoded growth_rate: 10.0)
    let ecosystem_system =
        EcosystemFeedbackSystem::new(EcosystemFeedbackParameters::default(), width, height);

    // Create temperature field with realistic values
    let mut temperature_field = TemperatureField::new(width, height, 15.0);
    for x in 0..width {
        for y in 0..height {
            // Create temperature gradient from 25°C (tropical) to 5°C (tundra)
            let temp = 25.0 - (y as f32 / height as f32) * 20.0;
            temperature_field.set_temperature(x, y, temp);
        }
    }

    // Create water layer with varied water depth for diverse biomes
    let mut water_layer = WaterLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            // Create water distribution: more water in lowlands, rivers
            let base_water = 0.1;
            let river_water = if x == width / 2 { 0.8 } else { 0.0 }; // Vertical river
            let lake_water = if (x as i32 - width as i32 / 4).abs() < 3
                && (y as i32 - height as i32 / 4).abs() < 2
            {
                0.6
            } else {
                0.0
            }; // Small lake
            let total_water = base_water + river_water + lake_water;
            water_layer.add_water(x, y, total_water);
        }
    }

    // Create surface moisture layer
    let mut moisture_layer = SurfaceMoistureLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            // Moisture correlates with temperature and water
            let water_depth = water_layer.get_water_depth(x, y);
            let moisture = 0.1 + water_depth * 0.3;
            moisture_layer.set_moisture(x, y, moisture);
        }
    }

    // Create flow engine (needed for ecosystem update)
    let flow_engine = FlowEngine::new(
        kosmarium::engine::physics::flow_engine::FlowAlgorithm::Gradient,
        width,
        height,
        &world_scale,
    );

    (
        ecosystem_system,
        temperature_field,
        water_layer,
        moisture_layer,
        flow_engine,
        world_scale,
    )
}

/// Test that captures baseline ecosystem behavior before temporal scaling integration
#[test]
fn test_capture_baseline_ecosystem_behavior() {
    let (
        mut ecosystem_system,
        mut temperature_field,
        mut water_layer,
        mut moisture_layer,
        flow_engine,
        world_scale,
    ) = create_test_setup();

    // Initialize biome map with some variety
    let biome_map = ecosystem_system.biome_map_mut();
    biome_map.set_biome(5, 5, BiomeType::Forest);
    biome_map.set_biome(10, 8, BiomeType::Desert);
    biome_map.set_biome(15, 3, BiomeType::Grassland);
    biome_map.set_biome(8, 12, BiomeType::Wetland);

    // Capture initial state
    let initial_state = EcosystemState::capture(ecosystem_system.biome_map());

    // Run ecosystem update cycle (simulating current behavior without temporal scaling)
    let dt = 0.1; // Standard timestep
    ecosystem_system.update(
        &mut temperature_field,
        &mut water_layer,
        &mut moisture_layer,
        &flow_engine,
        &world_scale,
        dt,
    );

    // Capture state after 1 update
    let post_update_state = EcosystemState::capture(ecosystem_system.biome_map());

    // Verify that ecosystem state changes (growth occurs)
    assert!(!initial_state.is_identical(&post_update_state, 1e-6));
    assert!(
        post_update_state.total_biomass > initial_state.total_biomass,
        "Expected ecosystem growth in favorable conditions"
    );

    println!("Baseline behavior captured:");
    println!(
        "  Initial total biomass: {:.6}",
        initial_state.total_biomass
    );
    println!(
        "  Post-update total biomass: {:.6}",
        post_update_state.total_biomass
    );
    println!(
        "  Biomass growth: {:.6}",
        post_update_state.total_biomass - initial_state.total_biomass
    );
    println!(
        "  Average vegetation density: {:.6}",
        post_update_state.avg_vegetation_density
    );
}

/// Test bit-identical behavior preservation after temporal scaling integration
/// This test will validate that Demo mode produces exactly the same results
/// NOTE: This test will initially fail until temporal scaling is integrated
#[test]
#[ignore] // Remove this once temporal scaling is integrated
fn test_demo_mode_bit_identical_behavior() {
    let (
        mut baseline_system,
        mut baseline_temp,
        mut baseline_water,
        mut baseline_moisture,
        baseline_flow,
        world_scale,
    ) = create_test_setup();
    let (mut demo_system, mut demo_temp, mut demo_water, mut demo_moisture, demo_flow, _) =
        create_test_setup();

    // Set up identical initial conditions
    for x in 0..baseline_system.biome_map().dimensions().0 {
        for y in 0..baseline_system.biome_map().dimensions().1 {
            // Sync biome types
            let biome = baseline_system.biome_map().get_biome(x, y);
            demo_system.biome_map_mut().set_biome(x, y, biome);

            // Sync biomass
            let biomass = baseline_system.biome_map().get_biomass(x, y);
            demo_system.biome_map_mut().set_biomass(x, y, biomass);

            // Sync vegetation density
            let vegetation = baseline_system.biome_map().get_vegetation_density(x, y);
            demo_system
                .biome_map_mut()
                .set_vegetation_density(x, y, vegetation);
        }
    }

    // Create temporal scaling service in Demo mode
    let temporal_service = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Demo,
        ..Default::default()
    });

    // Verify Demo mode settings
    assert_eq!(temporal_service.biological_scaling_factor(), 1.0);
    assert_eq!(temporal_service.mode(), TemporalMode::Demo);

    let dt = 0.1;
    let num_ticks = 1000; // Test over 1000 simulation ticks for comprehensive validation

    // Run baseline simulation (current implementation)
    for _ in 0..num_ticks {
        baseline_system.update(
            &mut baseline_temp,
            &mut baseline_water,
            &mut baseline_moisture,
            &baseline_flow,
            &world_scale,
            dt,
        );
    }

    // Run temporal scaling simulation in Demo mode
    // TODO: This will need to be updated once temporal scaling is integrated
    // For now, simulate what the behavior should be
    for _ in 0..num_ticks {
        // Simulate temporal scaling integration with Demo mode
        // This should call the new temporally-scaled update method
        // demo_system.update_with_temporal_scaling(&mut demo_temp, &mut demo_water, &mut demo_moisture, &demo_flow, &world_scale, dt, &temporal_service);

        // For now, use existing update method (will be identical)
        demo_system.update(
            &mut demo_temp,
            &mut demo_water,
            &mut demo_moisture,
            &demo_flow,
            &world_scale,
            dt,
        );
    }

    // Capture final states
    let baseline_final = EcosystemState::capture(baseline_system.biome_map());
    let demo_final = EcosystemState::capture(demo_system.biome_map());

    // Critical validation: Demo mode must be bit-identical to baseline
    let tolerance = 1e-12; // Extremely strict tolerance for bit-identical requirement
    assert!(
        baseline_final.is_identical(&demo_final, tolerance),
        "Demo mode failed to preserve bit-identical behavior!"
    );

    println!("✓ Demo mode bit-identical validation passed:");
    println!(
        "  Final total biomass: {:.12}",
        baseline_final.total_biomass
    );
    println!("  Max biomass: {:.6}", baseline_final.max_biomass);
    println!("  Min biomass: {:.6}", baseline_final.min_biomass);
    println!(
        "  Average vegetation density: {:.12}",
        baseline_final.avg_vegetation_density
    );
}

/// Performance benchmark test for temporal scaling overhead
#[test]
#[ignore] // Remove this once temporal scaling is integrated  
fn test_demo_mode_performance_overhead() {
    let (
        mut baseline_system,
        mut baseline_temp,
        mut baseline_water,
        mut baseline_moisture,
        baseline_flow,
        world_scale,
    ) = create_test_setup();
    let (mut demo_system, mut demo_temp, mut demo_water, mut demo_moisture, demo_flow, _) =
        create_test_setup();

    let temporal_service = TemporalScalingService::new(TemporalScalingConfig {
        mode: TemporalMode::Demo,
        ..Default::default()
    });

    let dt = 0.1;
    let num_ticks = 1000;

    // Benchmark baseline performance
    let baseline_start = std::time::Instant::now();
    for _ in 0..num_ticks {
        baseline_system.update(
            &mut baseline_temp,
            &mut baseline_water,
            &mut baseline_moisture,
            &baseline_flow,
            &world_scale,
            dt,
        );
    }
    let baseline_duration = baseline_start.elapsed();

    // Benchmark temporal scaling performance
    let demo_start = std::time::Instant::now();
    for _ in 0..num_ticks {
        // TODO: Use temporal scaling update method once implemented
        demo_system.update(
            &mut demo_temp,
            &mut demo_water,
            &mut demo_moisture,
            &demo_flow,
            &world_scale,
            dt,
        );
    }
    let demo_duration = demo_start.elapsed();

    // Calculate performance overhead
    let overhead_ratio = demo_duration.as_secs_f64() / baseline_duration.as_secs_f64();
    let overhead_percent = (overhead_ratio - 1.0) * 100.0;

    println!("Performance benchmark results:");
    println!(
        "  Baseline duration: {:.3}ms",
        baseline_duration.as_secs_f64() * 1000.0
    );
    println!(
        "  Demo mode duration: {:.3}ms",
        demo_duration.as_secs_f64() * 1000.0
    );
    println!("  Overhead: {:.2}%", overhead_percent);

    // Critical performance gate: <5% overhead
    assert!(
        overhead_percent < 5.0,
        "Demo mode performance overhead ({:.2}%) exceeds 5% limit",
        overhead_percent
    );

    // Calculate ticks per 10s baseline
    let ticks_per_10s = (num_ticks as f64) * (10.0 / baseline_duration.as_secs_f64());
    println!("  Baseline performance: {:.0} ticks/10s", ticks_per_10s);

    // Performance gate: >333 ticks/10s (95% of 350 baseline)
    assert!(
        ticks_per_10s > 333.0,
        "Performance ({:.0} ticks/10s) below minimum threshold (333 ticks/10s)",
        ticks_per_10s
    );
}
