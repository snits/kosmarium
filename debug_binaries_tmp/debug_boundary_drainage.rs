// ABOUTME: Quick diagnostic to check actual boundary outflow behavior
// ABOUTME: Verifies if water is being removed from system vs just tracked

use sim_protoype::engine::core::scale::{DetailLevel, ScaleAware, WorldScale};
use sim_protoype::engine::physics::worldgen::{
    TectonicConfig, TectonicGenerator, TerrainGenerator,
};
use sim_protoype::engine::sim::Simulation;

fn main() {
    println!("BOUNDARY DRAINAGE DEBUG ANALYSIS");
    println!("Testing if water is actually being removed vs just tracked");

    // Test continental scale where Jerry sees the problem
    let scale_km = 4096.0;
    let resolution = 128usize;
    let world_scale = WorldScale::new(
        scale_km,
        (resolution as u32, resolution as u32),
        DetailLevel::Standard,
    );

    println!(
        "Domain: {:.0}km x {:.0}km at {} resolution",
        scale_km, scale_km, resolution
    );

    // Generate terrain
    let generator = TectonicGenerator::new(12345);
    let mut config = TectonicConfig::default();
    config.enable_geological_evolution = false;
    let scaled_config = config.derive_parameters(&world_scale);
    let heightmap = generator.generate(resolution, resolution, &scaled_config);

    // Create simulation
    let mut simulation = Simulation::_new_with_scale(heightmap, world_scale);

    println!("\nInitial state:");
    let initial_water = simulation.water.get_total_water();
    println!("  Total water: {:.6}", initial_water);

    // Run 10 timesteps and track water removal
    for step in 1..=10 {
        let pre_tick_water = simulation.water.get_total_water();

        simulation.tick();

        let post_tick_water = simulation.water.get_total_water();
        let water_change = post_tick_water - pre_tick_water;

        let metrics = &simulation.water_system.drainage_metrics;
        let boundary_outflow_this_tick = metrics.boundary_outflow_rate;

        println!(
            "Step {}: Water {:.6} -> {:.6} (change: {:.6}), Boundary outflow: {:.6}",
            step, pre_tick_water, post_tick_water, water_change, boundary_outflow_this_tick
        );

        // The key test: If boundary outflow > 0 but water increases, then tracking is wrong
        if boundary_outflow_this_tick > 0.0 && water_change > 0.0 {
            println!("  🚨 PROBLEM: Boundary outflow tracked but water still increased!");
        }

        if step == 10 {
            println!("\nFinal drainage metrics:");
            println!(
                "  Total rainfall input: {:.6}",
                metrics.total_rainfall_input
            );
            println!("  Total evaporation: {:.6}", metrics.total_evaporation);
            println!(
                "  Total boundary outflow: {:.6}",
                metrics.total_boundary_outflow
            );
            println!(
                "  Current water storage: {:.6}",
                metrics.current_water_storage
            );
            println!("  Mass balance error: {:.8}", metrics.mass_balance_error);

            // Calculate expected vs actual
            let expected_water = initial_water + metrics.total_rainfall_input
                - metrics.total_evaporation
                - metrics.total_boundary_outflow;
            let actual_water = simulation.water.get_total_water();

            println!("\nExpected vs Actual:");
            println!("  Expected final water: {:.6}", expected_water);
            println!("  Actual final water: {:.6}", actual_water);
            println!("  Difference: {:.6}", actual_water - expected_water);

            if (actual_water - expected_water).abs() < 0.001 {
                println!("  ✅ GOOD: Actual matches expected (boundary drainage working)");
            } else {
                println!("  ❌ BAD: Actual doesn't match expected (boundary drainage broken)");
            }
        }
    }
}
