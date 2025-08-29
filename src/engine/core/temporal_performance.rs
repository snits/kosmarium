// ABOUTME: Performance monitoring and statistics for temporal scaling operations
// ABOUTME: Tracks scaling overhead, operation counts, and provides performance transparency for scientific validation

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Performance monitoring for temporal scaling operations
///
/// Provides real-time performance statistics to ensure temporal scaling
/// introduces minimal overhead while maintaining scientific transparency
pub struct TemporalPerformanceMonitor {
    /// Total number of scaling operations performed
    scaling_call_count: u64,
    /// Cumulative time spent in scaling operations
    total_scaling_time: Duration,
    /// Timestamp when monitoring started
    start_time: Instant,
    /// Last performance summary generated
    last_performance_summary: Option<PerformanceSummary>,
    /// Running average operation time (for efficiency)
    running_average_time: Duration,
}

/// Performance statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Total scaling operations performed
    pub total_scaling_operations: u64,
    /// Average time per scaling operation
    pub average_operation_time: Duration,
    /// Scaling operations per second
    pub operations_per_second: f64,
    /// Percentage of total simulation time spent in scaling
    pub percentage_of_simulation_time: f64,
    /// Human-readable assessment of scaling overhead
    pub scaling_overhead_assessment: String,
    /// Timestamp when this summary was generated
    pub snapshot_time: Duration,
}

/// Current performance statistics for real-time display
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Operations in the last measurement period
    pub recent_operations: u64,
    /// Operations per second in last period
    pub current_ops_per_second: f64,
    /// Current overhead percentage
    pub current_overhead_percent: f64,
    /// Performance trend (improving, stable, degrading)
    pub performance_trend: PerformanceTrend,
}

/// Performance trend indicator
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
}

impl TemporalPerformanceMonitor {
    /// Create new performance monitor
    pub fn new() -> Self {
        Self {
            scaling_call_count: 0,
            total_scaling_time: Duration::ZERO,
            start_time: Instant::now(),
            last_performance_summary: None,
            running_average_time: Duration::ZERO,
        }
    }

    /// Record a temporal scaling operation with its execution time
    ///
    /// This should be called after each scaling operation to track performance.
    /// For hot path performance, the timing measurement should be done externally.
    pub fn record_scaling_operation(&mut self, duration: Duration) {
        self.scaling_call_count += 1;
        self.total_scaling_time += duration;

        // Update running average for efficiency
        if self.scaling_call_count == 1 {
            self.running_average_time = duration;
        } else {
            // Exponential moving average with alpha = 0.1
            let alpha = 0.1;
            let new_time_nanos = duration.as_nanos() as f64;
            let avg_time_nanos = self.running_average_time.as_nanos() as f64;
            let new_avg = (alpha * new_time_nanos) + ((1.0 - alpha) * avg_time_nanos);
            self.running_average_time = Duration::from_nanos(new_avg as u64);
        }
    }

    /// Generate comprehensive performance summary
    ///
    /// # Arguments
    /// * `simulation_ticks` - Number of simulation ticks elapsed (for overhead calculation)
    ///
    /// # Returns
    /// Complete performance analysis including overhead assessment
    pub fn generate_summary(&mut self, simulation_ticks: u64) -> PerformanceSummary {
        let elapsed_time = self.start_time.elapsed();

        let average_operation_time = if self.scaling_call_count > 0 {
            self.total_scaling_time / self.scaling_call_count as u32
        } else {
            Duration::ZERO
        };

        let operations_per_second = if elapsed_time.as_secs_f64() > 0.0 {
            self.scaling_call_count as f64 / elapsed_time.as_secs_f64()
        } else {
            0.0
        };

        let percentage_of_simulation_time = if elapsed_time > Duration::ZERO {
            (self.total_scaling_time.as_secs_f64() / elapsed_time.as_secs_f64()) * 100.0
        } else {
            0.0
        };

        let scaling_overhead_assessment = assess_performance_overhead(
            percentage_of_simulation_time,
            operations_per_second,
            average_operation_time,
        );

        let summary = PerformanceSummary {
            total_scaling_operations: self.scaling_call_count,
            average_operation_time,
            operations_per_second,
            percentage_of_simulation_time,
            scaling_overhead_assessment,
            snapshot_time: elapsed_time,
        };

        self.last_performance_summary = Some(summary.clone());
        summary
    }

    /// Get current performance statistics for real-time monitoring
    pub fn current_stats(&self) -> PerformanceStats {
        let elapsed = self.start_time.elapsed();
        let current_ops_per_second = if elapsed.as_secs_f64() > 0.0 {
            self.scaling_call_count as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let current_overhead_percent = if elapsed > Duration::ZERO {
            (self.total_scaling_time.as_secs_f64() / elapsed.as_secs_f64()) * 100.0
        } else {
            0.0
        };

        let performance_trend = if let Some(ref last_summary) = self.last_performance_summary {
            if current_ops_per_second > last_summary.operations_per_second * 1.05 {
                PerformanceTrend::Improving
            } else if current_ops_per_second < last_summary.operations_per_second * 0.95 {
                PerformanceTrend::Degrading
            } else {
                PerformanceTrend::Stable
            }
        } else {
            PerformanceTrend::Stable
        };

        PerformanceStats {
            recent_operations: self.scaling_call_count,
            current_ops_per_second,
            current_overhead_percent,
            performance_trend,
        }
    }

    /// Reset performance monitoring (useful for benchmarking specific phases)
    pub fn reset(&mut self) {
        self.scaling_call_count = 0;
        self.total_scaling_time = Duration::ZERO;
        self.start_time = Instant::now();
        self.last_performance_summary = None;
        self.running_average_time = Duration::ZERO;
    }

    /// Check if performance is meeting scientific computing standards
    ///
    /// Scientific computing standard: < 5% overhead for auxiliary operations
    pub fn is_performance_acceptable(&self) -> bool {
        let elapsed = self.start_time.elapsed();
        if elapsed == Duration::ZERO {
            return true;
        }

        let overhead_percent =
            (self.total_scaling_time.as_secs_f64() / elapsed.as_secs_f64()) * 100.0;
        overhead_percent < 5.0
    }

    /// Get total operations performed
    pub fn total_operations(&self) -> u64 {
        self.scaling_call_count
    }

    /// Get total time spent in scaling operations
    pub fn total_scaling_time(&self) -> Duration {
        self.total_scaling_time
    }

    /// Get monitoring duration
    pub fn monitoring_duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for TemporalPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Assess performance overhead and provide human-readable evaluation
fn assess_performance_overhead(
    percentage_overhead: f64,
    ops_per_second: f64,
    avg_operation_time: Duration,
) -> String {
    let overhead_category = if percentage_overhead < 1.0 {
        "Excellent"
    } else if percentage_overhead < 2.5 {
        "Very Good"
    } else if percentage_overhead < 5.0 {
        "Acceptable"
    } else if percentage_overhead < 10.0 {
        "High"
    } else {
        "Excessive"
    };

    let efficiency_note = if ops_per_second > 1_000_000.0 {
        "Highly optimized"
    } else if ops_per_second > 100_000.0 {
        "Well optimized"
    } else if ops_per_second > 10_000.0 {
        "Adequately optimized"
    } else {
        "May benefit from optimization"
    };

    format!(
        "{} ({:.2}% overhead) - {}",
        overhead_category, percentage_overhead, efficiency_note
    )
}

/// Display performance statistics in a formatted way
impl PerformanceSummary {
    /// Format performance summary for console display
    pub fn format_for_console(&self) -> String {
        format!(
            "Temporal Scaling Performance:\n\
             ├─ Operations: {}\n\
             ├─ Avg time: {:.2} μs per operation\n\
             ├─ Throughput: {:.0} ops/sec\n\
             ├─ Overhead: {:.2}% of simulation time\n\
             └─ Assessment: {}",
            self.total_scaling_operations,
            self.average_operation_time.as_nanos() as f64 / 1000.0,
            self.operations_per_second,
            self.percentage_of_simulation_time,
            self.scaling_overhead_assessment
        )
    }

    /// Format performance summary for scientific reporting
    pub fn format_for_scientific_report(&self) -> String {
        format!(
            "Temporal Scaling Performance Report\n\
             ====================================\n\
             Total scaling operations: {}\n\
             Average operation time: {:.3} μs\n\
             Operations per second: {:.0}\n\
             Percentage of simulation time: {:.3}%\n\
             Performance assessment: {}\n\
             Monitoring duration: {:.2} seconds\n\
             \n\
             Performance meets scientific computing standards: {}",
            self.total_scaling_operations,
            self.average_operation_time.as_nanos() as f64 / 1000.0,
            self.operations_per_second,
            self.percentage_of_simulation_time,
            self.scaling_overhead_assessment,
            self.snapshot_time.as_secs_f64(),
            if self.percentage_of_simulation_time < 5.0 {
                "Yes"
            } else {
                "No"
            }
        )
    }
}

/// Helper for timing temporal scaling operations in hot paths
///
/// Usage example:
/// ```rust
/// let timer = TemporalScalingTimer::new();
/// let scaled_value = temporal_service.scale_biological_rate(rate, dt);
/// monitor.record_scaling_operation(timer.elapsed());
/// ```
pub struct TemporalScalingTimer {
    start: Instant,
}

impl TemporalScalingTimer {
    /// Start timing a temporal scaling operation
    #[inline]
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time since timer creation
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Default for TemporalScalingTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_monitor_basic_functionality() {
        let mut monitor = TemporalPerformanceMonitor::new();

        // Record some operations
        monitor.record_scaling_operation(Duration::from_nanos(100));
        monitor.record_scaling_operation(Duration::from_nanos(150));
        monitor.record_scaling_operation(Duration::from_nanos(200));

        assert_eq!(monitor.total_operations(), 3);
        assert!(monitor.total_scaling_time() > Duration::ZERO);
    }

    #[test]
    fn test_performance_summary_generation() {
        let mut monitor = TemporalPerformanceMonitor::new();

        // Record operations
        for _ in 0..1000 {
            monitor.record_scaling_operation(Duration::from_nanos(50));
        }

        let summary = monitor.generate_summary(100);

        assert_eq!(summary.total_scaling_operations, 1000);
        assert!(summary.operations_per_second > 0.0);
        assert!(summary.percentage_of_simulation_time >= 0.0);
    }

    #[test]
    fn test_performance_acceptable_threshold() {
        let mut monitor = TemporalPerformanceMonitor::new();

        // Sleep a small amount to ensure elapsed time > 0
        thread::sleep(Duration::from_millis(1));

        // Record very fast operations (should be acceptable)
        monitor.record_scaling_operation(Duration::from_nanos(10));

        assert!(monitor.is_performance_acceptable());
    }

    #[test]
    fn test_performance_stats() {
        let mut monitor = TemporalPerformanceMonitor::new();

        monitor.record_scaling_operation(Duration::from_nanos(100));

        let stats = monitor.current_stats();
        assert_eq!(stats.recent_operations, 1);
        assert!(stats.current_ops_per_second > 0.0);
    }

    #[test]
    fn test_timer_functionality() {
        let timer = TemporalScalingTimer::new();
        thread::sleep(Duration::from_nanos(100));

        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_nanos(100));
    }

    #[test]
    fn test_reset_functionality() {
        let mut monitor = TemporalPerformanceMonitor::new();

        monitor.record_scaling_operation(Duration::from_nanos(100));
        assert_eq!(monitor.total_operations(), 1);

        monitor.reset();
        assert_eq!(monitor.total_operations(), 0);
        assert_eq!(monitor.total_scaling_time(), Duration::ZERO);
    }

    #[test]
    fn test_overhead_assessment_categories() {
        // Test different overhead categories
        let low_overhead = assess_performance_overhead(0.5, 1_000_000.0, Duration::from_nanos(10));
        assert!(low_overhead.contains("Excellent"));

        let high_overhead = assess_performance_overhead(8.0, 10_000.0, Duration::from_micros(10));
        assert!(high_overhead.contains("High"));
    }

    #[test]
    fn test_performance_summary_formatting() {
        let summary = PerformanceSummary {
            total_scaling_operations: 1000,
            average_operation_time: Duration::from_nanos(150),
            operations_per_second: 500_000.0,
            percentage_of_simulation_time: 1.5,
            scaling_overhead_assessment: "Very Good (1.50% overhead) - Highly optimized"
                .to_string(),
            snapshot_time: Duration::from_secs(10),
        };

        let console_format = summary.format_for_console();
        assert!(console_format.contains("1,000"));
        assert!(console_format.contains("0.15 μs"));
        assert!(console_format.contains("500000 ops/sec"));

        let scientific_format = summary.format_for_scientific_report();
        assert!(scientific_format.contains("Performance Report"));
        assert!(scientific_format.contains("Yes")); // Should meet scientific standards
    }
}
