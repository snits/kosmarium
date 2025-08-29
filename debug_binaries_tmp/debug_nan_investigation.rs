// ABOUTME: Comprehensive debugging script to identify NaN/infinity sources in layered terrain generation
// ABOUTME: Tests mathematical operations and validates floating point values before OpenGL rendering

use kosmarium::worldgen::{TectonicGenerator, TerrainGenerator, TectonicConfig};
use std::collections::HashMap;

fn main() {
    println!("=== NaN/Infinity Investigation for Layered Tectonic Terrain ===\n");
    
    // Test with various configurations to isolate problematic scenarios
    let test_configs = vec![
        ("Default Config", TectonicConfig::default()),
        ("High Detail", TectonicConfig {
            surface_detail: 1.0,
            continental_roughness: 0.9,
            oceanic_roughness: 0.1,
            ..TectonicConfig::default()
        }),
        ("Extreme Blending", TectonicConfig {
            coastal_blending: 50.0,
            tectonic_influence: 1.0,
            ..TectonicConfig::default()
        }),
        ("Zero Blending", TectonicConfig {
            coastal_blending: 0.0,
            surface_detail: 0.8,
            ..TectonicConfig::default()
        }),
    ];
    
    for (name, config) in test_configs {
        println!("Testing configuration: {}", name);
        test_configuration(&config);
        println!();
    }
}

fn test_configuration(config: &TectonicConfig) {
    let generator = TectonicGenerator::new(12345);
    let width = 64;
    let height = 64;
    
    println!("  Generating {}x{} terrain...", width, height);
    let heightmap = generator.generate(width, height, config);
    
    // Comprehensive validation
    let validation_results = validate_heightmap(&heightmap);
    print_validation_results(&validation_results);
    
    // Test specific mathematical operations that could produce NaN
    test_mathematical_operations(config);
}

#[derive(Debug, Default)]
struct ValidationResults {
    total_cells: usize,
    finite_cells: usize,
    nan_cells: usize,
    infinite_cells: usize,
    negative_infinite_cells: usize,
    min_value: f32,
    max_value: f32,
    mean_value: f32,
    problematic_coordinates: Vec<(usize, usize, f32)>,
}

fn validate_heightmap(heightmap: &[Vec<f32>]) -> ValidationResults {
    let mut results = ValidationResults::default();
    let mut sum = 0.0;
    
    results.min_value = f32::INFINITY;
    results.max_value = f32::NEG_INFINITY;
    
    for (y, row) in heightmap.iter().enumerate() {
        for (x, &value) in row.iter().enumerate() {
            results.total_cells += 1;
            
            if value.is_nan() {
                results.nan_cells += 1;
                results.problematic_coordinates.push((x, y, value));
                println!("    NaN found at ({}, {})", x, y);
            } else if value.is_infinite() {
                if value.is_sign_positive() {
                    results.infinite_cells += 1;
                } else {
                    results.negative_infinite_cells += 1;
                }
                results.problematic_coordinates.push((x, y, value));
                println!("    Infinity found at ({}, {}): {}", x, y, value);
            } else {
                results.finite_cells += 1;
                sum += value;
                results.min_value = results.min_value.min(value);
                results.max_value = results.max_value.max(value);
            }
        }
    }
    
    if results.finite_cells > 0 {
        results.mean_value = sum / results.finite_cells as f32;
    }
    
    results
}

fn print_validation_results(results: &ValidationResults) {
    println!("  Validation Results:");
    println!("    Total cells: {}", results.total_cells);
    println!("    Finite cells: {}", results.finite_cells);
    println!("    NaN cells: {}", results.nan_cells);
    println!("    +Infinity cells: {}", results.infinite_cells);
    println!("    -Infinity cells: {}", results.negative_infinite_cells);
    
    if results.finite_cells > 0 {
        println!("    Value range: {:.6} to {:.6}", results.min_value, results.max_value);
        println!("    Mean value: {:.6}", results.mean_value);
        println!("    Range span: {:.6}", results.max_value - results.min_value);
    }
    
    if !results.problematic_coordinates.is_empty() {
        println!("    ⚠️  {} problematic values found!", results.problematic_coordinates.len());
    } else {
        println!("    ✅ All values are finite and valid for OpenGL");
    }
}

fn test_mathematical_operations(config: &TectonicConfig) {
    println!("  Testing mathematical operations for NaN/infinity sources:");
    
    // Test division operations that could produce NaN
    test_division_operations(config);
    
    // Test square root operations
    test_sqrt_operations();
    
    // Test blending calculations
    test_blending_calculations(config);
    
    // Test elevation factor calculations
    test_elevation_factor_calculations();
}

fn test_division_operations(config: &TectonicConfig) {
    println!("    Division operations:");
    
    // Test coastal blending division
    let test_distances = vec![0.0, 0.1, 1.0, 10.0, f32::INFINITY];
    for distance in test_distances {
        let blend_factor = distance / config.coastal_blending;
        if !blend_factor.is_finite() {
            println!("      ⚠️  blend_factor = {} / {} = {} (problematic)", 
                     distance, config.coastal_blending, blend_factor);
        }
    }
    
    // Test normalization divisions
    let test_ranges = vec![0.0, 0.001, 1.0, f32::INFINITY];
    for range in test_ranges {
        if range > 0.0 {
            let normalized = 1.0 / range;
            if !normalized.is_finite() {
                println!("      ⚠️  normalization = 1.0 / {} = {} (problematic)", range, normalized);
            }
        } else {
            println!("      ⚠️  zero range detected - would cause division by zero");
        }
    }
}

fn test_sqrt_operations() {
    println!("    Square root operations:");
    
    // Test distance calculations
    let test_values = vec![-1.0, 0.0, 1.0, 4.0, f32::INFINITY];
    for value in test_values {
        let sqrt_result = value.sqrt();
        if !sqrt_result.is_finite() {
            println!("      ⚠️  sqrt({}) = {} (problematic)", value, sqrt_result);
        }
    }
    
    // Test magnitude calculations
    let test_vectors = vec![
        (0.0, 0.0),
        (1.0, 1.0),
        (f32::INFINITY, 1.0),
        (1.0, f32::INFINITY),
        (f32::NAN, 1.0),
    ];
    
    for (x, y) in test_vectors {
        let magnitude = (x * x + y * y).sqrt();
        if !magnitude.is_finite() {
            println!("      ⚠️  magnitude of ({}, {}) = {} (problematic)", x, y, magnitude);
        }
    }
}

fn test_blending_calculations(config: &TectonicConfig) {
    println!("    Blending calculations:");
    
    let test_scenarios = vec![
        ("Normal", 0.5, 0.3, true, 5.0),
        ("Zero distance", 0.5, 0.3, true, 0.0),
        ("Infinite distance", 0.5, 0.3, true, f32::INFINITY),
        ("NaN detail", f32::NAN, 0.3, true, 5.0),
        ("Infinite detail", f32::INFINITY, 0.3, true, 5.0),
    ];
    
    for (name, continental_detail, oceanic_detail, is_continental, coastal_distance) in test_scenarios {
        let result = simulate_blend_terrain_detail(
            continental_detail,
            oceanic_detail,
            is_continental,
            coastal_distance,
            config.coastal_blending,
        );
        
        if !result.is_finite() {
            println!("      ⚠️  {} scenario: blend_result = {} (problematic)", name, result);
        }
    }
}

fn test_elevation_factor_calculations() {
    println!("    Elevation factor calculations:");
    
    let test_elevations = vec![
        -2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0, 
        f32::INFINITY, f32::NEG_INFINITY, f32::NAN
    ];
    
    for elevation in test_elevations {
        let factor = simulate_calculate_elevation_detail_factor(elevation);
        if !factor.is_finite() {
            println!("      ⚠️  elevation {} -> factor {} (problematic)", elevation, factor);
        }
    }
}

// Simulate the blend_terrain_detail method to test for NaN production
fn simulate_blend_terrain_detail(
    continental_detail: f32,
    oceanic_detail: f32,
    is_continental: bool,
    coastal_distance: f32,
    blending_distance: f32,
) -> f32 {
    if coastal_distance >= blending_distance {
        if is_continental {
            continental_detail
        } else {
            oceanic_detail
        }
    } else {
        let blend_factor = coastal_distance / blending_distance;
        
        if is_continental {
            continental_detail * blend_factor + oceanic_detail * (1.0 - blend_factor)
        } else {
            oceanic_detail * blend_factor + continental_detail * (1.0 - blend_factor)
        }
    }
}

// Simulate the calculate_elevation_detail_factor method
fn simulate_calculate_elevation_detail_factor(tectonic_elevation: f32) -> f32 {
    if tectonic_elevation > 0.0 {
        1.0 + tectonic_elevation * 0.5
    } else {
        (1.0 + tectonic_elevation * 0.3).max(0.3)
    }
}