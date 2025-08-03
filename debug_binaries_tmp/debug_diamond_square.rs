use sim_protoype::engine::physics::worldgen::{
    DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator,
};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let seed = 12345;
    println!(
        "Investigating Diamond-Square (0,0) mountain artifact with seed: {}",
        seed
    );

    let generator = DiamondSquareGenerator::new(seed);
    let config = DiamondSquareConfig {
        initial_corners: [0.3, 0.7, 0.4, 0.6], // Same as main.rs
        roughness: 0.7,
        persistence: 0.6,
        wrap_edges: false,
    };

    // Generate a small map to examine individual values
    let width = 16;
    let height = 16;
    let heightmap = generator.generate(width, height, &config);

    println!("\nElevation values for {}x{} map:", width, height);
    println!("Legend: Values are normalized to [0.0, 1.0] range");
    println!("Expected corner values after normalization depend on min/max of generated terrain");

    // Print top-left corner area in detail
    println!("\nTop-left 8x8 corner (row, col format):");
    for y in 0..8.min(height) {
        print!("Row {}: ", y);
        for x in 0..8.min(width) {
            print!("{:6.3} ", heightmap[y][x]);
        }
        println!();
    }

    // Find min/max values to understand normalization
    let min_val = heightmap
        .iter()
        .flatten()
        .cloned()
        .fold(f32::INFINITY, f32::min);
    let max_val = heightmap
        .iter()
        .flatten()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max);
    let avg_val = heightmap.iter().flatten().cloned().sum::<f32>() / (width * height) as f32;

    println!("\nStatistics:");
    println!("Min elevation: {:.6}", min_val);
    println!("Max elevation: {:.6}", max_val);
    println!("Average elevation: {:.6}", avg_val);
    println!("Range: {:.6}", max_val - min_val);

    // Check (0,0) specifically
    println!("\nPosition (0,0) analysis:");
    println!("Elevation at (0,0): {:.6}", heightmap[0][0]);
    println!(
        "Is (0,0) the maximum? {}",
        heightmap[0][0] >= max_val - 1e-6
    );
    println!(
        "Elevation percentile of (0,0): {:.2}%",
        (heightmap[0][0] - min_val) / (max_val - min_val) * 100.0
    );

    // Check other corners for comparison
    println!("\nCorner comparison:");
    println!("Top-left (0,0): {:.6}", heightmap[0][0]);
    println!(
        "Top-right (0,{}): {:.6}",
        width - 1,
        heightmap[0][width - 1]
    );
    println!(
        "Bottom-left ({},0): {:.6}",
        height - 1,
        heightmap[height - 1][0]
    );
    println!(
        "Bottom-right ({},{}): {:.6}",
        height - 1,
        width - 1,
        heightmap[height - 1][width - 1]
    );

    // Test with different initial corner configurations
    println!("\n=== Testing different initial corner configurations ===");

    let test_configs = vec![
        ("All equal (0.5)", [0.5, 0.5, 0.5, 0.5]),
        ("Original varied", [0.3, 0.7, 0.4, 0.6]),
        ("Zero corners", [0.0, 0.0, 0.0, 0.0]),
        ("Random corners", [0.1, 0.9, 0.2, 0.8]),
    ];

    for (name, corners) in test_configs {
        let test_config = DiamondSquareConfig {
            initial_corners: corners,
            roughness: 0.7,
            persistence: 0.6,
            wrap_edges: false,
        };

        let test_map = generator.generate(8, 8, &test_config);
        let test_max = test_map
            .iter()
            .flatten()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        let corner_val = test_map[0][0];

        println!(
            "{}: (0,0)={:.6}, max={:.6}, is_max={}",
            name,
            corner_val,
            test_max,
            corner_val >= test_max - 1e-6
        );
    }
}
