// Quick test of the corrected concentration factor for continental drainage
use kosmarium::WaterFlowSystem;
use kosmarium::engine::core::heightmap::HeightMap;
use kosmarium::engine::physics::drainage::DrainageNetwork;
use kosmarium::engine::physics::water::WaterLayer;

fn main() {
    println!("Testing Corrected Concentration Factor (Scale Factor = 5000)");
    println!("=========================================================\n");

    // Small test for quick validation
    let width = 64;
    let height = 32;

    // Generate simple heightmap - higher on left, lower on right for clear flow direction
    let mut heightmap = HeightMap::new(width, height, 0.5);
    for y in 0..height {
        for x in 0..width {
            // Linear gradient from left (high) to right (low)
            let elevation = 1.0 - (x as f32 / width as f32);
            heightmap.set(x, y, elevation);
        }
    }

    // Create drainage network (need to create a temporary WorldScale)
    // For testing purposes, create a WorldScale that represents continental scale
    use kosmarium::engine::core::scale::{DetailLevel, WorldScale};
    let temp_scale = WorldScale::new(
        32.0 * width as f64,
        (width as u32, height as u32),
        DetailLevel::Standard,
    );
    let drainage_network = DrainageNetwork::from_heightmap(&heightmap, &temp_scale);
    let stats = drainage_network.get_statistics();

    println!("Test setup:");
    println!("  Grid: {}x{}", width, height);
    println!("  Max flow accumulation: {:.1}", stats.max_accumulation);

    // Create water system with default scale (it will estimate grid spacing)
    let mut water_system = WaterFlowSystem::from_parameters(Default::default(), &temp_scale);
    let mut water_layer = WaterLayer::new(width, height);

    // Add initial water
    for y in 0..height {
        for x in 0..width {
            water_layer.depth.set(x, y, 0.01); // 1cm initial depth
        }
    }

    println!("\nRunning simulation with corrected concentration factor...");

    // Run one simulation step
    water_system.update_water_flow_with_drainage(
        &mut heightmap,
        &mut water_layer,
        &drainage_network,
    );

    // Check results
    let mut total_water = 0.0;
    let mut max_velocity = 0.0f32;
    let mut cells_with_significant_velocity = 0;

    for y in 0..height {
        for x in 0..width {
            total_water += water_layer.depth.get(x, y);
            let (vx, vy) = water_layer.velocity.get(x, y);
            let vel_mag = (vx * vx + vy * vy).sqrt();
            max_velocity = max_velocity.max(vel_mag);

            if vel_mag > 0.01 {
                // 1 cm/s threshold
                cells_with_significant_velocity += 1;
            }
        }
    }

    // Check boundary outflow by examining edge cells
    let mut boundary_has_flow = false;
    for y in 0..height {
        let (vx, _vy) = water_layer.velocity.get(width - 1, y); // Right edge
        if vx > 0.0 {
            // Flow towards right boundary
            boundary_has_flow = true;
            break;
        }
    }

    println!("\nResults:");
    println!("  Total water: {:.6}", total_water);
    println!("  Max velocity: {:.6} m/s", max_velocity);
    println!(
        "  Cells with significant velocity: {}",
        cells_with_significant_velocity
    );
    println!("  Boundary flow detected: {}", boundary_has_flow);

    if max_velocity > 0.1 {
        println!("\n✅ SUCCESS! Velocities are now in realistic range!");
        if boundary_has_flow {
            println!("✅ BOUNDARY DRAINAGE WORKING!");
            println!("The mathematical fix has resolved the continental drainage problem!");
        }
    } else if max_velocity > 0.001 {
        println!("\n⚠️  Velocities improved but still low");
        println!("Max velocity: {:.6} m/s (need > 0.1 m/s)", max_velocity);
    } else {
        println!("\n❌ Still no significant flow");
    }
}
