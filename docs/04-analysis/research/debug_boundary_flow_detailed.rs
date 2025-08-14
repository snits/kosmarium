// ABOUTME: Detailed step-by-step analysis of boundary outflow during water flow simulation
// ABOUTME: Tracks water mass through each phase: rainfall -> flow -> evaporation

use sim_prototype::engine::core::heightmap::HeightMap;
use sim_prototype::engine::sim::Simulation;

fn main() {
    println!("=== DETAILED BOUNDARY FLOW ANALYSIS ===");

    // Create same test case as boundary outflow test
    let heightmap = HeightMap::from_nested(vec![
        vec![1.0, 0.8, 0.6, 0.4, 0.2], // Steep slope toward right boundary
        vec![1.0, 0.8, 0.6, 0.4, 0.2],
        vec![1.0, 0.8, 0.6, 0.4, 0.2],
    ]);

    let mut test_sim = Simulation::new(heightmap);

    // Add water only to left side
    for y in 0..3 {
        for x in 0..2 {
            test_sim.water.depth.set(x, y, 0.2); // 20cm water depth
        }
    }

    let initial_water = test_sim.water.get_total_water();
    println!("Initial water: {:.6} m³", initial_water);
    println!(
        "Flow threshold: {:.8}",
        test_sim.water_system.evaporation_threshold * 10.0
    );

    println!("\n=== DETAILED FLOW ANALYSIS ===");

    // Track changes over multiple ticks
    for tick in 0..3 {
        println!("\n--- TICK {} ---", tick + 1);

        println!("\nCell analysis before tick:");
        let mut cells_above_flow_threshold = 0;
        let mut total_potential_flow = 0.0f32;
        let flow_threshold = test_sim.water_system.evaporation_threshold * 10.0;

        for y in 0..3 {
            for x in 0..5 {
                let water_depth = test_sim.water.depth.get(x, y);
                let (vx, vy) = test_sim.water.velocity.get(x, y);
                let velocity_mag = (vx * vx + vy * vy).sqrt();
                let max_velocity = 0.5;
                let flow_amount = water_depth * velocity_mag.min(max_velocity);

                if water_depth > 0.0 || flow_amount > 1e-8 {
                    println!(
                        "  Cell ({},{}): depth={:.6}m, vel=({:.6},{:.6}), |v|={:.6}, flow_amt={:.8}m",
                        x, y, water_depth, vx, vy, velocity_mag, flow_amount
                    );

                    if flow_amount > flow_threshold {
                        cells_above_flow_threshold += 1;
                    }
                    total_potential_flow += flow_amount;
                }
            }
        }

        println!("Cells above flow threshold: {}", cells_above_flow_threshold);
        println!("Total potential flow: {:.8} m³", total_potential_flow);

        // Record water before tick components
        let water_before_tick = test_sim.water.get_total_water();

        // Run full tick and analyze results
        test_sim.tick();

        println!("\nTick completed - analyzing results:");

        let water_after_tick = test_sim.water.get_total_water();
        let total_change = water_after_tick - water_before_tick;

        println!(
            "Total change: {:.8} m³ ({:.3}%)",
            total_change,
            (total_change / water_before_tick) * 100.0
        );

        // Show water distribution after tick
        println!("\nWater distribution after tick:");
        for y in 0..3 {
            print!("  Row {}: ", y);
            for x in 0..5 {
                print!("{:.4} ", test_sim.water.depth.get(x, y));
            }
            println!();
        }

        // Analyze why outflow might be limited
        if total_change.abs() < 1e-6 {
            println!("WARNING: Very small water change detected!");

            // Check velocities after tick
            println!("Velocity analysis after tick:");
            for y in 0..3 {
                for x in 0..5 {
                    let (vx, vy) = test_sim.water.velocity.get(x, y);
                    let water_depth = test_sim.water.depth.get(x, y);
                    if (vx.abs() > 1e-6 || vy.abs() > 1e-6) || water_depth > 1e-6 {
                        println!(
                            "  Cell ({},{}) after tick: depth={:.6}, vel=({:.6},{:.6})",
                            x, y, water_depth, vx, vy
                        );
                    }
                }
            }
        }
    }

    let final_water = test_sim.water.get_total_water();
    let total_loss = initial_water - final_water;
    let loss_percentage = (total_loss / initial_water) * 100.0;

    println!("\n=== FINAL RESULTS ===");
    println!("Initial water: {:.6} m³", initial_water);
    println!("Final water: {:.6} m³", final_water);
    println!("Total loss: {:.6} m³ ({:.2}%)", total_loss, loss_percentage);

    if loss_percentage < 1.0 {
        println!(
            "FAILURE: Boundary outflow test requires >1% loss, got {:.2}%",
            loss_percentage
        );
    } else {
        println!(
            "SUCCESS: Boundary outflow test passed with {:.2}% loss",
            loss_percentage
        );
    }
}
