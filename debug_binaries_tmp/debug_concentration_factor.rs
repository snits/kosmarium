use sim_protoype::engine::{
    core::{heightmap::HeightMap, world_scale::WorldScale},
    physics::{drainage::DrainageNetwork, water::WaterLayer},
    sim::WaterFlowSystem,
};
use sim_protoype::worldgen::{DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator};

/// Test the new concentration factor system for continental-scale drainage
fn main() {
    println!("Testing Concentration Factor Water Flow System");
    println!("==============================================\n");

    // Test at Jerry's problematic size: 1024x512 at 32km/pixel
    let width = 1024;
    let height = 512;
    println!("Continental Scale Test: {}x{} at 32km/pixel", width, height);
    println!(
        "Physical dimensions: {:.0}km x {:.0}km",
        width * 32,
        height * 32
    );

    // Create WorldScale for continental scale (32km/pixel)
    let world_scale = WorldScale::from_total_size(
        32000.0 * width as f32,
        32000.0 * height as f32,
        width,
        height,
    );

    // Generate terrain
    let config = DiamondSquareConfig::default();
    let mut generator = DiamondSquareGenerator::from_config(config, 1234);
    let heightmap = generator.generate_heightmap(width, height);

    println!("Generated heightmap with {} cells", width * height);

    // Create water system
    let mut water_system = WaterFlowSystem::new_for_scale(&world_scale);
    let mut water_layer = WaterLayer::new(width, height);

    // Create drainage network - this is required for concentration factor
    println!("Creating drainage network...");
    let drainage_network = DrainageNetwork::from_heightmap(&heightmap, &world_scale);
    let stats = drainage_network.get_statistics();
    println!("Drainage network stats:");
    println!("  Max accumulation: {:.2}", stats.max_accumulation);
    println!("  Mean accumulation: {:.2}", stats.mean_accumulation);
    println!("  River cells: {}", stats.river_cells);

    // Test several ticks to see if boundary drainage works
    println!("\n=== Water Flow Test (with Concentration Factor) ===");

    for tick in 1..=10 {
        // Reset metrics for this tick
        water_system.reset_drainage_metrics();

        // Add some initial water to test boundary flow
        if tick == 1 {
            // Add water near the center to give it time to flow
            let center_x = width / 2;
            let center_y = height / 2;
            for dy in -20..=20 {
                for dx in -20..=20 {
                    let x = center_x as i32 + dx;
                    let y = center_y as i32 + dy;
                    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                        water_layer.depth.set(x as usize, y as usize, 0.1); // Initial water depth
                    }
                }
            }
        }

        // Update water flow with the new concentration factor system
        water_system.update_water_flow_with_drainage(
            &heightmap,
            &mut water_layer,
            &drainage_network,
        );

        // Get drainage metrics
        let metrics = water_system.get_drainage_metrics();

        // Total water in system
        let mut total_water = 0.0;
        let mut max_velocity: f32 = 0.0;
        let mut velocities_above_threshold = 0;
        for y in 0..height {
            for x in 0..width {
                total_water += water_layer.depth.get(x, y);
                let (vx, vy) = water_layer.velocity.get(x, y);
                let vel_mag = (vx * vx + vy * vy).sqrt();
                max_velocity = max_velocity.max(vel_mag);
                if vel_mag > 0.0001 {
                    // 0.1 mm/s
                    velocities_above_threshold += 1;
                }
            }
        }

        println!(
            "Tick {}: total_water={:.6}, boundary_outflow={:.6}, max_velocity={:.6}, cells_with_flow={}",
            tick,
            total_water,
            metrics.boundary_outflow_rate,
            max_velocity,
            velocities_above_threshold
        );

        if metrics.boundary_outflow_rate > 0.0 {
            println!(
                "✅ SUCCESS: Boundary drainage is working! Outflow = {:.6}",
                metrics.boundary_outflow_rate
            );
            break;
        }
    }

    // Skip detailed validation for now
    println!("\nSkipping detailed validation for initial test...");

    // Get final drainage metrics
    let final_metrics = water_system.get_drainage_metrics();
    println!("\nFinal Drainage Metrics:");
    println!(
        "  Total boundary outflow: {:.8}",
        final_metrics.total_boundary_outflow
    );
    println!(
        "  Boundary outflow rate: {:.8}",
        final_metrics.boundary_outflow_rate
    );
    println!(
        "  Drainage efficiency: {:.4}%",
        final_metrics.drainage_efficiency * 100.0
    );

    if final_metrics.boundary_outflow_rate > 0.0 {
        println!("\n🎉 CONCENTRATION FACTOR SUCCESS!");
        println!("Water is now draining at continental scales!");
        println!("The mathematical solution has fixed the 'aquarium effect'!");
    } else {
        println!("\n❌ Still no boundary drainage");
        println!("Need to investigate concentration factor parameters");
    }
}
