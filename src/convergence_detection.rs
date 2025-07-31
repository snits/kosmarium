// ABOUTME: Convergence detection system for early termination of geological simulation iterations
// ABOUTME: Implements multiple convergence criteria and adaptive stopping conditions for performance optimization

use crate::optimized_heightmap::FlatHeightmap;
use std::collections::VecDeque;

/// Different types of convergence criteria
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConvergenceCriterion {
    /// Total change magnitude falls below threshold
    TotalChangeMagnitude,
    /// Average change per cell falls below threshold  
    AverageChangePerCell,
    /// Maximum single-cell change falls below threshold
    MaximumSingleChange,
    /// Rate of change (derivative) approaches zero
    ChangeRateStabilization,
    /// Statistical variance in changes approaches zero
    ChangeVarianceStabilization,
}

/// Configuration for convergence detection
#[derive(Debug, Clone)]
pub struct ConvergenceConfig {
    /// Minimum iterations before convergence can be detected
    pub min_iterations: usize,

    /// Convergence thresholds for different criteria
    pub total_change_threshold: f32,
    pub average_change_threshold: f32,
    pub max_change_threshold: f32,
    pub change_rate_threshold: f32,
    pub variance_threshold: f32,

    /// Window size for rolling average calculations
    pub rolling_window_size: usize,

    /// How many consecutive iterations must meet criteria
    pub consecutive_iterations_required: usize,

    /// Which criteria must be satisfied (can require multiple)
    pub required_criteria: Vec<ConvergenceCriterion>,

    /// Enable adaptive thresholds that tighten over time
    pub adaptive_thresholds: bool,

    /// Progress reporting interval
    pub progress_report_interval: usize,
}

impl Default for ConvergenceConfig {
    fn default() -> Self {
        Self {
            min_iterations: 100,
            total_change_threshold: 0.001,
            average_change_threshold: 0.0001,
            max_change_threshold: 0.01,
            change_rate_threshold: 0.00001,
            variance_threshold: 0.000001,
            rolling_window_size: 50,
            consecutive_iterations_required: 10,
            required_criteria: vec![
                ConvergenceCriterion::AverageChangePerCell,
                ConvergenceCriterion::ChangeRateStabilization,
            ],
            adaptive_thresholds: true,
            progress_report_interval: 500,
        }
    }
}

/// Tracks convergence metrics over time
#[derive(Debug, Clone)]
pub struct ConvergenceTracker {
    config: ConvergenceConfig,

    // Historical data for analysis
    total_changes: VecDeque<f32>,
    average_changes: VecDeque<f32>,
    max_changes: VecDeque<f32>,

    // Convergence state
    iterations_meeting_criteria: usize,
    current_iteration: usize,
    is_converged: bool,
    convergence_iteration: Option<usize>,

    // Statistics
    initial_change_magnitude: Option<f32>,
    convergence_ratio: f32,
}

impl ConvergenceTracker {
    pub fn new(config: ConvergenceConfig) -> Self {
        let window_size = config.rolling_window_size;

        Self {
            config,
            total_changes: VecDeque::with_capacity(window_size),
            average_changes: VecDeque::with_capacity(window_size),
            max_changes: VecDeque::with_capacity(window_size),
            iterations_meeting_criteria: 0,
            current_iteration: 0,
            is_converged: false,
            convergence_iteration: None,
            initial_change_magnitude: None,
            convergence_ratio: 0.0,
        }
    }

    /// Record changes from current iteration and check convergence
    pub fn record_iteration(
        &mut self,
        old_heightmap: &FlatHeightmap,
        new_heightmap: &FlatHeightmap,
        water_changes: Option<f32>,
    ) -> ConvergenceResult {
        self.current_iteration += 1;

        // Calculate change metrics
        let change_metrics =
            self.calculate_change_metrics(old_heightmap, new_heightmap, water_changes);

        // Store in rolling windows
        self.store_metrics(&change_metrics);

        // Track initial magnitude for ratio calculations
        if self.initial_change_magnitude.is_none() {
            self.initial_change_magnitude = Some(change_metrics.total_change);
        }

        // Check convergence criteria
        let meets_criteria = self.check_convergence_criteria(&change_metrics);

        if meets_criteria {
            self.iterations_meeting_criteria += 1;
        } else {
            self.iterations_meeting_criteria = 0; // Reset counter
        }

        // Determine if converged
        let newly_converged = !self.is_converged
            && self.current_iteration >= self.config.min_iterations
            && self.iterations_meeting_criteria >= self.config.consecutive_iterations_required;

        if newly_converged {
            self.is_converged = true;
            self.convergence_iteration = Some(self.current_iteration);

            // Calculate convergence ratio
            if let Some(initial) = self.initial_change_magnitude {
                self.convergence_ratio = change_metrics.total_change / initial;
            }
        }

        // Generate progress report if needed
        let progress_info = if self.config.progress_report_interval > 0
            && self.current_iteration % self.config.progress_report_interval == 0
        {
            Some(self.generate_progress_report(&change_metrics))
        } else {
            None
        };

        ConvergenceResult {
            is_converged: self.is_converged,
            newly_converged,
            current_iteration: self.current_iteration,
            iterations_since_convergence: self
                .convergence_iteration
                .map(|c| self.current_iteration - c),
            change_metrics,
            progress_info,
            estimated_iterations_remaining: self.estimate_remaining_iterations(&change_metrics),
        }
    }

    /// Get current convergence status
    pub fn is_converged(&self) -> bool {
        self.is_converged
    }

    /// Get current iteration number
    pub fn current_iteration(&self) -> usize {
        self.current_iteration
    }

    /// Get convergence statistics
    pub fn get_convergence_stats(&self) -> ConvergenceStats {
        ConvergenceStats {
            total_iterations: self.current_iteration,
            convergence_iteration: self.convergence_iteration,
            convergence_ratio: self.convergence_ratio,
            criteria_met_iterations: self.iterations_meeting_criteria,
            final_total_change: self.total_changes.back().copied().unwrap_or(0.0),
            final_average_change: self.average_changes.back().copied().unwrap_or(0.0),
            final_max_change: self.max_changes.back().copied().unwrap_or(0.0),
        }
    }

    /// Reset tracker for new simulation
    pub fn reset(&mut self) {
        self.total_changes.clear();
        self.average_changes.clear();
        self.max_changes.clear();
        self.iterations_meeting_criteria = 0;
        self.current_iteration = 0;
        self.is_converged = false;
        self.convergence_iteration = None;
        self.initial_change_magnitude = None;
        self.convergence_ratio = 0.0;
    }

    // Private methods

    fn calculate_change_metrics(
        &self,
        old_heightmap: &FlatHeightmap,
        new_heightmap: &FlatHeightmap,
        water_changes: Option<f32>,
    ) -> ChangeMetrics {
        let old_data = old_heightmap.data();
        let new_data = new_heightmap.data();

        let mut total_change = 0.0;
        let mut max_change = 0.0;
        let mut change_count = 0;

        for (old_val, new_val) in old_data.iter().zip(new_data.iter()) {
            let change = (new_val - old_val).abs();
            total_change += change;
            max_change = max_change.max(change);

            if change > 0.0001 {
                // Count significant changes
                change_count += 1;
            }
        }

        // Include water changes if provided
        if let Some(water_change) = water_changes {
            total_change += water_change;
            max_change = max_change.max(water_change);
        }

        let average_change = if old_data.len() > 0 {
            total_change / old_data.len() as f32
        } else {
            0.0
        };

        ChangeMetrics {
            total_change,
            average_change,
            max_change,
            significant_changes: change_count,
        }
    }

    fn store_metrics(&mut self, metrics: &ChangeMetrics) {
        // Store in circular buffers
        if self.total_changes.len() >= self.config.rolling_window_size {
            self.total_changes.pop_front();
            self.average_changes.pop_front();
            self.max_changes.pop_front();
        }

        self.total_changes.push_back(metrics.total_change);
        self.average_changes.push_back(metrics.average_change);
        self.max_changes.push_back(metrics.max_change);
    }

    fn check_convergence_criteria(&self, current_metrics: &ChangeMetrics) -> bool {
        let mut criteria_met = 0;
        let total_criteria = self.config.required_criteria.len();

        for criterion in &self.config.required_criteria {
            if self.check_single_criterion(*criterion, current_metrics) {
                criteria_met += 1;
            }
        }

        criteria_met == total_criteria // All required criteria must be met
    }

    fn check_single_criterion(
        &self,
        criterion: ConvergenceCriterion,
        metrics: &ChangeMetrics,
    ) -> bool {
        match criterion {
            ConvergenceCriterion::TotalChangeMagnitude => {
                metrics.total_change
                    < self.get_adaptive_threshold(self.config.total_change_threshold)
            }

            ConvergenceCriterion::AverageChangePerCell => {
                metrics.average_change
                    < self.get_adaptive_threshold(self.config.average_change_threshold)
            }

            ConvergenceCriterion::MaximumSingleChange => {
                metrics.max_change < self.get_adaptive_threshold(self.config.max_change_threshold)
            }

            ConvergenceCriterion::ChangeRateStabilization => self.check_change_rate_stabilization(),

            ConvergenceCriterion::ChangeVarianceStabilization => {
                self.check_variance_stabilization()
            }
        }
    }

    fn get_adaptive_threshold(&self, base_threshold: f32) -> f32 {
        if !self.config.adaptive_thresholds {
            return base_threshold;
        }

        // Gradually tighten thresholds as simulation progresses
        let progress_factor = (self.current_iteration as f32 / 10000.0).min(1.0);
        let tightening_factor = 1.0 - progress_factor * 0.5; // Up to 50% tighter

        base_threshold * tightening_factor
    }

    fn check_change_rate_stabilization(&self) -> bool {
        if self.total_changes.len() < 10 {
            return false;
        }

        // Calculate rate of change (derivative)
        let recent_values: Vec<_> = self.total_changes.iter().rev().take(10).collect();
        let mut rate_sum = 0.0;

        for i in 1..recent_values.len() {
            let rate = (recent_values[i - 1] - recent_values[i]).abs();
            rate_sum += rate;
        }

        let average_rate = rate_sum / (recent_values.len() - 1) as f32;
        average_rate < self.config.change_rate_threshold
    }

    fn check_variance_stabilization(&self) -> bool {
        if self.average_changes.len() < 20 {
            return false;
        }

        // Calculate variance of recent changes
        let recent_changes: Vec<_> = self.average_changes.iter().rev().take(20).collect();
        let mean = recent_changes.iter().copied().sum::<f32>() / recent_changes.len() as f32;

        let variance = recent_changes
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>()
            / recent_changes.len() as f32;

        variance < self.config.variance_threshold
    }

    fn generate_progress_report(&self, metrics: &ChangeMetrics) -> ProgressInfo {
        let progress_ratio = if let Some(initial) = self.initial_change_magnitude {
            if initial > 0.0 {
                1.0 - (metrics.total_change / initial)
            } else {
                1.0
            }
        } else {
            0.0
        };

        ProgressInfo {
            iteration: self.current_iteration,
            total_change: metrics.total_change,
            average_change: metrics.average_change,
            max_change: metrics.max_change,
            progress_ratio: progress_ratio.clamp(0.0, 1.0),
            criteria_met_iterations: self.iterations_meeting_criteria,
        }
    }

    fn estimate_remaining_iterations(&self, current_metrics: &ChangeMetrics) -> Option<usize> {
        if self.current_iteration < 50 || current_metrics.total_change == 0.0 {
            return None;
        }

        // Simple linear extrapolation based on recent rate of change
        let recent_rate = self.calculate_recent_change_rate();
        if recent_rate <= 0.0 {
            return None;
        }

        let target_change = self.config.total_change_threshold;
        let remaining_change = current_metrics.total_change - target_change;

        if remaining_change <= 0.0 {
            Some(0)
        } else {
            Some((remaining_change / recent_rate) as usize)
        }
    }

    fn calculate_recent_change_rate(&self) -> f32 {
        if self.total_changes.len() < 10 {
            return 0.0;
        }

        let recent_values: Vec<_> = self.total_changes.iter().rev().take(10).collect();
        let start_value = *recent_values.last().unwrap();
        let end_value = *recent_values.first().unwrap();

        if start_value <= end_value {
            return 0.0;
        }

        (start_value - end_value) / recent_values.len() as f32
    }
}

/// Metrics calculated for each iteration
#[derive(Debug, Clone)]
pub struct ChangeMetrics {
    pub total_change: f32,
    pub average_change: f32,
    pub max_change: f32,
    pub significant_changes: usize,
}

/// Result of convergence check
#[derive(Debug, Clone)]
pub struct ConvergenceResult {
    pub is_converged: bool,
    pub newly_converged: bool,
    pub current_iteration: usize,
    pub iterations_since_convergence: Option<usize>,
    pub change_metrics: ChangeMetrics,
    pub progress_info: Option<ProgressInfo>,
    pub estimated_iterations_remaining: Option<usize>,
}

/// Progress information for reporting
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub iteration: usize,
    pub total_change: f32,
    pub average_change: f32,
    pub max_change: f32,
    pub progress_ratio: f32,
    pub criteria_met_iterations: usize,
}

/// Final convergence statistics
#[derive(Debug, Clone)]
pub struct ConvergenceStats {
    pub total_iterations: usize,
    pub convergence_iteration: Option<usize>,
    pub convergence_ratio: f32,
    pub criteria_met_iterations: usize,
    pub final_total_change: f32,
    pub final_average_change: f32,
    pub final_max_change: f32,
}

impl ConvergenceStats {
    pub fn iterations_saved(&self, max_iterations: usize) -> usize {
        if let Some(convergence_iter) = self.convergence_iteration {
            max_iterations.saturating_sub(convergence_iter)
        } else {
            0
        }
    }

    pub fn efficiency_gain(&self, max_iterations: usize) -> f32 {
        let saved = self.iterations_saved(max_iterations);
        saved as f32 / max_iterations as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convergence_detection_basic() {
        let config = ConvergenceConfig {
            min_iterations: 5,
            consecutive_iterations_required: 2,
            average_change_threshold: 0.01,
            required_criteria: vec![ConvergenceCriterion::AverageChangePerCell],
            ..Default::default()
        };

        let mut tracker = ConvergenceTracker::new(config);

        // Create heightmaps with decreasing changes
        let mut heightmap1 = FlatHeightmap::new(10, 10);
        let mut heightmap2 = FlatHeightmap::new(10, 10);

        // Large initial change
        heightmap2.set(5, 5, 0.1);
        let result = tracker.record_iteration(&heightmap1, &heightmap2, None);
        assert!(!result.is_converged);

        // Small change (below threshold)
        heightmap1 = heightmap2.clone();
        heightmap2.set(5, 5, 0.105); // 0.005 change
        let result = tracker.record_iteration(&heightmap1, &heightmap2, None);
        assert!(!result.is_converged); // Need consecutive iterations

        // Another small change
        heightmap1 = heightmap2.clone();
        heightmap2.set(5, 5, 0.108); // 0.003 change
        let result = tracker.record_iteration(&heightmap1, &heightmap2, None);

        // Should be converged after meeting min iterations
        for _ in 0..5 {
            heightmap1 = heightmap2.clone();
            heightmap2.set(5, 5, heightmap2.get(5, 5) + 0.001);
            let result = tracker.record_iteration(&heightmap1, &heightmap2, None);
            if result.is_converged {
                break;
            }
        }

        assert!(tracker.is_converged());
    }

    #[test]
    fn convergence_stats() {
        let config = ConvergenceConfig::default();
        let mut tracker = ConvergenceTracker::new(config);

        // Simulate convergence after 500 iterations
        let mut heightmap1 = FlatHeightmap::new(5, 5);
        let mut heightmap2 = FlatHeightmap::new(5, 5);

        for i in 0..600 {
            let change_amount = 0.1 * (-i as f32 * 0.01).exp(); // Exponential decay
            heightmap2.set(2, 2, change_amount);

            let result = tracker.record_iteration(&heightmap1, &heightmap2, None);
            if result.is_converged {
                break;
            }

            heightmap1 = heightmap2.clone();
        }

        let stats = tracker.get_convergence_stats();
        assert!(stats.convergence_iteration.is_some());
        assert!(stats.iterations_saved(10000) > 0);
        assert!(stats.efficiency_gain(10000) > 0.0);
    }

    #[test]
    fn adaptive_thresholds() {
        let mut config = ConvergenceConfig::default();
        config.adaptive_thresholds = true;

        let tracker = ConvergenceTracker::new(config);

        // Early iteration should have loose threshold
        let early_threshold = tracker.get_adaptive_threshold(0.01);
        assert_eq!(early_threshold, 0.01); // No change early on

        // Later iterations should have tighter thresholds
        let mut late_tracker = tracker.clone();
        late_tracker.current_iteration = 5000;
        let late_threshold = late_tracker.get_adaptive_threshold(0.01);
        assert!(late_threshold < 0.01);
    }
}
