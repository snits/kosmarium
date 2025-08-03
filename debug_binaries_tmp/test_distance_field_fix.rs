// ABOUTME: Test script to validate the coastal distance field fix for NaN/infinity issues
// ABOUTME: Demonstrates the problem and validates the proposed solution

use sim_protoype::worldgen::{TectonicConfig, TectonicGenerator, TerrainGenerator};

fn main() {
    println!("=== Testing Distance Field Fix for NaN/Infinity Issues ===\n");

    // Test with a configuration that might produce problematic distance fields
    let config = TectonicConfig {
        coastal_blending: 0.0, // This was causing division by zero
        surface_detail: 0.8,
        ..TectonicConfig::default()
    };

    println!("Testing with coastal_blending = 0.0 (problematic case)");
    test_distance_field_handling(&config);

    // Test with normal configuration
    let normal_config = TectonicConfig::default();
    println!("\nTesting with default configuration");
    test_distance_field_handling(&normal_config);
}

fn test_distance_field_handling(config: &TectonicConfig) {
    let generator = TectonicGenerator::new(12345);
    let width = 32;
    let height = 32;

    println!(
        "  Generating {}x{} terrain with coastal_blending = {}",
        width, height, config.coastal_blending
    );

    let heightmap = generator.generate(width, height, config);

    // Check for problematic values
    let mut nan_count = 0;
    let mut inf_count = 0;
    let mut finite_count = 0;
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;

    for y in 0..heightmap.height() {
        for x in 0..heightmap.width() {
            let value = heightmap.get(x, y);
            if value.is_nan() {
                nan_count += 1;
            } else if value.is_infinite() {
                inf_count += 1;
            } else {
                finite_count += 1;
                min_val = min_val.min(value);
                max_val = max_val.max(value);
            }
        }
    }

    println!("  Results:");
    println!("    Finite values: {}", finite_count);
    println!("    NaN values: {}", nan_count);
    println!("    Infinite values: {}", inf_count);

    if finite_count > 0 {
        println!("    Value range: {:.6} to {:.6}", min_val, max_val);
    }

    if nan_count > 0 || inf_count > 0 {
        println!("    ⚠️  Problematic values detected!");
    } else {
        println!("    ✅ All values are finite and safe for OpenGL");
    }
}

// Test the specific mathematical operations that could cause issues
fn test_blend_factor_calculation() {
    println!("\n=== Testing Blend Factor Calculations ===");

    let test_cases = vec![
        ("Normal case", 5.0, 15.0),
        ("Zero distance", 0.0, 15.0),
        ("Infinite distance", f32::INFINITY, 15.0),
        ("Zero blending distance", 5.0, 0.0),
        ("Both zero", 0.0, 0.0),
    ];

    for (name, distance, blending_distance) in test_cases {
        let blend_factor = if blending_distance > 0.0 {
            distance / blending_distance
        } else {
            1.0 // Safe fallback when blending_distance is zero
        };

        println!(
            "  {}: distance={}, blending_distance={} -> blend_factor={}",
            name, distance, blending_distance, blend_factor
        );

        if !blend_factor.is_finite() {
            println!("    ⚠️  Problematic blend factor!");
        }
    }
}
