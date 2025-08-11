// ABOUTME: Water flow physics diagnostic framework for real-time validation of hydrodynamics
// ABOUTME: Comprehensive validation of mass conservation, CFL stability, and physics quality metrics

use crate::engine::core::scale::WorldScale;
use crate::engine::physics::water::WaterLayer;
use crate::engine::core::heightmap::HeightMap;
use crate::engine::sim::WaterFlowSystem;

/// Safety parameters derived from SageMath mathematical analysis
/// Following successful atmospheric physics pattern (F_THRESHOLD → 99.6% improvement)
pub mod safety_parameters {
    /// Minimum water depth threshold to prevent numerical instability (√h → 0)
    /// Derived from mathematical analysis: balances numerical precision with physical realism
    pub const H_MIN_THRESHOLD: f32 = 1e-6; // meters
    
    /// CFL safety factor for shallow water equations including gravity wave speed
    /// More conservative than atmospheric physics (0.5) due to gravity wave speeds
    pub const CFL_SAFETY_FACTOR: f32 = 0.25;
    
    /// Maximum realistic water flow velocity in m/s
    /// Based on hydrology literature: river flood speeds
    pub const MAX_REALISTIC_VELOCITY_MS: f32 = 10.0;
    
    /// Minimum realistic water flow velocity in m/s  
    /// Based on hydrology literature: active surface water flow
    pub const MIN_REALISTIC_VELOCITY_MS: f32 = 0.01;
    
    /// Absolute maximum physical velocity (dam break, catastrophic flow)
    pub const ABSOLUTE_MAX_VELOCITY_MS: f32 = 20.0;
    
    /// Mass conservation error tolerance (fraction)
    /// Same precision as successful atmospheric physics validation
    pub const MASS_CONSERVATION_TOLERANCE: f32 = 1e-6;
    
    /// Earth gravity acceleration for shallow water equations
    pub const GRAVITY_ACCELERATION: f32 = 9.81; // m/s²
    
    /// Diagnostic warning thresholds
    pub const VELOCITY_WARNING_THRESHOLD: f32 = 8.0; // 80% of max realistic
    pub const CFL_WARNING_THRESHOLD: f32 = 0.8; // CFL ratio warning
    pub const MASS_ERROR_WARNING: f32 = 1e-5; // 10x tolerance warning
}

/// Comprehensive water flow validation results
#[derive(Debug, Clone)]
pub struct WaterFlowValidation {
    pub is_mass_conserved: bool,
    pub mass_conservation_error: f32,
    pub is_cfl_stable: bool,
    pub max_cfl_violation: f32,
    pub velocity_statistics: VelocityStatistics,
    pub boundary_flux_balance: BoundaryFluxAnalysis,
    pub physics_quality_score: f32, // 0.0-1.0, higher is better
    pub scale_consistency: ScaleConsistencyAnalysis,
}

#[derive(Debug, Clone)]
pub struct VelocityStatistics {
    pub max_velocity_ms: f32,
    pub mean_velocity_ms: f32,
    pub velocity_magnitude_percentiles: Vec<f32>, // [10th, 25th, 50th, 75th, 90th]
    pub realistic_velocity_fraction: f32, // Fraction of cells with realistic velocities
}

#[derive(Debug, Clone)]
pub struct BoundaryFluxAnalysis {
    pub total_inflow_rate: f32,   // Rainfall - evaporation (m/s equivalent)
    pub total_outflow_rate: f32,  // Boundary losses (m/s equivalent)
    pub flux_balance_error: f32,  // |inflow - outflow - storage_change|
    pub boundary_loss_fraction: f32, // Fraction of water lost at boundaries
}

#[derive(Debug, Clone)]
pub struct ScaleConsistencyAnalysis {
    pub domain_size_km: f32,
    pub grid_spacing_m: f32,
    pub cfl_timestep_limit: f32,
    pub scale_appropriate_parameters: bool,
}

/// Water flow physics diagnostic system
pub struct WaterFlowDiagnostics {
    world_scale: WorldScale,
    gravity: f32,
    previous_total_mass: Option<f32>,
    mass_history: Vec<f32>, // Rolling history for trend analysis
}

impl WaterFlowDiagnostics {
    /// Create new water flow diagnostics for given world scale
    pub fn new(world_scale: WorldScale) -> Self {
        Self {
            world_scale,
            gravity: safety_parameters::GRAVITY_ACCELERATION,
            previous_total_mass: None,
            mass_history: Vec::new(),
        }
    }
    
    /// Comprehensive validation of water flow physics
    pub fn validate_water_flow_physics(
        &mut self,
        water_system: &WaterFlowSystem,
        heightmap: &HeightMap,
        water: &WaterLayer,
    ) -> WaterFlowValidation {
        // 1. Mass Conservation Analysis
        let mass_conservation = self.validate_mass_conservation(water_system, water);
        
        // 2. CFL Stability Analysis  
        let cfl_analysis = self.validate_cfl_stability(water, heightmap);
        
        // 3. Velocity Statistics and Realism
        let velocity_stats = self.analyze_velocity_statistics(water);
        
        // 4. Boundary Flux Analysis
        let boundary_analysis = self.analyze_boundary_flux(water_system, water);
        
        // 5. Scale Consistency Check
        let scale_analysis = self.analyze_scale_consistency(water_system);
        
        // 6. Overall Physics Quality Score
        let physics_quality = self.calculate_physics_quality_score(
            &mass_conservation,
            &cfl_analysis,
            &velocity_stats,
            &boundary_analysis,
        );
        
        WaterFlowValidation {
            is_mass_conserved: mass_conservation.0,
            mass_conservation_error: mass_conservation.1,
            is_cfl_stable: cfl_analysis.0,
            max_cfl_violation: cfl_analysis.1,
            velocity_statistics: velocity_stats,
            boundary_flux_balance: boundary_analysis,
            physics_quality_score: physics_quality,
            scale_consistency: scale_analysis,
        }
    }
    
    /// Validate mass conservation: d(total_mass)/dt ≈ rainfall - evaporation - boundary_outflow
    fn validate_mass_conservation(
        &mut self,
        water_system: &WaterFlowSystem,
        water: &WaterLayer,
    ) -> (bool, f32) {
        let current_total_mass = water.get_total_water();
        self.mass_history.push(current_total_mass);
        
        // Keep only recent history (rolling window)
        if self.mass_history.len() > 100 {
            self.mass_history.remove(0);
        }
        
        let mass_conservation_error = if let Some(previous_mass) = self.previous_total_mass {
            let mass_change_rate = current_total_mass - previous_mass;
            
            // Expected mass change from rainfall and evaporation
            let total_cells = (water.width() * water.height()) as f32;
            let expected_change = total_cells * (
                water_system.effective_rainfall_rate - 
                water_system.parameters.evaporation_rate * current_total_mass / total_cells
            );
            
            let error = (mass_change_rate - expected_change).abs();
            let relative_error = if current_total_mass > 0.0 {
                error / current_total_mass
            } else {
                error
            };
            
            relative_error
        } else {
            0.0 // First measurement, no comparison
        };
        
        self.previous_total_mass = Some(current_total_mass);
        
        let is_conserved = mass_conservation_error < safety_parameters::MASS_CONSERVATION_TOLERANCE;
        (is_conserved, mass_conservation_error)
    }
    
    /// Validate CFL stability condition including gravity wave speed
    fn validate_cfl_stability(&self, water: &WaterLayer, heightmap: &HeightMap) -> (bool, f32) {
        let dx = self.world_scale.meters_per_pixel() as f32;
        let mut max_cfl_violation: f32 = 0.0;
        let mut total_violations = 0;
        let total_cells = water.width() * water.height();
        
        for y in 0..water.height() {
            for x in 0..water.width() {
                let (u, v) = water.velocity.get(x, y);
                let h = water.get_water_depth(x, y).max(safety_parameters::H_MIN_THRESHOLD);
                
                // Calculate maximum wave speed: |u| + √(gh) (shallow water equations)
                let velocity_magnitude = (u * u + v * v).sqrt();
                let gravity_wave_speed = (self.gravity * h).sqrt();
                let max_wave_speed = velocity_magnitude + gravity_wave_speed;
                
                // CFL condition: dt ≤ CFL_SAFETY * dx / max_wave_speed
                let cfl_timestep_limit = safety_parameters::CFL_SAFETY_FACTOR * dx / max_wave_speed;
                
                // Current simulation assumes dt = 1 (one timestep per update)
                let current_dt = 1.0;
                let cfl_ratio = current_dt / cfl_timestep_limit;
                
                if cfl_ratio > 1.0 {
                    max_cfl_violation = max_cfl_violation.max(cfl_ratio);
                    total_violations += 1;
                }
            }
        }
        
        let violation_fraction = total_violations as f32 / total_cells as f32;
        let is_stable = max_cfl_violation < 1.0 && violation_fraction < 0.01; // Allow <1% violations
        
        (is_stable, max_cfl_violation)
    }
    
    /// Analyze velocity statistics and realism
    fn analyze_velocity_statistics(&self, water: &WaterLayer) -> VelocityStatistics {
        let mut velocities_ms = Vec::new();
        let dx = self.world_scale.meters_per_pixel() as f32;
        
        for y in 0..water.height() {
            for x in 0..water.width() {
                let (u, v) = water.velocity.get(x, y);
                let velocity_magnitude = (u * u + v * v).sqrt();
                
                // Convert from simulation units to m/s (approximate)
                let velocity_ms = velocity_magnitude * dx;
                velocities_ms.push(velocity_ms);
            }
        }
        
        // Sort for percentile calculations
        velocities_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = velocities_ms.len();
        
        let max_velocity_ms = velocities_ms.last().copied().unwrap_or(0.0);
        let mean_velocity_ms = velocities_ms.iter().sum::<f32>() / n as f32;
        
        let percentiles = vec![
            velocities_ms[n * 10 / 100],  // 10th percentile
            velocities_ms[n * 25 / 100],  // 25th percentile
            velocities_ms[n * 50 / 100],  // 50th percentile (median)
            velocities_ms[n * 75 / 100],  // 75th percentile
            velocities_ms[n * 90 / 100],  // 90th percentile
        ];
        
        // Count realistic velocities
        let realistic_count = velocities_ms
            .iter()
            .filter(|&&v| {
                v >= safety_parameters::MIN_REALISTIC_VELOCITY_MS
                    && v <= safety_parameters::MAX_REALISTIC_VELOCITY_MS
            })
            .count();
        let realistic_velocity_fraction = realistic_count as f32 / n as f32;
        
        VelocityStatistics {
            max_velocity_ms,
            mean_velocity_ms,
            velocity_magnitude_percentiles: percentiles,
            realistic_velocity_fraction,
        }
    }
    
    /// Analyze boundary flux balance
    fn analyze_boundary_flux(
        &self,
        water_system: &WaterFlowSystem,
        water: &WaterLayer,
    ) -> BoundaryFluxAnalysis {
        let total_cells = (water.width() * water.height()) as f32;
        let current_total_mass = water.get_total_water();
        
        // Calculate input rate (rainfall) and output rate (evaporation)
        let rainfall_rate = water_system.effective_rainfall_rate * total_cells;
        let mean_water_depth = current_total_mass / total_cells;
        let evaporation_rate = water_system.parameters.evaporation_rate * current_total_mass;
        
        let total_inflow_rate = rainfall_rate;
        let total_internal_loss = evaporation_rate;
        
        // Estimate boundary outflow (this is approximate without detailed flux tracking)
        let net_input = total_inflow_rate - total_internal_loss;
        let storage_change_rate = if let Some(prev_mass) = self.previous_total_mass {
            current_total_mass - prev_mass
        } else {
            0.0
        };
        
        // Boundary outflow = net_input - storage_change
        let estimated_boundary_outflow = net_input - storage_change_rate;
        let total_outflow_rate = total_internal_loss + estimated_boundary_outflow.max(0.0);
        
        let flux_balance_error = (net_input - storage_change_rate).abs();
        let boundary_loss_fraction = if total_inflow_rate > 0.0 {
            estimated_boundary_outflow.max(0.0) / total_inflow_rate
        } else {
            0.0
        };
        
        BoundaryFluxAnalysis {
            total_inflow_rate,
            total_outflow_rate,
            flux_balance_error,
            boundary_loss_fraction,
        }
    }
    
    /// Analyze scale consistency and parameter appropriateness
    fn analyze_scale_consistency(&self, water_system: &WaterFlowSystem) -> ScaleConsistencyAnalysis {
        let domain_size_km = self.world_scale.physical_size_km as f32;
        let grid_spacing_m = self.world_scale.meters_per_pixel() as f32;
        
        // Calculate CFL timestep limit for this scale
        let typical_velocity = water_system.parameters.max_expected_velocity_ms;
        let typical_depth = 1.0; // Assume 1m typical depth
        let gravity_wave_speed = (self.gravity * typical_depth).sqrt();
        let max_wave_speed = typical_velocity + gravity_wave_speed;
        
        let cfl_timestep_limit = safety_parameters::CFL_SAFETY_FACTOR * grid_spacing_m / max_wave_speed;
        
        // Check if parameters are appropriate for this scale
        let scale_appropriate = domain_size_km >= 1.0 && domain_size_km <= 40_000.0
            && grid_spacing_m >= 10.0 && grid_spacing_m <= 10_000.0
            && cfl_timestep_limit > 0.001; // Minimum reasonable timestep
        
        ScaleConsistencyAnalysis {
            domain_size_km,
            grid_spacing_m,
            cfl_timestep_limit,
            scale_appropriate_parameters: scale_appropriate,
        }
    }
    
    /// Calculate overall physics quality score (0.0-1.0)
    fn calculate_physics_quality_score(
        &self,
        mass_conservation: &(bool, f32),
        cfl_analysis: &(bool, f32),
        velocity_stats: &VelocityStatistics,
        boundary_analysis: &BoundaryFluxAnalysis,
    ) -> f32 {
        // Mass conservation score (0.0-1.0)
        let mass_score = if mass_conservation.0 {
            1.0
        } else {
            (1.0 - mass_conservation.1.min(1.0)).max(0.0)
        };
        
        // CFL stability score (0.0-1.0)
        let cfl_score = if cfl_analysis.0 {
            1.0
        } else {
            (2.0 - cfl_analysis.1).max(0.0).min(1.0)
        };
        
        // Velocity realism score (0.0-1.0)
        let velocity_score = velocity_stats.realistic_velocity_fraction;
        
        // Boundary flux score (0.0-1.0)
        let flux_score = if boundary_analysis.flux_balance_error < 0.1 {
            1.0
        } else {
            (1.0 - boundary_analysis.flux_balance_error.min(1.0)).max(0.0)
        };
        
        // Weighted average (mass conservation and CFL stability are most critical)
        0.4 * mass_score + 0.3 * cfl_score + 0.2 * velocity_score + 0.1 * flux_score
    }
    
    /// Generate diagnostic report for debugging and validation
    pub fn generate_diagnostic_report(&self, validation: &WaterFlowValidation) -> String {
        format!(
            "=== WATER FLOW PHYSICS DIAGNOSTIC REPORT ===\n\
            Overall Physics Quality Score: {:.3}/1.0\n\
            \n\
            MASS CONSERVATION:\n\
            - Conserved: {}\n\
            - Error: {:.2e}\n\
            - Tolerance: {:.2e}\n\
            \n\
            CFL STABILITY:\n\
            - Stable: {}\n\
            - Max Violation: {:.3}x\n\
            - Safety Factor: {:.3}\n\
            \n\
            VELOCITY ANALYSIS:\n\
            - Max Velocity: {:.3} m/s\n\
            - Mean Velocity: {:.3} m/s\n\
            - Realistic Fraction: {:.3}\n\
            - 50th Percentile: {:.3} m/s\n\
            \n\
            BOUNDARY FLUX:\n\
            - Inflow Rate: {:.2e} m³/s\n\
            - Outflow Rate: {:.2e} m³/s\n\
            - Balance Error: {:.2e}\n\
            - Boundary Loss: {:.1}%\n\
            \n\
            SCALE CONSISTENCY:\n\
            - Domain Size: {:.1} km\n\
            - Grid Spacing: {:.1} m\n\
            - CFL Limit: {:.4} s\n\
            - Scale Appropriate: {}\n\
            ",
            validation.physics_quality_score,
            validation.is_mass_conserved,
            validation.mass_conservation_error,
            safety_parameters::MASS_CONSERVATION_TOLERANCE,
            validation.is_cfl_stable,
            validation.max_cfl_violation,
            safety_parameters::CFL_SAFETY_FACTOR,
            validation.velocity_statistics.max_velocity_ms,
            validation.velocity_statistics.mean_velocity_ms,
            validation.velocity_statistics.realistic_velocity_fraction,
            validation.velocity_statistics.velocity_magnitude_percentiles[2], // median
            validation.boundary_flux_balance.total_inflow_rate,
            validation.boundary_flux_balance.total_outflow_rate,
            validation.boundary_flux_balance.flux_balance_error,
            validation.boundary_flux_balance.boundary_loss_fraction * 100.0,
            validation.scale_consistency.domain_size_km,
            validation.scale_consistency.grid_spacing_m,
            validation.scale_consistency.cfl_timestep_limit,
            validation.scale_consistency.scale_appropriate_parameters,
        )
    }
}

/// Convenience function for quick water flow validation during development
pub fn validate_water_flow_quick(
    world_scale: &WorldScale,
    water_system: &WaterFlowSystem,
    heightmap: &HeightMap,
    water: &WaterLayer,
) -> WaterFlowValidation {
    let mut diagnostics = WaterFlowDiagnostics::new(world_scale.clone());
    diagnostics.validate_water_flow_physics(water_system, heightmap, water)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::core::scale::{DetailLevel, WorldScale};
    use crate::engine::sim::WaterFlowParameters;
    use crate::engine::core::heightmap::HeightMap;
    
    fn create_test_setup() -> (WorldScale, WaterFlowSystem, HeightMap, WaterLayer) {
        let scale = WorldScale::new(100.0, (50, 50), DetailLevel::Standard);
        let water_system = WaterFlowSystem::new_for_scale(&scale);
        let heightmap = HeightMap::new(50, 50, 1.0);
        let water = WaterLayer::new(50, 50);
        
        (scale, water_system, heightmap, water)
    }
    
    #[test]
    fn test_mass_conservation_validation() {
        let (scale, water_system, heightmap, mut water) = create_test_setup();
        let mut diagnostics = WaterFlowDiagnostics::new(scale);
        
        // Add some water
        water.add_water(25, 25, 1.0);
        
        let validation = diagnostics.validate_water_flow_physics(&water_system, &heightmap, &water);
        
        // First run should have no mass conservation error (no previous measurement)
        assert!(validation.mass_conservation_error == 0.0);
        
        // Second run should detect changes
        water.add_water(25, 25, 0.5);
        let validation2 = diagnostics.validate_water_flow_physics(&water_system, &heightmap, &water);
        
        // Should detect the mass change
        assert!(validation2.mass_conservation_error > 0.0);
    }
    
    #[test]
    fn test_cfl_stability_analysis() {
        let (scale, water_system, heightmap, mut water) = create_test_setup();
        let diagnostics = WaterFlowDiagnostics::new(scale);
        
        // Set reasonable velocity and depth
        water.add_water(25, 25, 1.0);
        water.velocity.set(25, 25, (0.1, 0.1)); // Small velocity
        
        let validation = diagnostics.validate_water_flow_physics(&water_system, &heightmap, &water);
        
        // Should be stable with small velocities
        assert!(validation.is_cfl_stable);
        assert!(validation.max_cfl_violation <= 1.0);
    }
    
    #[test]
    fn test_velocity_statistics() {
        let (scale, water_system, heightmap, mut water) = create_test_setup();
        let diagnostics = WaterFlowDiagnostics::new(scale);
        
        // Set up various velocities
        for i in 0..10 {
            water.add_water(i, 25, 0.5);
            water.velocity.set(i, 25, (i as f32 * 0.01, 0.0));
        }
        
        let validation = diagnostics.validate_water_flow_physics(&water_system, &heightmap, &water);
        
        // Should have calculated velocity statistics
        assert!(validation.velocity_statistics.max_velocity_ms >= 0.0);
        assert!(validation.velocity_statistics.mean_velocity_ms >= 0.0);
        assert_eq!(validation.velocity_statistics.velocity_magnitude_percentiles.len(), 5);
    }
    
    #[test]
    fn test_physics_quality_score() {
        let (scale, water_system, heightmap, water) = create_test_setup();
        let diagnostics = WaterFlowDiagnostics::new(scale);
        
        let validation = diagnostics.validate_water_flow_physics(&water_system, &heightmap, &water);
        
        // Physics quality score should be between 0 and 1
        assert!(validation.physics_quality_score >= 0.0);
        assert!(validation.physics_quality_score <= 1.0);
    }
    
    #[test]
    fn test_diagnostic_report_generation() {
        let (scale, water_system, heightmap, water) = create_test_setup();
        let diagnostics = WaterFlowDiagnostics::new(scale);
        
        let validation = diagnostics.validate_water_flow_physics(&water_system, &heightmap, &water);
        let report = diagnostics.generate_diagnostic_report(&validation);
        
        // Should contain key sections
        assert!(report.contains("MASS CONSERVATION"));
        assert!(report.contains("CFL STABILITY"));
        assert!(report.contains("VELOCITY ANALYSIS"));
        assert!(report.contains("BOUNDARY FLUX"));
        assert!(report.contains("SCALE CONSISTENCY"));
    }
}