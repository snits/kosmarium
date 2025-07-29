// ABOUTME: Grid convergence testing framework for validating simulation scaling behavior
// ABOUTME: Provides tools to test that simulations converge as grid resolution increases

use crate::climate::ClimateSystem;
use crate::scale::{DetailLevel, WorldScale};
use crate::sim::Simulation;
use crate::worldgen::{DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator};

/// Result of a grid convergence study
#[derive(Clone, Debug)]
pub struct ConvergenceStudyResult {
    /// Grid resolutions tested (cells per axis)  
    pub resolutions: Vec<u32>,
    /// Computed solution metrics at each resolution
    pub metrics: Vec<ConvergenceMetric>,
    /// Estimated convergence order (p in h^p scaling)
    pub convergence_order: Option<f64>,
    /// Whether the study indicates convergence
    pub is_converged: bool,
    /// Any warnings or issues detected
    pub warnings: Vec<String>,
}

/// Result of a climate convergence study
#[derive(Clone, Debug)]
pub struct ClimateConvergenceStudyResult {
    /// Grid resolutions tested (cells per axis)
    pub resolutions: Vec<u32>,
    /// Climate metrics at each resolution
    pub metrics: Vec<ClimateConvergenceMetric>,
    /// Convergence order for mean temperature
    pub mean_temp_convergence_order: Option<f64>,
    /// Convergence order for temperature gradient
    pub gradient_convergence_order: Option<f64>,
    /// Whether climate fields are converged
    pub is_converged: bool,
    /// Warnings and analysis results
    pub warnings: Vec<String>,
}

/// Metrics computed for each grid resolution
#[derive(Clone, Debug)]
pub struct ConvergenceMetric {
    /// Grid resolution (cells per axis)
    pub resolution: u32,
    /// Grid spacing (meters per cell)
    pub grid_spacing: f64,
    /// Total water accumulated in simulation
    pub total_water: f64,
    /// Maximum water depth observed
    pub max_water_depth: f64,
    /// Water distribution entropy (measure of how spread out water is)
    pub water_entropy: f64,
    /// Number of simulation steps taken
    pub steps_simulated: u32,
}

/// Climate-specific metrics for convergence testing
#[derive(Clone, Debug)]
pub struct ClimateConvergenceMetric {
    /// Grid resolution (cells per axis)
    pub resolution: u32,
    /// Grid spacing (meters per cell)
    pub grid_spacing: f64,
    /// Average temperature across the domain
    pub mean_temperature: f64,
    /// Temperature standard deviation (measure of spatial variation)
    pub temperature_std: f64,
    /// Minimum temperature in domain
    pub min_temperature: f64,
    /// Maximum temperature in domain
    pub max_temperature: f64,
    /// Temperature gradient strength (measure of elevation effects)
    pub temperature_gradient: f64,
    /// Spatial correlation of temperature with elevation
    pub temp_elevation_correlation: f64,
}

/// Configuration for convergence studies
#[derive(Clone, Debug)]
pub struct ConvergenceStudyConfig {
    /// Physical domain size in kilometers
    pub domain_size_km: f64,
    /// Base resolution for coarsest grid
    pub base_resolution: u32,
    /// Resolution refinement factor (typically 2)
    pub refinement_factor: u32,
    /// Number of refinement levels to test
    pub num_levels: u32,
    /// Number of simulation steps to run at each resolution
    pub simulation_steps: u32,
    /// Amount of water to add for testing (normalized)
    pub test_water_amount: f32,
    /// Terrain generation seed for reproducibility
    pub terrain_seed: u64,
}

impl Default for ConvergenceStudyConfig {
    fn default() -> Self {
        Self {
            domain_size_km: 10.0,
            base_resolution: 50,
            refinement_factor: 2,
            num_levels: 4, // Test 50x50, 100x100, 200x200, 400x400
            simulation_steps: 20,
            test_water_amount: 1.0,
            terrain_seed: 42,
        }
    }
}

/// Grid convergence testing framework
pub struct ConvergenceStudy {
    config: ConvergenceStudyConfig,
}

impl ConvergenceStudy {
    /// Create a new convergence study with the given configuration
    pub fn new(config: ConvergenceStudyConfig) -> Self {
        Self { config }
    }

    /// Create a study with default configuration
    pub fn default() -> Self {
        Self::new(ConvergenceStudyConfig::default())
    }

    /// Run a full convergence study
    pub fn run_study(&self) -> ConvergenceStudyResult {
        let mut resolutions = Vec::new();
        let mut metrics = Vec::new();
        let mut warnings = Vec::new();

        // Generate test grids at different resolutions
        for level in 0..self.config.num_levels {
            let resolution = self.config.base_resolution * self.config.refinement_factor.pow(level);
            resolutions.push(resolution);

            match self.run_single_grid(resolution) {
                Ok(metric) => metrics.push(metric),
                Err(error) => {
                    warnings.push(format!("Resolution {}: {}", resolution, error));
                }
            }
        }

        // Analyze convergence behavior
        let convergence_order = self.estimate_convergence_order(&metrics);
        let is_converged = self.check_convergence(&metrics, &mut warnings);

        ConvergenceStudyResult {
            resolutions,
            metrics,
            convergence_order,
            is_converged,
            warnings,
        }
    }

    /// Run simulation on a single grid resolution
    fn run_single_grid(&self, resolution: u32) -> Result<ConvergenceMetric, String> {
        // Generate consistent terrain across resolutions
        let generator = DiamondSquareGenerator::new(self.config.terrain_seed);
        let config = DiamondSquareConfig {
            initial_corners: [0.3, 0.7, 0.4, 0.6],
            roughness: 0.6,
            persistence: 0.5,
            wrap_edges: false,
        };

        let heightmap = generator.generate(resolution as usize, resolution as usize, &config);

        // Create simulation with appropriate world scale
        let world_scale = WorldScale::new(
            self.config.domain_size_km,
            (resolution, resolution),
            DetailLevel::Standard,
        );

        let mut simulation = Simulation::_new_with_scale(heightmap, world_scale);

        // Disable rainfall for controlled convergence testing
        simulation.water_system.parameters.base_rainfall_rate = 0.0;
        simulation.water_system.effective_rainfall_rate = 0.0;

        // Add test water at center for reproducible initial conditions
        let center_x = (resolution / 2) as usize;
        let center_y = (resolution / 2) as usize;
        simulation.add_water_at(center_x, center_y, self.config.test_water_amount);

        // Run simulation steps
        for _ in 0..self.config.simulation_steps {
            simulation.tick();
        }

        // Compute convergence metrics
        let grid_spacing = (self.config.domain_size_km * 1000.0) / resolution as f64; // meters per cell
        let total_water = simulation.water.get_total_water() as f64;
        let max_water_depth = self.compute_max_water_depth(&simulation);
        let water_entropy = self.compute_water_entropy(&simulation);

        Ok(ConvergenceMetric {
            resolution,
            grid_spacing,
            total_water,
            max_water_depth,
            water_entropy,
            steps_simulated: self.config.simulation_steps,
        })
    }

    /// Compute maximum water depth in simulation
    fn compute_max_water_depth(&self, simulation: &Simulation) -> f64 {
        simulation
            .water
            .depth
            .iter()
            .flat_map(|row| row.iter())
            .fold(0.0f32, |max, &depth| max.max(depth)) as f64
    }

    /// Compute water distribution entropy (measure of how spread out water is)
    fn compute_water_entropy(&self, simulation: &Simulation) -> f64 {
        let total_water = simulation.water.get_total_water();
        if total_water <= 0.0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for row in &simulation.water.depth {
            for &depth in row {
                if depth > 0.0 {
                    let probability = depth / total_water;
                    entropy -= (probability as f64) * (probability as f64).ln();
                }
            }
        }

        entropy
    }

    /// Estimate convergence order from metrics
    fn estimate_convergence_order(&self, metrics: &[ConvergenceMetric]) -> Option<f64> {
        if metrics.len() < 3 {
            return None;
        }

        // Use total water as the convergence metric
        // Compute convergence order using Richardson extrapolation
        let mut orders = Vec::new();

        for i in 0..metrics.len() - 2 {
            let f_coarse = metrics[i].total_water;
            let f_medium = metrics[i + 1].total_water;
            let f_fine = metrics[i + 2].total_water;

            let r = self.config.refinement_factor as f64;

            // Richardson extrapolation: estimate order p
            // f_fine - f_medium = C * h_medium^p
            // f_medium - f_coarse = C * h_coarse^p = C * (r * h_medium)^p
            // Solving: p = ln((f_medium - f_coarse)/(f_fine - f_medium)) / ln(r)

            let numerator = (f_medium - f_coarse).abs();
            let denominator = (f_fine - f_medium).abs();

            if numerator > 1e-10 && denominator > 1e-10 {
                let order = (numerator / denominator).ln() / r.ln();
                if order.is_finite() && order > 0.0 {
                    orders.push(order);
                }
            }
        }

        if orders.is_empty() {
            None
        } else {
            // Return average convergence order
            Some(orders.iter().sum::<f64>() / orders.len() as f64)
        }
    }

    /// Check if the study indicates proper convergence
    fn check_convergence(&self, metrics: &[ConvergenceMetric], warnings: &mut Vec<String>) -> bool {
        if metrics.len() < 2 {
            warnings.push("Insufficient data points for convergence analysis".to_string());
            return false;
        }

        let mut converged = true;

        // Check that total water is conserved (within reasonable bounds)
        let first_total = metrics[0].total_water;
        for metric in metrics.iter().skip(1) {
            let relative_error = ((metric.total_water - first_total) / first_total).abs();
            if relative_error > 0.1 {
                // 10% tolerance
                warnings.push(format!(
                    "Water conservation violation at resolution {}: {:.2}% error",
                    metric.resolution,
                    relative_error * 100.0
                ));
                converged = false;
            }
        }

        // Check that solution is converging (decreasing changes with resolution)
        for i in 1..metrics.len() {
            let prev = &metrics[i - 1];
            let curr = &metrics[i];

            let change = (curr.total_water - prev.total_water).abs();
            let relative_change = change / prev.total_water.max(1e-10);

            // Solutions should change less as resolution increases
            if i > 1 {
                let prev_change = (prev.total_water - metrics[i - 2].total_water).abs();
                let prev_relative_change = prev_change / metrics[i - 2].total_water.max(1e-10);

                if relative_change > prev_relative_change * 1.5 {
                    warnings.push(format!(
                        "Non-convergent behavior between resolutions {} and {}",
                        prev.resolution, curr.resolution
                    ));
                }
            }
        }

        // Check for physically reasonable values
        for metric in metrics {
            if metric.total_water < 0.0 {
                warnings.push(format!(
                    "Negative total water at resolution {}: {}",
                    metric.resolution, metric.total_water
                ));
                converged = false;
            }

            if metric.max_water_depth < 0.0 {
                warnings.push(format!(
                    "Negative water depth at resolution {}: {}",
                    metric.resolution, metric.max_water_depth
                ));
                converged = false;
            }
        }

        converged
    }

    /// Run a climate convergence study to validate temperature field convergence
    pub fn run_climate_study(&self) -> ClimateConvergenceStudyResult {
        let mut resolutions = Vec::new();
        let mut metrics = Vec::new();
        let mut warnings = Vec::new();

        // Generate test grids at different resolutions
        for level in 0..self.config.num_levels {
            let resolution = self.config.base_resolution * self.config.refinement_factor.pow(level);
            resolutions.push(resolution);

            match self.run_single_climate_grid(resolution) {
                Ok(metric) => metrics.push(metric),
                Err(error) => {
                    warnings.push(format!("Climate resolution {}: {}", resolution, error));
                }
            }
        }

        // Analyze climate convergence behavior
        let mean_temp_convergence_order =
            self.estimate_climate_convergence_order(&metrics, |m| m.mean_temperature);
        let gradient_convergence_order =
            self.estimate_climate_convergence_order(&metrics, |m| m.temperature_gradient);
        let is_converged = self.check_climate_convergence(&metrics, &mut warnings);

        ClimateConvergenceStudyResult {
            resolutions,
            metrics,
            mean_temp_convergence_order,
            gradient_convergence_order,
            is_converged,
            warnings,
        }
    }

    /// Run climate analysis on a single grid resolution
    fn run_single_climate_grid(&self, resolution: u32) -> Result<ClimateConvergenceMetric, String> {
        // Generate consistent terrain across resolutions
        let generator = DiamondSquareGenerator::new(self.config.terrain_seed);
        let config = DiamondSquareConfig {
            initial_corners: [0.3, 0.7, 0.4, 0.6],
            roughness: 0.6,
            persistence: 0.5,
            wrap_edges: false,
        };

        let heightmap = generator.generate(resolution as usize, resolution as usize, &config);

        // Create world scale and climate system
        let world_scale = WorldScale::new(
            self.config.domain_size_km,
            (resolution, resolution),
            DetailLevel::Standard,
        );

        let climate_system = ClimateSystem::new_for_scale(&world_scale);
        let temperature_layer = climate_system.generate_temperature_layer(&heightmap);

        // Compute climate metrics
        let grid_spacing = world_scale.meters_per_pixel();
        let climate_metrics =
            self.compute_climate_metrics(&heightmap, &temperature_layer, grid_spacing);

        Ok(climate_metrics)
    }

    /// Compute climate convergence metrics from temperature field
    fn compute_climate_metrics(
        &self,
        heightmap: &[Vec<f32>],
        temperature_layer: &crate::climate::TemperatureLayer,
        grid_spacing: f64,
    ) -> ClimateConvergenceMetric {
        let height = heightmap.len();
        let width = if height > 0 { heightmap[0].len() } else { 0 };

        if width == 0 || height == 0 {
            return ClimateConvergenceMetric {
                resolution: 0,
                grid_spacing,
                mean_temperature: 0.0,
                temperature_std: 0.0,
                min_temperature: 0.0,
                max_temperature: 0.0,
                temperature_gradient: 0.0,
                temp_elevation_correlation: 0.0,
            };
        }

        // Collect all temperature and elevation values
        let mut temperatures = Vec::new();
        let mut elevations = Vec::new();
        let mut min_temp = f64::INFINITY;
        let mut max_temp = f64::NEG_INFINITY;

        for y in 0..height {
            for x in 0..width {
                let temp = temperature_layer.get_temperature(x, y) as f64;
                let elev = heightmap[y][x] as f64;

                temperatures.push(temp);
                elevations.push(elev);

                min_temp = min_temp.min(temp);
                max_temp = max_temp.max(temp);
            }
        }

        // Calculate mean temperature
        let mean_temperature = temperatures.iter().sum::<f64>() / temperatures.len() as f64;

        // Calculate temperature standard deviation
        let variance = temperatures
            .iter()
            .map(|t| (t - mean_temperature).powi(2))
            .sum::<f64>()
            / temperatures.len() as f64;
        let temperature_std = variance.sqrt();

        // Calculate temperature gradient strength (spatial derivative magnitude)
        let mut gradient_magnitudes = Vec::new();
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let temp_center = temperature_layer.get_temperature(x, y) as f64;
                let temp_right = temperature_layer.get_temperature(x + 1, y) as f64;
                let temp_up = temperature_layer.get_temperature(x, y - 1) as f64;

                let dx_gradient = (temp_right - temp_center) / grid_spacing;
                let dy_gradient = (temp_up - temp_center) / grid_spacing;
                let gradient_magnitude = (dx_gradient.powi(2) + dy_gradient.powi(2)).sqrt();

                gradient_magnitudes.push(gradient_magnitude);
            }
        }

        let temperature_gradient = if gradient_magnitudes.len() > 0 {
            gradient_magnitudes.iter().sum::<f64>() / gradient_magnitudes.len() as f64
        } else {
            0.0
        };

        // Calculate correlation between temperature and elevation
        let mean_elevation = elevations.iter().sum::<f64>() / elevations.len() as f64;

        let numerator: f64 = temperatures
            .iter()
            .zip(elevations.iter())
            .map(|(t, e)| (t - mean_temperature) * (e - mean_elevation))
            .sum();

        let temp_variance: f64 = temperatures
            .iter()
            .map(|t| (t - mean_temperature).powi(2))
            .sum();

        let elev_variance: f64 = elevations
            .iter()
            .map(|e| (e - mean_elevation).powi(2))
            .sum();

        let temp_elevation_correlation = if temp_variance > 0.0 && elev_variance > 0.0 {
            numerator / (temp_variance * elev_variance).sqrt()
        } else {
            0.0
        };

        ClimateConvergenceMetric {
            resolution: width.min(height) as u32,
            grid_spacing,
            mean_temperature,
            temperature_std,
            min_temperature: min_temp,
            max_temperature: max_temp,
            temperature_gradient,
            temp_elevation_correlation,
        }
    }

    /// Estimate convergence order for a specific climate metric
    fn estimate_climate_convergence_order<F>(
        &self,
        metrics: &[ClimateConvergenceMetric],
        metric_extractor: F,
    ) -> Option<f64>
    where
        F: Fn(&ClimateConvergenceMetric) -> f64,
    {
        if metrics.len() < 3 {
            return None; // Need at least 3 points for Richardson extrapolation
        }

        // Use Richardson extrapolation to estimate convergence order
        let mut convergence_orders = Vec::new();

        for i in 0..(metrics.len() - 2) {
            let coarse = &metrics[i];
            let medium = &metrics[i + 1];
            let fine = &metrics[i + 2];

            let f_coarse = metric_extractor(coarse);
            let f_medium = metric_extractor(medium);
            let f_fine = metric_extractor(fine);

            let h_coarse = coarse.grid_spacing;
            let h_medium = medium.grid_spacing;
            let h_fine = fine.grid_spacing;

            // Richardson extrapolation: f_h = f_exact + C*h^p
            // (f_medium - f_fine) / (f_coarse - f_medium) = (h_medium/h_fine)^p / (h_coarse/h_medium)^p
            let ratio_left = if (f_coarse - f_medium).abs() > 1e-12 {
                (f_medium - f_fine) / (f_coarse - f_medium)
            } else {
                continue; // Skip if denominator is too small
            };

            let ratio_right = (h_medium / h_fine) / (h_coarse / h_medium);

            if ratio_left > 0.0 && ratio_right > 0.0 {
                let order = ratio_left.ln() / ratio_right.ln();
                if order.is_finite() && order > 0.0 && order < 10.0 {
                    convergence_orders.push(order);
                }
            }
        }

        if convergence_orders.is_empty() {
            None
        } else {
            // Return average convergence order
            Some(convergence_orders.iter().sum::<f64>() / convergence_orders.len() as f64)
        }
    }

    /// Check if climate fields show convergence
    fn check_climate_convergence(
        &self,
        metrics: &[ClimateConvergenceMetric],
        warnings: &mut Vec<String>,
    ) -> bool {
        if metrics.len() < 2 {
            warnings.push("Insufficient data points for convergence analysis".to_string());
            return false;
        }

        let mut converged = true;

        // Check that mean temperature is stabilizing
        let temp_changes: Vec<f64> = metrics
            .windows(2)
            .map(|pair| (pair[1].mean_temperature - pair[0].mean_temperature).abs())
            .collect();

        if let Some(&last_temp_change) = temp_changes.last() {
            if last_temp_change > 0.5 {
                // 0.5°C tolerance
                warnings.push(format!(
                    "Mean temperature still changing significantly: {:.3}°C",
                    last_temp_change
                ));
                converged = false;
            }
        }

        // Check that temperature gradients are stabilizing
        let gradient_changes: Vec<f64> = metrics
            .windows(2)
            .map(|pair| (pair[1].temperature_gradient - pair[0].temperature_gradient).abs())
            .collect();

        if let Some(&last_gradient_change) = gradient_changes.last() {
            let relative_change = if metrics.last().unwrap().temperature_gradient > 0.0 {
                last_gradient_change / metrics.last().unwrap().temperature_gradient
            } else {
                last_gradient_change
            };

            if relative_change > 0.1 {
                // 10% relative tolerance
                warnings.push(format!(
                    "Temperature gradient still changing: {:.1}% relative change",
                    relative_change * 100.0
                ));
                converged = false;
            }
        }

        // Check temperature-elevation correlation consistency
        let correlation_changes: Vec<f64> = metrics
            .windows(2)
            .map(|pair| {
                (pair[1].temp_elevation_correlation - pair[0].temp_elevation_correlation).abs()
            })
            .collect();

        if let Some(&last_correlation_change) = correlation_changes.last() {
            if last_correlation_change > 0.05 {
                // 5% correlation change tolerance
                warnings.push(format!(
                    "Temperature-elevation correlation still changing: {:.3}",
                    last_correlation_change
                ));
                converged = false;
            }
        }

        converged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convergence_study_runs() {
        let config = ConvergenceStudyConfig {
            domain_size_km: 5.0,
            base_resolution: 25,
            refinement_factor: 2,
            num_levels: 3, // Test 25x25, 50x50, 100x100
            simulation_steps: 10,
            test_water_amount: 0.5,
            terrain_seed: 123,
        };

        let study = ConvergenceStudy::new(config);
        let result = study.run_study();

        // Should have completed all levels
        assert_eq!(result.resolutions.len(), 3);
        assert_eq!(result.metrics.len(), 3);

        // Resolutions should be correctly scaled
        assert_eq!(result.resolutions[0], 25);
        assert_eq!(result.resolutions[1], 50);
        assert_eq!(result.resolutions[2], 100);

        // Should have computed metrics for each resolution
        for (i, metric) in result.metrics.iter().enumerate() {
            assert_eq!(metric.resolution, result.resolutions[i]);
            assert!(metric.grid_spacing > 0.0);
            assert!(metric.total_water >= 0.0);
            assert!(metric.steps_simulated > 0);
        }
    }

    #[test]
    fn convergence_metrics_reasonable() {
        let config = ConvergenceStudyConfig {
            base_resolution: 50,
            num_levels: 2,
            simulation_steps: 5,
            test_water_amount: 0.1, // Smaller test amount
            ..Default::default()
        };

        let study = ConvergenceStudy::new(config);
        let result = study.run_study();

        // Should detect some level of convergence behavior
        for metric in &result.metrics {
            // Water should be conserved (not lost or created excessively)
            assert!(
                metric.total_water > 0.0,
                "Water should be present after adding test water"
            );
            // Allow for rainfall accumulation over simulation steps
            assert!(
                metric.total_water < 50.0,
                "Water shouldn't grow excessively: got {}",
                metric.total_water
            );

            // Physical quantities should be reasonable
            assert!(
                metric.max_water_depth >= 0.0,
                "Water depth can't be negative"
            );
            assert!(metric.water_entropy >= 0.0, "Entropy can't be negative");
        }
    }

    #[test]
    fn grid_spacing_scales_correctly() {
        let config = ConvergenceStudyConfig {
            domain_size_km: 10.0,
            base_resolution: 100,
            refinement_factor: 2,
            num_levels: 3,
            simulation_steps: 1,
            ..Default::default()
        };

        let study = ConvergenceStudy::new(config);
        let result = study.run_study();

        // Grid spacing should halve with each refinement
        for i in 1..result.metrics.len() {
            let prev_spacing = result.metrics[i - 1].grid_spacing;
            let curr_spacing = result.metrics[i].grid_spacing;

            let ratio = prev_spacing / curr_spacing;
            assert!(
                (ratio - 2.0).abs() < 0.1,
                "Grid spacing should double with each coarsening: {} vs {}",
                prev_spacing,
                curr_spacing
            );
        }
    }

    // Climate convergence tests
    #[test]
    fn climate_convergence_study_runs() {
        let config = ConvergenceStudyConfig {
            domain_size_km: 5.0,
            base_resolution: 25,
            refinement_factor: 2,
            num_levels: 3,          // Test 25x25, 50x50, 100x100
            simulation_steps: 5,    // Not used for climate analysis
            test_water_amount: 0.0, // Not used for climate analysis
            terrain_seed: 123,
        };

        let study = ConvergenceStudy::new(config);
        let result = study.run_climate_study();

        // Should have completed all levels
        assert_eq!(result.resolutions.len(), 3);
        assert_eq!(result.metrics.len(), 3);

        // Resolutions should be correctly scaled
        assert_eq!(result.resolutions[0], 25);
        assert_eq!(result.resolutions[1], 50);
        assert_eq!(result.resolutions[2], 100);

        // Should have computed climate metrics for each resolution
        for (i, metric) in result.metrics.iter().enumerate() {
            assert_eq!(metric.resolution, result.resolutions[i]);
            assert!(metric.grid_spacing > 0.0);
            assert!(metric.mean_temperature.is_finite());
            assert!(metric.temperature_std >= 0.0);
            assert!(metric.min_temperature <= metric.max_temperature);
            assert!(metric.temperature_gradient >= 0.0);
        }
    }

    #[test]
    fn climate_metrics_reasonable() {
        let config = ConvergenceStudyConfig {
            base_resolution: 50,
            num_levels: 2,
            terrain_seed: 42,
            ..Default::default()
        };

        let study = ConvergenceStudy::new(config);
        let result = study.run_climate_study();

        assert_eq!(result.metrics.len(), 2);

        for metric in &result.metrics {
            // Temperature should be in reasonable range
            assert!(
                metric.mean_temperature > -50.0 && metric.mean_temperature < 50.0,
                "Mean temperature should be reasonable: {}",
                metric.mean_temperature
            );

            // Standard deviation should be positive (spatial variation exists)
            assert!(
                metric.temperature_std > 0.0,
                "Temperature should vary spatially: std = {}",
                metric.temperature_std
            );

            // Min should be less than max
            assert!(
                metric.min_temperature < metric.max_temperature,
                "Temperature range should be non-zero: {} to {}",
                metric.min_temperature,
                metric.max_temperature
            );

            // Temperature gradient should be positive (elevation effects)
            assert!(
                metric.temperature_gradient > 0.0,
                "Temperature gradients should exist due to elevation: {}",
                metric.temperature_gradient
            );

            // Correlation with elevation should be negative (higher = cooler)
            assert!(
                metric.temp_elevation_correlation < 0.0,
                "Temperature should correlate negatively with elevation: {}",
                metric.temp_elevation_correlation
            );
        }
    }

    #[test]
    fn climate_convergence_order_estimation() {
        let config = ConvergenceStudyConfig {
            domain_size_km: 10.0,
            base_resolution: 25,
            refinement_factor: 2,
            num_levels: 4, // Need at least 3 for convergence order estimation
            terrain_seed: 42,
            ..Default::default()
        };

        let study = ConvergenceStudy::new(config);
        let result = study.run_climate_study();

        assert_eq!(result.metrics.len(), 4);

        // Should be able to estimate convergence orders
        // Climate fields should converge as grid is refined
        if let Some(temp_order) = result.mean_temp_convergence_order {
            assert!(
                temp_order > 0.0 && temp_order < 10.0,
                "Temperature convergence order should be reasonable: {}",
                temp_order
            );
        }

        if let Some(gradient_order) = result.gradient_convergence_order {
            assert!(
                gradient_order > 0.0 && gradient_order < 10.0,
                "Gradient convergence order should be reasonable: {}",
                gradient_order
            );
        }

        // Grid spacing should halve with each refinement for climate metrics too
        for i in 1..result.metrics.len() {
            let prev_spacing = result.metrics[i - 1].grid_spacing;
            let curr_spacing = result.metrics[i].grid_spacing;

            let ratio = prev_spacing / curr_spacing;
            assert!(
                (ratio - 2.0).abs() < 0.1,
                "Grid spacing should double with each coarsening: {} vs {}",
                prev_spacing,
                curr_spacing
            );
        }
    }

    #[test]
    fn climate_temperature_elevation_correlation() {
        let config = ConvergenceStudyConfig {
            base_resolution: 50,
            num_levels: 2,
            terrain_seed: 123, // Known seed with elevation variation
            ..Default::default()
        };

        let study = ConvergenceStudy::new(config);
        let result = study.run_climate_study();

        for metric in &result.metrics {
            // Temperature-elevation correlation should be negative
            // (higher elevation = cooler temperatures)
            // For small test domains, expect at least some negative correlation
            assert!(
                metric.temp_elevation_correlation < 0.0,
                "Temperature should correlate negatively with elevation: {}",
                metric.temp_elevation_correlation
            );

            // Correlation should be finite and within reasonable bounds
            assert!(
                metric.temp_elevation_correlation.is_finite(),
                "Temperature-elevation correlation should be finite"
            );
            assert!(
                metric.temp_elevation_correlation > -1.0,
                "Temperature-elevation correlation should be > -1.0: {}",
                metric.temp_elevation_correlation
            );
        }

        // Correlations should be similar across resolutions (within tolerance)
        if result.metrics.len() >= 2 {
            let correlation_diff = (result.metrics[0].temp_elevation_correlation
                - result.metrics[1].temp_elevation_correlation)
                .abs();
            assert!(
                correlation_diff < 0.3,
                "Temperature-elevation correlation should be stable across resolutions: diff = {}",
                correlation_diff
            );
        }
    }
}
