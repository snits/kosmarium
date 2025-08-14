// Test to reproduce Jerry's water accumulation issue at continental scales

use sim_prototype::engine::core::heightmap::HeightMap;
use sim_prototype::engine::core::scale::{DetailLevel, WorldScale};
use sim_prototype::engine::physics::worldgen::{TectonicConfig, TectonicGenerator, TerrainGenerator};
use sim_prototype::engine::physics::water::WaterLayer;
use sim_prototype::engine::sim::{Simulation, WaterFlowSystem};

fn test_scale_water_behavior(scale_km: f64, label: &str) {
    println!("\n=== Testing {} Scale ({:.0}km domain) ===", label, scale_km);
    
    // Create appropriate resolution for the scale
    let resolution = if scale_km < 500.0 { 64 } else { 128 };
    let world_scale = WorldScale::new(scale_km, (resolution, resolution), DetailLevel::Standard);
    
    println!("Domain: {:.0}km x {:.0}km", scale_km, scale_km);
    println!("Resolution: {}x{} pixels", resolution, resolution);
    println!("Meters per pixel: {:.0}", world_scale.meters_per_pixel());
    
    // Generate terrain with consistent configuration
    let generator = TectonicGenerator::new(12345);
    let mut config = TectonicConfig::default();
    config.enable_geological_evolution = false; // Disable for faster testing
    
    // Apply scale-aware parameters
    let scaled_config = config.derive_parameters(&world_scale);
    println!("Tectonic plates: {}", scaled_config.num_plates);
    println!("Coastal blending: {:.1} pixels", scaled_config.coastal_blending);
    
    let heightmap = generator.generate(resolution, resolution, &scaled_config);
    
    // Create water system for this scale
    let water_system = WaterFlowSystem::new_for_scale(&world_scale);
    let mut water = WaterLayer::new(resolution, resolution);
    
    println!("\nWater System Parameters:");
    println!("- Rainfall rate: {:.6}", water_system.effective_rainfall_rate);
    println!("- Drainage threshold: {:.6}", water_system.parameters.flow_speed_threshold);
    println!("- Evaporation rate: {:.6}", water_system.parameters.evaporation_rate);
    
    // Simulate water accumulation for a few timesteps
    let mut sim_heightmap = heightmap.clone();
    
    // Run simulation steps and track water accumulation
    println!("\nSimulation Progress:");
    for step in 1..=20 {
        // Add rainfall
        water_system.add_rainfall(&mut water);
        
        // Move water
        water_system.move_water(&mut water);
        
        // Apply erosion and evaporation
        water_system.apply_erosion(&mut sim_heightmap, &mut water);
        water_system.apply_evaporation(&mut water);
        
        if step % 5 == 0 {
            let total_water = water.get_total_water();
            let mean_water_per_cell = total_water / (resolution * resolution) as f32;
            println!("Step {}: Total water = {:.3}, Mean per cell = {:.6}", 
                    step, total_water, mean_water_per_cell);
        }
    }
    
    // Final analysis
    let final_total_water = water.get_total_water();
    let mean_water_depth = final_total_water / (resolution * resolution) as f32;
    
    println!("\nFinal Water Analysis:");
    println!("- Total water: {:.3}", final_total_water);
    println!("- Mean depth per cell: {:.6}", mean_water_depth);
    
    // Count cells with significant water accumulation
    let mut high_water_cells = 0;
    let mut water_distribution = vec![0; 10]; // Binned distribution
    
    for y in 0..resolution {
        for x in 0..resolution {
            let depth = water.get_water_depth(x, y);
            if depth > 0.1 {
                high_water_cells += 1;
            }
            
            // Bin the water depth for distribution analysis
            let bin = (depth * 10.0).min(9.0) as usize;
            water_distribution[bin] += 1;
        }
    }
    
    println!("- Cells with >0.1 water depth: {} ({:.1}%)", 
             high_water_cells, 
             high_water_cells as f32 / (resolution * resolution) as f32 * 100.0);
    
    println!("Water depth distribution (bins 0.0-0.1, 0.1-0.2, etc.):");
    for (i, count) in water_distribution.iter().enumerate() {
        if *count > 0 {
            println!("  {:.1}-{:.1}: {} cells", i as f32 * 0.1, (i + 1) as f32 * 0.1, count);
        }
    }
}

fn main() {
    println!("Water Accumulation Scale Analysis");
    println!("Testing Jerry's reported issue with excessive water at large scales");
    
    // Test different scales from regional to continental
    test_scale_water_behavior(100.0, "Regional");      // 100km - typical regional model
    test_scale_water_behavior(500.0, "Large Regional"); // 500km - large regional
    test_scale_water_behavior(1000.0, "Small Continental"); // 1000km - small continental
    test_scale_water_behavior(4000.0, "Continental");   // 4000km - full continental (Jerry's problematic scale)
    
    println!("\n=== Analysis Complete ===");
    println!("Look for:");
    println!("1. Increasing water accumulation with scale");
    println!("2. Changes in water distribution patterns");
    println!("3. Scale-dependent parameter effectiveness");
}