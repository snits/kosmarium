// ABOUTME: Geological time scale evolution system for pre-aging terrain through erosion processes
// ABOUTME: Runs accelerated water flow and erosion over geological timescales before real-time simulation

use crate::climate::{ClimateSystem, TemperatureLayer};
use crate::heightmap::HeightMap;
use crate::scale::{DetailLevel, WorldScale};
use crate::sim::{WaterFlowParameters, WaterFlowSystem, WaterLayer};
use crate::tectonics::TectonicSystem;

/// Configuration for geological time scale evolution
#[derive(Clone, Debug)]
pub struct GeologicalEvolutionConfig {
    /// Number of geological time iterations to run (thousands to millions)
    pub evolution_iterations: usize,

    /// Accelerated water flow parameters for geological timescales
    pub geological_water_params: WaterFlowParameters,

    /// Climate evolution settings
    pub enable_climate_cycles: bool,
    pub temperature_variation: f32, // Temperature swings during evolution

    /// Erosion acceleration factor (speeds up geological processes)
    pub erosion_acceleration: f32,

    /// Progress reporting interval (0 = no progress reports)
    pub progress_interval: usize,

    /// Enable detailed logging of geological processes
    pub verbose_logging: bool,
}

impl Default for GeologicalEvolutionConfig {
    fn default() -> Self {
        Self {
            evolution_iterations: 10000, // 10K iterations for initial testing
            geological_water_params: Self::geological_water_params(),
            enable_climate_cycles: true,
            temperature_variation: 10.0, // ±10°C variation over geological time
            erosion_acceleration: 5.0,   // 5x faster erosion for geological time
            progress_interval: 1000,     // Report every 1000 iterations
            verbose_logging: false,
        }
    }
}

impl GeologicalEvolutionConfig {
    /// Create water flow parameters optimized for geological time scale evolution
    fn geological_water_params() -> WaterFlowParameters {
        let mut params = WaterFlowParameters::default();

        // Accelerated geological processes
        params.flow_rate = 0.2; // Faster water flow
        params.evaporation_rate = 0.01; // Higher evaporation for balance
        params.erosion_strength = 0.05; // Stronger erosion for geological time
        params.deposition_rate = 0.1; // Faster sediment settling
        params.base_rainfall_rate = 0.005; // Higher rainfall for active erosion

        // Use mass-conserving scaling for realistic water budgets
        params.rainfall_scaling = crate::sim::RainfallScaling::MassConserving;

        params
    }
}

/// Geological evolution system that pre-ages terrain through erosion processes
pub struct GeologicalEvolution {
    config: GeologicalEvolutionConfig,
    seed: u64,
}

/// Results from geological evolution process
#[derive(Debug)]
pub struct EvolutionResults {
    /// The evolved heightmap after geological processes
    pub evolved_heightmap: Vec<Vec<f32>>,

    /// Final water distribution (rivers, lakes)
    pub final_water_state: WaterLayer,

    /// Statistics about the evolution process
    pub stats: EvolutionStats,
}

/// Statistics tracking geological evolution process
#[derive(Debug, Default)]
pub struct EvolutionStats {
    pub total_iterations: usize,
    pub total_erosion: f32,            // Total material eroded
    pub total_deposition: f32,         // Total material deposited
    pub river_network_length: f32,     // Approximate length of river networks
    pub average_elevation_change: f32, // Average change in elevation
    pub max_elevation_change: f32,     // Maximum elevation change at any point
}

impl GeologicalEvolution {
    pub fn new(config: GeologicalEvolutionConfig, seed: u64) -> Self {
        Self { config, seed }
    }

    /// Run geological evolution on a heightmap, returning evolved terrain
    pub fn evolve_terrain(
        &self,
        initial_heightmap: Vec<Vec<f32>>,
        tectonic_system: Option<&TectonicSystem>,
    ) -> EvolutionResults {
        let height = initial_heightmap.len();
        let width = initial_heightmap[0].len();

        if self.config.verbose_logging {
            println!(
                "Starting geological evolution: {}x{} map, {} iterations",
                width, height, self.config.evolution_iterations
            );
        }

        // Create world scale for physics calculations
        let world_scale =
            WorldScale::new(10.0, (width as u32, height as u32), DetailLevel::Standard);

        // Initialize water flow system with geological parameters
        let water_system = WaterFlowSystem::from_parameters(
            self.config.geological_water_params.clone(),
            &world_scale,
        );

        // Initialize water layer
        let mut water_layer = WaterLayer::new(width, height);

        // Initialize climate system for temperature-dependent processes
        let climate_system = ClimateSystem::new_for_scale(&world_scale);
        let mut temperature_layer = TemperatureLayer::new(width, height);

        // Initialize statistics tracking
        let mut stats = EvolutionStats::default();
        let mut evolved_heightmap = initial_heightmap.clone();

        // Track initial state for statistics
        let initial_total_elevation: f32 =
            initial_heightmap.iter().flat_map(|row| row.iter()).sum();

        // Run geological evolution iterations
        for iteration in 0..self.config.evolution_iterations {
            // Update temperature layer (regenerate from climate system)
            // For geological timescales, we'll use the base climate without variation
            // More complex climate cycles can be added later if needed
            temperature_layer = climate_system.generate_temperature_layer(&evolved_heightmap);

            // Store pre-erosion state for statistics
            let pre_erosion_elevation: f32 =
                evolved_heightmap.iter().flat_map(|row| row.iter()).sum();

            // Run one step of accelerated water flow and erosion
            // Convert to HeightMap for the water system
            let mut heightmap_for_water = HeightMap::from_nested(evolved_heightmap.clone());
            water_system.update_water_flow_with_climate(
                &mut heightmap_for_water,
                &mut water_layer,
                &temperature_layer,
                &climate_system,
            );
            // Convert back to nested format
            evolved_heightmap = heightmap_for_water.to_nested();

            // Apply erosion acceleration factor
            if self.config.erosion_acceleration > 1.0 {
                self.apply_erosion_acceleration(&mut evolved_heightmap, &water_layer);
            }

            // Update statistics
            let post_erosion_elevation: f32 =
                evolved_heightmap.iter().flat_map(|row| row.iter()).sum();

            let elevation_change = (post_erosion_elevation - pre_erosion_elevation).abs();
            stats.total_erosion += elevation_change * 0.5; // Approximate erosion amount
            stats.total_deposition += elevation_change * 0.5; // Approximate deposition amount

            // Progress reporting
            if self.config.progress_interval > 0 && iteration % self.config.progress_interval == 0 {
                let progress = (iteration as f32 / self.config.evolution_iterations as f32) * 100.0;
                println!(
                    "Geological evolution progress: {:.1}% ({}/{})",
                    progress, iteration, self.config.evolution_iterations
                );
            }
        }

        // Calculate final statistics
        let final_total_elevation: f32 = evolved_heightmap.iter().flat_map(|row| row.iter()).sum();

        stats.total_iterations = self.config.evolution_iterations;
        stats.average_elevation_change =
            (final_total_elevation - initial_total_elevation).abs() / (width * height) as f32;
        stats.river_network_length = self.calculate_river_network_length(&water_layer);
        stats.max_elevation_change =
            self.calculate_max_elevation_change(&initial_heightmap, &evolved_heightmap);

        if self.config.verbose_logging {
            println!("Geological evolution complete:");
            println!(
                "  Average elevation change: {:.4}",
                stats.average_elevation_change
            );
            println!("  Max elevation change: {:.4}", stats.max_elevation_change);
            println!("  River network length: {:.1}", stats.river_network_length);
            println!("  Total erosion: {:.2}", stats.total_erosion);
        }

        EvolutionResults {
            evolved_heightmap,
            final_water_state: water_layer,
            stats,
        }
    }

    /// Apply additional erosion acceleration for geological timescales
    fn apply_erosion_acceleration(&self, heightmap: &mut Vec<Vec<f32>>, water_layer: &WaterLayer) {
        let acceleration = self.config.erosion_acceleration - 1.0; // Additional acceleration beyond base rate

        for y in 0..heightmap.len() {
            for x in 0..heightmap[0].len() {
                let water_amount = water_layer.depth[y][x];
                let sediment_amount = water_layer.sediment[y][x];

                // Additional erosion where water is flowing
                if water_amount > 0.01 {
                    let additional_erosion = water_amount * acceleration * 0.001;
                    heightmap[y][x] -= additional_erosion;

                    // Ensure elevation stays reasonable
                    heightmap[y][x] = heightmap[y][x].clamp(-2.0, 2.0);
                }

                // Additional deposition where sediment is high
                if sediment_amount > 0.01 {
                    let additional_deposition = sediment_amount * acceleration * 0.0005;
                    heightmap[y][x] += additional_deposition;

                    // Ensure elevation stays reasonable
                    heightmap[y][x] = heightmap[y][x].clamp(-2.0, 2.0);
                }
            }
        }
    }

    /// Calculate approximate river network length based on water distribution
    fn calculate_river_network_length(&self, water_layer: &WaterLayer) -> f32 {
        let mut river_length = 0.0;
        let river_threshold = 0.05; // Minimum water depth to consider as river

        for y in 0..water_layer.height() {
            for x in 0..water_layer.width() {
                if water_layer.depth[y][x] > river_threshold {
                    // Count connected water cells (simple approximation)
                    let mut connections = 0;

                    // Check 8-connected neighbors
                    for dy in -1_i32..=1 {
                        for dx in -1_i32..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }

                            let ny = y as i32 + dy;
                            let nx = x as i32 + dx;

                            if ny >= 0
                                && ny < water_layer.height() as i32
                                && nx >= 0
                                && nx < water_layer.width() as i32
                            {
                                if water_layer.depth[ny as usize][nx as usize] > river_threshold {
                                    connections += 1;
                                }
                            }
                        }
                    }

                    // Add to river length based on connectivity
                    if connections > 0 {
                        river_length += (connections as f32).sqrt() * 0.5;
                    }
                }
            }
        }

        river_length
    }

    /// Calculate maximum elevation change between initial and final heightmaps
    fn calculate_max_elevation_change(&self, initial: &[Vec<f32>], final_map: &[Vec<f32>]) -> f32 {
        let mut max_change: f32 = 0.0;

        for y in 0..initial.len() {
            for x in 0..initial[0].len() {
                let change = (final_map[y][x] - initial[y][x]).abs();
                max_change = max_change.max(change);
            }
        }

        max_change
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geological_evolution_creates_system() {
        let config = GeologicalEvolutionConfig::default();
        let evolution = GeologicalEvolution::new(config, 12345);

        // Test with small heightmap
        let heightmap = vec![vec![0.5; 10]; 10];
        let results = evolution.evolve_terrain(heightmap, None);

        assert_eq!(results.evolved_heightmap.len(), 10);
        assert_eq!(results.evolved_heightmap[0].len(), 10);
        assert_eq!(results.stats.total_iterations, 10000);
    }

    #[test]
    fn geological_evolution_modifies_terrain() {
        let mut config = GeologicalEvolutionConfig::default();
        config.evolution_iterations = 100; // Short test
        config.progress_interval = 0; // No progress output
        config.verbose_logging = false;

        let evolution = GeologicalEvolution::new(config, 12345);

        // Create simple heightmap with a mountain
        let mut heightmap = vec![vec![0.0; 5]; 5];
        heightmap[2][2] = 1.0; // Central mountain

        let initial_heightmap = heightmap.clone();
        let results = evolution.evolve_terrain(heightmap, None);

        // Terrain should be modified by erosion
        let has_changes = results
            .evolved_heightmap
            .iter()
            .zip(initial_heightmap.iter())
            .any(|(final_row, initial_row)| {
                final_row
                    .iter()
                    .zip(initial_row.iter())
                    .any(|(final_val, initial_val)| (final_val - initial_val).abs() > 0.001)
            });

        assert!(has_changes, "Geological evolution should modify terrain");
        assert!(
            results.stats.total_erosion > 0.0,
            "Should have some erosion"
        );
    }
}
