// ABOUTME: Debug the actual flow threshold values being calculated
// ABOUTME: Check if thresholds are too high to allow any boundary flow

use kosmarium::engine::core::scale::{DetailLevel, ScaleAware, WorldScale};
use kosmarium::engine::physics::worldgen::{
    TectonicConfig, TectonicGenerator, TerrainGenerator,
};
use kosmarium::engine::sim::Simulation;

fn main() {
    println!("FLOW THRESHOLD DEBUG ANALYSIS");
    println!("Checking if flow thresholds are preventing boundary drainage");

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
    println!("Meters per pixel: {:.0}", world_scale.meters_per_pixel());

    // Generate terrain
    let generator = TectonicGenerator::new(12345);
    let mut config = TectonicConfig::default();
    config.enable_geological_evolution = false;
    let scaled_config = config.derive_parameters(&world_scale);
    let heightmap = generator.generate(resolution, resolution, &scaled_config);

    // Create simulation
    let simulation = Simulation::_new_with_scale(heightmap, world_scale.clone());

    // Extract the key parameters
    let evaporation_threshold = simulation.water_system.evaporation_threshold;
    let effective_rainfall_rate = simulation.water_system.effective_rainfall_rate;
    let evaporation_rate = simulation.water_system.parameters.evaporation_rate;

    println!("\nWater System Parameters:");
    println!("  Evaporation threshold: {:.8}", evaporation_threshold);
    println!("  Effective rainfall rate: {:.8}", effective_rainfall_rate);
    println!("  Evaporation rate: {:.8}", evaporation_rate);

    // Calculate the scale-aware flow threshold manually (from sim.rs lines 844-846)
    let meters_per_pixel = world_scale.meters_per_pixel();
    let physical_threshold = 0.001 * meters_per_pixel as f32 / 1000.0; // 1mm depth scaled to pixel
    let flow_threshold = 1e-8_f32.max(physical_threshold); // Ensure non-zero minimum

    println!("\nFlow Threshold Calculation:");
    println!("  Meters per pixel: {:.0}", meters_per_pixel);
    println!(
        "  Physical threshold (1mm scaled): {:.8}",
        physical_threshold
    );
    println!("  Final flow threshold: {:.8}", flow_threshold);

    // Estimate typical flow amounts
    let typical_water_depth = effective_rainfall_rate; // Water after one rainfall event
    let max_velocity = 0.5; // From move_water_with_boundaries
    let typical_flow_amount = typical_water_depth * max_velocity;

    println!("\nFlow Analysis:");
    println!(
        "  Typical water depth (1 rainfall): {:.8}",
        typical_water_depth
    );
    println!("  Max velocity: {:.2}", max_velocity);
    println!("  Typical flow amount: {:.8}", typical_flow_amount);
    println!("  Flow threshold: {:.8}", flow_threshold);

    if typical_flow_amount > flow_threshold {
        println!("  ✅ Typical flow EXCEEDS threshold - boundary flow should occur");
    } else {
        println!("  ❌ Typical flow BELOW threshold - boundary flow blocked!");
        println!(
            "     Ratio: {:.2}% of threshold",
            (typical_flow_amount / flow_threshold) * 100.0
        );
    }

    // Calculate how many rainfall events would be needed to exceed threshold
    let water_needed_for_flow = flow_threshold / max_velocity;
    let rainfall_events_needed = water_needed_for_flow / effective_rainfall_rate;

    println!("\nThreshold Requirements:");
    println!(
        "  Water depth needed for flow: {:.8}",
        water_needed_for_flow
    );
    println!("  Rainfall events needed: {:.1}", rainfall_events_needed);

    if rainfall_events_needed > 1000.0 {
        println!(
            "  🚨 PROBLEM: Would need {:.0} rainfall events - practically impossible!",
            rainfall_events_needed
        );
    } else if rainfall_events_needed > 100.0 {
        println!(
            "  ⚠️  ISSUE: Would need {:.0} rainfall events - very slow drainage",
            rainfall_events_needed
        );
    } else {
        println!(
            "  ✅ REASONABLE: {:.0} rainfall events to start boundary flow",
            rainfall_events_needed
        );
    }
}
