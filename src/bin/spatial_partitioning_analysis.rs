// ABOUTME: Detailed analysis of spatial partitioning effectiveness in geological evolution
// ABOUTME: Measures actual active cell percentages and optimization gains across scenarios

use sim_protoype::engine::core::{DetailLevel, WorldScale};
use sim_protoype::engine::physics::convergence_detection::{
    ConvergenceConfig, ConvergenceCriterion,
};
use sim_protoype::engine::physics::optimized_geological_evolution::{
    OptimizedGeologicalConfig, OptimizedGeologicalEvolution,
};
use sim_protoype::engine::physics::worldgen::{
    DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator,
};
use std::time::Instant;

#[derive(Debug, Clone)]
struct SpatialPartitioningResult {
    scenario: String,
    map_size: usize,
    total_cells: usize,
    avg_active_cells: f32,
    peak_active_cells: usize,
    min_active_cells: usize,
    active_cell_percentage: f32,
    performance_gain: f32,
    convergence_iterations: usize,
    cache_hit_rate: f32,
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                  SPATIAL PARTITIONING ANALYSIS                     ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let mut results = Vec::new();

    // Test different terrain scenarios to measure spatial partitioning effectiveness
    let test_scenarios: Vec<(&str, fn(usize) -> Vec<Vec<f32>>)> = vec![
        ("Flat Terrain", create_flat_terrain),
        ("Single Mountain", create_single_mountain),
        ("Multiple Peaks", create_multiple_peaks),
        ("Rough Terrain", create_rough_terrain),
        ("Sparse Features", create_sparse_features),
    ];

    let test_size = 200; // Fixed size for comparison

    for (scenario_name, terrain_generator) in test_scenarios {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Testing Scenario: {}", scenario_name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let result = test_spatial_partitioning(scenario_name, test_size, terrain_generator);
        print_scenario_results(&result);
        results.push(result);
        println!();
    }

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                      COMPARATIVE ANALYSIS                          ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    compare_spatial_partitioning_effectiveness(&results);

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                    OPTIMIZATION RECOMMENDATIONS                     ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    provide_spatial_optimization_recommendations(&results);
}

fn test_spatial_partitioning(
    scenario: &str,
    size: usize,
    terrain_generator: fn(usize) -> Vec<Vec<f32>>,
) -> SpatialPartitioningResult {
    // Generate terrain using the provided generator
    let heightmap = terrain_generator(size);

    // Create world scale
    let world_scale = WorldScale::new(10.0, (size as u32, size as u32), DetailLevel::Standard);

    // Configure for detailed analysis
    let mut config = OptimizedGeologicalConfig::default();
    config.max_iterations = 1000; // Longer run for better statistics
    config.performance_report_interval = 0; // No progress output
    config.enable_performance_logging = false;

    // Make convergence detection more lenient to see full partitioning behavior
    config.convergence_config = ConvergenceConfig {
        min_iterations: 50,
        total_change_threshold: 0.001,
        average_change_threshold: 0.0001,
        max_change_threshold: 0.01,
        change_rate_threshold: 0.0001,
        variance_threshold: 0.0001,
        rolling_window_size: 10,
        consecutive_iterations_required: 10,
        required_criteria: vec![ConvergenceCriterion::TotalChangeMagnitude],
        adaptive_thresholds: false,
        progress_report_interval: 0,
    };

    let mut evolution = OptimizedGeologicalEvolution::new(size, size, config, &world_scale);

    println!("   Running evolution for {} scenario...", scenario);
    let start = Instant::now();
    let results = evolution.evolve_terrain_optimized(heightmap);
    let _duration = start.elapsed();

    SpatialPartitioningResult {
        scenario: scenario.to_string(),
        map_size: size,
        total_cells: size * size,
        avg_active_cells: results.performance_stats.average_active_cells_per_iteration,
        peak_active_cells: results.performance_stats.peak_active_cells,
        min_active_cells: results.performance_stats.minimum_active_cells,
        active_cell_percentage: (results.performance_stats.average_active_cells_per_iteration
            / (size * size) as f32)
            * 100.0,
        performance_gain: results.performance_stats.performance_gain,
        convergence_iterations: results.performance_stats.total_iterations,
        cache_hit_rate: results.cache_stats.hit_rate,
    }
}

fn create_flat_terrain(size: usize) -> Vec<Vec<f32>> {
    let mut heightmap = vec![vec![0.5; size]; size];

    // Add minimal noise to avoid completely static scenario
    for y in 0..size {
        for x in 0..size {
            let noise = ((x + y) as f32 * 0.01).sin() * 0.01;
            heightmap[y][x] += noise;
        }
    }

    heightmap
}

fn create_single_mountain(size: usize) -> Vec<Vec<f32>> {
    let mut heightmap = vec![vec![0.1; size]; size];
    let center_x = size / 2;
    let center_y = size / 2;
    let max_radius = (size / 4) as f32;

    for y in 0..size {
        for x in 0..size {
            let dx = (x as f32 - center_x as f32).abs();
            let dy = (y as f32 - center_y as f32).abs();
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < max_radius {
                let height = 0.8 * (1.0 - distance / max_radius).powf(2.0);
                heightmap[y][x] = 0.1 + height;
            }
        }
    }

    heightmap
}

fn create_multiple_peaks(size: usize) -> Vec<Vec<f32>> {
    let mut heightmap = vec![vec![0.1; size]; size];

    // Create several smaller peaks
    let peaks = vec![
        (size / 4, size / 4, size / 8),
        (3 * size / 4, size / 4, size / 10),
        (size / 2, 3 * size / 4, size / 8),
        (size / 8, 3 * size / 4, size / 12),
        (7 * size / 8, 2 * size / 3, size / 10),
    ];

    for (peak_x, peak_y, radius) in peaks {
        for y in 0..size {
            for x in 0..size {
                let dx = (x as f32 - peak_x as f32).abs();
                let dy = (y as f32 - peak_y as f32).abs();
                let distance = (dx * dx + dy * dy).sqrt();

                if distance < radius as f32 {
                    let height = 0.6 * (1.0 - distance / radius as f32).powf(1.5);
                    heightmap[y][x] = (heightmap[y][x] + height).min(1.0);
                }
            }
        }
    }

    heightmap
}

fn create_rough_terrain(size: usize) -> Vec<Vec<f32>> {
    // Use Diamond-Square for naturally rough terrain
    let generator = DiamondSquareGenerator::new(12345);
    let config = DiamondSquareConfig {
        initial_corners: [0.2, 0.8, 0.3, 0.7],
        roughness: 0.8, // High roughness for varied terrain
        persistence: 0.6,
        wrap_edges: false,
    };

    let heightmap = generator.generate(size, size, &config);
    heightmap.to_nested()
}

fn create_sparse_features(size: usize) -> Vec<Vec<f32>> {
    let mut heightmap = vec![vec![0.1; size]; size];

    // Create sparse, isolated features that should have minimal interaction
    let features = vec![
        (size / 8, size / 8, size / 16, 0.8),
        (7 * size / 8, size / 8, size / 20, 0.6),
        (size / 8, 7 * size / 8, size / 18, 0.7),
        (7 * size / 8, 7 * size / 8, size / 16, 0.9),
        (size / 2, size / 2, size / 20, 0.5),
    ];

    for (feature_x, feature_y, radius, max_height) in features {
        for y in 0..size {
            for x in 0..size {
                let dx = (x as f32 - feature_x as f32).abs();
                let dy = (y as f32 - feature_y as f32).abs();
                let distance = (dx * dx + dy * dy).sqrt();

                if distance < radius as f32 {
                    let height = max_height * (1.0 - distance / radius as f32).powf(3.0);
                    heightmap[y][x] = (heightmap[y][x] + height).min(1.0);
                }
            }
        }
    }

    heightmap
}

fn print_scenario_results(result: &SpatialPartitioningResult) {
    println!("   Total Cells:         {:>8}", result.total_cells);
    println!("   Avg Active Cells:    {:>8.1}", result.avg_active_cells);
    println!("   Peak Active Cells:   {:>8}", result.peak_active_cells);
    println!("   Min Active Cells:    {:>8}", result.min_active_cells);
    println!(
        "   Active Cell %:       {:>7.2}%",
        result.active_cell_percentage
    );
    println!("   Performance Gain:    {:>7.2}x", result.performance_gain);
    println!(
        "   Convergence Iters:   {:>8}",
        result.convergence_iterations
    );
    println!(
        "   Cache Hit Rate:      {:>7.1}%",
        result.cache_hit_rate * 100.0
    );

    // Analyze partitioning effectiveness
    let theoretical_max_gain = 100.0 / result.active_cell_percentage;
    let actual_efficiency = (result.performance_gain / theoretical_max_gain) * 100.0;
    println!("   Theoretical Max:     {:>7.2}x", theoretical_max_gain);
    println!("   Actual Efficiency:   {:>7.1}%", actual_efficiency);
}

fn compare_spatial_partitioning_effectiveness(results: &[SpatialPartitioningResult]) {
    println!("Scenario Comparison:");
    println!(
        "   {:16} {:>8} {:>10} {:>12} {:>10} {:>8}",
        "Scenario", "Active %", "Perf Gain", "Efficiency %", "Conv Iters", "Cache %"
    );
    println!(
        "   {:->16} {:->8} {:->10} {:->12} {:->10} {:->8}",
        "", "", "", "", "", ""
    );

    for result in results {
        let theoretical_max = 100.0 / result.active_cell_percentage;
        let efficiency = (result.performance_gain / theoretical_max) * 100.0;

        println!(
            "   {:16} {:>7.1}% {:>9.1}x {:>11.1}% {:>10} {:>7.1}%",
            result.scenario,
            result.active_cell_percentage,
            result.performance_gain,
            efficiency,
            result.convergence_iterations,
            result.cache_hit_rate * 100.0
        );
    }

    // Find best and worst cases
    let best_partitioning = results
        .iter()
        .min_by(|a, b| {
            a.active_cell_percentage
                .partial_cmp(&b.active_cell_percentage)
                .unwrap()
        })
        .unwrap();
    let worst_partitioning = results
        .iter()
        .max_by(|a, b| {
            a.active_cell_percentage
                .partial_cmp(&b.active_cell_percentage)
                .unwrap()
        })
        .unwrap();

    println!(
        "\nBest Spatial Partitioning: {} ({:.1}% active cells)",
        best_partitioning.scenario, best_partitioning.active_cell_percentage
    );
    println!(
        "Worst Spatial Partitioning: {} ({:.1}% active cells)",
        worst_partitioning.scenario, worst_partitioning.active_cell_percentage
    );

    // Calculate average performance
    let avg_active_percentage: f32 = results
        .iter()
        .map(|r| r.active_cell_percentage)
        .sum::<f32>()
        / results.len() as f32;
    let avg_performance_gain: f32 =
        results.iter().map(|r| r.performance_gain).sum::<f32>() / results.len() as f32;

    println!("Average Active Cells: {:.1}%", avg_active_percentage);
    println!("Average Performance Gain: {:.1}x", avg_performance_gain);

    // Validate the 5-20% claim
    let within_claim = results
        .iter()
        .filter(|r| r.active_cell_percentage >= 5.0 && r.active_cell_percentage <= 20.0)
        .count();
    println!(
        "Scenarios within 5-20% claim: {}/{}",
        within_claim,
        results.len()
    );
}

fn provide_spatial_optimization_recommendations(results: &[SpatialPartitioningResult]) {
    println!("Spatial Partitioning Analysis:");

    let avg_active_percentage: f32 = results
        .iter()
        .map(|r| r.active_cell_percentage)
        .sum::<f32>()
        / results.len() as f32;
    let avg_efficiency: f32 = results
        .iter()
        .map(|r| {
            let theoretical_max = 100.0 / r.active_cell_percentage;
            (r.performance_gain / theoretical_max) * 100.0
        })
        .sum::<f32>()
        / results.len() as f32;

    println!(
        "• Average active cell percentage: {:.1}%",
        avg_active_percentage
    );
    println!("• Average partitioning efficiency: {:.1}%", avg_efficiency);

    if avg_active_percentage < 10.0 {
        println!("✅ Excellent spatial partitioning: <10% active cells");
    } else if avg_active_percentage < 20.0 {
        println!("✅ Good spatial partitioning: <20% active cells");
    } else if avg_active_percentage < 50.0 {
        println!("⚠ Moderate spatial partitioning: 20-50% active cells");
    } else {
        println!("❌ Poor spatial partitioning: >50% active cells");
    }

    println!("\n🔧 Optimization Opportunities:");

    // Analyze convergence speed impact
    let fast_convergence = results
        .iter()
        .filter(|r| r.convergence_iterations < 200)
        .count();
    if fast_convergence > 0 {
        println!(
            "• Convergence detection is working well ({} scenarios converged quickly)",
            fast_convergence
        );
        println!("  Consider tightening convergence thresholds for even faster termination");
    }

    // Cache effectiveness analysis
    let avg_cache_hit_rate: f32 =
        results.iter().map(|r| r.cache_hit_rate).sum::<f32>() / results.len() as f32;
    if avg_cache_hit_rate > 0.8 {
        println!(
            "• Cache system is highly effective ({:.1}% hit rate)",
            avg_cache_hit_rate * 100.0
        );
    } else if avg_cache_hit_rate > 0.6 {
        println!(
            "• Cache system is moderately effective ({:.1}% hit rate)",
            avg_cache_hit_rate * 100.0
        );
        println!("  Consider increasing cache lifetime or size");
    } else {
        println!(
            "• Cache system needs improvement ({:.1}% hit rate)",
            avg_cache_hit_rate * 100.0
        );
        println!("  Investigate cache invalidation strategy");
    }

    // Specific recommendations based on scenario performance
    let flat_terrain = results.iter().find(|r| r.scenario.contains("Flat"));
    let rough_terrain = results.iter().find(|r| r.scenario.contains("Rough"));

    if let (Some(flat), Some(rough)) = (flat_terrain, rough_terrain) {
        if rough.active_cell_percentage > flat.active_cell_percentage * 3.0 {
            println!("• Rough terrain shows poor partitioning compared to flat terrain");
            println!("  Consider adaptive threshold adjustment based on terrain roughness");
        }
    }

    println!("\n🎯 Recommended Improvements:");
    println!("1. Multi-level spatial partitioning for better locality");
    println!("2. Adaptive change thresholds based on terrain characteristics");
    println!("3. Predictive cell activation based on flow direction");
    println!("4. Dynamic load balancing for uneven terrain distribution");

    if avg_efficiency < 70.0 {
        println!("5. Investigation needed: actual performance not matching theoretical maximum");
        println!("   This suggests overhead in the partitioning system itself");
    }
}
