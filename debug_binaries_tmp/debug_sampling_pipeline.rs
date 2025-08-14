use sim_prototype::worldgen::{DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator};

fn main() {
    let seed = 12345;
    println!("=== Diamond-Square Sampling Pipeline Debug ===");
    println!("Investigating the generation → sampling → normalization pipeline");
    println!("Seed: {}", seed);

    let generator = DiamondSquareGenerator::new(seed);
    let config = DiamondSquareConfig {
        initial_corners: [0.3, 0.7, 0.4, 0.6],
        roughness: 0.7,
        persistence: 0.6,
        wrap_edges: false,
    };

    // Test small power-of-2 size first to see raw Diamond-Square output
    println!("\n=== Step 1: Raw Diamond-Square on power-of-2 grid ===");

    // We need to access the internal generate_power_of_two method
    // Since it's private, let's test with dimensions that ARE power-of-2
    let size = 16; // 16x16 is power of 2
    let raw_result = generator.generate(size, size, &config);

    println!("Generated {}x{} heightmap (power of 2)", size, size);

    // Show corner values from raw generation
    println!("Raw corners before any processing:");
    println!("  Top-left (0,0): {:.6}", raw_result[0][0]);
    println!(
        "  Top-right (0,{}): {:.6}",
        size - 1,
        raw_result[0][size - 1]
    );
    println!(
        "  Bottom-left ({},0): {:.6}",
        size - 1,
        raw_result[size - 1][0]
    );
    println!(
        "  Bottom-right ({},{}): {:.6}",
        size - 1,
        size - 1,
        raw_result[size - 1][size - 1]
    );

    // Find raw statistics
    let raw_min = raw_result
        .iter()
        .flatten()
        .cloned()
        .fold(f32::INFINITY, f32::min);
    let raw_max = raw_result
        .iter()
        .flatten()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max);
    let raw_avg = raw_result.iter().sum::<f32>() / (size * size) as f32;

    println!("Raw statistics:");
    println!("  Min: {:.6}", raw_min);
    println!("  Max: {:.6}", raw_max);
    println!("  Avg: {:.6}", raw_avg);
    println!("  Range: {:.6}", raw_max - raw_min);

    println!("\n=== Step 2: Test arbitrary dimensions (triggers sampling) ===");

    // Now test with non-power-of-2 dimensions
    let target_width = 15; // Not power of 2
    let target_height = 10;
    let sampled_result = generator.generate(target_width, target_height, &config);

    println!(
        "Generated {}x{} heightmap (arbitrary dimensions)",
        target_width, target_height
    );

    println!("Sampled corners:");
    println!("  Top-left (0,0): {:.6}", sampled_result[0][0]);
    println!(
        "  Top-right (0,{}): {:.6}",
        target_width - 1,
        sampled_result[0][target_width - 1]
    );
    println!(
        "  Bottom-left ({},0): {:.6}",
        target_height - 1,
        sampled_result[target_height - 1][0]
    );
    println!(
        "  Bottom-right ({},{}): {:.6}",
        target_height - 1,
        target_width - 1,
        sampled_result[target_height - 1][target_width - 1]
    );

    // Check sampled statistics
    let sampled_min = sampled_result
        .iter()
        .flatten()
        .cloned()
        .fold(f32::INFINITY, f32::min);
    let sampled_max = sampled_result
        .iter()
        .flatten()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max);
    let sampled_avg = sampled_result.iter().flatten().cloned().sum::<f32>()
        / (target_width * target_height) as f32;

    println!("Sampled statistics:");
    println!("  Min: {:.6}", sampled_min);
    println!("  Max: {:.6}", sampled_max);
    println!("  Avg: {:.6}", sampled_avg);
    println!("  Range: {:.6}", sampled_max - sampled_min);

    // Key question: Is (0,0) already maximum before normalization?
    println!("\nKey analysis:");
    println!(
        "Is (0,0) the maximum value in sampled data? {}",
        sampled_result[0][0] >= sampled_max - 1e-6
    );
    println!(
        "(0,0) percentile in sampled data: {:.2}%",
        (sampled_result[0][0] - sampled_min) / (sampled_max - sampled_min) * 100.0
    );

    println!("\n=== Step 3: Test different sizes to find pattern ===");

    let test_sizes = vec![
        (8, 8),     // Power of 2
        (16, 16),   // Power of 2
        (15, 15),   // Non-power of 2
        (17, 17),   // Non-power of 2
        (240, 120), // Main app dimensions
        (32, 32),   // Power of 2
        (31, 31),   // Non-power of 2
    ];

    for (w, h) in test_sizes {
        let test_result = generator.generate(w, h, &config);
        let test_00 = test_result[0][0];
        let test_max = test_result
            .iter()
            .flatten()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        let is_max = test_00 >= test_max - 1e-6;

        println!(
            "{}x{}: (0,0)={:.6}, max={:.6}, is_max={}",
            w, h, test_00, test_max, is_max
        );
    }

    println!("\n=== Step 4: Test with different initial corners ===");

    // Test if the issue is related to specific initial corner values
    let corner_tests = vec![
        ("All equal", [0.5, 0.5, 0.5, 0.5]),
        ("All zero", [0.0, 0.0, 0.0, 0.0]),
        ("Top-left high", [1.0, 0.0, 0.0, 0.0]),
        ("Top-left low", [0.0, 1.0, 1.0, 1.0]),
    ];

    for (name, corners) in corner_tests {
        let test_config = DiamondSquareConfig {
            initial_corners: corners,
            roughness: 0.7,
            persistence: 0.6,
            wrap_edges: false,
        };

        let test_result = generator.generate(16, 16, &test_config);
        let test_00 = test_result[0][0];
        let test_max = test_result
            .iter()
            .flatten()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        let is_max = test_00 >= test_max - 1e-6;

        println!(
            "{}: (0,0)={:.6}, max={:.6}, is_max={}",
            name, test_00, test_max, is_max
        );
    }
}
