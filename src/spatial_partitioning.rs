// ABOUTME: Spatial partitioning system for selective cell updates in geological simulation
// ABOUTME: Implements multi-tier update regions for water/evaporation coupling and convergence tracking

use crate::optimized_heightmap::FlatHeightmap;
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

                    // Add to next iteration's active cells
                    if propagated_magnitude > self.min_change_threshold * 0.1 {
                        self.next_active_cells.insert(index);
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
pub struct OptimizedWaterFlowSystem {
    update_tracker: SpatialUpdateTracker,
    cached_temperature_valid: bool,
    last_temperature_update: u64,
    temperature_cache_lifetime: u64,
}

impl OptimizedWaterFlowSystem {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            update_tracker: SpatialUpdateTracker::new(width, height),
            cached_temperature_valid: false,
            last_temperature_update: 0,
            temperature_cache_lifetime: 100, // Recompute temperature every 100 iterations
        }
    }

    /// Update water flow only for active cells
    pub fn update_water_flow_selective(
        &mut self,
        heightmap: &mut FlatHeightmap,
        water_depth: &mut Vec<f32>,
        water_velocity: &mut Vec<(f32, f32)>,
        sediment: &mut Vec<f32>,
        iteration: u64,
    ) -> bool {
        let mut any_changes = false;
        let (width, height) = heightmap.dimensions();

        // Process only active cells
        for (x, y) in self.update_tracker.active_cells() {
            let index = y * width + x;

            // Store previous values to detect changes
            let prev_elevation = heightmap.get(x, y);
            let prev_water = water_depth[index];

            // Apply water flow, erosion, etc. only to this cell
            let flow_magnitude =
                self.calculate_flow_at_cell(heightmap, water_depth, water_velocity, x, y);
            let erosion_amount = self.apply_erosion_at_cell(heightmap, water_depth, sediment, x, y);

            // Detect changes and mark neighboring cells if needed
            let elevation_change = (heightmap.get(x, y) - prev_elevation).abs();
            let water_change = (water_depth[index] - prev_water).abs();

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
                if water_depth[index] > 0.01 {
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

    // Placeholder methods for actual water simulation logic
    fn calculate_flow_at_cell(
        &self,
        _heightmap: &FlatHeightmap,
        _water_depth: &[f32],
        _water_velocity: &mut [(f32, f32)],
        _x: usize,
        _y: usize,
    ) -> f32 {
        // Implementation would go here
        0.0
    }

    fn apply_erosion_at_cell(
        &self,
        _heightmap: &mut FlatHeightmap,
        _water_depth: &[f32],
        _sediment: &mut [f32],
        _x: usize,
        _y: usize,
    ) -> f32 {
        // Implementation would go here
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

        let system = OptimizedWaterFlowSystem {
            update_tracker: tracker,
            cached_temperature_valid: false,
            last_temperature_update: 0,
            temperature_cache_lifetime: 100,
        };

        let stats = system.get_performance_stats();
        assert_eq!(stats.total_cells, 10000);
        assert!(stats.active_cells > 0);
        assert!(stats.efficiency_ratio > 0.0);
        assert!(stats.performance_gain() > 1.0);
    }
}
