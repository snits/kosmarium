// ABOUTME: High-performance geological evolution system integrating all optimization techniques
// ABOUTME: Demonstrates spatial partitioning, caching, flat data structures, and convergence detection

use super::super::core::cache_system::{CacheStats, CachedClimateSystem};
use super::super::core::optimized_heightmap::FlatHeightmap;
use super::super::core::scale::WorldScale;
use super::climate::ClimateSystem;
use super::convergence_detection::{
    ConvergenceConfig, ConvergenceResult, ConvergenceStats, ConvergenceTracker,
};
use super::spatial_partitioning::{OptimizedWaterFlowSystem, PerformanceStats};

/// Configuration for optimized geological evolution
#[derive(Clone, Debug)]
pub struct OptimizedGeologicalConfig {
    /// Maximum iterations (early exit may terminate sooner)
    pub max_iterations: usize,

    /// Convergence detection configuration
    pub convergence_config: ConvergenceConfig,

    /// Performance reporting interval
    pub performance_report_interval: usize,

    /// Enable detailed performance logging
    pub enable_performance_logging: bool,

    /// Enable progress visualization
    pub enable_progress_visualization: bool,

    /// Water flow parameters
    pub flow_rate: f32,
    pub evaporation_rate: f32,
    pub erosion_strength: f32,
    pub deposition_rate: f32,
}

impl Default for OptimizedGeologicalConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10000,
            convergence_config: ConvergenceConfig::default(),
            performance_report_interval: 1000,
            enable_performance_logging: true,
            enable_progress_visualization: false,
            flow_rate: 0.2,
            evaporation_rate: 0.01,
            erosion_strength: 0.05,
            deposition_rate: 0.1,
        }
    }
}

/// Results from optimized geological evolution
#[derive(Debug)]
pub struct OptimizedEvolutionResults {
    /// Final evolved heightmap
    pub evolved_heightmap: FlatHeightmap,

    /// Final water distribution
    pub final_water_depths: Vec<f32>,
    pub final_water_velocities: Vec<(f32, f32)>,
    pub final_sediment: Vec<f32>,

    /// Performance statistics
    pub performance_stats: OptimizedPerformanceStats,

    /// Convergence information
    pub convergence_stats: ConvergenceStats,

    /// Cache performance
    pub cache_stats: CacheStats,
}

/// Comprehensive performance statistics
#[derive(Debug, Clone)]
pub struct OptimizedPerformanceStats {
    pub total_iterations: usize,
    pub total_cells: usize,
    pub average_active_cells_per_iteration: f32,
    pub peak_active_cells: usize,
    pub minimum_active_cells: usize,
    pub total_cells_processed: u64,
    pub cells_skipped: u64,
    pub performance_gain: f32,
    pub convergence_efficiency: f32,
    pub cache_hit_rate: f32,
}

/// High-performance geological evolution system
pub struct OptimizedGeologicalEvolution {
    config: OptimizedGeologicalConfig,

    // Core systems
    water_flow_system: OptimizedWaterFlowSystem,
    cached_climate_system: CachedClimateSystem,
    convergence_tracker: ConvergenceTracker,

    // Performance tracking
    iteration_count: usize,
    active_cells_history: Vec<usize>,
    total_cells_processed: u64,

    // Dimensions
    width: usize,
    height: usize,
}

impl OptimizedGeologicalEvolution {
    /// Create new optimized geological evolution system
    pub fn new(
        width: usize,
        height: usize,
        config: OptimizedGeologicalConfig,
        world_scale: &WorldScale,
    ) -> Self {
        // Create climate system and wrap with caching
        let base_climate_system = ClimateSystem::new_for_scale(world_scale);
        let cached_climate_system = CachedClimateSystem::new(base_climate_system);

        Self {
            config: config.clone(),
            water_flow_system: OptimizedWaterFlowSystem::new(width, height),
            cached_climate_system,
            convergence_tracker: ConvergenceTracker::new(config.convergence_config),
            iteration_count: 0,
            active_cells_history: Vec::new(),
            total_cells_processed: 0,
            width,
            height,
        }
    }

    /// Run optimized geological evolution with all performance enhancements
    pub fn evolve_terrain_optimized(
        &mut self,
        initial_heightmap: Vec<Vec<f32>>,
    ) -> OptimizedEvolutionResults {
        println!(
            "Starting optimized geological evolution: {}x{} map",
            self.width, self.height
        );

        // Convert to flat heightmap for performance
        let mut heightmap = FlatHeightmap::from_nested(initial_heightmap);
        let mut prev_heightmap = heightmap.clone();

        // Initialize water systems with flat storage
        let total_cells = self.width * self.height;
        let mut water_depths = vec![0.0; total_cells];
        let mut water_velocities = vec![(0.0, 0.0); total_cells];
        let mut sediment = vec![0.0; total_cells];

        // Initialize active regions around any existing water
        self.water_flow_system
            .initialize_active_regions(&heightmap, &water_depths);

        // Performance tracking
        let start_time = std::time::Instant::now();
        let mut peak_active_cells = 0;
        let mut min_active_cells = usize::MAX;

        // Main evolution loop with early exit
        for iteration in 0..self.config.max_iterations {
            self.iteration_count = iteration + 1;

            // Store previous state for convergence detection
            prev_heightmap = heightmap.clone();
            let prev_total_water: f32 = water_depths.iter().sum();

            // Get cached temperature layer (major performance optimization)
            let _temperature_layer = self
                .cached_climate_system
                .get_cached_temperature_layer(&heightmap);

            // Update water flow only for active cells (spatial partitioning optimization)
            let water_changes_occurred = self.water_flow_system.update_water_flow_selective(
                &mut heightmap,
                &mut water_depths,
                &mut water_velocities,
                &mut sediment,
                iteration as u64,
            );

            // Track performance metrics
            let current_stats = self.water_flow_system.get_performance_stats();
            self.active_cells_history.push(current_stats.active_cells);
            self.total_cells_processed += current_stats.active_cells as u64;

            peak_active_cells = peak_active_cells.max(current_stats.active_cells);
            if current_stats.active_cells > 0 {
                min_active_cells = min_active_cells.min(current_stats.active_cells);
            }

            // Calculate water change magnitude
            let current_total_water: f32 = water_depths.iter().sum();
            let water_change_magnitude = (current_total_water - prev_total_water).abs();

            // Check convergence (early exit optimization)
            let convergence_result = self.convergence_tracker.record_iteration(
                &prev_heightmap,
                &heightmap,
                Some(water_change_magnitude),
            );

            // Handle convergence
            if convergence_result.is_converged {
                println!("Simulation converged after {} iterations!", iteration + 1);
                break;
            }

            // Progress reporting with performance metrics
            if self.config.performance_report_interval > 0
                && iteration % self.config.performance_report_interval == 0
            {
                self.report_progress(iteration, &convergence_result, &current_stats);
            }

            // Advance climate system iteration
            self.cached_climate_system.advance_iteration();

            // Early termination if no changes are occurring (but respect convergence detection time)
            let min_iterations_for_convergence = self.config.convergence_config.min_iterations
                + self
                    .config
                    .convergence_config
                    .consecutive_iterations_required;
            if !water_changes_occurred
                && current_stats.active_cells == 0
                && iteration >= min_iterations_for_convergence
            {
                println!(
                    "No active changes detected - terminating early at iteration {}",
                    iteration + 1
                );
                break;
            }
        }

        let total_time = start_time.elapsed();

        // Generate comprehensive results
        let performance_stats =
            self.calculate_performance_stats(peak_active_cells, min_active_cells);
        let convergence_stats = self.convergence_tracker.get_convergence_stats();
        let cache_stats = self.cached_climate_system.get_performance_stats();

        // Final performance report
        if self.config.enable_performance_logging {
            self.print_final_performance_report(
                &performance_stats,
                &convergence_stats,
                &cache_stats,
                total_time,
            );
        }

        OptimizedEvolutionResults {
            evolved_heightmap: heightmap,
            final_water_depths: water_depths,
            final_water_velocities: water_velocities,
            final_sediment: sediment,
            performance_stats,
            convergence_stats,
            cache_stats,
        }
    }

    /// Get current optimization statistics without running evolution
    pub fn get_current_stats(&self) -> (PerformanceStats, CacheStats) {
        let spatial_stats = self.water_flow_system.get_performance_stats();
        let cache_stats = self.cached_climate_system.get_performance_stats();
        (spatial_stats, cache_stats)
    }

    /// Check if simulation would benefit from optimization
    pub fn analyze_optimization_potential(
        &self,
        heightmap: &FlatHeightmap,
    ) -> OptimizationAnalysis {
        let total_cells = heightmap.len();

        // Estimate active region size based on terrain variation
        let mut variation_count = 0;
        let mean_elevation = heightmap.data().iter().sum::<f32>() / total_cells as f32;

        for &elevation in heightmap.data() {
            if (elevation - mean_elevation).abs() > 0.1 {
                variation_count += 1;
            }
        }

        let estimated_active_ratio = variation_count as f32 / total_cells as f32;
        let potential_speedup = if estimated_active_ratio > 0.0 {
            1.0 / estimated_active_ratio
        } else {
            1.0
        };

        OptimizationAnalysis {
            total_cells,
            estimated_active_cells: variation_count,
            estimated_active_ratio,
            potential_spatial_speedup: potential_speedup,
            cache_benefit_score: self.estimate_cache_benefit(),
            convergence_benefit_score: self.estimate_convergence_benefit(),
            overall_optimization_score: (potential_speedup - 1.0) * 0.4 + 0.3 + 0.3, // Weighted average
        }
    }

    // Private methods

    fn calculate_performance_stats(
        &self,
        peak_active: usize,
        min_active: usize,
    ) -> OptimizedPerformanceStats {
        let total_cells = self.width * self.height;
        let average_active = if !self.active_cells_history.is_empty() {
            self.active_cells_history.iter().sum::<usize>() as f32
                / self.active_cells_history.len() as f32
        } else {
            0.0
        };

        let total_possible_cells = total_cells as u64 * self.iteration_count as u64;
        let cells_skipped = total_possible_cells - self.total_cells_processed;

        let performance_gain = if self.total_cells_processed > 0 {
            total_possible_cells as f32 / self.total_cells_processed as f32
        } else {
            1.0
        };

        let convergence_efficiency = self
            .convergence_tracker
            .get_convergence_stats()
            .efficiency_gain(self.config.max_iterations);

        let cache_stats = self.cached_climate_system.get_performance_stats();

        OptimizedPerformanceStats {
            total_iterations: self.iteration_count,
            total_cells,
            average_active_cells_per_iteration: average_active,
            peak_active_cells: peak_active,
            minimum_active_cells: if min_active == usize::MAX {
                0
            } else {
                min_active
            },
            total_cells_processed: self.total_cells_processed,
            cells_skipped,
            performance_gain,
            convergence_efficiency,
            cache_hit_rate: cache_stats.hit_rate,
        }
    }

    fn report_progress(
        &self,
        iteration: usize,
        convergence: &ConvergenceResult,
        spatial_stats: &PerformanceStats,
    ) {
        println!("\n=== Progress Report - Iteration {} ===", iteration + 1);
        println!(
            "Convergence: {:.1}% complete",
            convergence
                .progress_info
                .as_ref()
                .map(|p| p.progress_ratio * 100.0)
                .unwrap_or(0.0)
        );
        println!(
            "Active cells: {} / {} ({:.1}%)",
            spatial_stats.active_cells,
            spatial_stats.total_cells,
            spatial_stats.active_cells as f32 / spatial_stats.total_cells as f32 * 100.0
        );
        println!("Performance gain: {:.2}x", spatial_stats.performance_gain());

        if let Some(remaining) = convergence.estimated_iterations_remaining {
            println!("Estimated iterations remaining: {}", remaining);
        }

        let cache_stats = self.cached_climate_system.get_performance_stats();
        println!("Cache hit rate: {:.1}%", cache_stats.hit_rate * 100.0);
        println!("========================================\n");
    }

    fn print_final_performance_report(
        &self,
        perf_stats: &OptimizedPerformanceStats,
        conv_stats: &ConvergenceStats,
        cache_stats: &CacheStats,
        total_time: std::time::Duration,
    ) {
        println!("\n╔══════════ OPTIMIZATION PERFORMANCE REPORT ══════════╗");
        println!("║                                                       ║");
        println!("║ SPATIAL PARTITIONING PERFORMANCE:                    ║");
        println!(
            "║   Total cells: {:<10} Peak active: {:<10}      ║",
            perf_stats.total_cells, perf_stats.peak_active_cells
        );
        println!(
            "║   Avg active: {:<11.1} Min active: {:<11}      ║",
            perf_stats.average_active_cells_per_iteration, perf_stats.minimum_active_cells
        );
        println!("║   Cells skipped: {:<35} ║", perf_stats.cells_skipped);
        println!(
            "║   Performance gain: {:<6.2}x                         ║",
            perf_stats.performance_gain
        );
        println!("║                                                       ║");
        println!("║ CONVERGENCE DETECTION PERFORMANCE:                   ║");
        println!(
            "║   Iterations completed: {:<10}                       ║",
            conv_stats.total_iterations
        );
        println!(
            "║   Converged at: {:<35} ║",
            conv_stats
                .convergence_iteration
                .map(|i| format!("iteration {}", i))
                .unwrap_or("did not converge".to_string())
        );
        println!(
            "║   Iterations saved: {:<10}                           ║",
            conv_stats.iterations_saved(self.config.max_iterations)
        );
        println!(
            "║   Convergence efficiency: {:<6.1}%                   ║",
            perf_stats.convergence_efficiency * 100.0
        );
        println!("║                                                       ║");
        println!("║ CACHE SYSTEM PERFORMANCE:                            ║");
        println!(
            "║   Cache hits: {:<10} Cache misses: {:<10}      ║",
            cache_stats.cache_hits, cache_stats.cache_misses
        );
        println!(
            "║   Hit rate: {:<6.1}%                                 ║",
            cache_stats.hit_rate * 100.0
        );
        println!(
            "║   Computational savings: {:<6.1}%                    ║",
            cache_stats.computational_savings() * 100.0
        );
        println!("║                                                       ║");
        println!("║ OVERALL PERFORMANCE:                                 ║");
        println!(
            "║   Total execution time: {:<6.2}s                     ║",
            total_time.as_secs_f32()
        );
        println!(
            "║   Combined optimization gain: {:<6.2}x               ║",
            perf_stats.performance_gain
                * (1.0 + perf_stats.convergence_efficiency)
                * (1.0 + cache_stats.computational_savings())
        );
        println!("║                                                       ║");
        println!("╚═══════════════════════════════════════════════════════╝\n");
    }

    fn estimate_cache_benefit(&self) -> f32 {
        // Temperature calculations are expensive, so caching provides significant benefit
        0.7 // Estimated 70% benefit from temperature caching
    }

    fn estimate_convergence_benefit(&self) -> f32 {
        // Early termination can save significant iterations
        0.6 // Estimated 60% benefit from convergence detection
    }
}

/// Analysis of optimization potential for a given heightmap
#[derive(Debug, Clone)]
pub struct OptimizationAnalysis {
    pub total_cells: usize,
    pub estimated_active_cells: usize,
    pub estimated_active_ratio: f32,
    pub potential_spatial_speedup: f32,
    pub cache_benefit_score: f32,
    pub convergence_benefit_score: f32,
    pub overall_optimization_score: f32,
}

impl OptimizationAnalysis {
    pub fn should_use_optimization(&self) -> bool {
        self.overall_optimization_score > 1.5 // Worth optimizing if >50% potential improvement
    }

    pub fn optimization_recommendation(&self) -> String {
        if self.overall_optimization_score > 3.0 {
            "Highly recommended - significant performance gains expected".to_string()
        } else if self.overall_optimization_score > 2.0 {
            "Recommended - moderate performance gains expected".to_string()
        } else if self.overall_optimization_score > 1.5 {
            "Consider optimization - small performance gains expected".to_string()
        } else {
            "Optimization may not provide significant benefits".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::core::scale::{DetailLevel, WorldScale};

    #[test]
    fn optimized_evolution_basic_functionality() {
        let width = 20;
        let height = 20;
        let world_scale =
            WorldScale::new(1.0, (width as u32, height as u32), DetailLevel::Standard);

        let mut config = OptimizedGeologicalConfig::default();
        config.max_iterations = 100; // Short test
        config.enable_performance_logging = false;

        let mut evolution = OptimizedGeologicalEvolution::new(width, height, config, &world_scale);

        // Create test heightmap with some variation
        let mut heightmap = vec![vec![0.5; width]; height];
        heightmap[10][10] = 1.0; // Mountain
        heightmap[5][5] = 0.0; // Valley

        let results = evolution.evolve_terrain_optimized(heightmap);

        // Verify results structure
        assert_eq!(results.evolved_heightmap.dimensions(), (width, height));
        assert_eq!(results.final_water_depths.len(), width * height);
        assert_eq!(results.final_water_velocities.len(), width * height);
        assert_eq!(results.final_sediment.len(), width * height);

        // Verify performance stats are reasonable
        assert!(results.performance_stats.total_iterations > 0);
        assert!(results.performance_stats.performance_gain >= 1.0);
        assert!(results.cache_stats.total_requests() > 0);
    }

    #[test]
    fn optimization_analysis() {
        let width = 50;
        let height = 50;
        let world_scale =
            WorldScale::new(5.0, (width as u32, height as u32), DetailLevel::Standard);
        let config = OptimizedGeologicalConfig::default();

        let evolution = OptimizedGeologicalEvolution::new(width, height, config, &world_scale);

        // Create heightmap with localized variation (optimization-friendly)
        let mut heightmap = FlatHeightmap::new(width, height);
        for y in 0..height {
            for x in 0..width {
                // Create a few peaks with mostly flat terrain
                let dist_to_peak1 = ((x as f32 - 15.0).powi(2) + (y as f32 - 15.0).powi(2)).sqrt();
                let dist_to_peak2 = ((x as f32 - 35.0).powi(2) + (y as f32 - 35.0).powi(2)).sqrt();

                let elevation = if dist_to_peak1 < 8.0 {
                    0.5 + (8.0 - dist_to_peak1) / 8.0 // Peak 1
                } else if dist_to_peak2 < 6.0 {
                    0.4 + (6.0 - dist_to_peak2) / 6.0 // Peak 2
                } else {
                    0.05 // Mostly flat baseline
                };

                heightmap.set(x, y, elevation);
            }
        }

        let analysis = evolution.analyze_optimization_potential(&heightmap);

        assert!(analysis.total_cells == width * height);
        assert!(analysis.estimated_active_ratio > 0.0);
        assert!(analysis.potential_spatial_speedup >= 1.0);
        assert!(analysis.overall_optimization_score > 0.0);

        // With significant terrain variation, optimization should be recommended
        assert!(analysis.should_use_optimization());
    }

    #[test]
    fn convergence_early_termination() {
        let width = 10;
        let height = 10;
        let world_scale =
            WorldScale::new(1.0, (width as u32, height as u32), DetailLevel::Standard);

        let mut config = OptimizedGeologicalConfig::default();
        config.max_iterations = 1000;
        config.convergence_config.min_iterations = 10;
        config.convergence_config.consecutive_iterations_required = 5;
        config.enable_performance_logging = false;

        let mut evolution = OptimizedGeologicalEvolution::new(width, height, config, &world_scale);

        // Create simple heightmap that should converge quickly
        let heightmap = vec![vec![0.5; width]; height];

        let results = evolution.evolve_terrain_optimized(heightmap);

        // Should converge before max iterations
        assert!(results.performance_stats.total_iterations < 1000);
        assert!(results.convergence_stats.convergence_iteration.is_some());
        assert!(results.convergence_stats.iterations_saved(1000) > 0);
    }
}
