// ABOUTME: Validation script to test OpenGL data safety for layered terrain generation
// ABOUTME: Implements defensive programming patterns and validates all floating point values

use sim_protoype::engine::physics::worldgen::{
    TectonicConfig, TectonicGenerator, TerrainGenerator,
};
use sim_protoype::engine::sim::Simulation;

fn main() {
    println!("=== OpenGL Data Validation for Layered Terrain ===\n");

    // Test various problematic configurations
    let test_configs = vec![
        ("Default", TectonicConfig::default()),
        (
            "Zero Coastal Blending",
            TectonicConfig {
                coastal_blending: 0.0,
                surface_detail: 0.8,
                ..TectonicConfig::default()
            },
        ),
        (
            "High Surface Detail",
            TectonicConfig {
                surface_detail: 1.0,
                continental_roughness: 0.9,
                oceanic_roughness: 0.1,
                ..TectonicConfig::default()
            },
        ),
        (
            "Extreme Blending Distance",
            TectonicConfig {
                coastal_blending: 100.0,
                tectonic_influence: 1.0,
                ..TectonicConfig::default()
            },
        ),
    ];

    for (name, config) in test_configs {
        println!("Testing configuration: {}", name);
        test_opengl_safety(&config);
        println!();
    }

    // Test the complete simulation pipeline
    println!("Testing complete simulation pipeline:");
    test_simulation_pipeline();
}

fn test_opengl_safety(config: &TectonicConfig) {
    let generator = TectonicGenerator::new(42);
    let width = 128;
    let height = 64;

    println!("  Generating {}x{} terrain...", width, height);
    let heightmap = generator.generate(width, height, config);

    // Validate heightmap for OpenGL safety
    let validation = validate_for_opengl(&heightmap);
    print_opengl_validation(&validation);

    // Test color conversion (simulates graphics rendering)
    test_color_conversion(&heightmap);
}

fn test_simulation_pipeline() {
    // Create a heightmap using layered tectonic generation
    let generator = TectonicGenerator::new(12345);
    let config = TectonicConfig::default();
    let heightmap = generator.generate(64, 64, &config);

    // Create simulation (this is what the graphics system uses)
    let simulation = Simulation::new(heightmap);

    println!("  Simulation created successfully");
    println!(
        "  Dimensions: {}x{}",
        simulation.get_width(),
        simulation.get_height()
    );

    // Test accessing elevation data (what graphics rendering does)
    let mut problematic_coords = Vec::new();
    for y in 0..simulation.get_height() {
        for x in 0..simulation.get_width() {
            let elevation = simulation.get_elevation(x, y);
            if !elevation.is_finite() {
                problematic_coords.push((x, y, elevation));
            }
        }
    }

    if problematic_coords.is_empty() {
        println!("  ✅ All elevation values are finite and safe for OpenGL");
    } else {
        println!(
            "  ⚠️  {} problematic elevation values found:",
            problematic_coords.len()
        );
        for (x, y, val) in problematic_coords.iter().take(5) {
            println!("    ({}, {}) = {}", x, y, val);
        }
    }
}

#[derive(Debug)]
struct OpenGLValidation {
    total_values: usize,
    finite_values: usize,
    nan_values: usize,
    positive_inf_values: usize,
    negative_inf_values: usize,
    min_value: f32,
    max_value: f32,
    mean_value: f32,
    opengl_safe: bool,
}

fn validate_for_opengl(heightmap: &[Vec<f32>]) -> OpenGLValidation {
    let mut validation = OpenGLValidation {
        total_values: 0,
        finite_values: 0,
        nan_values: 0,
        positive_inf_values: 0,
        negative_inf_values: 0,
        min_value: f32::INFINITY,
        max_value: f32::NEG_INFINITY,
        mean_value: 0.0,
        opengl_safe: true,
    };

    let mut sum = 0.0;

    for row in heightmap {
        for &value in row {
            validation.total_values += 1;

            if value.is_nan() {
                validation.nan_values += 1;
                validation.opengl_safe = false;
            } else if value.is_infinite() {
                if value.is_sign_positive() {
                    validation.positive_inf_values += 1;
                } else {
                    validation.negative_inf_values += 1;
                }
                validation.opengl_safe = false;
            } else {
                validation.finite_values += 1;
                sum += value;
                validation.min_value = validation.min_value.min(value);
                validation.max_value = validation.max_value.max(value);
            }
        }
    }

    if validation.finite_values > 0 {
        validation.mean_value = sum / validation.finite_values as f32;
    }

    validation
}

fn print_opengl_validation(validation: &OpenGLValidation) {
    println!("  OpenGL Validation Results:");
    println!("    Total values: {}", validation.total_values);
    println!("    Finite values: {}", validation.finite_values);
    println!("    NaN values: {}", validation.nan_values);
    println!("    +Infinity values: {}", validation.positive_inf_values);
    println!("    -Infinity values: {}", validation.negative_inf_values);

    if validation.finite_values > 0 {
        println!(
            "    Value range: {:.6} to {:.6}",
            validation.min_value, validation.max_value
        );
        println!("    Mean value: {:.6}", validation.mean_value);
    }

    if validation.opengl_safe {
        println!("    ✅ SAFE for OpenGL rendering");
    } else {
        println!("    ❌ UNSAFE for OpenGL rendering - contains NaN/infinity values");
    }
}

fn test_color_conversion(heightmap: &[Vec<f32>]) {
    println!("  Testing color conversion (graphics pipeline simulation):");

    let mut conversion_issues = 0;
    let mut sample_count = 0;

    // Sample some values to test color conversion
    for (y, row) in heightmap.iter().enumerate().step_by(8) {
        for (x, &elevation) in row.iter().enumerate().step_by(8) {
            sample_count += 1;

            // Simulate the elevation_to_color function from graphics_render.rs
            let color_result = simulate_elevation_to_color(elevation);

            if !color_result.is_valid() {
                conversion_issues += 1;
                if conversion_issues <= 3 {
                    println!(
                        "    Color conversion issue at ({}, {}): elevation={}, color={:?}",
                        x, y, elevation, color_result
                    );
                }
            }
        }
    }

    if conversion_issues == 0 {
        println!(
            "    ✅ All {} sampled values convert to valid colors",
            sample_count
        );
    } else {
        println!(
            "    ⚠️  {} out of {} sampled values had color conversion issues",
            conversion_issues, sample_count
        );
    }
}

#[derive(Debug)]
struct ColorResult {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl ColorResult {
    fn is_valid(&self) -> bool {
        self.r.is_finite()
            && self.g.is_finite()
            && self.b.is_finite()
            && self.a.is_finite()
            && self.r >= 0.0
            && self.r <= 1.0
            && self.g >= 0.0
            && self.g <= 1.0
            && self.b >= 0.0
            && self.b <= 1.0
            && self.a >= 0.0
            && self.a <= 1.0
    }
}

fn simulate_elevation_to_color(elevation: f32) -> ColorResult {
    // Simulate the elevation_to_color function from graphics_render.rs
    let (r, g, b, a) = match elevation {
        e if e < 0.2 => (0.0, 0.0, 1.0, 1.0), // BLUE - Water
        e if e < 0.4 => (0.5, 0.8, 1.0, 1.0), // SKYBLUE - Coast
        e if e < 0.6 => (0.0, 1.0, 0.0, 1.0), // GREEN - Plains
        e if e < 0.8 => (1.0, 1.0, 0.0, 1.0), // YELLOW - Hills
        _ => (1.0, 0.0, 0.0, 1.0),            // RED - Mountains
    };

    ColorResult { r, g, b, a }
}
