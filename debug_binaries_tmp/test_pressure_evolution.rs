// ABOUTME: Test program to verify temporal integration fix for atmospheric pressure system
// ABOUTME: Monitors pressure evolution over time to ensure gradual changes instead of regeneration
use kosmarium::engine::{
    core::{
        heightmap::HeightMap,
        scale::{DetailLevel, WorldScale},
    },
    sim::Simulation,
};
use std::time::Instant;

fn main() {
    println!("Testing temporal integration fix for atmospheric pressure...\n");

    // Create a small test simulation
    let world_scale = WorldScale::new(1000.0, (50, 50), DetailLevel::Standard);

    // Create a simple heightmap with some variation
    let mut heightmap = HeightMap::new(50, 50, 0.0);
    for y in 0..50 {
        for x in 0..50 {
            // Create some terrain variation
            let center_dist = ((x as f32 - 25.0).powi(2) + (y as f32 - 25.0).powi(2)).sqrt();
            let elevation = (0.5 + 0.3 * (center_dist / 25.0).cos()).max(0.0).min(1.0);
            heightmap.set(x, y, elevation);
        }
    }

    // Create simulation
    let start_time = Instant::now();
    let mut simulation = Simulation::_new_with_scale(heightmap, world_scale);

    println!("Simulation created in {:?}", start_time.elapsed());

    // Sample some pressure values at specific locations
    let sample_points = [(10, 10), (25, 25), (40, 40)];

    println!("Initial pressure values:");
    for (i, &(x, y)) in sample_points.iter().enumerate() {
        let pressure = simulation.get_pressure_at(x, y);
        println!(
            "  Point {}: ({}, {}) = {:.2} Pa ({:.2} kPa)",
            i + 1,
            x,
            y,
            pressure,
            pressure / 1000.0
        );
    }

    // Calculate initial pressure statistics
    let initial_stats = calculate_pressure_stats(&simulation);
    println!("\nInitial pressure statistics:");
    println!(
        "  Range: {:.2} - {:.2} kPa",
        initial_stats.min / 1000.0,
        initial_stats.max / 1000.0
    );
    println!("  Mean: {:.2} kPa", initial_stats.mean / 1000.0);
    println!(
        "  Spread: {:.2} kPa",
        (initial_stats.max - initial_stats.min) / 1000.0
    );

    // Run simulation for several ticks and track pressure changes
    println!("\nRunning temporal evolution test...");
    let mut previous_values = sample_points
        .iter()
        .map(|&(x, y)| simulation.get_pressure_at(x, y))
        .collect::<Vec<_>>();

    for tick in 1..=20 {
        simulation.tick();

        // Check pressure values at sample points
        let current_values = sample_points
            .iter()
            .map(|&(x, y)| simulation.get_pressure_at(x, y))
            .collect::<Vec<_>>();

        // Calculate changes
        let max_change = previous_values
            .iter()
            .zip(&current_values)
            .map(|(prev, curr)| (curr - prev).abs())
            .fold(0.0, f32::max);

        // Calculate current statistics
        let current_stats = calculate_pressure_stats(&simulation);

        println!(
            "Tick {}: Range {:.2}-{:.2} kPa, Mean {:.2} kPa, Max change {:.2} Pa",
            tick,
            current_stats.min / 1000.0,
            current_stats.max / 1000.0,
            current_stats.mean / 1000.0,
            max_change
        );

        // Check for signs of the old bug (sudden uniform values)
        let spread = current_stats.max - current_stats.min;
        if spread < 100.0 {
            // Less than 0.1 kPa spread indicates uniform pressure
            println!(
                "WARNING: Pressure field became nearly uniform (spread: {:.2} Pa)",
                spread
            );
        }

        // Check for pressure clamp hitting (old bug symptom)
        let clamped_count = count_clamped_values(&simulation);
        if clamped_count > 0 {
            println!("WARNING: {} cells hit pressure clamps", clamped_count);
        }

        previous_values = current_values;
    }

    // Final pressure values
    println!("\nFinal pressure values:");
    for (i, &(x, y)) in sample_points.iter().enumerate() {
        let pressure = simulation.get_pressure_at(x, y);
        println!(
            "  Point {}: ({}, {}) = {:.2} Pa ({:.2} kPa)",
            i + 1,
            x,
            y,
            pressure,
            pressure / 1000.0
        );
    }

    let final_stats = calculate_pressure_stats(&simulation);
    println!("\nFinal pressure statistics:");
    println!(
        "  Range: {:.2} - {:.2} kPa",
        final_stats.min / 1000.0,
        final_stats.max / 1000.0
    );
    println!("  Mean: {:.2} kPa", final_stats.mean / 1000.0);
    println!(
        "  Spread: {:.2} kPa",
        (final_stats.max - final_stats.min) / 1000.0
    );

    // Test success criteria
    let initial_spread = initial_stats.max - initial_stats.min;
    let final_spread = final_stats.max - final_stats.min;

    println!("\n=== TEMPORAL INTEGRATION TEST RESULTS ===");
    println!("Initial spread: {:.2} kPa", initial_spread / 1000.0);
    println!("Final spread: {:.2} kPa", final_spread / 1000.0);

    if final_spread > 1000.0 {
        // At least 1 kPa spread maintained
        println!("✓ SUCCESS: Pressure gradients preserved over time");
    } else {
        println!("✗ FAILED: Pressure field became too uniform");
    }

    if final_stats.max < 109000.0 {
        // Well below the 110 kPa clamp
        println!("✓ SUCCESS: No pressure accumulation to clamp limit");
    } else {
        println!("✗ FAILED: Pressure values accumulated near clamp limit");
    }

    println!("Test completed successfully!");
}

struct PressureStats {
    min: f32,
    max: f32,
    mean: f32,
}

fn calculate_pressure_stats(simulation: &Simulation) -> PressureStats {
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    let mut sum = 0.0;
    let mut count = 0;

    for y in 0..50 {
        for x in 0..50 {
            let pressure = simulation.get_pressure_at(x, y);
            min = min.min(pressure);
            max = max.max(pressure);
            sum += pressure;
            count += 1;
        }
    }

    PressureStats {
        min,
        max,
        mean: sum / count as f32,
    }
}

fn count_clamped_values(simulation: &Simulation) -> usize {
    let mut count = 0;
    for y in 0..50 {
        for x in 0..50 {
            let pressure = simulation.get_pressure_at(x, y);
            // Check for both upper and lower clamps
            if pressure >= 109900.0 || pressure <= 50100.0 {
                count += 1;
            }
        }
    }
    count
}
