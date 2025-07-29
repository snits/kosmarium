// ABOUTME: Verification program to test that the water system fix works for large maps  
// ABOUTME: Demonstrates that water now accumulates properly on 1024x512 maps with MassConserving scaling

mod scale;
mod sim;

use scale::{WorldScale, DetailLevel};
use sim::{Simulation, WaterFlowParameters, WaterFlowSystem, RainfallScaling};

fn main() {
    println!("Testing Water System Fix");
    println!("=======================");
    
    println!("\n=== Before vs After Fix ===");
    test_large_map_water_accumulation();
    
    println!("\n=== Threshold Analysis ===");
    analyze_thresholds();
    
    println!("\n=== Water Accumulation Test ===");
    test_water_accumulation_over_time();
}

fn test_large_map_water_accumulation() {
    // Test the problematic 1024x512 map size
    let heightmap = vec![vec![0.5; 1024]; 512]; // Flat terrain for predictable results
    let world_scale = WorldScale::new(10.0, (1024, 512), DetailLevel::Standard);
    let mut sim = Simulation::new_with_scale(heightmap, world_scale);
    
    println!("Map: 1024x512 ({} cells)", 1024 * 512);
    println!("Effective rainfall rate: {:.8}", sim.water_system.effective_rainfall_rate);
    println!("Evaporation threshold: {:.8}", sim.water_system.evaporation_threshold);
    println!("Evaporation rate: {:.3}", sim.water_system.parameters.evaporation_rate);
    
    // Test water accumulation over several ticks
    let mut tick_results = Vec::new();
    
    for tick in 1..=10 {
        sim.tick();
        let total_water = sim.water.get_total_water();
        let avg_depth = total_water / (1024.0 * 512.0);
        tick_results.push((tick, total_water, avg_depth));
        
        println!("Tick {}: total_water={:.6}, avg_depth={:.8}", tick, total_water, avg_depth);
    }
    
    let final_water = tick_results.last().unwrap().1;
    if final_water > 0.0 {
        println!("✅ SUCCESS: Water accumulates on large maps!");
    } else {
        println!("❌ FAILURE: Water still gets cleared to 0.0");
    }
}

fn analyze_thresholds() {
    let test_cases = vec![
        (240, 120, "Reference"),
        (480, 240, "Medium"),
        (1024, 512, "Large (Jerry's case)"),
        (2048, 1024, "Huge"),
    ];
    
    for (width, height, name) in test_cases {
        let world_scale = WorldScale::new(10.0, (width, height), DetailLevel::Standard);
        let system = WaterFlowSystem::new_for_scale(&world_scale);
        
        let post_evap_rainfall = system.effective_rainfall_rate * (1.0 - system.parameters.evaporation_rate);
        let old_threshold = 0.001; // The problematic hard-coded threshold
        let new_threshold = system.evaporation_threshold;
        
        println!("{} ({}x{}):", name, width, height);
        println!("  Effective rainfall: {:.8}", system.effective_rainfall_rate);
        println!("  Post-evap rainfall: {:.8}", post_evap_rainfall);
        println!("  Old threshold: {:.6} (would clear: {})", old_threshold, post_evap_rainfall < old_threshold);
        println!("  New threshold: {:.8} (would clear: {})", new_threshold, post_evap_rainfall < new_threshold);
        
        if post_evap_rainfall >= new_threshold {
            println!("  ✅ Water can accumulate");
        } else {
            println!("  ❌ Water would still be cleared");
        }
        println!();
    }
}

fn test_water_accumulation_over_time() {
    // Create a large map and run it for many ticks to see steady-state behavior
    let heightmap = vec![vec![0.5; 1024]; 512];
    let world_scale = WorldScale::new(10.0, (1024, 512), DetailLevel::Standard);
    let mut sim = Simulation::new_with_scale(heightmap, world_scale);
    
    println!("Running 1024x512 simulation for 100 ticks...");
    
    let mut sample_points = vec![1, 5, 10, 20, 50, 100];
    let mut sample_index = 0;
    
    for tick in 1..=100 {
        sim.tick();
        
        if sample_index < sample_points.len() && tick == sample_points[sample_index] {
            let total_water = sim.water.get_total_water();
            let avg_depth = total_water / (1024.0 * 512.0);
            println!("  Tick {}: avg_depth={:.8}, total_water={:.4}", tick, avg_depth, total_water);
            sample_index += 1;
        }
    }
    
    // Check if we reached a reasonable steady state
    let final_water = sim.water.get_total_water();
    let expected_steady_state = sim.water_system.effective_rainfall_rate / sim.water_system.parameters.evaporation_rate;
    let total_expected = expected_steady_state * (1024.0 * 512.0);
    
    println!("\nFinal Analysis:");
    println!("  Final total water: {:.4}", final_water);
    println!("  Expected steady-state total: {:.4}", total_expected);
    println!("  Ratio: {:.3}", final_water / total_expected);
    
    if final_water > total_expected * 0.8 {
        println!("  ✅ Reached reasonable steady state");
    } else {
        println!("  ⚠️  May not have reached steady state yet (or other factors involved)");
    }
}