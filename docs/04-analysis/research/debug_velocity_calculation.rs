// ABOUTME: Debug script to analyze velocity calculations for boundary outflow test case
// ABOUTME: Verifies if velocities are correctly calculated from heightmap gradients

use sim_prototype::engine::core::heightmap::HeightMap;
use sim_prototype::engine::sim::Simulation;

fn main() {
    println!("=== VELOCITY CALCULATION DEBUG ===");

    // Create same heightmap as boundary test
    let heightmap = HeightMap::from_nested(vec![
        vec![1.0, 0.8, 0.6, 0.4, 0.2], // Steep slope toward right boundary
        vec![1.0, 0.8, 0.6, 0.4, 0.2],
        vec![1.0, 0.8, 0.6, 0.4, 0.2],
    ]);

    let mut test_sim = Simulation::new(heightmap);

    // Add water that will flow toward boundary - only left side
    for y in 0..3 {
        for x in 0..2 {
            test_sim.water.depth.set(x, y, 0.2); // 20cm water depth
        }
    }

    println!("System parameters:");
    println!(
        "  Flow rate: {}",
        test_sim.water_system.parameters.flow_rate
    );
    println!(
        "  Evaporation threshold: {:.8}",
        test_sim.water_system.evaporation_threshold
    );
    println!(
        "  Effective rainfall rate: {:.8}",
        test_sim.water_system.effective_rainfall_rate
    );

    println!("\n=== BEFORE FLOW CALCULATION ===");
    for y in 0..3 {
        for x in 0..5 {
            let elevation = test_sim.heightmap.get(x, y);
            let water_depth = test_sim.water.depth.get(x, y);
            let total_elev = elevation + water_depth;
            let (vx, vy) = test_sim.water.velocity.get(x, y);
            println!(
                "Cell ({},{}): elev={:.1}, water={:.1}, total={:.1}, vel=({:.6},{:.6})",
                x, y, elevation, water_depth, total_elev, vx, vy
            );
        }
    }

    // Calculate flow directions manually using the water system
    test_sim
        .water_system
        .calculate_flow_directions(&test_sim.heightmap, &mut test_sim.water);

    println!("\n=== AFTER FLOW CALCULATION ===");
    for y in 0..3 {
        for x in 0..5 {
            let elevation = test_sim.heightmap.get(x, y);
            let water_depth = test_sim.water.depth.get(x, y);
            let total_elev = elevation + water_depth;
            let (vx, vy) = test_sim.water.velocity.get(x, y);
            let vel_mag = (vx * vx + vy * vy).sqrt();

            // Calculate expected slope to neighbor
            let mut max_slope: f32 = 0.0;
            if x < 4 {
                let neighbor_elev =
                    test_sim.heightmap.get(x + 1, y) + test_sim.water.depth.get(x + 1, y);
                max_slope = max_slope.max(total_elev - neighbor_elev);
            }

            println!(
                "Cell ({},{}): elev={:.1}, water={:.1}, total={:.1}, vel=({:.6},{:.6}), |v|={:.6}, max_slope={:.2}",
                x, y, elevation, water_depth, total_elev, vx, vy, vel_mag, max_slope
            );
        }
    }

    println!("\n=== FLOW AMOUNT CALCULATION ===");
    for y in 0..3 {
        for x in 0..5 {
            let water_depth = test_sim.water.depth.get(x, y);
            let (vx, vy) = test_sim.water.velocity.get(x, y);
            let velocity_mag = (vx * vx + vy * vy).sqrt();
            let max_velocity = 0.5; // CFL limit
            let flow_amount = water_depth * velocity_mag.min(max_velocity);
            let flow_threshold = 1e-8;

            if water_depth > 0.0 || velocity_mag > 0.0 {
                println!(
                    "Cell ({},{}): depth={:.3}m, vel_mag={:.6}, flow_amount={:.8}m, above_threshold={}",
                    x,
                    y,
                    water_depth,
                    velocity_mag,
                    flow_amount,
                    flow_amount > flow_threshold
                );
            }
        }
    }

    println!("\n=== HYDROLOGICAL EXPECTATIONS ===");

    // Manning's equation calculation for comparison
    let hydraulic_radius: f32 = 0.2; // water depth
    let slope_per_meter: f32 = 0.2 / 2000.0; // 0.2 elevation over 2km
    let manning_n: f32 = 0.035; // natural channels

    let manning_velocity =
        (1.0 / manning_n) * hydraulic_radius.powf(2.0 / 3.0) * slope_per_meter.powf(0.5);
    println!("Manning's velocity: {:.6} m/s", manning_velocity);

    // Convert to cells/tick (assuming 1 tick = 1 second, 2km per cell)
    let manning_cells_per_tick = manning_velocity / 2000.0;
    println!(
        "Manning's in simulation units: {:.8} cells/tick",
        manning_cells_per_tick
    );

    // Expected flow amount
    let expected_flow_amount = 0.2 * manning_cells_per_tick;
    println!("Expected flow amount: {:.8} m", expected_flow_amount);
    println!(
        "Is expected above threshold (1e-8)? {}",
        expected_flow_amount > 1e-8
    );
}
