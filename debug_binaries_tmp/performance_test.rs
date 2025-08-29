// ABOUTME: Performance testing script for flow accumulation algorithm scaling analysis
// ABOUTME: Measures performance across different map sizes to identify algorithmic complexity

use kosmarium::engine::core::heightmap::HeightMap;
use kosmarium::engine::physics::drainage::{FlowAccumulationMap, FlowDirectionMap};
use std::time::Instant;

fn generate_test_heightmap(width: usize, height: usize) -> HeightMap {
    // Create a simple gradient heightmap for consistent testing
    let mut heightmap = HeightMap::new(width, height, 0.0);

    for y in 0..height {
        for x in 0..width {
            // Create gradient from top-left (high) to bottom-right (low)
            let normalized_x = x as f32 / width as f32;
            let normalized_y = y as f32 / height as f32;
            let elevation = 1.0 - (normalized_x + normalized_y) * 0.5;
            heightmap.set(x, y, elevation);
        }
    }

    heightmap
}

fn benchmark_flow_accumulation(width: usize, height: usize) -> u128 {
    let heightmap = generate_test_heightmap(width, height);

    println!(
        "Testing {}x{} map ({} cells)...",
        width,
        height,
        width * height
    );

    // Time flow direction calculation
    let start = Instant::now();
    let flow_directions = FlowDirectionMap::from_heightmap(&heightmap);
    let flow_dir_time = start.elapsed();

    // Time flow accumulation calculation (this is the bottleneck)
    let start = Instant::now();
    let _flow_accumulation = FlowAccumulationMap::from_flow_directions(&flow_directions);
    let flow_acc_time = start.elapsed();

    println!("  Flow directions: {:?}", flow_dir_time);
    println!("  Flow accumulation: {:?}", flow_acc_time);

    flow_acc_time.as_millis()
}

fn main() {
    println!("=== Flow Accumulation Performance Analysis ===\n");

    let test_sizes = vec![
        (20, 10),   // 200 cells - baseline
        (50, 25),   // 1,250 cells
        (100, 50),  // 5,000 cells
        (150, 75),  // 11,250 cells
        (200, 100), // 20,000 cells
        (250, 125), // 31,250 cells
        (300, 150), // 45,000 cells
    ];

    let mut results = Vec::new();

    for (width, height) in test_sizes {
        let cells = width * height;
        let time_ms = benchmark_flow_accumulation(width, height);
        results.push((cells, time_ms));
        println!();
    }

    println!("=== Performance Scaling Analysis ===");
    println!(
        "{:>8} {:>10} {:>12} {:>15}",
        "Cells", "Time (ms)", "ms/cell", "Scale Factor"
    );
    println!("{:-<8} {:-<10} {:-<12} {:-<15}", "", "", "", "");

    let baseline_cells = results[0].0;
    let baseline_time = results[0].1;

    for (cells, time_ms) in &results {
        let ms_per_cell = *time_ms as f64 / *cells as f64;
        let cell_scale = *cells as f64 / baseline_cells as f64;
        let time_scale = *time_ms as f64 / baseline_time as f64;
        let complexity_ratio = time_scale / cell_scale;

        println!(
            "{:>8} {:>10} {:>12.4} {:>15.2}",
            cells, time_ms, ms_per_cell, complexity_ratio
        );
    }

    // Calculate empirical complexity
    if results.len() >= 2 {
        let (cells1, time1) = results[0];
        let (cells2, time2) = results[results.len() - 1];

        let cell_ratio = cells2 as f64 / cells1 as f64;
        let time_ratio = time2 as f64 / time1 as f64;

        // If O(n), time_ratio should equal cell_ratio
        // If O(n²), time_ratio should equal cell_ratio²
        // If O(n³), time_ratio should equal cell_ratio³

        let log_cell_ratio = cell_ratio.ln();
        let log_time_ratio = time_ratio.ln();
        let empirical_complexity = log_time_ratio / log_cell_ratio;

        println!("\n=== Complexity Analysis ===");
        println!("Cell ratio: {:.2}x", cell_ratio);
        println!("Time ratio: {:.2}x", time_ratio);
        println!("Empirical complexity: O(n^{:.2})", empirical_complexity);

        if empirical_complexity < 1.5 {
            println!("✓ Linear complexity - good scaling");
        } else if empirical_complexity < 2.5 {
            println!("⚠ Quadratic complexity - will not scale to large maps");
        } else {
            println!("❌ Cubic or worse complexity - serious optimization needed");
        }
    }
}
