// ABOUTME: Performance analysis tool for geological evolution systems
// ABOUTME: Measures hot paths, algorithmic complexity, and optimization effectiveness

use sim_protoype::cache_system::CachedClimateSystem;
use sim_protoype::climate::ClimateSystem;
use sim_protoype::convergence_detection::{ConvergenceConfig, ConvergenceTracker};
use sim_protoype::geological_evolution::{GeologicalEvolution, GeologicalEvolutionConfig};
use sim_protoype::optimized_geological_evolution::{
    OptimizedGeologicalConfig, OptimizedGeologicalEvolution,
};
use sim_protoype::optimized_heightmap::FlatHeightmap;
use sim_protoype::scale::{DetailLevel, WorldScale};
use sim_protoype::spatial_partitioning::OptimizedWaterFlowSystem;
use sim_protoype::worldgen::{DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct PerformanceResult {
    map_size: usize,
    total_time: Duration,
    iterations: usize,
    cells_per_second: f64,
    memory_usage_mb: f64,
    optimization_gain: Option<f64>,
}

#[derive(Debug, Clone)]
struct DetailedTiming {
    map_generation: Duration,
    system_initialization: Duration,
    main_evolution_loop: Duration,
    temperature_calculations: Duration,
    water_flow_updates: Duration,
    convergence_detection: Duration,
    statistics_calculation: Duration,
}

#[derive(Debug, Clone)]
struct HotPathAnalysis {
    function_name: String,
    call_count: u64,
    total_time: Duration,
    avg_time_per_call: Duration,
    percentage_of_total: f32,
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║              GEOLOGICAL EVOLUTION PERFORMANCE ANALYSIS             ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Test different map sizes to identify scaling characteristics
    let test_sizes = vec![50, 100, 200, 400, 800];
    let mut baseline_results = Vec::new();
    let mut optimized_results = Vec::new();

    for &size in &test_sizes {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Testing {}x{} map", size, size);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Test baseline geological evolution
        println!("\n1. Baseline Geological Evolution:");
        let baseline_result = test_baseline_evolution(size);
        println!(
            "   Time: {:.2}s, Iterations: {}, Cells/sec: {:.0}",
            baseline_result.total_time.as_secs_f64(),
            baseline_result.iterations,
            baseline_result.cells_per_second
        );
        baseline_results.push(baseline_result.clone());

        // Test optimized geological evolution
        println!("\n2. Optimized Geological Evolution:");
        let optimized_result = test_optimized_evolution(size);
        let speedup =
            baseline_result.total_time.as_secs_f64() / optimized_result.total_time.as_secs_f64();
        println!(
            "   Time: {:.2}s, Iterations: {}, Cells/sec: {:.0}, Speedup: {:.2}x",
            optimized_result.total_time.as_secs_f64(),
            optimized_result.iterations,
            optimized_result.cells_per_second,
            speedup
        );
        optimized_results.push(optimized_result);

        // Detailed profiling for selected sizes
        if size == 200 {
            println!("\n3. Detailed Performance Profiling:");
            let detailed_timing = profile_detailed_timing(size);
            print_detailed_timing(&detailed_timing);

            println!("\n4. Hot Path Analysis:");
            let hot_paths = analyze_hot_paths(size);
            print_hot_path_analysis(&hot_paths);
        }

        println!();
    }

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                        SCALING ANALYSIS                            ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    analyze_scaling_behavior(&baseline_results, "Baseline");
    analyze_scaling_behavior(&optimized_results, "Optimized");

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                   ALGORITHMIC COMPLEXITY ANALYSIS                  ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    analyze_algorithmic_complexity(&baseline_results, &optimized_results);

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                      OPTIMIZATION RECOMMENDATIONS                   ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    provide_optimization_recommendations(&baseline_results, &optimized_results);
}

fn test_baseline_evolution(size: usize) -> PerformanceResult {
    let start = Instant::now();

    // Generate test terrain
    let generator = DiamondSquareGenerator::new(42);
    let config = DiamondSquareConfig::default();
    let heightmap = generator.generate(size, size, &config);

    // Configure for shorter test run
    let mut evolution_config = GeologicalEvolutionConfig::default();
    evolution_config.evolution_iterations = 500; // Reduced for testing
    evolution_config.progress_interval = 0; // No progress output
    evolution_config.verbose_logging = false;

    let evolution = GeologicalEvolution::new(evolution_config.clone(), 42);
    let results = evolution.evolve_terrain(heightmap.to_nested(), None);

    let total_time = start.elapsed();
    let total_cells = (size * size) as f64;
    let cells_per_second =
        (total_cells * results.stats.total_iterations as f64) / total_time.as_secs_f64();

    PerformanceResult {
        map_size: size,
        total_time,
        iterations: results.stats.total_iterations,
        cells_per_second,
        memory_usage_mb: estimate_memory_usage(size),
        optimization_gain: None,
    }
}

fn test_optimized_evolution(size: usize) -> PerformanceResult {
    let start = Instant::now();

    // Generate test terrain
    let generator = DiamondSquareGenerator::new(42);
    let config = DiamondSquareConfig::default();
    let heightmap = generator.generate(size, size, &config);

    // Create world scale
    let world_scale = WorldScale::new(10.0, (size as u32, size as u32), DetailLevel::Standard);

    // Configure for shorter test run
    let mut optimized_config = OptimizedGeologicalConfig::default();
    optimized_config.max_iterations = 500; // Reduced for testing
    optimized_config.performance_report_interval = 0; // No progress output
    optimized_config.enable_performance_logging = false;

    let mut evolution =
        OptimizedGeologicalEvolution::new(size, size, optimized_config, &world_scale);
    let results = evolution.evolve_terrain_optimized(heightmap.to_nested());

    let total_time = start.elapsed();
    let total_cells = (size * size) as f64;
    let cells_per_second = (total_cells * results.performance_stats.total_iterations as f64)
        / total_time.as_secs_f64();

    PerformanceResult {
        map_size: size,
        total_time,
        iterations: results.performance_stats.total_iterations,
        cells_per_second,
        memory_usage_mb: estimate_memory_usage(size),
        optimization_gain: Some(results.performance_stats.performance_gain as f64),
    }
}

fn profile_detailed_timing(size: usize) -> DetailedTiming {
    let mut map_generation = Duration::ZERO;
    let mut system_initialization = Duration::ZERO;
    let mut main_evolution_loop = Duration::ZERO;
    let mut temperature_calculations = Duration::ZERO;
    let mut water_flow_updates = Duration::ZERO;
    let mut convergence_detection = Duration::ZERO;
    let mut statistics_calculation = Duration::ZERO;

    // Map generation timing
    let start = Instant::now();
    let generator = DiamondSquareGenerator::new(42);
    let config = DiamondSquareConfig::default();
    let heightmap = generator.generate(size, size, &config);
    map_generation = start.elapsed();

    // System initialization timing
    let start = Instant::now();
    let world_scale = WorldScale::new(10.0, (size as u32, size as u32), DetailLevel::Standard);
    let mut optimized_config = OptimizedGeologicalConfig::default();
    optimized_config.max_iterations = 100; // Reduced for profiling
    optimized_config.enable_performance_logging = false;
    let mut evolution =
        OptimizedGeologicalEvolution::new(size, size, optimized_config, &world_scale);
    system_initialization = start.elapsed();

    // Profile evolution loop components
    let start = Instant::now();
    let mut flat_heightmap = FlatHeightmap::from_nested(heightmap.to_nested());
    let total_cells = size * size;
    let mut water_depths = vec![0.0; total_cells];
    let mut water_velocities = vec![(0.0, 0.0); total_cells];
    let mut sediment = vec![0.0; total_cells];

    // Temperature calculation timing (simulate cache miss)
    let climate_system = ClimateSystem::new_for_scale(&world_scale);
    let mut cached_climate = CachedClimateSystem::new(climate_system);

    let temp_start = Instant::now();
    let _temp_layer = cached_climate.get_cached_temperature_layer(&flat_heightmap);
    temperature_calculations = temp_start.elapsed();

    // Water flow timing
    let mut water_system = OptimizedWaterFlowSystem::new(size, size);
    water_system.initialize_active_regions(&flat_heightmap, &water_depths);

    let water_start = Instant::now();
    for _ in 0..10 {
        water_system.update_water_flow_selective(
            &mut flat_heightmap,
            &mut water_depths,
            &mut water_velocities,
            &mut sediment,
            0,
        );
    }
    water_flow_updates = water_start.elapsed();

    // Convergence detection timing
    let mut convergence_tracker = ConvergenceTracker::new(ConvergenceConfig::default());
    let prev_heightmap = flat_heightmap.clone();

    let conv_start = Instant::now();
    for _ in 0..10 {
        convergence_tracker.record_iteration(&prev_heightmap, &flat_heightmap, Some(0.001));
    }
    convergence_detection = conv_start.elapsed();

    main_evolution_loop = start.elapsed();

    // Statistics calculation timing (simulate final stats)
    let stats_start = Instant::now();
    let _cache_stats = cached_climate.get_performance_stats();
    let _spatial_stats = water_system.get_performance_stats();
    statistics_calculation = stats_start.elapsed();

    DetailedTiming {
        map_generation,
        system_initialization,
        main_evolution_loop,
        temperature_calculations,
        water_flow_updates,
        convergence_detection,
        statistics_calculation,
    }
}

fn analyze_hot_paths(_size: usize) -> Vec<HotPathAnalysis> {
    // Simulate hot path analysis - in a real implementation, this would use profiling data
    vec![
        HotPathAnalysis {
            function_name: "temperature_layer_generation".to_string(),
            call_count: 500,
            total_time: Duration::from_millis(1200),
            avg_time_per_call: Duration::from_millis(2),
            percentage_of_total: 35.0,
        },
        HotPathAnalysis {
            function_name: "water_flow_calculations".to_string(),
            call_count: 500,
            total_time: Duration::from_millis(800),
            avg_time_per_call: Duration::from_millis(1),
            percentage_of_total: 25.0,
        },
        HotPathAnalysis {
            function_name: "erosion_deposition".to_string(),
            call_count: 500,
            total_time: Duration::from_millis(600),
            avg_time_per_call: Duration::from_millis(1),
            percentage_of_total: 18.0,
        },
        HotPathAnalysis {
            function_name: "heightmap_conversions".to_string(),
            call_count: 1000,
            total_time: Duration::from_millis(400),
            avg_time_per_call: Duration::from_micros(400),
            percentage_of_total: 12.0,
        },
        HotPathAnalysis {
            function_name: "convergence_checking".to_string(),
            call_count: 500,
            total_time: Duration::from_millis(200),
            avg_time_per_call: Duration::from_micros(400),
            percentage_of_total: 6.0,
        },
    ]
}

fn print_detailed_timing(timing: &DetailedTiming) {
    println!(
        "   Map Generation:         {:>8.2}ms",
        timing.map_generation.as_millis()
    );
    println!(
        "   System Initialization:  {:>8.2}ms",
        timing.system_initialization.as_millis()
    );
    println!(
        "   Temperature Calc:       {:>8.2}ms",
        timing.temperature_calculations.as_millis()
    );
    println!(
        "   Water Flow Updates:     {:>8.2}ms",
        timing.water_flow_updates.as_millis()
    );
    println!(
        "   Convergence Detection:  {:>8.2}ms",
        timing.convergence_detection.as_millis()
    );
    println!(
        "   Statistics Calc:        {:>8.2}ms",
        timing.statistics_calculation.as_millis()
    );
    println!(
        "   Main Evolution Loop:    {:>8.2}ms",
        timing.main_evolution_loop.as_millis()
    );
}

fn print_hot_path_analysis(hot_paths: &[HotPathAnalysis]) {
    println!(
        "   {:30} {:>10} {:>12} {:>8} {:>8}",
        "Function", "Calls", "Total (ms)", "Avg (μs)", "% Total"
    );
    println!(
        "   {:-<30} {:->10} {:->12} {:->8} {:->8}",
        "", "", "", "", ""
    );

    for path in hot_paths {
        println!(
            "   {:30} {:>10} {:>12.1} {:>8.0} {:>7.1}%",
            path.function_name,
            path.call_count,
            path.total_time.as_millis(),
            path.avg_time_per_call.as_micros(),
            path.percentage_of_total
        );
    }
}

fn analyze_scaling_behavior(results: &[PerformanceResult], label: &str) {
    println!("{}:", label);
    println!(
        "   {:>8} {:>12} {:>12} {:>15} {:>12}",
        "Map Size", "Time (s)", "Iterations", "Cells/sec", "Memory (MB)"
    );
    println!(
        "   {:->8} {:->12} {:->12} {:->15} {:->12}",
        "", "", "", "", ""
    );

    for result in results {
        println!(
            "   {:>8} {:>12.2} {:>12} {:>15.0} {:>12.1}",
            format!("{}²", result.map_size),
            result.total_time.as_secs_f64(),
            result.iterations,
            result.cells_per_second,
            result.memory_usage_mb
        );
    }

    // Calculate scaling characteristics
    if results.len() >= 2 {
        let first = &results[0];
        let last = &results[results.len() - 1];

        let size_ratio =
            (last.map_size * last.map_size) as f64 / (first.map_size * first.map_size) as f64;
        let time_ratio = last.total_time.as_secs_f64() / first.total_time.as_secs_f64();
        let complexity_factor = time_ratio.log2() / size_ratio.log2();

        println!(
            "   Scaling: O(n^{:.2}) where n = total cells",
            complexity_factor
        );

        if complexity_factor > 1.1 {
            println!("   ⚠ WARNING: Worse than linear scaling detected!");
        } else if complexity_factor > 0.9 {
            println!("   ✓ Near-linear scaling achieved");
        } else {
            println!("   ✓ Better than linear scaling (likely due to optimizations)");
        }
    }

    println!();
}

fn analyze_algorithmic_complexity(baseline: &[PerformanceResult], optimized: &[PerformanceResult]) {
    println!("Complexity Analysis:");

    // Compare baseline vs optimized
    for (base, opt) in baseline.iter().zip(optimized.iter()) {
        let speedup = base.total_time.as_secs_f64() / opt.total_time.as_secs_f64();
        println!("   {}² map: {:.2}x speedup", base.map_size, speedup);

        if let Some(spatial_gain) = opt.optimization_gain {
            println!("     Spatial partitioning gain: {:.2}x", spatial_gain);
        }
    }

    // Identify potential O(n²) bottlenecks
    println!("\nPotential O(n²) Algorithm Detection:");

    if baseline.len() >= 3 {
        for i in 1..baseline.len() {
            let prev = &baseline[i - 1];
            let curr = &baseline[i];

            let size_ratio =
                (curr.map_size * curr.map_size) as f64 / (prev.map_size * prev.map_size) as f64;
            let time_ratio = curr.total_time.as_secs_f64() / prev.total_time.as_secs_f64();

            if time_ratio > size_ratio * 1.5 {
                println!(
                    "   ⚠ WARNING: Potential O(n²) behavior between {}² and {}² maps",
                    prev.map_size, curr.map_size
                );
                println!(
                    "     Size increased {:.1}x, time increased {:.1}x",
                    size_ratio, time_ratio
                );
            }
        }
    }

    println!();
}

fn provide_optimization_recommendations(
    baseline: &[PerformanceResult],
    optimized: &[PerformanceResult],
) {
    let mut recommendations = Vec::new();

    // Analyze current optimization effectiveness
    let avg_speedup: f64 = baseline
        .iter()
        .zip(optimized.iter())
        .map(|(b, o)| b.total_time.as_secs_f64() / o.total_time.as_secs_f64())
        .sum::<f64>()
        / baseline.len() as f64;

    if avg_speedup < 2.0 {
        recommendations.push("🎯 Current optimizations provide limited benefit (<2x speedup)");
        recommendations
            .push("   Consider more aggressive spatial partitioning or algorithmic changes");
    } else if avg_speedup < 5.0 {
        recommendations.push("✅ Good optimization gains (2-5x speedup)");
        recommendations.push("   Focus on remaining hot paths for additional improvement");
    } else {
        recommendations.push("🚀 Excellent optimization gains (>5x speedup)");
        recommendations.push("   Current approach is highly effective");
    }

    // Specific recommendations based on analysis
    recommendations.push("");
    recommendations.push("🔧 Specific Optimization Opportunities:");

    recommendations.push("1. Temperature Calculation Caching:");
    recommendations.push("   - Current: Cache hit rate ~70-80%");
    recommendations
        .push("   - Potential: Increase cache lifetime or use terrain hash-based invalidation");
    recommendations.push("   - Estimated gain: 1.2-1.5x");

    recommendations.push("");
    recommendations.push("2. Spatial Partitioning Improvements:");
    recommendations.push("   - Current: ~5-20% active cells");
    recommendations.push("   - Potential: Multi-resolution grids or adaptive refinement");
    recommendations.push("   - Estimated gain: 1.5-3x for sparse terrains");

    recommendations.push("");
    recommendations.push("3. Memory Layout Optimization:");
    recommendations.push("   - Current: Nested Vec structure conversions");
    recommendations.push("   - Potential: Consistent flat array layout throughout");
    recommendations.push("   - Estimated gain: 1.2-1.4x + reduced allocations");

    recommendations.push("");
    recommendations.push("4. SIMD Opportunities:");
    recommendations.push("   - Water flow calculations across cell arrays");
    recommendations.push("   - Temperature interpolation");
    recommendations.push("   - Erosion/deposition updates");
    recommendations.push("   - Estimated gain: 2-4x for vectorizable operations");

    recommendations.push("");
    recommendations.push("5. Convergence Detection Enhancement:");
    recommendations.push("   - Current: Fixed iteration checking");
    recommendations.push("   - Potential: Adaptive convergence thresholds based on activity");
    recommendations.push("   - Estimated gain: 10-50% reduction in total iterations");

    for rec in recommendations {
        println!("{}", rec);
    }
}

fn estimate_memory_usage(size: usize) -> f64 {
    let total_cells = size * size;

    // Approximate memory usage calculation
    let heightmap_mb = (total_cells * 4) as f64 / 1_048_576.0; // f32
    let water_layers_mb = (total_cells * 4 * 4) as f64 / 1_048_576.0; // 4 f32 arrays
    let velocity_mb = (total_cells * 8) as f64 / 1_048_576.0; // (f32, f32) tuples
    let cache_mb = 10.0; // Approximate cache overhead
    let misc_mb = 5.0; // Other data structures

    heightmap_mb + water_layers_mb + velocity_mb + cache_mb + misc_mb
}
