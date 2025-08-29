// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Integration interface specifications and usage examples for temporal scaling framework
// ABOUTME: Shows how ecosystem_feedback.rs and other temporal processes will use TemporalScalingService

use super::temporal_scaling::{TemporalScalingService, TemporalMode, TemporalScalingConfig};

/// Example integration pattern for ecosystem_feedback.rs
/// 
/// This demonstrates how the ecosystem feedback system will integrate with temporal scaling
/// to preserve bit-perfect Demo mode behavior while enabling realistic scientific rates.
pub struct EcosystemFeedbackIntegrationExample {
    temporal_scaling: TemporalScalingService,
    // Would include actual ecosystem parameters in real implementation
}

impl EcosystemFeedbackIntegrationExample {
    pub fn new(config: TemporalScalingConfig) -> Self {
        Self {
            temporal_scaling: TemporalScalingService::new(config),
        }
    }

    /// Integration pattern for ecosystem_feedback.rs:272 growth rate scaling
    /// 
    /// This method demonstrates the exact integration pattern that will be used
    /// to replace the current line:
    /// ```rust
    /// growth_rate: 10.0,  // 10 kg/m²/day growth under optimal conditions
    /// ```
    /// 
    /// With temporal scaling:
    /// ```rust
    /// let scaled_growth_rate = temporal_scaling.scale_ecosystem_growth_rate(10.0, dt_hours);
    /// ```
    pub fn scale_growth_rate_example(&self, dt_hours: f64) -> f64 {
        let base_growth_rate_per_day = 10.0; // kg/m²/day - current value from line 272
        
        // This is the exact integration pattern for ecosystem_feedback.rs
        self.temporal_scaling.scale_ecosystem_growth_rate(base_growth_rate_per_day, dt_hours)
    }

    /// Example showing how different temporal modes affect the same growth rate
    pub fn demonstrate_scaling_modes(&self, dt_hours: f64) -> ScalingModeComparison {
        let base_rate = 10.0; // kg/m²/day

        ScalingModeComparison {
            base_rate,
            demo_mode_result: {
                let demo_service = TemporalScalingService::new(TemporalScalingConfig {
                    mode: TemporalMode::Demo,
                    ..Default::default()
                });
                demo_service.scale_ecosystem_growth_rate(base_rate, dt_hours)
            },
            realistic_mode_result: {
                let realistic_service = TemporalScalingService::new(TemporalScalingConfig {
                    mode: TemporalMode::Realistic,
                    ..Default::default()
                });
                realistic_service.scale_ecosystem_growth_rate(base_rate, dt_hours)
            },
            research_mode_result: {
                let research_service = TemporalScalingService::new(TemporalScalingConfig {
                    mode: TemporalMode::Research,
                    custom_scaling_factor: 0.1, // 10% of demo rate
                    ..Default::default()
                });
                research_service.scale_ecosystem_growth_rate(base_rate, dt_hours)
            },
        }
    }

    /// Integration pattern for other temporal processes
    /// 
    /// Shows how different types of temporal processes can be scaled
    pub fn demonstrate_process_scaling(&self, dt_hours: f64) -> ProcessScalingExamples {
        ProcessScalingExamples {
            biological_process: self.temporal_scaling.scale_biological_rate(5.0, dt_hours),
            geological_process: self.temporal_scaling.scale_geological_rate(0.001, dt_hours), 
            atmospheric_process: self.temporal_scaling.scale_atmospheric_rate(1.2, dt_hours),
        }
    }

    /// Performance benchmarking example
    /// 
    /// Demonstrates that temporal scaling has minimal performance impact
    pub fn performance_benchmark_example(&self) -> PerformanceBenchmark {
        let iterations = 1_000_000;
        let dt_hours = 0.1;
        let base_rate = 10.0;

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _scaled_rate = self.temporal_scaling.scale_ecosystem_growth_rate(base_rate, dt_hours);
        }
        let duration = start.elapsed();

        PerformanceBenchmark {
            iterations,
            total_duration: duration,
            per_operation: duration / iterations,
            operations_per_second: iterations as f64 / duration.as_secs_f64(),
        }
    }
}

/// Results of comparing scaling modes
#[derive(Debug, Clone)]
pub struct ScalingModeComparison {
    pub base_rate: f64,
    pub demo_mode_result: f64,
    pub realistic_mode_result: f64,
    pub research_mode_result: f64,
}

impl ScalingModeComparison {
    /// Show the scaling factors applied by each mode
    pub fn scaling_factors(&self) -> ScalingFactors {
        ScalingFactors {
            demo_factor: self.demo_mode_result / self.base_rate,
            realistic_factor: self.realistic_mode_result / self.base_rate, 
            research_factor: self.research_mode_result / self.base_rate,
        }
    }

    /// Validate that demo mode preserves exact behavior
    pub fn validate_demo_preservation(&self, dt_hours: f64) -> bool {
        let expected_demo_result = self.base_rate * dt_hours / 24.0; // Current behavior
        (self.demo_mode_result - expected_demo_result).abs() < 1e-10
    }
}

#[derive(Debug, Clone)]
pub struct ScalingFactors {
    pub demo_factor: f64,
    pub realistic_factor: f64,
    pub research_factor: f64,
}

/// Examples of scaling different process types
#[derive(Debug, Clone)]
pub struct ProcessScalingExamples {
    pub biological_process: f64,
    pub geological_process: f64,
    pub atmospheric_process: f64,
}

/// Performance benchmark results
#[derive(Debug, Clone)]
pub struct PerformanceBenchmark {
    pub iterations: u32,
    pub total_duration: std::time::Duration,
    pub per_operation: std::time::Duration,
    pub operations_per_second: f64,
}

/// Integration interface for simulation setup
/// 
/// This shows how the simulation system will create and manage temporal scaling services
pub struct SimulationTemporalScalingInterface;

impl SimulationTemporalScalingInterface {
    /// Create temporal scaling service from configuration
    /// 
    /// This demonstrates how the simulation will instantiate temporal scaling
    /// from the configuration system
    pub fn create_from_config(config: &crate::engine::config::SimulationDefaults) -> TemporalScalingService {
        TemporalScalingService::new(config.temporal_scaling.clone())
    }

    /// Integration point for ecosystem feedback system
    /// 
    /// Shows how ecosystem_feedback.rs will receive the temporal scaling service
    pub fn integrate_with_ecosystem_feedback(
        temporal_scaling: &TemporalScalingService,
    ) -> EcosystemFeedbackTemporalIntegration {
        EcosystemFeedbackTemporalIntegration {
            scaling_service: temporal_scaling,
        }
    }
}

/// Temporal integration wrapper for ecosystem feedback
/// 
/// This represents the pattern that will be used in ecosystem_feedback.rs
pub struct EcosystemFeedbackTemporalIntegration<'a> {
    scaling_service: &'a TemporalScalingService,
}

impl<'a> EcosystemFeedbackTemporalIntegration<'a> {
    /// Apply temporal scaling to growth rates
    /// 
    /// This method shows the exact integration pattern for line 272 in ecosystem_feedback.rs
    pub fn apply_growth_rate_scaling(&self, base_growth_rate: f64, dt_hours: f64) -> f64 {
        self.scaling_service.scale_ecosystem_growth_rate(base_growth_rate, dt_hours)
    }

    /// Apply temporal scaling to other biological processes
    /// 
    /// This shows how other rates in the ecosystem feedback system can be scaled
    pub fn apply_biological_rate_scaling(&self, base_rate: f64, dt_hours: f64) -> f64 {
        self.scaling_service.scale_biological_rate(base_rate, dt_hours)
    }

    /// Get current temporal mode for conditional logic
    /// 
    /// This allows ecosystem feedback to adjust behavior based on temporal mode
    pub fn get_temporal_mode(&self) -> TemporalMode {
        self.scaling_service.mode()
    }

    /// Check if we're in demo mode for bit-perfect preservation
    /// 
    /// This allows conditional logic that ensures exact demo behavior
    pub fn is_demo_mode(&self) -> bool {
        matches!(self.scaling_service.mode(), TemporalMode::Demo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_preserves_demo_behavior() {
        let demo_config = TemporalScalingConfig {
            mode: TemporalMode::Demo,
            ..Default::default()
        };
        
        let integration = EcosystemFeedbackIntegrationExample::new(demo_config);
        let dt_hours = 0.1;
        
        let scaled_rate = integration.scale_growth_rate_example(dt_hours);
        let expected_rate = 10.0 * dt_hours / 24.0; // Current behavior
        
        assert!((scaled_rate - expected_rate).abs() < 1e-10);
    }

    #[test]
    fn test_integration_applies_realistic_scaling() {
        let realistic_config = TemporalScalingConfig {
            mode: TemporalMode::Realistic,
            ..Default::default()
        };
        
        let integration = EcosystemFeedbackIntegrationExample::new(realistic_config);
        let dt_hours = 0.1;
        
        let scaled_rate = integration.scale_growth_rate_example(dt_hours);
        let demo_rate = 10.0 * dt_hours / 24.0;
        
        // Realistic mode should produce much smaller rates
        assert!(scaled_rate < demo_rate * 0.001);
    }

    #[test]
    fn test_scaling_mode_comparison() {
        let demo_config = TemporalScalingConfig {
            mode: TemporalMode::Demo,
            ..Default::default()
        };
        
        let integration = EcosystemFeedbackIntegrationExample::new(demo_config);
        let dt_hours = 0.1;
        
        let comparison = integration.demonstrate_scaling_modes(dt_hours);
        
        // Validate demo mode preservation
        assert!(comparison.validate_demo_preservation(dt_hours));
        
        // Validate scaling relationships
        let factors = comparison.scaling_factors();
        assert!((factors.demo_factor - dt_hours / 24.0).abs() < 1e-10);
        assert!(factors.realistic_factor < factors.demo_factor * 0.001);
        assert!(factors.research_factor < factors.demo_factor);
    }

    #[test]
    fn test_performance_benchmark() {
        let demo_config = TemporalScalingConfig {
            mode: TemporalMode::Demo,
            ..Default::default()
        };
        
        let integration = EcosystemFeedbackIntegrationExample::new(demo_config);
        let benchmark = integration.performance_benchmark_example();
        
        // Temporal scaling should be very fast
        assert!(benchmark.operations_per_second > 1_000_000.0); // At least 1M ops/sec
        assert!(benchmark.per_operation.as_nanos() < 1000); // Less than 1 microsecond per operation
    }

    #[test] 
    fn test_simulation_integration_interface() {
        use crate::engine::config::SimulationDefaults;
        
        let defaults = SimulationDefaults::default();
        let temporal_scaling = SimulationTemporalScalingInterface::create_from_config(&defaults);
        
        let integration = SimulationTemporalScalingInterface::integrate_with_ecosystem_feedback(&temporal_scaling);
        
        // Test integration methods
        let scaled_growth = integration.apply_growth_rate_scaling(10.0, 0.1);
        assert!(scaled_growth > 0.0);
        
        let scaled_bio = integration.apply_biological_rate_scaling(5.0, 0.1);
        assert!(scaled_bio > 0.0);
        
        // Test mode queries
        assert_eq!(integration.get_temporal_mode(), TemporalMode::Demo);
        assert!(integration.is_demo_mode());
    }
}