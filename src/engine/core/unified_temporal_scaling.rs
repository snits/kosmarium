// ABOUTME: Unified temporal scaling architecture that fixes physics violations from selective scaling
// ABOUTME: Provides single temporal factor for all physics systems to maintain causality and conservation laws

use serde::{Deserialize, Serialize};
use crate::engine::core::temporal_scaling::{TemporalMode, TemporalScalingConfig};

/// Unified temporal scaling context for physics consistency
/// 
/// This replaces the problematic selective scaling approach (scale_biological, 
/// scale_geological, scale_atmospheric) with a single unified temporal factor
/// that applies to ALL physics systems. This ensures:
/// 
/// 1. **Temporal Coupling**: All systems evolve at the same temporal rate
/// 2. **Causality Preservation**: No system can evolve faster than its inputs
/// 3. **Conservation Laws**: Energy and mass conservation maintained across systems
/// 4. **Physics Consistency**: All coupled equations use the same dt scaling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalScale {
    /// Active temporal scaling mode
    pub mode: TemporalMode,
    
    /// Unified temporal scaling factor applied to ALL physics systems
    /// - 1.0 = demo mode (simulation timescale)
    /// - 0.000274 ≈ realistic mode (1/3650, scientific timescales)  
    /// - Custom values for research studies
    pub global_temporal_factor: f64,
    
    /// Study phenomenon that configured this temporal scale (for documentation)
    /// Preserves the user-friendly --study-phenomenon CLI experience
    pub study_phenomenon: Option<String>,
}

impl TemporalScale {
    /// Create temporal scale configuration from study phenomenon preset
    ///
    /// This provides scientist-friendly presets that configure unified temporal scaling
    /// based on research intent, preserving the valuable CLI experience while
    /// fixing the physics violations from selective scaling.
    ///
    /// # Available Presets
    /// - `drought`: Global factor 0.2 for long-term ecosystem stress analysis
    /// - `ecosystem`: Global factor ~0.000274 for realistic biological timescales
    /// - `climate`: Global factor ~0.000274 for realistic climate-ecosystem coupling
    /// - `storm`: Global factor 1.0 for observable short-term weather dynamics
    ///
    /// # Arguments
    /// * `phenomenon` - Study phenomenon name (case-insensitive)
    ///
    /// # Returns
    /// * `Ok(TemporalScale)` - Unified temporal scaling for the phenomenon
    /// * `Err(String)` - Error message if phenomenon is unknown
    ///
    /// # Mathematical Basis
    /// All presets use a single global factor applied to ALL physics systems,
    /// ensuring temporal coupling and conservation law compliance.
    ///
    /// # Example
    /// ```rust
    /// use sim_prototype::engine::core::unified_temporal_scaling::TemporalScale;
    ///
    /// let temporal_scale = TemporalScale::from_study_phenomenon("drought").unwrap();
    /// assert_eq!(temporal_scale.global_temporal_factor, 0.2);
    /// assert_eq!(temporal_scale.study_phenomenon, Some("drought".to_string()));
    /// ```
    pub fn from_study_phenomenon(phenomenon: &str) -> Result<TemporalScale, String> {
        let phenomenon_lower = phenomenon.to_lowercase();
        
        match phenomenon_lower.as_str() {
            "drought" => Ok(TemporalScale {
                mode: TemporalMode::Research,
                global_temporal_factor: 0.2, // Slower for long-term drought effects
                study_phenomenon: Some(phenomenon_lower),
            }),
            
            "ecosystem" => Ok(TemporalScale {
                mode: TemporalMode::Realistic,
                global_temporal_factor: 2.5 / 3650.0, // Scientific ecological timescales
                study_phenomenon: Some(phenomenon_lower),
            }),
            
            "climate" => Ok(TemporalScale {
                mode: TemporalMode::Realistic,
                global_temporal_factor: 2.5 / 3650.0, // Scientific climate timescales
                study_phenomenon: Some(phenomenon_lower),
            }),
            
            "storm" => Ok(TemporalScale {
                mode: TemporalMode::Demo,
                global_temporal_factor: 1.0, // Demo rate for observable storm dynamics
                study_phenomenon: Some(phenomenon_lower),
            }),
            
            _ => Err(format!(
                "Unknown study phenomenon '{}'. Valid options: drought, ecosystem, climate, storm",
                phenomenon
            )),
        }
    }
    
    /// Create new temporal scale with explicit configuration
    pub fn new(mode: TemporalMode, global_temporal_factor: f64, study_phenomenon: Option<String>) -> Self {
        Self {
            mode,
            global_temporal_factor,
            study_phenomenon,
        }
    }
    
    /// Create default temporal scale (demo mode, no scaling)
    pub fn default_demo() -> Self {
        Self {
            mode: TemporalMode::Demo,
            global_temporal_factor: 1.0,
            study_phenomenon: None,
        }
    }
    
    /// Create realistic temporal scale (scientific timescales)
    pub fn realistic() -> Self {
        Self {
            mode: TemporalMode::Realistic,
            global_temporal_factor: 2.5 / 3650.0, // 1/1460 for realistic annual rates
            study_phenomenon: None,
        }
    }
    
    /// Scale any temporal rate using the unified scaling factor
    ///
    /// This method replaces the problematic separate scaling methods
    /// (scale_biological_rate, scale_geological_rate, scale_atmospheric_rate)
    /// with a single unified scaling that applies to ALL physics systems.
    ///
    /// # Arguments
    /// * `rate` - Rate per simulation tick (typically per day in demo mode)
    /// * `dt_hours` - Simulation timestep in hours
    ///
    /// # Returns
    /// Scaled rate appropriate for the current unified temporal mode
    ///
    /// # Physics Consistency
    /// By using the same scaling for all systems, this ensures:
    /// - Temporal coupling between all physics systems
    /// - Conservation of energy and mass across system boundaries
    /// - Proper causality (no system evolves faster than its inputs)
    pub fn scale_rate(&self, rate: f64, dt_hours: f64) -> f64 {
        // Convert to rate per hour first, then apply unified scaling
        let rate_per_hour = rate * dt_hours / 24.0; // Assuming input rate is per day
        rate_per_hour * self.global_temporal_factor
    }
    
    /// Get the unified temporal scaling factor (optimized for hot paths)
    #[inline]
    pub fn temporal_factor(&self) -> f64 {
        self.global_temporal_factor
    }
    
    /// Get human-readable description of current unified scaling
    pub fn scaling_description(&self) -> String {
        let base_description = match self.mode {
            TemporalMode::Demo => "Demo mode: Unscaled simulation time (1.0x)".to_string(),
            TemporalMode::Realistic => format!(
                "Realistic mode: Scientific ecological scaling ({:.6}x = 2.5 kg/m²/year target)",
                self.global_temporal_factor
            ),
            TemporalMode::Research => format!(
                "Research mode: Custom unified scaling ({:.6}x)",
                self.global_temporal_factor
            ),
        };
        
        if let Some(ref phenomenon) = self.study_phenomenon {
            format!("{} [Study: {}]", base_description, phenomenon)
        } else {
            base_description
        }
    }
}

impl Default for TemporalScale {
    fn default() -> Self {
        Self::default_demo()
    }
}

/// Conversion from legacy TemporalScalingConfig to unified TemporalScale
/// 
/// This provides backward compatibility for existing code while migrating
/// to the unified temporal scaling architecture that fixes physics violations.
impl From<TemporalScalingConfig> for TemporalScale {
    fn from(old_config: TemporalScalingConfig) -> Self {
        // Calculate unified factor from old selective scaling approach
        let base_factor = match old_config.mode {
            TemporalMode::Demo => 1.0,
            TemporalMode::Realistic => 2.5 / 3650.0, // Match realistic scaling
            TemporalMode::Research => old_config.custom_scaling_factor,
        };
        
        // For migration, we use the base factor uniformly
        // This fixes the physics violations by eliminating selective scaling
        TemporalScale {
            mode: old_config.mode,
            global_temporal_factor: base_factor,
            study_phenomenon: None, // Legacy config doesn't track study phenomenon
        }
    }
}

/// Builder for TemporalScale with fluent interface
pub struct TemporalScaleBuilder {
    temporal_scale: TemporalScale,
}

impl TemporalScaleBuilder {
    pub fn new() -> Self {
        Self {
            temporal_scale: TemporalScale::default(),
        }
    }
    
    pub fn mode(mut self, mode: TemporalMode) -> Self {
        self.temporal_scale.mode = mode;
        self
    }
    
    pub fn global_temporal_factor(mut self, factor: f64) -> Self {
        self.temporal_scale.global_temporal_factor = factor;
        self
    }
    
    pub fn study_phenomenon(mut self, phenomenon: String) -> Self {
        self.temporal_scale.study_phenomenon = Some(phenomenon);
        self
    }
    
    pub fn build(self) -> TemporalScale {
        self.temporal_scale
    }
}

impl Default for TemporalScaleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_study_phenomenon_presets() {
        // Test drought preset
        let drought = TemporalScale::from_study_phenomenon("drought").unwrap();
        assert_eq!(drought.mode, TemporalMode::Research);
        assert_eq!(drought.global_temporal_factor, 0.2);
        assert_eq!(drought.study_phenomenon, Some("drought".to_string()));
        
        // Test ecosystem preset
        let ecosystem = TemporalScale::from_study_phenomenon("ecosystem").unwrap();
        assert_eq!(ecosystem.mode, TemporalMode::Realistic);
        assert!((ecosystem.global_temporal_factor - (2.5 / 3650.0)).abs() < 1e-10);
        assert_eq!(ecosystem.study_phenomenon, Some("ecosystem".to_string()));
        
        // Test case insensitive
        let climate = TemporalScale::from_study_phenomenon("CLIMATE").unwrap();
        assert_eq!(climate.study_phenomenon, Some("climate".to_string()));
    }
    
    #[test]
    fn test_unified_scaling_consistency() {
        let temporal_scale = TemporalScale::from_study_phenomenon("ecosystem").unwrap();
        
        // All systems should get the same scaling factor
        let rate = 10.0; // kg/m²/day
        let dt_hours = 0.1;
        
        let scaled_rate = temporal_scale.scale_rate(rate, dt_hours);
        
        // Should be much smaller than demo mode
        let demo_rate = rate * dt_hours / 24.0;
        assert!(scaled_rate < demo_rate * 0.001);
        
        // Should be consistent with the global factor
        let expected_rate = (rate * dt_hours / 24.0) * temporal_scale.global_temporal_factor;
        assert!((scaled_rate - expected_rate).abs() < 1e-10);
    }
    
    #[test]
    fn test_legacy_config_conversion() {
        let old_config = TemporalScalingConfig {
            mode: TemporalMode::Realistic,
            custom_scaling_factor: 1.0, // Not used in realistic mode
            scale_biological: true,      // These flags are ignored in conversion
            scale_geological: false,
            scale_atmospheric: true,
        };
        
        let temporal_scale = TemporalScale::from(old_config);
        assert_eq!(temporal_scale.mode, TemporalMode::Realistic);
        assert!((temporal_scale.global_temporal_factor - (2.5 / 3650.0)).abs() < 1e-10);
        assert_eq!(temporal_scale.study_phenomenon, None);
    }
    
    #[test]
    fn test_builder_pattern() {
        let temporal_scale = TemporalScaleBuilder::new()
            .mode(TemporalMode::Research)
            .global_temporal_factor(0.5)
            .study_phenomenon("custom_experiment".to_string())
            .build();
        
        assert_eq!(temporal_scale.mode, TemporalMode::Research);
        assert_eq!(temporal_scale.global_temporal_factor, 0.5);
        assert_eq!(temporal_scale.study_phenomenon, Some("custom_experiment".to_string()));
    }
    
    #[test]
    fn test_physics_consistency() {
        let temporal_scale = TemporalScale::from_study_phenomenon("climate").unwrap();
        
        // All physics systems should use the same temporal factor
        let factor = temporal_scale.temporal_factor();
        
        // Verify this is the expected realistic factor
        assert!((factor - (2.5 / 3650.0)).abs() < 1e-10);
        
        // All systems will use this same factor, ensuring physics consistency
        let biological_rate = temporal_scale.scale_rate(5.0, 0.1);
        let geological_rate = temporal_scale.scale_rate(0.001, 0.1); 
        let atmospheric_rate = temporal_scale.scale_rate(1.2, 0.1);
        
        // All should use the same temporal scaling factor
        assert!((biological_rate / (5.0 * 0.1 / 24.0) - factor).abs() < 1e-10);
        assert!((geological_rate / (0.001 * 0.1 / 24.0) - factor).abs() < 1e-10);
        assert!((atmospheric_rate / (1.2 * 0.1 / 24.0) - factor).abs() < 1e-10);
    }
    
    #[test]
    fn test_unknown_phenomenon() {
        let result = TemporalScale::from_study_phenomenon("unknown");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown study phenomenon"));
    }
}