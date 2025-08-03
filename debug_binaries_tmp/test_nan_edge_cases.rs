// ABOUTME: Comprehensive test for NaN/infinity edge cases in layered terrain generation
// ABOUTME: Tests extreme configurations and validates defensive programming measures

use sim_protoype::worldgen::{TectonicConfig, TectonicGenerator, TerrainGenerator};

fn main() {
    println!("=== Testing NaN/Infinity Edge Cases ===\n");

    // Test extreme configurations that could trigger mathematical issues
    let extreme_configs = vec![
        (
            "Zero Everything",
            TectonicConfig {
                num_plates: 1,
                surface_detail: 0.0,
                mountain_scale: 0.0,
                ocean_depth_scale: 0.0,
                continental_roughness: 0.0,
                oceanic_roughness: 0.0,
                detail_persistence: 0.0,
                tectonic_influence: 0.0,
                coastal_blending: 0.0,

                // Disable geological evolution for testing
                enable_geological_evolution: false,
                geological_evolution_config: None,
            },
        ),
        (
            "Maximum Values",
            TectonicConfig {
                num_plates: 20,
                surface_detail: 2.0,
                mountain_scale: 5.0,
                ocean_depth_scale: 5.0,
                continental_roughness: 1.0,
                oceanic_roughness: 1.0,
                detail_persistence: 1.0,
                tectonic_influence: 1.0,
                coastal_blending: 200.0,

                // Disable geological evolution for testing
                enable_geological_evolution: false,
                geological_evolution_config: None,
            },
        ),
        (
            "Tiny Blending Distance",
            TectonicConfig {
                coastal_blending: 0.001,
                surface_detail: 1.0,
                ..TectonicConfig::default()
            },
        ),
        (
            "Huge Blending Distance",
            TectonicConfig {
                coastal_blending: 1000.0,
                surface_detail: 1.0,
                ..TectonicConfig::default()
            },
        ),
        (
            "Single Plate",
            TectonicConfig {
                num_plates: 1,
                surface_detail: 0.8,
                ..TectonicConfig::default()
            },
        ),
        (
            "Many Plates",
            TectonicConfig {
                num_plates: 50,
                surface_detail: 0.8,
                ..TectonicConfig::default()
            },
        ),
    ];

    for (name, config) in extreme_configs {
        println!("Testing extreme configuration: {}", name);
        test_extreme_configuration(&config);
        println!();
    }

    // Test various map sizes that could cause scaling issues
    test_various_map_sizes();
}

fn test_extreme_configuration(config: &TectonicConfig) {
    let generator = TectonicGenerator::new(12345);

    // Test multiple map sizes
    let sizes = vec![(32, 32), (64, 64), (128, 64)];

    for (width, height) in sizes {
        println!("  Testing {}x{} map...", width, height);

        let heightmap = generator.generate(width, height, config);

        let mut issues = Vec::new();
        let mut min_val = f32::INFINITY;
        let mut max_val = f32::NEG_INFINITY;
        let mut finite_count = 0;

        for (y, row) in heightmap.iter().enumerate() {
            for (x, &value) in row.iter().enumerate() {
                if value.is_nan() {
                    issues.push(format!("NaN at ({}, {})", x, y));
                } else if value.is_infinite() {
                    issues.push(format!("Infinity at ({}, {}): {}", x, y, value));
                } else {
                    finite_count += 1;
                    min_val = min_val.min(value);
                    max_val = max_val.max(value);
                }
            }
        }

        let total_cells = width * height;

        if issues.is_empty() {
            println!("    ✅ All {} values are finite", total_cells);
            if finite_count > 0 {
                println!("    Range: {:.6} to {:.6}", min_val, max_val);
            }
        } else {
            println!("    ❌ {} issues found:", issues.len());
            for issue in issues.iter().take(3) {
                println!("      {}", issue);
            }
            if issues.len() > 3 {
                println!("      ... and {} more", issues.len() - 3);
            }
        }
    }
}

fn test_various_map_sizes() {
    println!("Testing various map sizes for scaling robustness:");

    let config = TectonicConfig::default();
    let generator = TectonicGenerator::new(42);

    let sizes = vec![
        (8, 8),     // Very small
        (16, 16),   // Small
        (64, 32),   // Rectangular
        (128, 128), // Square medium
        (256, 128), // Large rectangular
        (512, 256), // Very large
    ];

    for (width, height) in sizes {
        println!("  Testing {}x{} map...", width, height);

        let start_time = std::time::Instant::now();
        let heightmap = generator.generate(width, height, &config);
        let generation_time = start_time.elapsed();

        // Validate the result
        let mut all_finite = true;
        let mut min_val = f32::INFINITY;
        let mut max_val = f32::NEG_INFINITY;

        for row in &heightmap {
            for &value in row {
                if !value.is_finite() {
                    all_finite = false;
                    break;
                } else {
                    min_val = min_val.min(value);
                    max_val = max_val.max(value);
                }
            }
            if !all_finite {
                break;
            }
        }

        if all_finite {
            println!(
                "    ✅ Generated in {:.2}ms, range: {:.3} to {:.3}",
                generation_time.as_millis(),
                min_val,
                max_val
            );
        } else {
            println!("    ❌ Contains non-finite values");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_coastal_blending_no_nan() {
        let config = TectonicConfig {
            coastal_blending: 0.0,
            surface_detail: 1.0,
            ..TectonicConfig::default()
        };

        let generator = TectonicGenerator::new(12345);
        let heightmap = generator.generate(32, 32, &config);

        for row in &heightmap {
            for &value in row {
                assert!(value.is_finite(), "Value should be finite, got: {}", value);
            }
        }
    }

    #[test]
    fn test_extreme_values_no_nan() {
        let config = TectonicConfig {
            surface_detail: 2.0,
            mountain_scale: 10.0,
            ocean_depth_scale: 10.0,
            continental_roughness: 1.0,
            oceanic_roughness: 1.0,
            coastal_blending: 1000.0,
            ..TectonicConfig::default()
        };

        let generator = TectonicGenerator::new(12345);
        let heightmap = generator.generate(64, 64, &config);

        for row in &heightmap {
            for &value in row {
                assert!(value.is_finite(), "Value should be finite, got: {}", value);
                assert!(
                    value >= -10.0 && value <= 10.0,
                    "Value should be in reasonable bounds, got: {}",
                    value
                );
            }
        }
    }

    #[test]
    fn test_single_plate_no_nan() {
        let config = TectonicConfig {
            num_plates: 1,
            surface_detail: 0.8,
            ..TectonicConfig::default()
        };

        let generator = TectonicGenerator::new(12345);
        let heightmap = generator.generate(32, 32, &config);

        for row in &heightmap {
            for &value in row {
                assert!(value.is_finite(), "Value should be finite, got: {}", value);
            }
        }
    }
}
