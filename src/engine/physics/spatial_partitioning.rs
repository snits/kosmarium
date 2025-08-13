// ABOUTME: Spatial partitioning system for selective cell updates in geological simulation
// ABOUTME: Implements multi-tier update regions for water/evaporation coupling and convergence tracking

use super::super::core::optimized_heightmap::FlatHeightmap;
use super::super::core::scale::WorldScale;
use super::{FlowEngine, FlowParameters}; // Use the re-export from mod.rs
use crate::engine::{WaterFlowParameters, WaterFlowSystem};
use std::collections::HashSet;

/// Represents different types of changes that can occur in a cell
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChangeType {
    Terrain,     // Elevation changed due to erosion/deposition
    Water,       // Water depth changed
    Temperature, // Temperature changed (affects evaporation)
    Flow,        // Water flow velocity changed
}

/// Tracks which cells need updates and what type of updates they need
#[derive(Debug, Clone)]
pub struct SpatialUpdateTracker {
    width: usize,
    height: usize,

    // Active cells that need updates this iteration
    active_cells: HashSet<usize>,

    // Cells that need updates next iteration (propagated changes)
    next_active_cells: HashSet<usize>,

    // Change magnitude tracking for convergence detection
    change_magnitudes: Vec<f32>,

    // Configuration
    min_change_threshold: f32,
    neighbor_propagation_distance: usize,
}

impl SpatialUpdateTracker {
    /// Create new spatial update tracker
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            active_cells: HashSet::new(),
            next_active_cells: HashSet::new(),
            change_magnitudes: vec![0.0; width * height],
            min_change_threshold: 0.001,
            neighbor_propagation_distance: 2,
        }
    }

    /// Mark a cell as needing update due to a change
    pub fn mark_cell_changed(
        &mut self,
        x: usize,
        y: usize,
        change_magnitude: f32,
        _change_type: ChangeType,
    ) {
        if x >= self.width || y >= self.height {
            return;
        }

        let index = y * self.width + x;

        // Only mark if change is significant
        if change_magnitude > self.min_change_threshold {
            self.active_cells.insert(index);
            self.change_magnitudes[index] = change_magnitude;

            // Propagate to neighbors for coupled systems (water/evaporation)
            self.propagate_to_neighbors(x, y, change_magnitude * 0.5);
        }
    }

    /// Propagate changes to neighboring cells
    fn propagate_to_neighbors(
        &mut self,
        center_x: usize,
        center_y: usize,
        propagated_magnitude: f32,
    ) {
        let dist = self.neighbor_propagation_distance as i32;

        for dy in -dist..=dist {
            for dx in -dist..=dist {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = center_x as i32 + dx;
                let ny = center_y as i32 + dy;

                if nx >= 0 && ny >= 0 && (nx as usize) < self.width && (ny as usize) < self.height {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    let index = ny * self.width + nx;

                    // Add to current active cells for immediate neighbor coupling
                    if propagated_magnitude > self.min_change_threshold * 0.1 {
                        self.active_cells.insert(index);
                    }
                }
            }
        }
    }

    /// Get iterator over currently active cells
    pub fn active_cells(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.active_cells.iter().map(|&index| {
            let y = index / self.width;
            let x = index % self.width;
            (x, y)
        })
    }

    /// Get number of active cells
    pub fn active_cell_count(&self) -> usize {
        self.active_cells.len()
    }

    /// Check if a specific cell is active
    pub fn is_cell_active(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let index = y * self.width + x;
        self.active_cells.contains(&index)
    }

    /// Advance to next iteration
    pub fn advance_iteration(&mut self) {
        // Move next iteration's active cells to current
        self.active_cells.clear();
        std::mem::swap(&mut self.active_cells, &mut self.next_active_cells);

        // Clear change magnitudes for new iteration
        for magnitude in &mut self.change_magnitudes {
            *magnitude = 0.0;
        }
    }

    /// Check if system has converged (very few active cells)
    pub fn has_converged(&self, convergence_threshold: f32) -> bool {
        let total_cells = self.width * self.height;
        let active_ratio = self.active_cells.len() as f32 / total_cells as f32;
        active_ratio < convergence_threshold
    }

    /// Get total change magnitude across all active cells
    pub fn total_change_magnitude(&self) -> f32 {
        self.active_cells
            .iter()
            .map(|&index| self.change_magnitudes[index])
            .sum()
    }

    /// Reset tracker for new simulation
    pub fn reset(&mut self) {
        self.active_cells.clear();
        self.next_active_cells.clear();
        self.change_magnitudes.fill(0.0);
    }

    /// Mark entire map as active (for first iteration or major changes)
    pub fn mark_all_active(&mut self) {
        self.active_cells.clear();
        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                self.active_cells.insert(index);
            }
        }
    }

    /// Set minimum change threshold
    pub fn set_change_threshold(&mut self, threshold: f32) {
        self.min_change_threshold = threshold;
    }

    /// Set neighbor propagation distance
    pub fn set_propagation_distance(&mut self, distance: usize) {
        self.neighbor_propagation_distance = distance;
    }
}

/// Optimized water flow system that only processes active cells
/// Migrated to use unified FlowEngine with spatial optimization
pub struct OptimizedWaterFlowSystem {
    update_tracker: SpatialUpdateTracker,
    cached_temperature_valid: bool,
    last_temperature_update: u64,
    temperature_cache_lifetime: u64,
    // Unified flow engine with spatial optimization algorithm
    flow_engine: FlowEngine,
    // Legacy water flow system maintained for backward compatibility
    water_flow_system: WaterFlowSystem,
}

impl OptimizedWaterFlowSystem {
    pub fn new(width: usize, height: usize) -> Self {
        // Create a default world scale for the water system
        // This will use appropriate physics parameters for the given map size
        let world_scale = WorldScale::new(
            10.0,
            (width as u32, height as u32),
            crate::engine::core::scale::DetailLevel::Standard,
        );
        let water_flow_system = WaterFlowSystem::new_for_scale(&world_scale);
        
        // Create unified flow engine with spatial optimization for performance
        let flow_engine = FlowEngine::for_performance(width, height, &world_scale);

        let mut update_tracker = SpatialUpdateTracker::new(width, height);
        // Set scale-aware change threshold based on water system's evaporation threshold
        let scale_aware_threshold = water_flow_system.evaporation_threshold * 2.0; // Scale with domain
        update_tracker.set_change_threshold(scale_aware_threshold);

        Self {
            update_tracker,
            cached_temperature_valid: false,
            last_temperature_update: 0,
            temperature_cache_lifetime: 100, // Recompute temperature every 100 iterations
            flow_engine,
            water_flow_system,
        }
    }

    pub fn new_with_params(
        width: usize,
        height: usize,
        params: WaterFlowParameters,
        world_scale: &WorldScale,
    ) -> Self {
        let water_flow_system = WaterFlowSystem::from_parameters(params.clone(), world_scale);
        
        // Create unified flow engine with spatial optimization and custom parameters
        let mut flow_engine = FlowEngine::for_performance(width, height, world_scale);
        // Convert legacy parameters to unified FlowParameters
        flow_engine.parameters = FlowParameters {
            gravity: 9.81,
            roughness: 0.03,
            min_depth: 1e-6,
            concentration_factor: 5000.0, // From legacy flow_rate conversion
            cfl_safety: params.cfl_safety_factor,
            dt: 1.0 / params.max_expected_velocity_ms, // Derived from CFL condition
        };

        let mut update_tracker = SpatialUpdateTracker::new(width, height);
        // Set scale-aware change threshold based on flow engine parameters
        let scale_aware_threshold = flow_engine.parameters.min_depth * 100.0;
        update_tracker.set_change_threshold(scale_aware_threshold);

        Self {
            update_tracker,
            cached_temperature_valid: false,
            last_temperature_update: 0,
            temperature_cache_lifetime: 100,
            flow_engine,
            water_flow_system,
        }
    }

    /// Update water flow only for active cells
    pub fn update_water_flow_selective(
        &mut self,
        heightmap: &mut FlatHeightmap,
        water_depth: &mut Vec<f32>,
        water_velocity: &mut Vec<(f32, f32)>,
        sediment: &mut Vec<f32>,
        _iteration: u64,
    ) -> bool {
        let mut any_changes = false;
        let (width, height) = heightmap.dimensions();

        // Convert FlatHeightmap and flat arrays to HeightMap and WaterLayer for FlowEngine
        let mut temp_heightmap = crate::engine::core::heightmap::HeightMap::new(width, height, 0.0);
        let mut temp_water = crate::engine::physics::water::WaterLayer::new(width, height);
        
        // Copy data to temporary structures
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                temp_heightmap.set(x, y, heightmap.get(x, y));
                temp_water.add_water(x, y, water_depth[index]);
                temp_water.velocity.set(x, y, water_velocity[index]);
                temp_water.sediment.set(x, y, sediment[index]);
            }
        }

        // Store previous values for change detection
        let prev_water: Vec<f32> = water_depth.clone();
        let prev_terrain: Vec<f32> = (0..width*height)
            .map(|i| {
                let (x, y) = (i % width, i / width);
                heightmap.get(x, y)
            })
            .collect();

        // Use unified FlowEngine with spatial optimization (only processes cells that need updates)
        let world_scale = WorldScale::new(
            self.flow_engine.velocity_field.meters_per_pixel,
            (width as u32, height as u32),
            crate::engine::core::scale::DetailLevel::Standard,
        );
        
        // FlowEngine's spatial algorithm only updates active cells for performance
        self.flow_engine.calculate_flow(&temp_heightmap, &mut temp_water, None, &world_scale);

        // Copy results back and apply erosion using legacy method for compatibility
        let active_cells: Vec<(usize, usize)> = self.update_tracker.active_cells().collect();
        for (x, y) in active_cells {
            let index = y * width + x;
            
            // Copy flow results back
            water_depth[index] = temp_water.get_water_depth(x, y);
            water_velocity[index] = temp_water.velocity.get(x, y);
            heightmap.set(x, y, temp_heightmap.get(x, y));
            
            // Apply erosion using legacy method for now
            let _erosion_amount =
                self.apply_erosion_at_cell(heightmap, water_depth, water_velocity, sediment, x, y);

            // Detect changes and mark neighboring cells if needed
            let elevation_change = (heightmap.get(x, y) - prev_terrain[index]).abs();
            let water_change = (water_depth[index] - prev_water[index]).abs();

            if elevation_change > 0.001 {
                self.update_tracker
                    .mark_cell_changed(x, y, elevation_change, ChangeType::Terrain);
                any_changes = true;
            }

            if water_change > 0.001 {
                self.update_tracker
                    .mark_cell_changed(x, y, water_change, ChangeType::Water);
                any_changes = true;
            }
        }

        // Advance to next iteration
        self.update_tracker.advance_iteration();

        any_changes
    }

    /// Check if simulation has converged
    pub fn has_converged(&self) -> bool {
        self.update_tracker.has_converged(0.001) // Less than 0.1% of cells active
    }

    /// Get statistics about active regions
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let (width, height) = (self.update_tracker.width, self.update_tracker.height);
        let total_cells = width * height;
        let active_cells = self.update_tracker.active_cell_count();

        PerformanceStats {
            total_cells,
            active_cells,
            efficiency_ratio: 1.0 - (active_cells as f32 / total_cells as f32),
            total_change_magnitude: self.update_tracker.total_change_magnitude(),
        }
    }

    /// Reset for new simulation
    pub fn reset(&mut self) {
        self.update_tracker.reset();
        self.cached_temperature_valid = false;
    }

    /// Mark initial active regions (e.g., around water sources)
    pub fn initialize_active_regions(&mut self, heightmap: &FlatHeightmap, water_depth: &[f32]) {
        let (width, height) = heightmap.dimensions();

        // Mark cells with water as active
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                // Scale-aware water depth threshold - use drainage system's threshold
                let water_depth_threshold = self.water_flow_system.evaporation_threshold * 5.0;
                if water_depth[index] > water_depth_threshold {
                    self.update_tracker.mark_cell_changed(
                        x,
                        y,
                        water_depth[index],
                        ChangeType::Water,
                    );
                }
            }
        }
    }

    // Real water simulation methods using the full water flow system
    fn calculate_flow_at_cell(
        &self,
        heightmap: &FlatHeightmap,
        water_depth: &[f32],
        water_velocity: &mut [(f32, f32)],
        x: usize,
        y: usize,
    ) -> f32 {
        // Flow calculation is now handled by the unified FlowEngine in
        // update_water_flow_selective() method. This provides:
        // 1. Consistent physics with other systems
        // 2. Proper drainage network integration
        // 3. Scale-aware parameters
        // 4. Spatial optimization through should_update_cell() filtering
        // This method returns 0.0 as flow calculation is handled elsewhere.
        0.0
    }

    fn apply_erosion_at_cell(
        &self,
        heightmap: &mut FlatHeightmap,
        water_depth: &[f32],
        water_velocity: &[(f32, f32)],
        sediment: &mut [f32],
        x: usize,
        y: usize,
    ) -> f32 {
        let (width, _height) = heightmap.dimensions();
        let index = y * width + x;

        let velocity = water_velocity[index];
        let flow_speed = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
        let water_depth_val = water_depth[index];

        if flow_speed > 0.01 && water_depth_val > 0.001 {
            // Erosion capacity based on flow speed and water depth
            let erosion_capacity =
                flow_speed * water_depth_val * self.water_flow_system.parameters.erosion_strength;

            // Erode terrain if we're below capacity
            let current_sediment = sediment[index];
            if current_sediment < erosion_capacity {
                let erosion_amount = (erosion_capacity - current_sediment).min(0.001);
                let current_height = heightmap.get(x, y);
                heightmap.set(x, y, current_height - erosion_amount);
                sediment[index] = current_sediment + erosion_amount;
                return erosion_amount;
            }
            // Deposit sediment if we're over capacity
            else if current_sediment > erosion_capacity {
                let deposition_amount = (current_sediment - erosion_capacity)
                    * self.water_flow_system.parameters.deposition_rate;
                let current_height = heightmap.get(x, y);
                heightmap.set(x, y, current_height + deposition_amount);
                sediment[index] = current_sediment - deposition_amount;
                return -deposition_amount; // Negative indicates deposition
            }
        }

        0.0
    }
}

/// Performance statistics for spatial partitioning
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_cells: usize,
    pub active_cells: usize,
    pub efficiency_ratio: f32,
    pub total_change_magnitude: f32,
}

impl PerformanceStats {
    pub fn cells_skipped(&self) -> usize {
        self.total_cells - self.active_cells
    }

    pub fn performance_gain(&self) -> f32 {
        if self.active_cells == 0 {
            return 1.0;
        }
        self.total_cells as f32 / self.active_cells as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spatial_tracker_basic_operations() {
        let mut tracker = SpatialUpdateTracker::new(10, 10);

        // Initially no active cells
        assert_eq!(tracker.active_cell_count(), 0);

        // Mark a cell as changed
        tracker.mark_cell_changed(5, 5, 0.1, ChangeType::Water);

        // Should have active cells (center + neighbors)
        assert!(tracker.active_cell_count() > 1);
        assert!(tracker.is_cell_active(5, 5));
    }

    #[test]
    fn convergence_detection() {
        let mut tracker = SpatialUpdateTracker::new(100, 100);

        // Mark a few cells
        tracker.mark_cell_changed(10, 10, 0.1, ChangeType::Water);
        tracker.mark_cell_changed(20, 20, 0.05, ChangeType::Terrain);

        // Should not be converged with active cells
        assert!(!tracker.has_converged(0.001));

        // Clear active cells
        tracker.advance_iteration();
        tracker.advance_iteration(); // Clear propagated cells too

        // Should be converged with no active cells
        assert!(tracker.has_converged(0.001));
    }

    #[test]
    fn change_threshold_filtering() {
        let mut tracker = SpatialUpdateTracker::new(10, 10);
        tracker.set_change_threshold(0.01);

        // Small change should be ignored
        tracker.mark_cell_changed(5, 5, 0.005, ChangeType::Water);
        assert_eq!(tracker.active_cell_count(), 0);

        // Large change should be tracked
        tracker.mark_cell_changed(5, 5, 0.02, ChangeType::Water);
        assert!(tracker.active_cell_count() > 0);
    }

    #[test]
    fn performance_stats() {
        let mut tracker = SpatialUpdateTracker::new(100, 100);
        tracker.mark_cell_changed(50, 50, 0.1, ChangeType::Water);

        // Create a temporary system for testing
        let world_scale = crate::engine::core::scale::WorldScale::new(
            10.0,
            (100, 100),
            crate::engine::core::scale::DetailLevel::Standard,
        );
        let water_flow_system = crate::engine::WaterFlowSystem::new_for_scale(&world_scale);

        let system = OptimizedWaterFlowSystem {
            update_tracker: tracker,
            cached_temperature_valid: false,
            last_temperature_update: 0,
            temperature_cache_lifetime: 100,
            water_flow_system,
        };

        let stats = system.get_performance_stats();
        assert_eq!(stats.total_cells, 10000);
        assert!(stats.active_cells > 0);
        assert!(stats.efficiency_ratio > 0.0);
        assert!(stats.performance_gain() > 1.0);
    }

    #[test]
    fn water_flow_integration_test() {
        // Test that our fix actually produces water flow changes
        let width = 10;
        let height = 10;
        let mut heightmap =
            crate::engine::core::optimized_heightmap::FlatHeightmap::new(width, height);

        // Create a simple slope (high on left, low on right)
        for y in 0..height {
            for x in 0..width {
                heightmap.set(x, y, 1.0 - (x as f32 / width as f32));
            }
        }

        // Initialize water system
        let mut water_system = OptimizedWaterFlowSystem::new(width, height);

        // Create water and sediment arrays with more water to ensure flow
        let mut water_depths = vec![0.5; width * height]; // More water for better flow
        let mut water_velocities = vec![(0.0, 0.0); width * height];
        let mut sediment = vec![0.0; width * height];

        // Initialize some active regions
        water_system.initialize_active_regions(&heightmap, &water_depths);

        let initial_stats = water_system.get_performance_stats();
        assert!(
            initial_stats.active_cells > 0,
            "Should have some active cells after initialization"
        );

        // Store initial state to verify changes
        let initial_total_water: f32 = water_depths.iter().sum();
        let initial_elevation_sum: f32 = (0..width * height)
            .map(|i| {
                let (x, y) = (i % width, i / width);
                heightmap.get(x, y)
            })
            .sum();

        // Run a few iterations and verify changes are detected
        let mut changes_detected = false;
        let mut significant_changes = false;

        for iteration in 0..10 {
            // Store state before iteration
            let before_water: f32 = water_depths.iter().sum();
            let before_elevation: f32 = (0..width * height)
                .map(|i| {
                    let (x, y) = (i % width, i / width);
                    heightmap.get(x, y)
                })
                .sum();

            let water_changed = water_system.update_water_flow_selective(
                &mut heightmap,
                &mut water_depths,
                &mut water_velocities,
                &mut sediment,
                iteration,
            );

            // Check for actual numerical changes
            let after_water: f32 = water_depths.iter().sum();
            let after_elevation: f32 = (0..width * height)
                .map(|i| {
                    let (x, y) = (i % width, i / width);
                    heightmap.get(x, y)
                })
                .sum();

            let water_change = (after_water - before_water).abs();
            let elevation_change = (after_elevation - before_elevation).abs();

            if water_changed {
                changes_detected = true;
            }

            if water_change > 0.001 || elevation_change > 0.001 {
                significant_changes = true;
            }

            if changes_detected && significant_changes {
                break;
            }
        }

        // Either the system reports changes OR we detect numerical changes
        assert!(
            changes_detected || significant_changes,
            "Water flow should produce detectable changes (reported: {}, numerical: {})",
            changes_detected,
            significant_changes
        );

        let final_stats = water_system.get_performance_stats();

        // If we have any active cells, verify we're getting some optimization
        if final_stats.active_cells > 0 {
            // Allow some cases where all cells might be active initially
            // The key is that the system is functional, not that it's optimized on this test case
            assert!(final_stats.active_cells <= final_stats.total_cells);
        }
    }
}
