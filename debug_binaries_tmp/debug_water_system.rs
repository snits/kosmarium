// ABOUTME: Debug program to investigate water level issues on large maps
// ABOUTME: Tests rainfall scaling, evaporation rates, and water conservation across different map sizes

use std::env;
use std::path::Path;

// Import the simulation modules - need to add path reference
fn main() {
    // Add the src directory to the module path
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let src_path = current_dir.join("src");

    println!("Debug Water System");
    println!("=================");

    // Test different map sizes with mass-conserving scaling
    test_map_size("Reference (240x120)", 240, 120);
    test_map_size("Medium (480x240)", 480, 240);
    test_map_size("Large (1024x512)", 1024, 512);
    test_map_size("Huge (2048x1024)", 2048, 1024);

    println!("\n=== Water System Analysis ===");

    // Test individual components of the water system
    test_rainfall_scaling();
    test_evaporation_impact();
    test_water_simulation_tick();
}

fn test_map_size(name: &str, width: u32, height: u32) {
    println!("\n{}", name);
    println!("{}", "=".repeat(name.len()));

    let total_cells = width * height;
    let reference_cells = 240 * 120; // 28,800
    let area_ratio = reference_cells as f64 / total_cells as f64;

    // Mass-conserving scaling parameters
    let base_rainfall_rate = 0.002;
    let effective_rainfall_rate = base_rainfall_rate * area_ratio as f32;
    let total_water_per_tick = effective_rainfall_rate * total_cells as f32;
    let reference_total_water = base_rainfall_rate * reference_cells as f32;

    println!("  Dimensions: {}x{} ({} cells)", width, height, total_cells);
    println!("  Area ratio from reference: {:.6}", area_ratio);
    println!("  Base rainfall rate: {:.6}", base_rainfall_rate);
    println!("  Effective rainfall rate: {:.8}", effective_rainfall_rate);
    println!("  Total water per tick: {:.6}", total_water_per_tick);
    println!("  Reference total water: {:.6}", reference_total_water);
    println!(
        "  Water conservation ratio: {:.6}",
        total_water_per_tick / reference_total_water
    );

    // Calculate what happens with evaporation
    let evaporation_rate = 0.001; // Default from WaterFlowParameters
    let net_water_per_cell_per_tick = effective_rainfall_rate * (1.0 - evaporation_rate);

    println!(
        "  After evaporation per cell: {:.8}",
        net_water_per_cell_per_tick
    );

    // Estimate steady-state water level (rainfall input = evaporation output)
    // At steady state: rainfall_rate = water_depth * evaporation_rate
    // So: water_depth = rainfall_rate / evaporation_rate
    let steady_state_depth = effective_rainfall_rate / evaporation_rate;
    println!("  Predicted steady-state depth: {:.6}", steady_state_depth);

    if steady_state_depth < 0.001 {
        println!("  ⚠️  WARNING: Steady-state depth below evaporation threshold (0.001)!");
        println!("      Water will be cleared to 0.0 in apply_evaporation()");
    }
}

fn test_rainfall_scaling() {
    println!("\n=== Rainfall Scaling Analysis ===");

    // Test the actual WorldScale calculation
    println!("Testing actual WorldScale implementation:");

    let test_cases = vec![
        (240, 120, "Reference"),
        (480, 240, "2x larger"),
        (1024, 512, "Jerry's problematic size"),
        (2048, 1024, "4x Jerry's size"),
    ];

    for (width, height, description) in test_cases {
        let total_cells = width * height;
        let reference_cells = 240 * 120;

        // This simulates the calculation in WorldScale::scale_factor_from_reference
        let scale_factor = reference_cells as f64 / total_cells as f64;

        // This simulates the calculation in WaterFlowSystem::calculate_rainfall_rate
        let base_rate = 0.002;
        let effective_rate = base_rate * scale_factor as f32;

        println!(
            "  {} ({}x{}): scale_factor={:.6}, effective_rate={:.8}",
            description, width, height, scale_factor, effective_rate
        );
    }
}

fn test_evaporation_impact() {
    println!("\n=== Evaporation Impact Analysis ===");

    let evaporation_rate = 0.001;
    let evaporation_threshold = 0.001; // From apply_evaporation - depths below this are cleared to 0

    println!("Evaporation rate: {}", evaporation_rate);
    println!("Evaporation threshold: {}", evaporation_threshold);

    // Test what happens to different rainfall rates after evaporation
    let test_rates = vec![
        ("1024x512", 0.002 * (28800.0 / 524288.0) as f32), // Mass-conserving scaled
        ("Reference", 0.002),
        ("Small test", 0.0001),
    ];

    for (name, rate) in test_rates {
        let after_evaporation = rate * (1.0 - evaporation_rate);
        let will_be_cleared = after_evaporation < evaporation_threshold;

        println!(
            "  {}: rate={:.8}, after_evap={:.8}, cleared={}",
            name, rate, after_evaporation, will_be_cleared
        );

        if will_be_cleared {
            println!("    ⚠️  This rainfall rate will result in 0.0 water after evaporation!");
        }
    }
}

fn test_water_simulation_tick() {
    println!("\n=== Water Simulation Tick Analysis ===");

    // Simulate what happens during a tick for large maps
    let large_map_rate = 0.002 * (28800.0 / 524288.0) as f32; // ~0.0001097
    let evaporation_rate = 0.001;
    let evaporation_threshold = 0.001;

    println!("Large map effective rainfall: {:.8}", large_map_rate);

    // Simulate water depth over several ticks
    let mut water_depth = 0.0;

    for tick in 1..=10 {
        // Add rainfall
        water_depth += large_map_rate;

        // Apply evaporation
        water_depth *= 1.0 - evaporation_rate;

        // Check threshold clearing
        if water_depth < evaporation_threshold {
            water_depth = 0.0;
        }

        println!(
            "  Tick {}: depth after rainfall={:.8}, after evaporation={:.8}",
            tick,
            water_depth + large_map_rate,
            water_depth
        );

        if water_depth == 0.0 {
            println!("    💀 Water cleared to 0.0!");
            break;
        }
    }

    // Calculate steady state if it exists
    if large_map_rate > evaporation_threshold * evaporation_rate {
        let steady_state = large_map_rate / evaporation_rate;
        println!(
            "  Theoretical steady state (if no clearing): {:.8}",
            steady_state
        );
    } else {
        println!("  ⚠️  No steady state possible - evaporation clearing dominates!");
    }
}
