// ABOUTME: Temporal scaling framework for realistic scientific simulation rates
// ABOUTME: Provides demo/realistic/research modes with configurable temporal scaling factors

use serde::{Deserialize, Serialize};

/// Temporal scaling modes for simulation realism
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalMode {
    /// Demo mode - preserves exact current behavior for demonstrations
    /// Scaling factor = 1.0 (no scaling)
    Demo,

    /// Realistic mode - scientific ecological time scales
    /// Scaling factor = 0.000274 (1/3650) for realistic annual rates
    Realistic,

    /// Research mode - configurable custom scaling factors
    /// Allows experimentation with different temporal scales
    Research,
}

impl Default for TemporalMode {
    fn default() -> Self {
        // Default to Demo mode for backward compatibility
        TemporalMode::Demo
    }
}

/// Configuration for temporal scaling system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalScalingConfig {
    /// Active temporal scaling mode
    pub mode: TemporalMode,

    /// Custom scaling factor for Research mode
    /// Ignored in Demo and Realistic modes
    pub custom_scaling_factor: f64,

    /// Whether to apply scaling to biological processes
    pub scale_biological: bool,

    /// Whether to apply scaling to geological processes  
    pub scale_geological: bool,

    /// Whether to apply scaling to atmospheric processes
    pub scale_atmospheric: bool,
}

impl Default for TemporalScalingConfig {
    fn default() -> Self {
        Self {
            mode: TemporalMode::Demo,
            custom_scaling_factor: 1.0,
            scale_biological: true,
            scale_geological: false, // Geological processes typically use different scales
            scale_atmospheric: false, // Atmospheric processes typically run at simulation timescale
        }
    }
}

/// Temporal scaling service for converting between demo and realistic time scales
pub struct TemporalScalingService {
    config: TemporalScalingConfig,

    // Pre-calculated scaling factors for performance
    biological_scaling_factor: f64,
    geological_scaling_factor: f64,
    atmospheric_scaling_factor: f64,
}

impl TemporalScalingService {
    /// Create temporal scaling configuration from study phenomenon preset
    ///
    /// This provides scientist-friendly presets that auto-configure temporal scaling
    /// based on research intent rather than implementation details.
    ///
    /// # Available Presets
    /// - `drought`: Long-term ecosystem stress analysis (0.2x realistic rate)
    /// - `ecosystem`: Natural biological growth cycles (realistic scientific rates)
    /// - `climate`: Climate-ecosystem coupling studies (realistic rates)
    /// - `storm`: Short-term weather dynamics (demo rate, atmospheric focus)
    ///
    /// # Arguments
    /// * `phenomenon` - Study phenomenon name (case-insensitive)
    ///
    /// # Returns
    /// * `Ok(TemporalScalingConfig)` - Configured for the specified phenomenon
    /// * `Err(String)` - Error message if phenomenon is unknown
    ///
    /// # Example
    /// ```rust
    /// use kosmarium::engine::core::{TemporalScalingService, TemporalMode};
    ///
    /// let config = TemporalScalingService::from_study_phenomenon("drought").unwrap();
    /// assert_eq!(config.mode, TemporalMode::Research);
    /// assert_eq!(config.custom_scaling_factor, 0.2);
    /// assert!(config.scale_biological);
    /// assert!(config.scale_atmospheric);
    /// assert!(!config.scale_geological);
    /// ```
    pub fn from_study_phenomenon(phenomenon: &str) -> Result<TemporalScalingConfig, String> {
        match phenomenon.to_lowercase().as_str() {
            "drought" => Ok(TemporalScalingConfig {
                mode: TemporalMode::Research,
                custom_scaling_factor: 0.2, // Slower for long-term drought effects
                scale_biological: true,     // Focus on ecosystem stress response
                scale_geological: false,    // Geological processes not relevant
                scale_atmospheric: true,    // Include precipitation/evaporation changes
            }),

            "ecosystem" => Ok(TemporalScalingConfig {
                mode: TemporalMode::Realistic, // Scientific ecological timescales
                custom_scaling_factor: 1.0,    // Not used in realistic mode
                scale_biological: true,        // Primary focus on biological processes
                scale_geological: false,       // Geological processes on different timescales
                scale_atmospheric: true,       // Include climate-ecosystem coupling
            }),

            "climate" => Ok(TemporalScalingConfig {
                mode: TemporalMode::Realistic, // Scientific climate timescales
                custom_scaling_factor: 1.0,    // Not used in realistic mode
                scale_biological: true,        // Include vegetation-climate feedback
                scale_geological: false,       // Geological processes too slow for climate studies
                scale_atmospheric: true,       // Primary focus on atmospheric processes
            }),

            "storm" => Ok(TemporalScalingConfig {
                mode: TemporalMode::Demo,   // Demo mode for observable storm dynamics
                custom_scaling_factor: 1.0, // Demo mode uses default scaling
                scale_biological: false,    // Biological processes too slow for storm timescales
                scale_geological: false,    // Geological processes irrelevant for storms
                scale_atmospheric: false,   // Let atmospheric processes run at simulation rate
            }),

            _ => Err(format!(
                "Unknown study phenomenon '{}'. Valid options: drought, ecosystem, climate, storm",
                phenomenon
            )),
        }
    }

    /// Create new temporal scaling service with configuration
    pub fn new(config: TemporalScalingConfig) -> Self {
        let mut service = Self {
            config,
            biological_scaling_factor: 1.0,
            geological_scaling_factor: 1.0,
            atmospheric_scaling_factor: 1.0,
        };

        service.update_scaling_factors();
        service
    }

    /// Update configuration and recalculate scaling factors
    pub fn update_config(&mut self, config: TemporalScalingConfig) {
        self.config = config;
        self.update_scaling_factors();
    }

    /// Get current temporal mode
    pub fn mode(&self) -> TemporalMode {
        self.config.mode
    }

    /// Get current configuration (immutable reference)
    pub fn config(&self) -> &TemporalScalingConfig {
        &self.config
    }

    /// Calculate scaling factors based on current configuration
    fn update_scaling_factors(&mut self) {
        let base_factor = match self.config.mode {
            TemporalMode::Demo => 1.0,
            TemporalMode::Realistic => 2.5 / 3650.0, // Scale to achieve 2.5 kg/m²/year target from 10.0 kg/m²/day
            TemporalMode::Research => self.config.custom_scaling_factor,
        };

        self.biological_scaling_factor = if self.config.scale_biological {
            base_factor
        } else {
            1.0
        };

        self.geological_scaling_factor = if self.config.scale_geological {
            base_factor
        } else {
            1.0
        };

        self.atmospheric_scaling_factor = if self.config.scale_atmospheric {
            base_factor
        } else {
            1.0
        };
    }

    /// Scale a biological process rate (e.g., growth, decay)
    ///
    /// # Arguments
    /// * `rate` - Rate per simulation tick (typically per day in demo mode)
    /// * `dt_hours` - Simulation timestep in hours
    ///
    /// # Returns
    /// Scaled rate appropriate for the current temporal mode
    pub fn scale_biological_rate(&self, rate: f64, dt_hours: f64) -> f64 {
        // Convert to rate per hour first, then apply scaling
        let rate_per_hour = rate * dt_hours / 24.0; // Assuming input rate is per day
        rate_per_hour * self.biological_scaling_factor
    }

    /// Scale a geological process rate (e.g., erosion, sediment transport)
    pub fn scale_geological_rate(&self, rate: f64, dt_hours: f64) -> f64 {
        let rate_per_hour = rate * dt_hours / 24.0;
        rate_per_hour * self.geological_scaling_factor
    }

    /// Scale an atmospheric process rate (e.g., evaporation, condensation)
    pub fn scale_atmospheric_rate(&self, rate: f64, dt_hours: f64) -> f64 {
        let rate_per_hour = rate * dt_hours / 24.0;
        rate_per_hour * self.atmospheric_scaling_factor
    }

    /// Get biological scaling factor (optimized for hot paths)
    #[inline]
    pub fn biological_scaling_factor(&self) -> f64 {
        self.biological_scaling_factor
    }

    /// Get geological scaling factor (optimized for hot paths)
    #[inline]
    pub fn geological_scaling_factor(&self) -> f64 {
        self.geological_scaling_factor
    }

    /// Get atmospheric scaling factor (optimized for hot paths)
    #[inline]
    pub fn atmospheric_scaling_factor(&self) -> f64 {
        self.atmospheric_scaling_factor
    }

    /// Convenience method: Scale ecosystem growth rate
    ///
    /// This method specifically handles the ecosystem_feedback.rs:272 case
    /// where growth_rate is 10.0 kg/m²/day
    pub fn scale_ecosystem_growth_rate(&self, base_growth_rate_per_day: f64, dt_hours: f64) -> f64 {
        match self.config.mode {
            TemporalMode::Demo => {
                // Demo mode: preserve exact current behavior
                // Return rate scaled only by dt_hours (existing behavior)
                base_growth_rate_per_day * dt_hours / 24.0
            }
            TemporalMode::Realistic => {
                // Realistic mode: apply scientific scaling
                // 10.0 kg/m²/day becomes ~0.00274 kg/m²/day for realistic annual rates
                let realistic_daily_rate =
                    base_growth_rate_per_day * self.biological_scaling_factor;
                realistic_daily_rate * dt_hours / 24.0
            }
            TemporalMode::Research => {
                // Research mode: custom scaling factor
                let custom_daily_rate = base_growth_rate_per_day * self.biological_scaling_factor;
                custom_daily_rate * dt_hours / 24.0
            }
        }
    }

    /// Get human-readable description of current scaling
    pub fn scaling_description(&self) -> String {
        match self.config.mode {
            TemporalMode::Demo => "Demo mode: Unscaled simulation time (1.0x)".to_string(),
            TemporalMode::Realistic => format!(
                "Realistic mode: Scientific ecological scaling ({:.6}x = 2.5 kg/m²/year target)",
                self.biological_scaling_factor
            ),
            TemporalMode::Research => format!(
                "Research mode: Custom scaling ({:.6}x)",
                self.config.custom_scaling_factor
            ),
        }
    }

    /// Get expected performance impact description
    pub fn performance_impact_description(&self) -> String {
        match self.config.mode {
            TemporalMode::Demo => "No performance impact (baseline)".to_string(),
            TemporalMode::Realistic | TemporalMode::Research => {
                "Minimal performance impact (<1% overhead for scaling calculations)".to_string()
            }
        }
    }
}

/// Builder for TemporalScalingService with fluent interface
pub struct TemporalScalingBuilder {
    config: TemporalScalingConfig,
}

impl TemporalScalingBuilder {
    pub fn new() -> Self {
        Self {
            config: TemporalScalingConfig::default(),
        }
    }

    pub fn mode(mut self, mode: TemporalMode) -> Self {
        self.config.mode = mode;
        self
    }

    pub fn custom_scaling_factor(mut self, factor: f64) -> Self {
        self.config.custom_scaling_factor = factor;
        self
    }

    pub fn scale_biological(mut self, enable: bool) -> Self {
        self.config.scale_biological = enable;
        self
    }

    pub fn scale_geological(mut self, enable: bool) -> Self {
        self.config.scale_geological = enable;
        self
    }

    pub fn scale_atmospheric(mut self, enable: bool) -> Self {
        self.config.scale_atmospheric = enable;
        self
    }

    pub fn build(self) -> TemporalScalingService {
        TemporalScalingService::new(self.config)
    }
}

impl Default for TemporalScalingBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_mode_preserves_behavior() {
        let service = TemporalScalingService::new(TemporalScalingConfig {
            mode: TemporalMode::Demo,
            ..Default::default()
        });

        // Demo mode should preserve exact scaling behavior
        let growth_rate = 10.0; // kg/m²/day
        let dt_hours = 0.1; // 0.1 hour timestep

        let scaled_rate = service.scale_ecosystem_growth_rate(growth_rate, dt_hours);
        let expected_rate = growth_rate * dt_hours / 24.0; // Current behavior

        assert!((scaled_rate - expected_rate).abs() < 1e-10);
        assert_eq!(service.biological_scaling_factor(), 1.0);
    }

    #[test]
    fn test_realistic_mode_scaling() {
        let service = TemporalScalingService::new(TemporalScalingConfig {
            mode: TemporalMode::Realistic,
            ..Default::default()
        });

        let growth_rate = 10.0; // kg/m²/day
        let dt_hours = 0.1;

        let scaled_rate = service.scale_ecosystem_growth_rate(growth_rate, dt_hours);

        // Should be much smaller than demo mode
        let demo_rate = growth_rate * dt_hours / 24.0;
        assert!(scaled_rate < demo_rate * 0.001); // Much less than 0.1% of demo rate

        // Should be approximately 1/3650 of demo rate
        let expected_factor = 1.0 / 3650.0;
        assert!((service.biological_scaling_factor() - expected_factor).abs() < 1e-10);
    }

    #[test]
    fn test_research_mode_custom_scaling() {
        let custom_factor = 0.1;
        let service = TemporalScalingService::new(TemporalScalingConfig {
            mode: TemporalMode::Research,
            custom_scaling_factor: custom_factor,
            ..Default::default()
        });

        assert_eq!(service.biological_scaling_factor(), custom_factor);

        let growth_rate = 10.0;
        let dt_hours = 0.1;

        let scaled_rate = service.scale_ecosystem_growth_rate(growth_rate, dt_hours);
        let expected_rate = growth_rate * custom_factor * dt_hours / 24.0;

        assert!((scaled_rate - expected_rate).abs() < 1e-10);
    }

    #[test]
    fn test_builder_pattern() {
        let service = TemporalScalingBuilder::new()
            .mode(TemporalMode::Research)
            .custom_scaling_factor(0.5)
            .scale_biological(true)
            .scale_geological(false)
            .build();

        assert_eq!(service.mode(), TemporalMode::Research);
        assert_eq!(service.biological_scaling_factor(), 0.5);
        assert_eq!(service.geological_scaling_factor(), 1.0);
    }

    #[test]
    fn test_performance_optimized_getters() {
        let service = TemporalScalingService::new(TemporalScalingConfig {
            mode: TemporalMode::Realistic,
            ..Default::default()
        });

        // These methods should be inlined and very fast
        let bio_factor = service.biological_scaling_factor();
        let geo_factor = service.geological_scaling_factor();
        let atm_factor = service.atmospheric_scaling_factor();

        assert!(bio_factor > 0.0);
        assert_eq!(geo_factor, 1.0); // Geological scaling disabled by default
        assert_eq!(atm_factor, 1.0); // Atmospheric scaling disabled by default
    }

    #[test]
    fn test_scaling_factors_consistency() {
        let realistic_service = TemporalScalingService::new(TemporalScalingConfig {
            mode: TemporalMode::Realistic,
            scale_biological: true,
            scale_geological: true,
            scale_atmospheric: true,
            ..Default::default()
        });

        let expected_factor = 1.0 / 3650.0;
        assert!((realistic_service.biological_scaling_factor() - expected_factor).abs() < 1e-10);
        assert!((realistic_service.geological_scaling_factor() - expected_factor).abs() < 1e-10);
        assert!((realistic_service.atmospheric_scaling_factor() - expected_factor).abs() < 1e-10);
    }
}
