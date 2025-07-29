// ABOUTME: Test case demonstrating the water clearing bug on large maps
// ABOUTME: Shows that MassConserving rainfall + evaporation threshold causes 0.0 water levels

// This would need proper imports in a real test, but demonstrates the issue

fn main() {
    println!("Testing Water Bug on Large Maps");
    println!("===============================");
    
    // Simulate what happens in the actual water system
    test_large_map_water_clearing();
}

fn test_large_map_water_clearing() {
    // Parameters from WaterFlowSystem for 1024x512 map
    let width = 1024;
    let height = 512;
    let total_cells = width * height;
    
    // Mass-conserving scaling calculation
    let reference_cells = 240 * 120; // 28,800
    let scale_factor = reference_cells as f32 / total_cells as f32;
    let base_rainfall_rate = 0.002;
    let effective_rainfall_rate = base_rainfall_rate * scale_factor;
    
    // Evaporation parameters from WaterFlowParameters::default()
    let evaporation_rate = 0.001;
    let evaporation_threshold = 0.001; // Hard-coded in apply_evaporation()
    
    println!("Map size: {}x{} ({} cells)", width, height, total_cells);
    println!("Scale factor: {:.6}", scale_factor);  
    println!("Effective rainfall rate: {:.8}", effective_rainfall_rate);
    println!("Evaporation rate: {:.3}", evaporation_rate);
    println!("Evaporation threshold: {:.3}", evaporation_threshold);
    
    // Simulate a single water cell through one tick
    let mut water_depth = 0.0;
    
    println!("\nSimulating water system tick:");
    
    // Step 1: Add rainfall (from add_rainfall)
    water_depth += effective_rainfall_rate;
    println!("  After rainfall: {:.8}", water_depth);
    
    // Step 2: Apply evaporation (from apply_evaporation)  
    water_depth *= 1.0 - evaporation_rate;
    println!("  After evaporation multiplier: {:.8}", water_depth);
    
    // Step 3: Threshold clearing (from apply_evaporation)
    if water_depth < evaporation_threshold {
        println!("  ⚠️  Water depth {:.8} < threshold {:.3}", water_depth, evaporation_threshold);
        water_depth = 0.0;
        println!("  💀 Water cleared to 0.0!");
    } else {
        println!("  ✅ Water retained: {:.8}", water_depth);
    }
    
    println!("\nFinal water depth: {:.8}", water_depth);
    
    // Calculate what the threshold should be
    let max_post_evap_rainfall = effective_rainfall_rate * (1.0 - evaporation_rate);
    let suggested_threshold = max_post_evap_rainfall * 0.1; // 10% of max rainfall
    
    println!("\nSuggested fixes:");
    println!("  Current threshold: {:.6}", evaporation_threshold);
    println!("  Max post-evaporation rainfall: {:.8}", max_post_evap_rainfall);
    println!("  Suggested threshold: {:.8}", suggested_threshold);
    println!("  Or make threshold scale-aware based on effective rainfall rate");
}