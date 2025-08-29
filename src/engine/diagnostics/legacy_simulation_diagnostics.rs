// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Simulation diagnostics and monitoring system for ASCII stats output
// ABOUTME: Collects and formats key metrics for debugging scale-aware simulation parameters

use crate::engine::sim::Simulation;

/// Comprehensive simulation diagnostics for monitoring system health
#[derive(Debug, Clone)]
pub struct SimulationDiagnostics {
    /// Current simulation iteration
    pub iteration: usize,
    /// Physical scale information
    pub meters_per_pixel: f64,
    pub domain_size_km: f64,
    /// Water system metrics
    pub water_metrics: WaterSystemMetrics,
    /// Threshold information (before/after hardcoded fixes)
    pub thresholds: ThresholdMetrics,
    /// Biome distribution
    pub biome_metrics: BiomeMetrics,
    /// Atmospheric system status
    pub atmosphere_metrics: AtmosphereMetrics,
    /// Mass conservation tracking
    pub conservation_metrics: ConservationMetrics,
}

/// Water flow system diagnostics
#[derive(Debug, Clone)]
pub struct WaterSystemMetrics {
    pub current_flow_rate: f32,
    pub flow_threshold: f32,
    pub current_rainfall: f32,
    pub evaporation_rate: f32,
    pub water_balance: f32,
    pub active_cells: usize,
    pub total_cells: usize,
    pub active_percentage: f32,
    pub has_converged: bool,
    pub convergence_threshold: f32,
}

/// Scale-aware threshold information
#[derive(Debug, Clone)]
pub struct ThresholdMetrics {
    pub flow_threshold_current: f32,
    pub flow_threshold_original: f32,
    pub convergence_threshold_current: f32,
    pub convergence_threshold_original: f32,
    pub erosion_max_current: f32,
    pub erosion_max_original: f32,
    pub cfl_timestep_range: (f32, f32),
}

/// Biome distribution and diversity metrics
#[derive(Debug, Clone)]
pub struct BiomeMetrics {
    pub water_percentage: f32,
    pub desert_percentage: f32,
    pub grassland_percentage: f32,
    pub forest_percentage: f32,
    pub diversity_index: f32,
    pub river_cells: usize,
}

/// Atmospheric system diagnostics
#[derive(Debug, Clone)]
pub struct AtmosphereMetrics {
    pub pressure_range_kpa: (f32, f32),
    pub pressure_gradient: f32,
    pub wind_speed_avg: f32,
    pub wind_speed_max: f32,
    pub coriolis_active: bool,
    pub boundary_stable: bool,
    pub momentum_drift: f32,
}

/// Mass conservation tracking
#[derive(Debug, Clone)]
pub struct ConservationMetrics {
    pub total_water: f32,
    pub water_change_percent: f32,
    pub total_elevation: f32,
    pub elevation_change_percent: f32,
    pub total_sediment: f32,
    pub boundary_flux: f32,
}

impl SimulationDiagnostics {
    /// Collect comprehensive diagnostics from simulation state
    pub fn collect_from_simulation(simulation: &Simulation, iteration: usize) -> Self {
        let world_scale = simulation.get_world_scale();
        let meters_per_pixel = world_scale.meters_per_pixel();
        let domain_size_km = world_scale.physical_size_km;

        // Collect water system metrics
        let water_metrics = Self::collect_water_metrics(simulation);

        // Collect threshold information
        let thresholds = Self::collect_threshold_metrics(simulation);

        // Collect biome metrics
        let biome_metrics = Self::collect_biome_metrics(simulation);

        // Collect atmosphere metrics
        let atmosphere_metrics = Self::collect_atmosphere_metrics(simulation);

        // Collect conservation metrics
        let conservation_metrics = Self::collect_conservation_metrics(simulation);

        Self {
            iteration,
            meters_per_pixel,
            domain_size_km,
            water_metrics,
            thresholds,
            biome_metrics,
            atmosphere_metrics,
            conservation_metrics,
        }
    }

    /// Collect water system diagnostics
    fn collect_water_metrics(simulation: &Simulation) -> WaterSystemMetrics {
        let water_system = simulation.get_water_system();
        let spatial_system = simulation.get_spatial_system();

        // Get current flow rate from a representative cell
        let current_flow_rate = simulation.get_average_flow_rate();

        // Get scale-aware thresholds
        let flow_threshold = water_system.evaporation_threshold * 10.0; // Our scale-aware calculation

        // Get rainfall and evaporation rates
        let current_rainfall = water_system.effective_rainfall_rate;
        let evaporation_rate = water_system.evaporation_threshold;
        let water_balance = current_rainfall - evaporation_rate;

        // Get spatial partitioning stats
        let stats = spatial_system.get_performance_stats();
        let active_cells = stats.active_cells;
        let total_cells = stats.total_cells;
        let active_percentage = (active_cells as f32 / total_cells as f32) * 100.0;

        // Check convergence
        let has_converged = spatial_system.has_converged();
        let convergence_threshold = water_system.evaporation_threshold * 2.0; // Our scale-aware calculation

        WaterSystemMetrics {
            current_flow_rate,
            flow_threshold,
            current_rainfall,
            evaporation_rate,
            water_balance,
            active_cells,
            total_cells,
            active_percentage,
            has_converged,
            convergence_threshold,
        }
    }

    /// Collect threshold comparison metrics
    fn collect_threshold_metrics(simulation: &Simulation) -> ThresholdMetrics {
        let water_system = simulation.get_water_system();

        // Current scale-aware thresholds (from our fixes)
        let flow_threshold_current = water_system.evaporation_threshold * 10.0;
        let convergence_threshold_current = water_system.evaporation_threshold * 2.0;
        let erosion_max_current = water_system.evaporation_threshold * 100.0;

        // Original hardcoded values (for comparison)
        let flow_threshold_original = 0.001;
        let convergence_threshold_original = 0.001;
        let erosion_max_original = 0.001;

        // CFL timestep range (scale-aware)
        let world_scale = simulation.get_world_scale();
        let grid_spacing_m = world_scale.meters_per_pixel() as f32;
        let min_timestep = (grid_spacing_m / 100000.0).max(0.001).min(10.0);
        let max_timestep = (grid_spacing_m / 100.0).max(60.0).min(3600.0);

        ThresholdMetrics {
            flow_threshold_current,
            flow_threshold_original,
            convergence_threshold_current,
            convergence_threshold_original,
            erosion_max_current,
            erosion_max_original,
            cfl_timestep_range: (min_timestep, max_timestep),
        }
    }

    /// Collect biome distribution metrics
    fn collect_biome_metrics(simulation: &Simulation) -> BiomeMetrics {
        let biome_map = simulation.generate_biome_map_basic();
        let total_cells = biome_map.len() as f32;

        let mut water_count = 0;
        let mut desert_count = 0;
        let mut grassland_count = 0;
        let mut forest_count = 0;
        let mut river_cells = 0;

        for (_, _, biome) in biome_map.iter_coords() {
            match biome {
crate::engine::agents::biome::BiomeType::Ocean
                | crate::engine::agents::biome::BiomeType::Lake
                | crate::engine::agents::biome::BiomeType::River
                | crate::engine::agents::biome::BiomeType::Wetland => water_count += 1,
crate::engine::agents::biome::BiomeType::Desert => desert_count += 1,
crate::engine::agents::biome::BiomeType::Grassland
                | crate::engine::agents::biome::BiomeType::Savanna => grassland_count += 1,
crate::engine::agents::biome::BiomeType::TemperateForest
                | crate::engine::agents::biome::BiomeType::RainForest
                | crate::engine::agents::biome::BiomeType::BorealForest => forest_count += 1,
                _ => {} // Handle other biome types (tundra, alpine, ice, etc.)
            }
        }

        // Calculate diversity index (Shannon diversity approximation)
        let proportions = [
            water_count as f32 / total_cells,
            desert_count as f32 / total_cells,
            grassland_count as f32 / total_cells,
            forest_count as f32 / total_cells,
        ];

        let diversity_index = proportions
            .iter()
            .filter(|&&p| p > 0.0)
            .map(|&p| -p * p.ln())
            .sum::<f32>();

        // Count river cells (water cells that are connected)
        river_cells = simulation.count_river_cells();

        BiomeMetrics {
            water_percentage: (water_count as f32 / total_cells) * 100.0,
            desert_percentage: (desert_count as f32 / total_cells) * 100.0,
            grassland_percentage: (grassland_count as f32 / total_cells) * 100.0,
            forest_percentage: (forest_count as f32 / total_cells) * 100.0,
            diversity_index,
            river_cells,
        }
    }

    /// Collect atmospheric system metrics
    fn collect_atmosphere_metrics(simulation: &Simulation) -> AtmosphereMetrics {
        let pressure_layer = simulation.get_pressure_layer();
        let wind_layer = simulation.get_wind_layer();
        let atmospheric_system = simulation.get_atmospheric_system();

        // Calculate pressure range using PhysicsGrid's optimized methods
        let min_pressure = pressure_layer.pressure.min();
        let max_pressure = pressure_layer.pressure.max();

        let pressure_range_kpa = (min_pressure / 1000.0, max_pressure / 1000.0);

        // Calculate pressure gradient (simplified)
        let pressure_gradient = (max_pressure - min_pressure) / 100000.0; // Per 100km

        // Wind statistics
        let wind_speed_avg = wind_layer.get_average_wind_speed();
        let mut wind_speed_max: f32 = 0.0;

        for y in 0..wind_layer.height() {
            for x in 0..wind_layer.width() {
                wind_speed_max = wind_speed_max.max(wind_layer.get_speed(x, y));
            }
        }

        // Atmospheric system status
        let coriolis_active = atmospheric_system.is_coriolis_active();

        // Boundary stability check
        let stability = atmospheric_system.validate_atmospheric_stability(&wind_layer);
        let boundary_stable = stability.is_system_stable;
        let momentum_drift = stability.momentum_magnitude;

        AtmosphereMetrics {
            pressure_range_kpa,
            pressure_gradient,
            wind_speed_avg,
            wind_speed_max,
            coriolis_active,
            boundary_stable,
            momentum_drift,
        }
    }

    /// Collect mass conservation metrics
    fn collect_conservation_metrics(simulation: &Simulation) -> ConservationMetrics {
        // These would need to be tracked over time in a real implementation
        // For now, provide placeholders that show the concept

        let total_water = simulation.calculate_total_water();
        let total_elevation = simulation.calculate_total_elevation();
        let total_sediment = simulation.calculate_total_sediment();

        // These would be calculated by comparing with previous values
        let water_change_percent = 0.0; // Placeholder
        let elevation_change_percent = 0.0; // Placeholder
        let boundary_flux = 0.0; // Placeholder

        ConservationMetrics {
            total_water,
            water_change_percent,
            total_elevation,
            elevation_change_percent,
            total_sediment,
            boundary_flux,
        }
    }

    /// Format diagnostics for compact ASCII display
    pub fn format_compact(&self) -> String {
        format!(
            r#"=== SIMULATION DIAGNOSTICS (Iter: {}, Scale: {:.0}m/px, Domain: {:.0}km) ===
WATER SYSTEM:
  Flow Rate: {:.5} | Threshold: {:.5} | Status: {} {}
  Rainfall:  {:.5} | Evap Rate: {:.5} | Balance: {:+.5}
  Active Cells: {}/{} ({:.1}%) | Converged: {}

THRESHOLDS (Scale-Aware):
  Flow Min: {:.4} (was {:.3}) | Convergence: {:.5} (was {:.3})
  Erosion Max: {:.4} (was {:.3}) | CFL Range: [{:.2}, {:.0}]s

BIOMES:
  Water: {:.0}% | Desert: {:.0}% | Grassland: {:.0}% | Forest: {:.0}%
  Diversity Index: {:.2} ({}) | River Cells: {}

ATMOSPHERE:
  Pressure Range: [{:.1}, {:.1}] kPa | Gradient: {:.3} kPa/100km
  Wind Speed Avg: {:.1} m/s | Max: {:.1} m/s | Coriolis: {}
  Boundary Stable: {} | Momentum Drift: {:.2} m/s ({})

MASS CONSERVATION:
  Water Total: {:.2e} (Δ: {:+.2}%) | Elevation Σ: {:.2e} (Δ: {:+.3}%)
  Sediment: {:.2e} | Boundary Flux: {:+.2} m³/s
==="#,
            self.iteration,
            self.meters_per_pixel,
            self.domain_size_km,
            self.water_metrics.current_flow_rate,
            self.water_metrics.flow_threshold,
            if self.water_metrics.current_flow_rate > 0.0001 {
                "FLOWING"
            } else {
                "STATIC"
            },
            if self.water_metrics.current_flow_rate < self.water_metrics.flow_threshold {
                "⚠️"
            } else {
                "✓"
            },
            self.water_metrics.current_rainfall,
            self.water_metrics.evaporation_rate,
            self.water_metrics.water_balance,
            self.water_metrics.active_cells,
            self.water_metrics.total_cells,
            self.water_metrics.active_percentage,
            if self.water_metrics.has_converged {
                "YES"
            } else {
                "NO"
            },
            self.thresholds.flow_threshold_current,
            self.thresholds.flow_threshold_original,
            self.thresholds.convergence_threshold_current,
            self.thresholds.convergence_threshold_original,
            self.thresholds.erosion_max_current,
            self.thresholds.erosion_max_original,
            self.thresholds.cfl_timestep_range.0,
            self.thresholds.cfl_timestep_range.1,
            self.biome_metrics.water_percentage,
            self.biome_metrics.desert_percentage,
            self.biome_metrics.grassland_percentage,
            self.biome_metrics.forest_percentage,
            self.biome_metrics.diversity_index,
            if self.biome_metrics.diversity_index > 0.5 {
                "Good"
            } else {
                "Poor"
            },
            self.biome_metrics.river_cells,
            self.atmosphere_metrics.pressure_range_kpa.0,
            self.atmosphere_metrics.pressure_range_kpa.1,
            self.atmosphere_metrics.pressure_gradient,
            self.atmosphere_metrics.wind_speed_avg,
            self.atmosphere_metrics.wind_speed_max,
            if self.atmosphere_metrics.coriolis_active {
                "ACTIVE"
            } else {
                "DISABLED"
            },
            if self.atmosphere_metrics.boundary_stable {
                "YES"
            } else {
                "NO"
            },
            self.atmosphere_metrics.momentum_drift,
            if self.atmosphere_metrics.momentum_drift < 1.0 {
                "OK"
            } else {
                "HIGH"
            },
            self.conservation_metrics.total_water,
            self.conservation_metrics.water_change_percent,
            self.conservation_metrics.total_elevation,
            self.conservation_metrics.elevation_change_percent,
            self.conservation_metrics.total_sediment,
            self.conservation_metrics.boundary_flux,
        )
    }
}
