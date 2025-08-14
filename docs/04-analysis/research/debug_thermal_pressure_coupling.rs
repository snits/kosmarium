// ABOUTME: Debug thermal-pressure coupling issue - 0.024% vs expected 57% variation
// ABOUTME: Analyzes temperature-pressure relationship to determine if values are realistic

use sim_prototype::engine::core::heightmap::HeightMap;
use sim_prototype::engine::core::scale::{DetailLevel, WorldScale};
use sim_prototype::engine::physics::climate::ClimateSystem;
use std::fs::File;
use std::io::Write;

/// Helper function to create test world scale
fn create_test_scale(width: u32, height: u32) -> WorldScale {
    WorldScale::new(10.0, (width, height), DetailLevel::Standard)
}

#[cfg(test)]
mod thermal_pressure_debug {
    use super::*;

    #[test]
    fn debug_thermal_pressure_coupling() -> std::io::Result<()> {
    let mut debug_file = File::create("thermal_pressure_analysis.log")?;
    writeln!(debug_file, "=== THERMAL-PRESSURE COUPLING ANALYSIS ===")?;
    
    // Replicate the exact test case that's failing
    let scale = create_test_scale(100, 50);
    let climate_system = ClimateSystem::new_for_scale(&scale);
    writeln!(debug_file, "Scale: {:?}", scale)?;
    
    // Create flat terrain to isolate thermal effects
    let heightmap = HeightMap::from_nested(vec![vec![0.0; 100]; 50]);
    writeln!(debug_file, "Heightmap: 100x50 flat terrain (elevation = 0.0)")?;

    // Create temperature gradient: warm on left (30°C), cool on right (10°C)
    let mut temp_layer = climate_system.generate_temperature_layer_optimized(&heightmap);
    for y in 0..50 {
        for x in 0..100 {
            let temp_gradient = 30.0 - (x as f32 / 100.0) * 20.0; // 30°C to 10°C
            temp_layer.temperature.set(x, y, temp_gradient);
        }
    }
    
    writeln!(debug_file, "Temperature gradient applied: 30°C (left) → 10°C (right)")?;

    // Generate pressure field from temperature
    let pressure_layer =
        climate_system.generate_pressure_layer_optimized(&temp_layer, &heightmap, &scale);

    // Analyze thermal-pressure relationship at test points
    let warm_side_temp = temp_layer.get_temperature(10, 25);
    let cool_side_temp = temp_layer.get_temperature(90, 25);
    let warm_side_pressure = pressure_layer.get_pressure(10, 25);
    let cool_side_pressure = pressure_layer.get_pressure(90, 25);

    writeln!(debug_file, "\n=== TEST POINT ANALYSIS ===")?;
    writeln!(debug_file, "Warm side (x=10): {:.1}°C, {:.0} Pa", warm_side_temp, warm_side_pressure)?;
    writeln!(debug_file, "Cool side (x=90): {:.1}°C, {:.0} Pa", cool_side_temp, cool_side_pressure)?;

    // Calculate variations
    let temp_difference = (warm_side_temp - cool_side_temp).abs();
    let pressure_difference = (warm_side_pressure - cool_side_pressure).abs();
    
    let temp_ratio = temp_difference / warm_side_temp.abs().max(1.0);
    let pressure_ratio = pressure_difference / warm_side_pressure.abs().max(1.0);

    writeln!(debug_file, "\n=== VARIATION ANALYSIS ===")?;
    writeln!(debug_file, "Temperature difference: {:.1}°C ({:.2}%)", temp_difference, temp_ratio * 100.0)?;
    writeln!(debug_file, "Pressure difference: {:.0} Pa ({:.4}%)", pressure_difference, pressure_ratio * 100.0)?;
    writeln!(debug_file, "Pressure/Temperature ratio: {:.4}", pressure_ratio / temp_ratio)?;

    // Check physics direction
    let physics_correct = warm_side_pressure < cool_side_pressure;
    writeln!(debug_file, "\n=== PHYSICS VALIDATION ===")?;
    writeln!(debug_file, "Physics correct (warm air = lower pressure): {}", physics_correct)?;
    
    if !physics_correct {
        writeln!(debug_file, "⚠️ PHYSICS ERROR: Warm air should have LOWER pressure than cool air")?;
    }

    // Test expectations
    let min_coupling_ratio = temp_ratio * 0.01; // Test requires at least 1% of temp variation
    let max_coupling_ratio = temp_ratio * 2.0;   // Test requires less than 200% of temp variation
    
    writeln!(debug_file, "\n=== TEST EXPECTATION ANALYSIS ===")?;
    writeln!(debug_file, "Expected pressure variation range: {:.4}% to {:.2}%", 
             min_coupling_ratio * 100.0, max_coupling_ratio * 100.0)?;
    writeln!(debug_file, "Actual pressure variation: {:.4}%", pressure_ratio * 100.0)?;
    
    let too_weak = pressure_ratio < min_coupling_ratio;
    let too_strong = pressure_ratio > max_coupling_ratio;
    
    if too_weak {
        writeln!(debug_file, "❌ FAIL: Pressure variation {:.4}% is too weak (< {:.4}%)", 
                 pressure_ratio * 100.0, min_coupling_ratio * 100.0)?;
    } else if too_strong {
        writeln!(debug_file, "❌ FAIL: Pressure variation {:.4}% is too strong (> {:.2}%)", 
                 pressure_ratio * 100.0, max_coupling_ratio * 100.0)?;
    } else {
        writeln!(debug_file, "✅ PASS: Pressure variation is within expected range")?;
    }

    // Detailed grid analysis
    writeln!(debug_file, "\n=== DETAILED GRID ANALYSIS ===")?;
    writeln!(debug_file, "Sample points across temperature gradient:")?;
    
    for sample_x in (0..100).step_by(20) {
        let temp = temp_layer.get_temperature(sample_x, 25);
        let pressure = pressure_layer.get_pressure(sample_x, 25);
        writeln!(debug_file, "  x={:2}: {:.1}°C, {:.0} Pa", sample_x, temp, pressure)?;
    }

    // Climate system parameters
    writeln!(debug_file, "\n=== CLIMATE SYSTEM PARAMETERS ===")?;
    writeln!(debug_file, "Investigating pressure_temperature_coupling parameter...")?;
    
    // Try to get climate parameters (may not be accessible)
    // This is diagnostic only - we'll see what we can access
    
    writeln!(debug_file, "\n=== CONCLUSION ===")?;
    if too_weak {
        writeln!(debug_file, "The thermal-pressure coupling appears to be calibrated too weakly.")?;
        writeln!(debug_file, "Physics direction is correct, but magnitude is insufficient.")?;
        writeln!(debug_file, "Recommended: Increase pressure_temperature_coupling parameter.")?;
    } else {
        writeln!(debug_file, "The thermal-pressure coupling appears to be working correctly.")?;
        writeln!(debug_file, "Issue may be with test expectations rather than physics implementation.")?;
    }

    println!("✓ Thermal-pressure analysis complete - see thermal_pressure_analysis.log");
    Ok(())
    }
}