// ABOUTME: Deep analysis of water flow calculations to find why velocities are zero
// ABOUTME: Investigates elevation gradients, flow direction calculation, and water movement

use crate::engine::core::heightmap::HeightMap;
use crate::engine::core::scale::{DetailLevel, WorldScale};
use crate::engine::sim::Simulation;

/// Detailed flow analysis to understand why velocities are zero
pub fn analyze_flow_calculation() {
    println!("=== FLOW CALCULATION ANALYSIS ===\n");

    // Create a simple test case with obvious slope
    let mut heightmap_data = vec![vec![0.0; 10]; 10];

    // Create a clear slope from left to right
    for y in 0..10 {
        for x in 0..10 {
            heightmap_data[y][x] = x as f32 * 0.1; // Slope from 0.0 to 0.9
        }
    }

    let heightmap = HeightMap::from_nested(heightmap_data);
    let world_scale = WorldScale::new(10.0, (10, 10), DetailLevel::Standard);
    let mut sim = Simulation::_new_with_scale(heightmap, world_scale.clone());

    // Add water to see if it flows
    sim.add_water_at(1, 5, 0.1); // Add water near the top of slope

    println!("Initial state:");
    print_detailed_state(&sim);

    // Manually trigger one water flow update to see what happens
    sim.water_system
        .calculate_flow_directions(&sim.heightmap, &mut sim.water);

    println!("\nAfter flow direction calculation:");
    print_detailed_state(&sim);

    // Check individual flow direction calculation
    analyze_single_cell_flow(&sim, 1, 5);

    // Test on flat terrain to see difference
    println!("\n=== FLAT TERRAIN TEST ===");
    let flat_heightmap = HeightMap::new(5, 5, 0.5); // Perfectly flat
    let mut flat_sim = Simulation::_new_with_scale(flat_heightmap, world_scale);
    flat_sim.add_water_at(2, 2, 0.1);

    println!("Flat terrain initial state:");
    print_detailed_state(&flat_sim);

    flat_sim
        .water_system
        .calculate_flow_directions(&flat_sim.heightmap, &mut flat_sim.water);

    println!("\nFlat terrain after flow calculation:");
    print_detailed_state(&flat_sim);
}

fn print_detailed_state(sim: &Simulation) {
    println!("Heightmap elevations:");
    for y in 0..sim.get_height().min(5) {
        for x in 0..sim.get_width().min(5) {
            print!("{:.2} ", sim.get_elevation(x, y));
        }
        println!();
    }

    println!("Water depths:");
    for y in 0..sim.get_height().min(5) {
        for x in 0..sim.get_width().min(5) {
            print!("{:.4} ", sim.water.depth.get(x, y));
        }
        println!();
    }

    println!("Water velocities:");
    for y in 0..sim.get_height().min(5) {
        for x in 0..sim.get_width().min(5) {
            let (vx, vy) = sim.water.velocity.get(x, y);
            print!("({:.3},{:.3}) ", vx, vy);
        }
        println!();
    }

    println!("Total water: {:.6}", sim.water.get_total_water());
}

fn analyze_single_cell_flow(sim: &Simulation, x: usize, y: usize) {
    println!("\n--- Analyzing cell ({}, {}) ---", x, y);

    let current_elevation = sim.heightmap.get(x, y) + sim.water.depth.get(x, y);
    println!("Current total elevation: {:.4}", current_elevation);

    let flow_rate = sim.water_system.parameters.flow_rate;
    println!("Flow rate parameter: {:.4}", flow_rate);

    let mut steepest_slope = 0.0f32;
    let mut best_direction = (0i32, 0i32);

    // Check all 8 neighbors manually
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < sim.get_width() as i32 && ny >= 0 && ny < sim.get_height() as i32 {
                let nx = nx as usize;
                let ny = ny as usize;

                let neighbor_elevation = sim.heightmap.get(nx, ny) + sim.water.depth.get(nx, ny);
                let slope = current_elevation - neighbor_elevation;

                println!(
                    "  Neighbor ({}, {}): elevation {:.4}, slope {:.4}",
                    nx, ny, neighbor_elevation, slope
                );

                if slope > steepest_slope {
                    steepest_slope = slope;
                    best_direction = (dx, dy);
                }
            }
        }
    }

    println!("Steepest slope: {:.4}", steepest_slope);
    println!(
        "Best direction: ({}, {})",
        best_direction.0, best_direction.1
    );

    if steepest_slope > 0.0 {
        let magnitude =
            ((best_direction.0 * best_direction.0 + best_direction.1 * best_direction.1) as f32)
                .sqrt();
        let normalized_vx = (best_direction.0 as f32 / magnitude) * steepest_slope * flow_rate;
        let normalized_vy = (best_direction.1 as f32 / magnitude) * steepest_slope * flow_rate;

        println!("Direction magnitude: {:.4}", magnitude);
        println!(
            "Expected velocity: ({:.4}, {:.4})",
            normalized_vx, normalized_vy
        );
    }

    let (actual_vx, actual_vy) = sim.water.velocity.get(x, y);
    println!("Actual velocity: ({:.4}, {:.4})", actual_vx, actual_vy);
}

/// Test the evaporation calculation to see if that's where water is disappearing
pub fn analyze_evaporation_loss() {
    println!("\n=== EVAPORATION ANALYSIS ===\n");

    let heightmap = HeightMap::new(5, 5, 0.5);
    let world_scale = WorldScale::new(10.0, (5, 5), DetailLevel::Standard);
    let mut sim = Simulation::_new_with_scale(heightmap, world_scale);

    // Add known amount of water
    let initial_water_per_cell = 0.1;
    for y in 0..5 {
        for x in 0..5 {
            sim.add_water_at(x, y, initial_water_per_cell);
        }
    }

    let total_initial = sim.water.get_total_water();
    println!("Initial total water: {:.6}", total_initial);

    // Check evaporation parameters
    println!(
        "Evaporation rate: {:.6}",
        sim.water_system.parameters.evaporation_rate
    );
    println!(
        "Evaporation threshold: {:.6}",
        sim.water_system.evaporation_threshold
    );
    println!(
        "Effective rainfall rate: {:.6}",
        sim.water_system.effective_rainfall_rate
    );

    // Apply one round of evaporation manually
    println!("\nBefore evaporation:");
    for y in 0..5 {
        for x in 0..5 {
            print!("{:.4} ", sim.water.depth.get(x, y));
        }
        println!();
    }

    // Calculate expected evaporation
    let expected_after_evaporation =
        total_initial * (1.0 - sim.water_system.parameters.evaporation_rate);
    println!(
        "Expected after evaporation: {:.6}",
        expected_after_evaporation
    );

    // Apply manual evaporation - simulate what happens in tick()
    // We can't access private method, so we'll simulate the effect
    for y in 0..5 {
        for x in 0..5 {
            let current_depth = sim.water.depth.get(x, y);
            let new_depth = current_depth * (1.0 - sim.water_system.parameters.evaporation_rate);
            if new_depth < sim.water_system.evaporation_threshold {
                sim.water.depth.set(x, y, 0.0);
            } else {
                sim.water.depth.set(x, y, new_depth);
            }
        }
    }

    let total_after_evaporation = sim.water.get_total_water();
    println!("Actual after evaporation: {:.6}", total_after_evaporation);
    println!(
        "Evaporation loss: {:.6}",
        total_initial - total_after_evaporation
    );

    println!("\nAfter evaporation:");
    for y in 0..5 {
        for x in 0..5 {
            print!("{:.4} ", sim.water.depth.get(x, y));
        }
        println!();
    }
}

/// Test if the temperature-dependent evaporation is causing excessive losses
pub fn analyze_temperature_evaporation() {
    println!("\n=== TEMPERATURE-DEPENDENT EVAPORATION ANALYSIS ===\n");

    let heightmap = HeightMap::new(3, 3, 0.5);
    let world_scale = WorldScale::new(10.0, (3, 3), DetailLevel::Standard);
    let mut sim = Simulation::_new_with_scale(heightmap, world_scale);

    // Add water
    let initial_water = 0.1;
    for y in 0..3 {
        for x in 0..3 {
            sim.add_water_at(x, y, initial_water);
        }
    }

    let total_before = sim.water.get_total_water();
    println!("Water before temperature evaporation: {:.6}", total_before);

    // Check temperature values
    println!("\nTemperature field:");
    for y in 0..3 {
        for x in 0..3 {
            let temp = sim.temperature_layer.get_current_temperature(
                x,
                y,
                sim.climate_system.current_season,
            );
            print!("{:.1}°C ", temp);
        }
        println!();
    }

    // Check evaporation multipliers
    println!("\nEvaporation multipliers:");
    for y in 0..3 {
        for x in 0..3 {
            let temp = sim.temperature_layer.get_current_temperature(
                x,
                y,
                sim.climate_system.current_season,
            );
            let multiplier = sim.climate_system.get_evaporation_multiplier(temp);
            print!("{:.3} ", multiplier);
        }
        println!();
    }

    // Apply temperature-dependent evaporation - simulate what happens in tick()
    // We can't access private method, so we'll simulate the effect
    for y in 0..3 {
        for x in 0..3 {
            let temp = sim.temperature_layer.get_current_temperature(
                x,
                y,
                sim.climate_system.current_season,
            );
            let temp_multiplier = sim.climate_system.get_evaporation_multiplier(temp);
            let effective_evaporation_rate =
                sim.water_system.parameters.evaporation_rate * temp_multiplier;

            let current_depth = sim.water.depth.get(x, y);
            let new_depth = current_depth * (1.0 - effective_evaporation_rate.min(1.0));

            if new_depth < sim.water_system.evaporation_threshold {
                sim.water.depth.set(x, y, 0.0);
            } else {
                sim.water.depth.set(x, y, new_depth);
            }
        }
    }

    let total_after = sim.water.get_total_water();
    println!("\nWater after temperature evaporation: {:.6}", total_after);
    println!(
        "Evaporation loss: {:.6} ({:.1}%)",
        total_before - total_after,
        100.0 * (total_before - total_after) / total_before
    );

    println!("\nWater after temperature evaporation:");
    for y in 0..3 {
        for x in 0..3 {
            print!("{:.4} ", sim.water.depth.get(x, y));
        }
        println!();
    }
}

/// Check the water flow update intervals to see if that's related to the issue
pub fn analyze_flow_update_intervals() {
    println!("\n=== FLOW UPDATE INTERVAL ANALYSIS ===\n");

    let heightmap = HeightMap::new(3, 3, 0.5);
    let world_scale = WorldScale::new(10.0, (3, 3), DetailLevel::Standard);
    let mut sim = Simulation::_new_with_scale(heightmap, world_scale);

    println!("Initial tick count: {}", sim.tick_count);

    // Run several ticks and track when water flow updates occur
    for tick in 0..10 {
        let water_before = sim.water.get_total_water();

        let will_update_water = sim.tick_count % 3 == 0; // WATER_FLOW_UPDATE_INTERVAL = 3
        println!(
            "Tick {}: Water flow will update: {}",
            tick, will_update_water
        );

        sim.tick();

        let water_after = sim.water.get_total_water();
        println!("  Water change: {:.6}", water_after - water_before);
    }
}
