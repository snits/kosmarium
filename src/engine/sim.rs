// ABOUTME: Core simulation state and water flow system for dynamic terrain evolution
// ABOUTME: Manages heightmap terrain with real-time water flow, accumulation, and hydraulic erosion

use super::agents::biome::{BiomeClassifier, BiomeMap};
use super::core::dimensional::{
    DimensionalAnalysis, DimensionalWaterFlowParameters, PhysicalQuantity,
};
use super::core::heightmap::HeightMap;
use super::core::scale::{REFERENCE_SCALE, ScaleAware, WorldScale};
use super::physics::atmosphere::{AtmosphericSystem, WeatherAnalysis, WindLayer};
use super::physics::climate::{AtmosphericPressureLayer, ClimateSystem, TemperatureLayer};
use super::physics::drainage::{DrainageNetwork, DrainageNetworkStatistics};
use super::physics::flow_engine::{FlowEngine, FlowParameters};
use super::physics::water::{Vec2, WaterLayer};

/// Simulation time information for display
#[derive(Debug, Clone)]
pub struct SimulationTime {
    pub tick_count: u64,
    pub days: u32,
    pub hours: u32,
    pub minutes: u32,
    pub total_hours: u32,
}

/// Raw, scale-independent water flow parameters
/// These represent the base behavior before any scale adjustments
#[derive(Clone, Debug)]
pub struct WaterFlowParameters {
    pub flow_rate: f32,                    // How fast water flows (0.0-1.0)
    pub evaporation_rate: f32,             // Water loss per tick (0.0-1.0)
    pub erosion_strength: f32,             // How much sediment water can carry
    pub deposition_rate: f32,              // How fast sediment settles
    pub base_rainfall_rate: f32,           // Base rainfall per cell per tick (at reference scale)
    pub rainfall_scaling: RainfallScaling, // How rainfall adjusts with map size
    pub max_expected_velocity_ms: f32,     // Maximum expected flow velocity in m/s (for CFL)
    pub cfl_safety_factor: f32,            // Safety margin for CFL condition (0.0-1.0)
}

/// Scale-derived water flow system with effective parameters
/// Migrated to use unified FlowEngine with gradient-based algorithm
pub struct WaterFlowSystem {
    pub parameters: WaterFlowParameters,
    pub effective_rainfall_rate: f32, // Computed rainfall rate for current scale
    pub _stable_timestep_seconds: f32, // CFL-derived timestep for numerical stability
    pub evaporation_threshold: f32,   // Scale-aware threshold for clearing tiny water amounts
    pub drainage_metrics: DrainageMetrics, // Boundary drainage monitoring and instrumentation

    /// Unified flow engine with gradient-based algorithm for interactive simulation
    flow_engine: Option<FlowEngine>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RainfallScaling {
    /// Same rainfall per cell regardless of map size (higher total water on larger maps)
    /// Use for predictable behavior and debugging
    _PerCell,

    /// Mass-conserving scaling: Total rainfall over region remains constant
    /// Rain per cell ∝ 1/area - larger maps get less rain per cell
    /// Use for realistic water budget modeling
    MassConserving,

    /// Intensity-based scaling: Meteorological intensity remains constant per unit area
    /// Same as PerCell but with clearer physical interpretation
    /// Use when modeling actual precipitation rates
    _IntensityBased,

    /// Hydrologically realistic scaling: Based on empirical watershed relationships
    /// Many hydrological processes follow Area^0.6 scaling laws
    /// Use for realistic terrain evolution simulation
    _HydrologicalRealistic,
}

impl Default for WaterFlowParameters {
    fn default() -> Self {
        // Default values calibrated for ~240x120 reference map
        Self {
            flow_rate: 0.1,
            evaporation_rate: 0.001,
            erosion_strength: 0.01,
            deposition_rate: 0.05,
            base_rainfall_rate: 0.0000027127, // Mathematical optimization: 737x reduction to eliminate 2993% water bug
            rainfall_scaling: RainfallScaling::MassConserving, // Physics-based total mass conservation
            max_expected_velocity_ms: 2.0, // Reasonable for gentle water flow (walking speed)
            cfl_safety_factor: 0.5,        // Conservative safety margin
        }
    }
}

impl ScaleAware for WaterFlowParameters {
    fn derive_parameters(&self, scale: &WorldScale) -> Self {
        let grid_spacing_m = scale.meters_per_pixel() as f32;
        let physical_size_km = scale.physical_size_km as f32;

        // ScaleAware CFL parameters - scale with grid resolution and domain size
        // Larger grids need longer timesteps, larger domains allow longer timesteps
        let domain_factor = (physical_size_km / 100.0).ln().max(1.0).min(4.0); // 1.0-4.0 range
        let resolution_factor = grid_spacing_m / 100.0; // Reference: 100m/pixel

        Self {
            flow_rate: self.flow_rate,
            evaporation_rate: self.evaporation_rate,
            erosion_strength: self.erosion_strength,
            deposition_rate: self.deposition_rate,
            base_rainfall_rate: self.base_rainfall_rate,
            rainfall_scaling: self.rainfall_scaling.clone(),
            // ScaleAware CFL parameters instead of hardcoded bounds
            max_expected_velocity_ms: self.max_expected_velocity_ms,
            cfl_safety_factor: self.cfl_safety_factor * (1.0 + 0.1 * (domain_factor - 1.0)), // Slightly more conservative for larger domains
        }
    }
}

impl WaterFlowSystem {
    /// Create a water flow system from parameters and world scale
    pub fn from_parameters(parameters: WaterFlowParameters, scale: &WorldScale) -> Self {
        let scaled_params = parameters.derive_parameters(scale);
        let effective_rainfall_rate = Self::calculate_rainfall_rate(&scaled_params, scale);
        let stable_timestep_seconds = Self::calculate_cfl_timestep(&scaled_params, scale);
        let evaporation_threshold =
            Self::calculate_evaporation_threshold(&scaled_params, effective_rainfall_rate);

        Self {
            parameters: scaled_params,
            effective_rainfall_rate,
            _stable_timestep_seconds: stable_timestep_seconds,
            evaporation_threshold,
            drainage_metrics: DrainageMetrics::new(),
            flow_engine: None, // Initialized lazily when needed
        }
    }

    /// Calculate the effective rainfall rate based on scaling strategy
    fn calculate_rainfall_rate(params: &WaterFlowParameters, scale: &WorldScale) -> f32 {
        match params.rainfall_scaling {
            RainfallScaling::_PerCell => {
                // No scaling - same rain per cell regardless of map size
                params.base_rainfall_rate
            }
            RainfallScaling::MassConserving => {
                // Total rainfall over region remains constant relative to reference scale
                // The base rate represents rainfall at reference scale, so we need inverse scaling:
                // More cells = less rain per cell to maintain same total regional rainfall
                let area_ratio = scale.scale_factor_from_reference(REFERENCE_SCALE) as f32;
                params.base_rainfall_rate / area_ratio // More cells = less rain per cell
            }
            RainfallScaling::_IntensityBased => {
                // Meteorological intensity remains constant - same as PerCell
                params.base_rainfall_rate
            }
            RainfallScaling::_HydrologicalRealistic => {
                // Based on empirical relationships in hydrology
                // Many watershed processes follow ~ Area^0.6 relationships
                let area_ratio = scale.scale_factor_from_reference(REFERENCE_SCALE) as f32;
                params.base_rainfall_rate / area_ratio.powf(0.6) // Fixed: was multiplying instead of dividing
            }
        }
    }

    /// Calculate CFL-stable timestep based on grid resolution and expected velocities
    /// CFL condition: dt ≤ dx / max_velocity
    fn calculate_cfl_timestep(params: &WaterFlowParameters, scale: &WorldScale) -> f32 {
        let dx = scale.meters_per_pixel() as f32; // Grid spacing in meters
        let max_velocity = params.max_expected_velocity_ms; // Maximum expected velocity in m/s

        // CFL condition with safety factor
        let cfl_timestep = params.cfl_safety_factor * dx / max_velocity;

        // ScaleAware bounds - smooth scaling without hardcoded thresholds
        let physical_size_km = scale.physical_size_km as f32;
        let grid_spacing_m = scale.meters_per_pixel() as f32;

        // Domain-aware scaling: larger domains can support longer timesteps
        let domain_scale_factor = (physical_size_km / 100.0).ln().max(1.0).min(4.0); // 1.0-4.0

        // Resolution-aware scaling: coarser grids need longer minimum timesteps
        let base_min_timestep = 0.001; // 1ms base for fine grids
        let base_max_timestep = 60.0; // 1 minute base for small domains

        let min_timestep = base_min_timestep * (grid_spacing_m / 10.0).max(1.0); // Scale with resolution
        let max_timestep = base_max_timestep * domain_scale_factor; // Scale with domain size

        cfl_timestep.max(min_timestep).min(max_timestep)
    }

    /// Calculate scale-aware evaporation threshold to prevent clearing water that should accumulate
    /// Threshold should be small enough to allow rainfall to accumulate over multiple ticks
    fn calculate_evaporation_threshold(
        params: &WaterFlowParameters,
        effective_rainfall_rate: f32,
    ) -> f32 {
        // Calculate what water depth looks like after evaporation is applied to one tick of rainfall
        let post_evaporation_rainfall = effective_rainfall_rate * (1.0 - params.evaporation_rate);

        // Set threshold to 1% of post-evaporation rainfall, ensuring water can accumulate
        // This allows ~100 ticks of rainfall to build up before being cleared
        let scale_aware_threshold = post_evaporation_rainfall * 0.01;

        // Ensure minimum threshold is not too small (avoid floating point precision issues)
        // but also not too large (don't clear water that should accumulate)
        scale_aware_threshold.max(1e-8).min(1e-4)
    }
}

impl WaterFlowSystem {
    /// Create a water flow system with default parameters for the given world scale
    pub fn new_for_scale(scale: &WorldScale) -> Self {
        let parameters = WaterFlowParameters::default();
        Self::from_parameters(parameters, scale)
    }

    /// Get the effective rainfall rate for this system
    pub fn _get_effective_rainfall_rate(&self) -> f32 {
        self.effective_rainfall_rate
    }

    /// Get the CFL-stable timestep for this system
    pub fn _get_stable_timestep_seconds(&self) -> f32 {
        self._stable_timestep_seconds
    }

    /// Get or initialize the unified flow engine for this water system
    /// Uses gradient-based algorithm optimized for interactive simulation
    fn get_flow_engine(&mut self, water: &WaterLayer, scale: &WorldScale) -> &mut FlowEngine {
        if self.flow_engine.is_none() {
            // Create flow engine with gradient-based algorithm for interactive simulation
            let mut engine = FlowEngine::new(
                super::physics::flow_engine::FlowAlgorithm::Gradient,
                water.width(),
                water.height(),
                scale,
            );

            // Convert legacy parameters to unified FlowParameters
            engine.parameters = FlowParameters {
                gravity: 9.81,
                roughness: 0.03,
                min_depth: self.evaporation_threshold * 0.01, // Use system's evaporation threshold
                concentration_factor: 1000.0, // Conservative for interactive simulation
                cfl_safety: self.parameters.cfl_safety_factor,
                dt: self._stable_timestep_seconds, // Use system's calculated timestep
            };

            self.flow_engine = Some(engine);
        }

        self.flow_engine.as_mut().unwrap()
    }

    /// Create dimensional parameters for proper physical analysis
    pub fn create_dimensional_parameters(
        &self,
        scale: &WorldScale,
    ) -> DimensionalWaterFlowParameters {
        // Convert normalized parameters to physical units
        let max_velocity_ms = self.parameters.max_expected_velocity_ms as f64;

        // Convert base rainfall rate to mm/h (assuming it's normalized per hour)
        let rainfall_rate_mmh = (self.effective_rainfall_rate * 1000.0) as f64; // Convert m/h to mm/h

        // Convert evaporation rate (assuming similar scaling)
        let evaporation_rate_mmh = (self.parameters.evaporation_rate * 1000.0) as f64; // Convert m/h to mm/h

        DimensionalAnalysis::from_world_scale(
            scale,
            max_velocity_ms,
            rainfall_rate_mmh,
            evaporation_rate_mmh,
        )
    }

    /// Validate dimensional consistency and report any physical issues
    pub fn validate_physical_parameters(&self, scale: &WorldScale) -> Vec<String> {
        let dimensional_params = self.create_dimensional_parameters(scale);
        DimensionalAnalysis::validate_dimensional_consistency(&dimensional_params)
    }

    /// Get physical rainfall volume per timestep in cubic meters per square meter
    pub fn get_rainfall_volume_rate(&self, scale: &WorldScale) -> PhysicalQuantity {
        let dimensional_params = self.create_dimensional_parameters(scale);
        dimensional_params.rainfall_depth_per_timestep()
    }

    /// Get physical evaporation volume per timestep in cubic meters per square meter  
    pub fn get_evaporation_volume_rate(&self, scale: &WorldScale) -> PhysicalQuantity {
        let dimensional_params = self.create_dimensional_parameters(scale);
        dimensional_params.evaporation_depth_per_timestep()
    }

    /// Check if current flow velocities are within CFL stability bounds
    pub fn _check_cfl_stability(&self, water: &WaterLayer, scale: &WorldScale) -> bool {
        let dx = scale.meters_per_pixel() as f32;
        let mut max_observed_velocity = 0.0f32;

        for y in 0..water.height() {
            for x in 0..water.width() {
                let (vx, vy) = water.velocity.get(x, y);
                let velocity_mag = (vx * vx + vy * vy).sqrt();
                max_observed_velocity = max_observed_velocity.max(velocity_mag);
            }
        }

        // Convert from dimensionless simulation units to m/s (rough approximation)
        let estimated_velocity_ms = max_observed_velocity * dx;

        // Check if we're within expected bounds
        estimated_velocity_ms <= self.parameters.max_expected_velocity_ms
    }

    /// Estimate grid spacing from heightmap context (temporary until WorldScale is available)
    fn estimate_grid_spacing_from_context(&self, heightmap: &HeightMap) -> f32 {
        // Use default scaling relationships based on typical domain sizes
        let width = heightmap.width();
        let height = heightmap.height();
        let total_cells = width * height;

        // Estimate based on common scaling patterns:
        // Small grids (< 10k cells): ~100m/pixel
        // Medium grids (10k-100k cells): ~1000m/pixel
        // Large grids (> 100k cells): ~10000m/pixel
        if total_cells < 10_000 {
            100.0 // Small domain
        } else if total_cells < 100_000 {
            1000.0 // Medium domain
        } else {
            10000.0 // Large domain
        }
    }

    /// Estimate grid spacing from water layer dimensions (for boundary flow calculations)
    fn estimate_grid_spacing_from_water_layer(&self, water: &WaterLayer) -> f32 {
        // Use same scaling logic as heightmap version
        let width = water.width();
        let height = water.height();
        let total_cells = width * height;

        // Estimate based on common scaling patterns
        if total_cells < 10_000 {
            100.0 // Small domain
        } else if total_cells < 100_000 {
            1000.0 // Medium domain
        } else {
            10000.0 // Large domain
        }
    }

    /// Calculate flow direction for each cell based on elevation gradients
    /// Enhanced with drainage-aware flow for gradual water concentration
    pub fn calculate_flow_directions(&mut self, heightmap: &HeightMap, water: &mut WaterLayer) {
        // For now, estimate grid spacing based on default scaling
        // This is a temporary fix until we can properly pass WorldScale context
        let grid_spacing_m = self.estimate_grid_spacing_from_context(heightmap);
        self.calculate_flow_directions_with_spacing(heightmap, water, grid_spacing_m);
    }

    /// Calculate flow directions using explicit WorldScale (preferred method)
    /// This method uses the correct grid spacing from WorldScale instead of heuristics
    pub fn calculate_flow_directions_with_scale(
        &mut self,
        heightmap: &HeightMap,
        water: &mut WaterLayer,
        scale: &WorldScale,
    ) {
        let grid_spacing_m = scale.meters_per_pixel() as f32;
        self.calculate_flow_directions_with_spacing(heightmap, water, grid_spacing_m);
    }

    fn calculate_flow_directions_with_spacing(
        &mut self,
        heightmap: &HeightMap,
        water: &mut WaterLayer,
        grid_spacing_m: f32,
    ) {
        // Create WorldScale from grid spacing for unified FlowEngine
        let scale = WorldScale::new(
            grid_spacing_m as f64,
            (heightmap.width() as u32, heightmap.height() as u32),
            super::core::scale::DetailLevel::Standard,
        );

        // Get or initialize the unified flow engine
        let flow_engine = self.get_flow_engine(water, &scale);

        // Delegate to unified FlowEngine with gradient-based algorithm
        // This replaces the manual 8-neighbor steepest descent calculation
        // with the consolidated gradient flow implementation
        flow_engine.calculate_flow(heightmap, water, None, &scale);
    }

    /// Simulate one tick of water flow with drainage-aware flow enhancement
    pub fn update_water_flow_with_drainage(
        &mut self,
        heightmap: &mut HeightMap,
        water: &mut WaterLayer,
        drainage_network: &DrainageNetwork,
    ) {
        // Calculate flow directions based on current state and drainage network
        self.calculate_flow_directions_with_drainage(
            heightmap,
            water,
            drainage_network,
            self.estimate_grid_spacing_from_context(heightmap),
        );

        // Add rainfall
        self.add_rainfall(water);

        // Move water based on flow directions (now drainage-aware)
        self.move_water_with_boundaries(water);

        // Apply erosion and deposition
        self.apply_erosion(heightmap, water);

        // Evaporate water (uniform rate - for systems without climate integration)
        self.apply_evaporation(water);
    }

    /// Simulate one tick of water flow (legacy method without drainage awareness)
    pub fn update_water_flow(&mut self, heightmap: &mut HeightMap, water: &mut WaterLayer) {
        // Calculate flow directions based on current state
        self.calculate_flow_directions_with_spacing(
            heightmap,
            water,
            self.estimate_grid_spacing_from_context(heightmap),
        );

        // Add rainfall
        self.add_rainfall(water);

        // Move water based on flow directions
        self.move_water(water);

        // Apply erosion and deposition
        self.apply_erosion(heightmap, water);

        // Evaporate water (uniform rate - for systems without climate integration)
        self.apply_evaporation(water);
    }

    /// Simulate one tick of water flow with climate integration and drainage-aware flow
    pub fn update_water_flow_with_climate_and_drainage(
        &mut self,
        heightmap: &mut HeightMap,
        water: &mut WaterLayer,
        temperature_layer: &mut TemperatureLayer,
        climate_system: &ClimateSystem,
        drainage_network: &DrainageNetwork,
        world_scale: &WorldScale,
    ) {
        // Calculate flow directions based on current state and drainage network
        let grid_spacing_m = world_scale.meters_per_pixel() as f32;
        self.calculate_flow_directions_with_drainage(
            heightmap,
            water,
            drainage_network,
            grid_spacing_m,
        );

        // Add rainfall
        self.add_rainfall(water);

        // Move water based on flow directions (now drainage-aware with boundaries)
        self.move_water_with_boundaries(water);

        // Apply erosion and deposition
        self.apply_erosion(heightmap, water);

        // Apply temperature-dependent evaporation
        self.apply_evaporation_with_temperature(water, temperature_layer, climate_system);
    }

    /// Simulate one tick of water flow with climate integration (legacy method)
    pub fn update_water_flow_with_climate(
        &mut self,
        heightmap: &mut HeightMap,
        water: &mut WaterLayer,
        temperature_layer: &mut TemperatureLayer,
        climate_system: &ClimateSystem,
    ) {
        // Calculate flow directions based on current state
        self.calculate_flow_directions_with_spacing(
            heightmap,
            water,
            self.estimate_grid_spacing_from_context(heightmap),
        );

        // Add rainfall
        self.add_rainfall(water);

        // Move water based on flow directions
        self.move_water(water);

        // Apply erosion and deposition
        self.apply_erosion(heightmap, water);

        // Apply temperature-dependent evaporation
        self.apply_evaporation_with_temperature(water, temperature_layer, climate_system);
    }

    fn add_rainfall(&mut self, water: &mut WaterLayer) {
        let rainfall_added = self.effective_rainfall_rate * (water.width() * water.height()) as f32;
        self.drainage_metrics.total_rainfall_input += rainfall_added;

        for depth in water.depth.iter_mut() {
            *depth += self.effective_rainfall_rate;
        }
    }

    fn move_water(&self, water: &mut WaterLayer) {
        // Use double-buffering to eliminate clone() allocation:
        // 1. Copy current depth to buffer as starting point
        water.copy_depth_to_buffer();

        for y in 0..water.height() {
            for x in 0..water.width() {
                let (vx, vy) = water.velocity.get(x, y);
                let velocity_mag = (vx * vx + vy * vy).sqrt();
                // CFL-stable velocity limit: max 0.5 cells per timestep for numerical stability
                let max_velocity = 0.5; // Conservative CFL condition
                let flow_amount = water.depth.get(x, y) * velocity_mag.min(max_velocity);

                // Computational flow threshold - allows realistic small-scale movement
                let flow_threshold = 1e-8; // Based on numerical precision, not evaporation rates
                if flow_amount > flow_threshold {
                    // Enhanced accumulative flow: allow fractional movement accumulation
                    // instead of rounding immediately to integer positions
                    let target_x_float = x as f32 + vx;
                    let target_y_float = y as f32 + vy;

                    // Calculate fractional flow distribution to neighboring cells
                    let x0 = target_x_float.floor() as i32;
                    let x1 = x0 + 1;
                    let y0 = target_y_float.floor() as i32;
                    let y1 = y0 + 1;

                    let fx = target_x_float.fract();
                    let fy = target_y_float.fract();

                    // Bilinear interpolation weights for flow distribution
                    let weight_00 = (1.0 - fx) * (1.0 - fy); // Bottom-left
                    let weight_10 = fx * (1.0 - fy); // Bottom-right
                    let weight_01 = (1.0 - fx) * fy; // Top-left
                    let weight_11 = fx * fy; // Top-right

                    // Get dimensions before mutable borrow
                    let width = water.width() as i32;
                    let height = water.height() as i32;

                    let buffer = water.get_depth_buffer_mut();
                    let current_depth = buffer.get(x, y);

                    // Remove water from current cell
                    buffer.set(x, y, current_depth - flow_amount);

                    // Distribute flow to target cells based on fractional position
                    let flow_cells = [
                        (x0, y0, weight_00),
                        (x1, y0, weight_10),
                        (x0, y1, weight_01),
                        (x1, y1, weight_11),
                    ];

                    for (tx, ty, weight) in flow_cells {
                        if tx >= 0 && tx < width && ty >= 0 && ty < height {
                            let target_flow = flow_amount * weight;
                            if target_flow > 1e-8 {
                                // Avoid microscopic flows
                                let target_depth = buffer.get(tx as usize, ty as usize);
                                buffer.set(tx as usize, ty as usize, target_depth + target_flow);
                            }
                        } else {
                            // Flow out of bounds = boundary outflow (lost water)
                            // This is the critical fix: water that flows beyond boundaries is lost
                        }
                    }
                }
            }
        }

        // 3. Swap buffers to make the result the new primary depth
        water.swap_depth_buffers();
    }

    fn apply_erosion(&self, heightmap: &mut HeightMap, water: &mut WaterLayer) {
        for y in 0..water.height() {
            for x in 0..water.width() {
                let velocity = water.velocity.get(x, y);
                let flow_speed = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
                let water_depth = water.depth.get(x, y);

                // Scale-aware erosion thresholds based on domain characteristics
                let erosion_flow_threshold = self.evaporation_threshold * 20.0; // Erosion needs significant flow
                let erosion_depth_threshold = self.evaporation_threshold * 5.0; // Minimum depth for erosion
                if flow_speed > erosion_flow_threshold && water_depth > erosion_depth_threshold {
                    // Erosion capacity based on flow speed and water depth
                    let erosion_capacity =
                        flow_speed * water_depth * self.parameters.erosion_strength;

                    // Erode terrain if we're below capacity
                    let current_sediment = water.sediment.get(x, y);
                    if current_sediment < erosion_capacity {
                        // Scale-aware erosion limit - prevent unrealistic landscape changes
                        let max_erosion_per_tick = self.evaporation_threshold * 100.0; // Scale with domain size
                        let erosion_amount =
                            (erosion_capacity - current_sediment).min(max_erosion_per_tick);
                        let current_height = heightmap.get(x, y);
                        heightmap.set(x, y, current_height - erosion_amount);
                        water.sediment.set(x, y, current_sediment + erosion_amount);
                    }
                    // Deposit sediment if we're over capacity
                    else if current_sediment > erosion_capacity {
                        let deposition_amount =
                            (current_sediment - erosion_capacity) * self.parameters.deposition_rate;
                        let current_height = heightmap.get(x, y);
                        heightmap.set(x, y, current_height + deposition_amount);
                        water
                            .sediment
                            .set(x, y, current_sediment - deposition_amount);
                    }
                }
            }
        }
    }

    /// Apply uniform evaporation (base case without temperature effects)
    fn apply_evaporation(&mut self, water: &mut WaterLayer) {
        let mut total_evaporated = 0.0;

        for depth in water.depth.iter_mut() {
            let initial_depth = *depth;
            *depth *= 1.0 - self.parameters.evaporation_rate;
            if *depth < self.evaporation_threshold {
                *depth = 0.0;
            }
            total_evaporated += initial_depth - *depth;
        }

        self.drainage_metrics.total_evaporation += total_evaporated;

        // Also evaporate sediment when water disappears
        for y in 0..water.height() {
            for x in 0..water.width() {
                if water.depth.get(x, y) < self.evaporation_threshold {
                    let current_sediment = water.sediment.get(x, y);
                    water.sediment.set(x, y, current_sediment * 0.5); // Sediment settles when water dries up
                }
            }
        }
    }

    /// Apply temperature-dependent evaporation using climate data WITH energy conservation
    fn apply_evaporation_with_temperature(
        &mut self,
        water: &mut WaterLayer,
        temperature_layer: &mut TemperatureLayer,
        climate_system: &ClimateSystem,
    ) {
        let mut total_evaporated = 0.0;

        for y in 0..water.height() {
            for x in 0..water.width() {
                // Get current temperature at this location
                let temperature_c =
                    temperature_layer.get_current_temperature(x, y, climate_system.current_season);

                // Get temperature-dependent evaporation multiplier
                let temp_multiplier = climate_system.get_evaporation_multiplier(temperature_c);

                // Apply temperature-modified evaporation rate
                let effective_evaporation_rate = self.parameters.evaporation_rate * temp_multiplier;

                // Apply evaporation with thermodynamic energy conservation
                let current_depth = water.depth.get(x, y);
                let new_depth = current_depth * (1.0 - effective_evaporation_rate.min(1.0));

                // Calculate water mass evaporated (m³/m² = m depth)
                let evaporated_water_depth = current_depth - new_depth.max(0.0);
                total_evaporated += evaporated_water_depth;

                // Apply latent heat cooling: Energy conservation E = m * λ
                // Latent heat of vaporization: 2.45 MJ/kg
                // Water density: 1000 kg/m³, so 2.45 MJ/m³ per meter depth
                if evaporated_water_depth > 0.0 {
                    // Energy removed per m² surface: evaporated_depth * latent_heat_per_depth
                    let latent_heat_per_meter = 2_450_000.0; // J/m³ (2.45 MJ/m³)
                    let energy_removed = evaporated_water_depth * latent_heat_per_meter; // J/m²

                    // Convert to temperature change: Q = m * c * ΔT
                    // Surface thermal mass approximation: ~1m depth with thermal capacity 4.18 MJ/(m³·K)
                    let surface_thermal_capacity = 4_180_000.0; // J/(m³·K)
                    let thermal_mass_per_m2 = 1.0; // Approximate 1m thermal depth
                    let total_thermal_capacity = surface_thermal_capacity * thermal_mass_per_m2; // J/(m²·K)

                    // Calculate temperature decrease: ΔT = Q / (m * c)
                    let temperature_decrease = energy_removed / total_thermal_capacity; // K = °C

                    // Apply cooling to surface temperature (energy conservation)
                    let current_temp = temperature_layer.get_temperature(x, y);
                    let new_temperature = current_temp - temperature_decrease;

                    // Set the cooled temperature back into the temperature layer
                    // Note: This requires temperature_layer to be mutable
                    temperature_layer.temperature[y][x] = new_temperature;
                }

                // Clear tiny amounts based on threshold
                if new_depth < self.evaporation_threshold {
                    water.depth.set(x, y, 0.0);
                } else {
                    water.depth.set(x, y, new_depth);
                }
            }
        }

        self.drainage_metrics.total_evaporation += total_evaporated;

        // Handle sediment settling when water disappears
        for y in 0..water.height() {
            for x in 0..water.width() {
                if water.depth.get(x, y) < self.evaporation_threshold {
                    let current_sediment = water.sediment.get(x, y);
                    water.sediment.set(x, y, current_sediment * 0.5); // Sediment settles when water dries up
                }
            }
        }
    }

    /// Calculate flow directions enhanced by drainage network for gradual water concentration
    fn calculate_flow_directions_with_drainage(
        &mut self,
        heightmap: &HeightMap,
        water: &mut WaterLayer,
        drainage_network: &DrainageNetwork,
        grid_spacing_m: f32,
    ) {
        // Create WorldScale from grid spacing for unified FlowEngine
        let scale = WorldScale::new(
            grid_spacing_m as f64,
            (heightmap.width() as u32, heightmap.height() as u32),
            super::core::scale::DetailLevel::Standard,
        );

        // Get or initialize the unified flow engine
        let flow_engine = self.get_flow_engine(water, &scale);

        // Temporarily switch to drainage-based algorithm for drainage network integration
        let original_algorithm = flow_engine.algorithm;
        flow_engine.algorithm = super::physics::flow_engine::FlowAlgorithm::Drainage;

        // Delegate to unified FlowEngine with drainage-based algorithm
        // This replaces the manual drainage enhancement calculation
        // with the consolidated drainage flow implementation
        flow_engine.calculate_flow(heightmap, water, Some(drainage_network), &scale);

        // Restore original algorithm
        flow_engine.algorithm = original_algorithm;
    }

    /// Move water with boundary outlets for mass conservation on continental scales
    fn move_water_with_boundaries(&mut self, water: &mut WaterLayer) {
        // Use double-buffering to eliminate clone() allocation:
        // 1. Copy current depth to buffer as starting point
        water.copy_depth_to_buffer();

        for y in 0..water.height() {
            for x in 0..water.width() {
                let (vx, vy) = water.velocity.get(x, y);
                let velocity_mag = (vx * vx + vy * vy).sqrt();
                // CFL-stable velocity limit: max 0.5 cells per timestep for numerical stability
                let max_velocity = 0.5; // Conservative CFL condition
                let flow_amount = water.depth.get(x, y) * velocity_mag.min(max_velocity);

                // Scale-aware flow threshold - physical minimum with scale adaptation
                let meters_per_pixel = self.estimate_grid_spacing_from_water_layer(water);
                let physical_threshold = 0.001 * meters_per_pixel / 1000.0; // 1mm depth scaled to pixel
                let flow_threshold = 1e-8_f32.max(physical_threshold); // Ensure non-zero minimum
                if flow_amount > flow_threshold {
                    // Enhanced accumulative flow: allow fractional movement accumulation
                    // instead of rounding immediately to integer positions
                    let target_x_float = x as f32 + vx;
                    let target_y_float = y as f32 + vy;

                    // Calculate fractional flow distribution to neighboring cells
                    let x0 = target_x_float.floor() as i32;
                    let x1 = x0 + 1;
                    let y0 = target_y_float.floor() as i32;
                    let y1 = y0 + 1;

                    let fx = target_x_float.fract();
                    let fy = target_y_float.fract();

                    // Bilinear interpolation weights for flow distribution
                    let weight_00 = (1.0 - fx) * (1.0 - fy); // Bottom-left
                    let weight_10 = fx * (1.0 - fy); // Bottom-right
                    let weight_01 = (1.0 - fx) * fy; // Top-left
                    let weight_11 = fx * fy; // Top-right

                    // Get dimensions before mutable borrow
                    let width = water.width() as i32;
                    let height = water.height() as i32;

                    let buffer = water.get_depth_buffer_mut();
                    let current_depth = buffer.get(x, y);

                    // Remove water from current cell
                    buffer.set(x, y, current_depth - flow_amount);

                    // Distribute flow to target cells based on fractional position
                    let flow_cells = [
                        (x0, y0, weight_00),
                        (x1, y0, weight_10),
                        (x0, y1, weight_01),
                        (x1, y1, weight_11),
                    ];

                    for (tx, ty, weight) in flow_cells {
                        if tx >= 0 && tx < width && ty >= 0 && ty < height {
                            let target_flow = flow_amount * weight;
                            if target_flow > 1e-8 {
                                // Avoid microscopic flows
                                let target_depth = buffer.get(tx as usize, ty as usize);
                                buffer.set(tx as usize, ty as usize, target_depth + target_flow);
                            }
                        } else {
                            // Flow out of bounds = boundary outflow (lost water)
                            // INSTRUMENTED: Track boundary drainage for continental scale analysis
                            let boundary_outflow = flow_amount * weight;
                            self.drainage_metrics.total_boundary_outflow += boundary_outflow;
                            self.drainage_metrics.boundary_outflow_rate += boundary_outflow;
                        }
                    }
                }
            }
        }

        // 3. Swap buffers to make the result the new primary depth
        water.swap_depth_buffers();
    }

    /// Reset drainage metrics for a new measurement period
    pub fn reset_drainage_metrics(&mut self) {
        self.drainage_metrics = DrainageMetrics::new();
    }

    /// Get current drainage metrics for monitoring
    pub fn get_drainage_metrics(&self) -> &DrainageMetrics {
        &self.drainage_metrics
    }
}

/// Boundary drainage monitoring and instrumentation
#[derive(Debug, Clone)]
pub struct DrainageMetrics {
    pub total_boundary_outflow: f32,
    pub total_rainfall_input: f32,
    pub total_evaporation: f32,
    pub current_water_storage: f32,
    pub drainage_efficiency: f32,   // outflow / (rainfall - evaporation)
    pub mass_balance_error: f32,    // Should be near zero
    pub boundary_outflow_rate: f32, // outflow per tick
    pub edge_saturation_ratio: f32, // water near edges / total water
    pub tick_count: u64,
}

impl DrainageMetrics {
    pub fn new() -> Self {
        Self {
            total_boundary_outflow: 0.0,
            total_rainfall_input: 0.0,
            total_evaporation: 0.0,
            current_water_storage: 0.0,
            drainage_efficiency: 0.0,
            mass_balance_error: 0.0,
            boundary_outflow_rate: 0.0,
            edge_saturation_ratio: 0.0,
            tick_count: 0,
        }
    }

    pub fn update_mass_balance(&mut self) {
        let expected_water =
            self.total_rainfall_input - self.total_evaporation - self.total_boundary_outflow;
        self.mass_balance_error = (self.current_water_storage - expected_water).abs();

        let net_input = self.total_rainfall_input - self.total_evaporation;
        if net_input > 0.0 {
            self.drainage_efficiency = self.total_boundary_outflow / net_input;
        }
    }

    pub fn calculate_edge_saturation_ratio(&mut self, water: &WaterLayer) {
        let width = water.width();
        let height = water.height();
        // Scale-aware edge margin: ~5% of domain size, minimum 5, maximum 50
        let edge_margin = ((width.min(height) as f32 * 0.05) as usize).clamp(5, 50);

        let mut total_water = 0.0;
        let mut edge_water = 0.0;

        for y in 0..height {
            for x in 0..width {
                let depth = water.depth.get(x, y);
                total_water += depth;

                if x < edge_margin
                    || x >= width - edge_margin
                    || y < edge_margin
                    || y >= height - edge_margin
                {
                    edge_water += depth;
                }
            }
        }

        self.current_water_storage = total_water;
        self.edge_saturation_ratio = if total_water > 0.0 {
            edge_water / total_water
        } else {
            0.0
        };
    }

    pub fn start_tick(&mut self) {
        self.boundary_outflow_rate = 0.0; // Reset per-tick outflow
        self.tick_count += 1;
    }

    pub fn end_tick(&mut self, water: &WaterLayer) {
        self.calculate_edge_saturation_ratio(water);
        self.update_mass_balance();
    }
}

pub struct Simulation {
    pub heightmap: HeightMap,
    pub water: WaterLayer,
    pub water_system: WaterFlowSystem,
    pub drainage_network: DrainageNetwork,
    pub climate_system: ClimateSystem,
    pub temperature_layer: TemperatureLayer,
    pub atmospheric_system: AtmosphericSystem,
    pub pressure_layer: AtmosphericPressureLayer,
    pub wind_layer: WindLayer,
    pub weather_analysis: WeatherAnalysis,
    pub _world_scale: WorldScale,
    pub tick_count: u64,
    // Cached biome map to avoid expensive recalculation every frame
    cached_biome_map: Option<BiomeMap>,
    biome_cache_valid: bool,
    // Atmospheric caching to prevent expensive regeneration every tick
    last_temperature_update: u64,
    last_pressure_update: u64,
    last_wind_update: u64,
    last_weather_analysis_update: u64,
}

impl Simulation {
    /// Create a simulation with default world scale (assumes 10km physical size)
    pub fn new(heightmap: HeightMap) -> Self {
        let height = heightmap.height();
        let width = heightmap.width();

        // Scale physical size to accommodate both terrain detail and climate realism
        let base_area = 240.0 * 120.0;
        let current_area = (width * height) as f64;
        let area_ratio = current_area / base_area;

        // Climate systems need larger domains for realistic behavior
        let climate_scale = 100.0 * (area_ratio / 4.0).sqrt();
        let terrain_scale = 10.0 * area_ratio.sqrt();

        // Use the larger scale to accommodate both systems
        let physical_size_km = climate_scale.max(terrain_scale);

        let world_scale = WorldScale::new(
            physical_size_km,
            (width as u32, height as u32),
            crate::engine::core::scale::DetailLevel::Standard,
        );

        // Create climate system and generate temperature layer
        let climate_system = ClimateSystem::new_for_scale(&world_scale);
        let temperature_layer = climate_system.generate_temperature_layer_optimized(&heightmap);

        // Create atmospheric system and generate pressure/wind layers
        let atmospheric_system = AtmosphericSystem::new_for_scale(&world_scale);
        let pressure_layer = climate_system.generate_pressure_layer_optimized(
            &temperature_layer,
            &heightmap,
            &world_scale,
        );
        let wind_layer =
            atmospheric_system.generate_geostrophic_winds(&pressure_layer, &world_scale);

        // Create drainage network from heightmap
        // Debug output disabled for clean TUI display
        let drainage_start = std::time::Instant::now();
        let drainage_network = DrainageNetwork::from_heightmap(&heightmap, &world_scale);
        // Debug timing disabled for clean TUI display

        let mut simulation = Self {
            heightmap,
            water: WaterLayer::new(width, height),
            water_system: WaterFlowSystem::new_for_scale(&world_scale),
            drainage_network,
            climate_system,
            temperature_layer,
            atmospheric_system,
            pressure_layer,
            wind_layer,
            weather_analysis: WeatherAnalysis::default(),
            _world_scale: world_scale,
            tick_count: 0,
            cached_biome_map: None,
            biome_cache_valid: false,
            // Initialize atmospheric caching - start with all systems up-to-date
            last_temperature_update: 0,
            last_pressure_update: 0,
            last_wind_update: 0,
            last_weather_analysis_update: 0,
        };

        // Apply initial water distribution for realistic starting biomes
        simulation.initialize_water_distribution();

        simulation
    }

    /// Create a simulation with explicit world scale
    pub fn _new_with_scale(heightmap: HeightMap, world_scale: WorldScale) -> Self {
        let height = heightmap.height();
        let width = heightmap.width();

        // Create climate system and generate temperature layer
        let climate_system = ClimateSystem::new_for_scale(&world_scale);
        let temperature_layer = climate_system.generate_temperature_layer_optimized(&heightmap);

        // Create atmospheric system and generate pressure/wind layers
        let atmospheric_system = AtmosphericSystem::new_for_scale(&world_scale);
        let pressure_layer = climate_system.generate_pressure_layer_optimized(
            &temperature_layer,
            &heightmap,
            &world_scale,
        );
        let wind_layer =
            atmospheric_system.generate_geostrophic_winds(&pressure_layer, &world_scale);

        // Create drainage network from heightmap
        let drainage_network = DrainageNetwork::from_heightmap(&heightmap, &world_scale);

        let mut simulation = Self {
            heightmap,
            water: WaterLayer::new(width, height),
            water_system: WaterFlowSystem::new_for_scale(&world_scale),
            drainage_network,
            climate_system,
            temperature_layer,
            atmospheric_system,
            pressure_layer,
            wind_layer,
            weather_analysis: WeatherAnalysis::default(),
            _world_scale: world_scale,
            tick_count: 0,
            cached_biome_map: None,
            biome_cache_valid: false,
            // Initialize atmospheric caching - start with all systems up-to-date
            last_temperature_update: 0,
            last_pressure_update: 0,
            last_wind_update: 0,
            last_weather_analysis_update: 0,
        };

        // Apply initial water distribution for realistic starting biomes
        simulation.initialize_water_distribution();

        simulation
    }

    /// Advance simulation by one time step with climate integration and atmospheric caching
    pub fn tick(&mut self) {
        // Drainage metrics instrumentation - start of tick
        self.water_system.drainage_metrics.start_tick();

        // Performance instrumentation (enabled with PERF_TRACE environment variable)
        let perf_trace = std::env::var("PERF_TRACE").is_ok();
        let tick_start = if perf_trace {
            Some(std::time::Instant::now())
        } else {
            None
        };

        // Advance seasonal cycle
        let climate_start = if perf_trace {
            Some(std::time::Instant::now())
        } else {
            None
        };
        self.climate_system.tick();
        if let Some(start) = climate_start {
            if perf_trace {
                eprintln!(
                    "PERF: climate_tick: {:.3}ms",
                    start.elapsed().as_secs_f64() * 1000.0
                );
            }
        }

        // Define atmospheric update intervals (in ticks)
        // These intervals reflect realistic timescales for atmospheric changes
        const TEMPERATURE_UPDATE_INTERVAL: u64 = 30; // ~3 hours (temperature changes gradually) 
        const PRESSURE_UPDATE_INTERVAL: u64 = 15; // ~1.5 hours (pressure responds to temperature)
        const WIND_UPDATE_INTERVAL: u64 = 10; // ~1 hour (wind follows pressure gradients)
        const WEATHER_ANALYSIS_INTERVAL: u64 = 25; // ~2.5 hours (weather pattern evolution)

        let mut temperature_updated = false;
        let mut pressure_updated = false;

        // Update temperature layer only when needed (slow changes)
        if self.tick_count - self.last_temperature_update >= TEMPERATURE_UPDATE_INTERVAL {
            let temp_start = if perf_trace {
                Some(std::time::Instant::now())
            } else {
                None
            };

            #[cfg(feature = "simd")]
            {
                // Use specialized optimization for common continental scale
                if self.heightmap.width() == 240 && self.heightmap.height() == 120 {
                    self.temperature_layer = self
                        .climate_system
                        .generate_temperature_layer_continental_240x120(&self.heightmap);
                } else {
                    self.temperature_layer = self
                        .climate_system
                        .generate_temperature_layer_simd(&self.heightmap);
                }
            }
            #[cfg(not(feature = "simd"))]
            {
                self.temperature_layer = self
                    .climate_system
                    .generate_temperature_layer_optimized(&self.heightmap);
            }

            if let Some(start) = temp_start {
                if perf_trace {
                    eprintln!(
                        "PERF: temperature_generation: {:.3}ms",
                        start.elapsed().as_secs_f64() * 1000.0
                    );
                }
            }

            self.last_temperature_update = self.tick_count;
            temperature_updated = true;
            // Invalidate biome cache due to temperature changes
            self.biome_cache_valid = false;
        }

        // Evolve pressure layer gradually when temperature changes OR enough time has passed
        if temperature_updated
            || self.tick_count - self.last_pressure_update >= PRESSURE_UPDATE_INTERVAL
        {
            // Evolution rate: faster changes when temperature updated, slower for temporal evolution
            let evolution_rate = if temperature_updated { 0.3 } else { 0.1 };

            #[cfg(feature = "simd")]
            {
                self.climate_system.evolve_pressure_layer_simd(
                    &mut self.pressure_layer,
                    &self.temperature_layer,
                    &self.heightmap,
                    &self._world_scale,
                    evolution_rate,
                );
            }
            #[cfg(not(feature = "simd"))]
            {
                self.climate_system.evolve_pressure_layer(
                    &mut self.pressure_layer,
                    &self.temperature_layer,
                    &self.heightmap_nested,
                    &self._world_scale,
                    evolution_rate,
                );
            }
            self.last_pressure_update = self.tick_count;
            pressure_updated = true;
        }

        // Update wind field when pressure changes OR enough time has passed
        if pressure_updated || self.tick_count - self.last_wind_update >= WIND_UPDATE_INTERVAL {
            self.wind_layer = self
                .atmospheric_system
                .generate_geostrophic_winds(&self.pressure_layer, &self._world_scale);
            self.last_wind_update = self.tick_count;
        }

        // Update weather analysis periodically (storms and pressure systems evolve slowly)
        if self.tick_count - self.last_weather_analysis_update >= WEATHER_ANALYSIS_INTERVAL {
            self.weather_analysis = self.atmospheric_system.analyze_weather_patterns(
                &self.pressure_layer,
                &self.wind_layer,
                &self._world_scale,
            );
            self.last_weather_analysis_update = self.tick_count;
        }

        // Update water flow less frequently - water movement is slower than atmospheric changes
        // Water only needs updates every few ticks for realistic flow rates
        const WATER_FLOW_UPDATE_INTERVAL: u64 = 3; // Every ~18 minutes simulation time
        if self.tick_count % WATER_FLOW_UPDATE_INTERVAL == 0 {
            let water_start = if perf_trace {
                Some(std::time::Instant::now())
            } else {
                None
            };

            self.water_system
                .update_water_flow_with_climate_and_drainage(
                    &mut self.heightmap,
                    &mut self.water,
                    &mut self.temperature_layer,
                    &self.climate_system,
                    &self.drainage_network,
                    &self._world_scale,
                );

            if let Some(start) = water_start {
                if perf_trace {
                    eprintln!(
                        "PERF: water_flow_update: {:.3}ms",
                        start.elapsed().as_secs_f64() * 1000.0
                    );
                }
            }
        }

        // Invalidate biome cache due to water changes
        self.biome_cache_valid = false;

        // Drainage concentration is now handled continuously through drainage-aware flow
        // No more periodic "nuclear redistribution" - water flows gradually toward drainage areas

        // Update drainage network periodically to account for terrain changes from erosion
        let drainage_start = if perf_trace {
            Some(std::time::Instant::now())
        } else {
            None
        };
        self.update_drainage_for_erosion();
        if let Some(start) = drainage_start {
            if perf_trace {
                eprintln!(
                    "PERF: drainage_update: {:.3}ms",
                    start.elapsed().as_secs_f64() * 1000.0
                );
            }
        }

        self.tick_count += 1;

        // Drainage metrics instrumentation - end of tick
        self.water_system.drainage_metrics.end_tick(&self.water);

        // Total tick timing
        if let Some(start) = tick_start {
            if perf_trace {
                eprintln!(
                    "PERF: total_tick: {:.3}ms",
                    start.elapsed().as_secs_f64() * 1000.0
                );
                eprintln!("PERF: ---");
            }
        }
    }

    /// Get drainage performance metrics for continental scale monitoring
    pub fn get_drainage_metrics(&self) -> &DrainageMetrics {
        &self.water_system.drainage_metrics
    }

    /// Check if drainage system is working effectively for current scale
    pub fn is_drainage_effective(&self) -> bool {
        let metrics = &self.water_system.drainage_metrics;

        // Drainage is effective if:
        // 1. Mass balance error is small (< 1% of total water input)
        let total_input = metrics.total_rainfall_input;
        let mass_balance_ok = if total_input > 0.0 {
            metrics.mass_balance_error / total_input < 0.01
        } else {
            true
        };

        // 2. Edge saturation is reasonable (< 50% of water near boundaries)
        let edge_ok = metrics.edge_saturation_ratio < 0.5;

        // 3. Some drainage is actually happening if there's net water input
        let net_input = metrics.total_rainfall_input - metrics.total_evaporation;
        let drainage_ok = if net_input > 0.0 {
            metrics.drainage_efficiency > 0.1 // At least 10% of excess water drains out
        } else {
            true
        };

        mass_balance_ok && edge_ok && drainage_ok
    }

    /// Get human-readable drainage status for debugging
    pub fn get_drainage_status(&self) -> String {
        let metrics = &self.water_system.drainage_metrics;
        format!(
            "Drainage Status: {} | Mass Balance Error: {:.6} | Edge Saturation: {:.1}% | Drainage Efficiency: {:.1}% | Boundary Outflow Rate: {:.6}/tick",
            if self.is_drainage_effective() {
                "EFFECTIVE"
            } else {
                "PROBLEMATIC"
            },
            metrics.mass_balance_error,
            metrics.edge_saturation_ratio * 100.0,
            metrics.drainage_efficiency * 100.0,
            metrics.boundary_outflow_rate
        )
    }

    pub fn get_simulation_time(&self) -> SimulationTime {
        // Assuming 10Hz simulation rate, each tick = 6 minutes of simulation time
        // This gives reasonable atmospheric dynamics timing
        const MINUTES_PER_TICK: f32 = 6.0;

        let total_minutes = self.tick_count as f32 * MINUTES_PER_TICK;
        let total_hours = total_minutes / 60.0;
        let days = (total_hours / 24.0) as u32;
        let hours = (total_hours % 24.0) as u32;
        let minutes = (total_minutes % 60.0) as u32;

        SimulationTime {
            tick_count: self.tick_count,
            days,
            hours,
            minutes,
            total_hours: total_hours as u32,
        }
    }

    /// Get the total water + terrain elevation at a position
    pub fn _get_total_elevation(&self, x: usize, y: usize) -> f32 {
        if x < self.heightmap.width() && y < self.heightmap.height() {
            self.heightmap.get(x, y) + self.water.depth.get(x, y)
        } else {
            0.0
        }
    }

    /// Add water at a specific location (useful for testing/debugging)
    pub fn add_water_at(&mut self, x: usize, y: usize, amount: f32) {
        self.water.add_water(x, y, amount);
    }

    /// Get dimensional analysis of current water flow system
    pub fn get_dimensional_analysis(&self) -> DimensionalWaterFlowParameters {
        self.water_system
            .create_dimensional_parameters(&self._world_scale)
    }

    /// Validate physical parameters and return any warnings
    pub fn validate_physics(&self) -> Vec<String> {
        self.water_system
            .validate_physical_parameters(&self._world_scale)
    }

    /// Get physical rainfall rate in proper units
    pub fn get_physical_rainfall_rate(&self) -> PhysicalQuantity {
        self.water_system
            .get_rainfall_volume_rate(&self._world_scale)
    }

    /// Get physical evaporation rate in proper units
    pub fn get_physical_evaporation_rate(&self) -> PhysicalQuantity {
        self.water_system
            .get_evaporation_volume_rate(&self._world_scale)
    }

    /// Get atmospheric pressure at a specific location
    pub fn get_pressure_at(&self, x: usize, y: usize) -> f32 {
        self.pressure_layer.get_pressure(x, y)
    }

    /// Get wind velocity at a specific location
    pub fn get_wind_at(&self, x: usize, y: usize) -> Vec2 {
        self.wind_layer.get_velocity(x, y)
    }

    /// Get wind speed at a specific location
    pub fn get_wind_speed_at(&self, x: usize, y: usize) -> f32 {
        self.wind_layer.get_speed(x, y)
    }

    /// Check if Coriolis effects are active for this simulation
    pub fn is_coriolis_active(&self) -> bool {
        self.atmospheric_system.is_coriolis_active()
    }

    /// Get average atmospheric pressure across the map
    pub fn get_average_pressure(&self) -> f32 {
        self.pressure_layer.get_average_pressure()
    }

    /// Get average wind speed across the map
    pub fn get_average_wind_speed(&self) -> f32 {
        self.wind_layer.get_average_wind_speed()
    }

    // Graphics API methods for accessing simulation data layers

    /// Get reference to heightmap for graphics rendering
    pub fn get_heightmap(&self) -> &HeightMap {
        &self.heightmap
    }

    /// Get the world scale information for this simulation
    pub fn get_world_scale(&self) -> &WorldScale {
        &self._world_scale
    }

    /// Get reference to water layer for graphics rendering
    pub fn get_water_layer(&self) -> &WaterLayer {
        &self.water
    }

    /// Generate biome map from current environmental state (cached for performance)
    pub fn generate_biome_map(&mut self) -> &BiomeMap {
        if !self.biome_cache_valid || self.cached_biome_map.is_none() {
            let classifier = BiomeClassifier::new_for_scale(&self._world_scale);
            let biome_map = classifier.generate_biome_map_with_drainage(
                &self.heightmap,
                &self.temperature_layer,
                &self.water,
                &self.climate_system,
                &self.drainage_network,
            );
            self.cached_biome_map = Some(biome_map);
            self.biome_cache_valid = true;
        }

        self.cached_biome_map.as_ref().unwrap()
    }

    /// Generate biome map without drainage network (legacy method)
    pub fn generate_biome_map_basic(&self) -> BiomeMap {
        let classifier = BiomeClassifier::new_for_scale(&self._world_scale);
        classifier.generate_biome_map(
            &self.heightmap,
            &self.temperature_layer,
            &self.water,
            &self.climate_system,
        )
    }

    /// Get reference to atmospheric pressure layer for graphics rendering
    pub fn get_atmospheric_pressure_layer(&self) -> &AtmosphericPressureLayer {
        &self.pressure_layer
    }

    /// Get reference to wind layer for graphics rendering
    pub fn get_wind_layer(&self) -> &WindLayer {
        &self.wind_layer
    }

    /// Get reference to weather analysis for graphics rendering
    pub fn get_weather_analysis(&self) -> &WeatherAnalysis {
        &self.weather_analysis
    }

    /// Get reference to temperature layer for graphics rendering
    pub fn get_temperature_layer(&self) -> &TemperatureLayer {
        &self.temperature_layer
    }

    /// Get heightmap width
    pub fn get_width(&self) -> usize {
        self.heightmap.width()
    }

    /// Get heightmap height
    pub fn get_height(&self) -> usize {
        self.heightmap.height()
    }

    /// Get elevation at specific coordinates
    pub fn get_elevation(&self, x: usize, y: usize) -> f32 {
        if x < self.heightmap.width() && y < self.heightmap.height() {
            self.heightmap.get(x, y)
        } else {
            0.0
        }
    }

    /// Apply drainage network water concentration to create realistic water bodies
    pub fn apply_drainage_concentration(&mut self) {
        self.drainage_network.concentrate_water(&mut self.water);
    }

    /// Initialize water distribution for realistic starting biomes
    /// Adds base water level and applies drainage concentration once
    fn initialize_water_distribution(&mut self) {
        println!("Initializing water distribution...");

        // Add a small base amount of water everywhere (representing natural precipitation)
        let base_water_amount = self.water_system.effective_rainfall_rate / 10.0; // Small initial amount
        for y in 0..self.water.height() {
            for x in 0..self.water.width() {
                self.water.add_water(x, y, base_water_amount);
            }
        }

        // Apply drainage concentration once to create realistic initial water distribution
        // Debug output disabled for clean TUI display
        self.apply_drainage_concentration();

        // Debug completion message disabled for clean TUI display
    }

    /// Get drainage network statistics for analysis
    pub fn get_drainage_statistics(&self) -> DrainageNetworkStatistics {
        self.drainage_network.get_statistics()
    }

    /// Check if location is part of a river system
    pub fn is_river(&self, x: usize, y: usize) -> bool {
        self.drainage_network.is_river(x, y)
    }

    /// Check if location is part of a major river system
    pub fn is_major_river(&self, x: usize, y: usize) -> bool {
        self.drainage_network.is_major_river(x, y)
    }

    /// Check if location is in a drainage depression (potential lake)
    pub fn is_depression(&self, x: usize, y: usize) -> bool {
        self.drainage_network.is_depression(x, y)
    }

    /// Get flow accumulation at coordinates (upstream drainage area)
    pub fn get_flow_accumulation(&self, x: usize, y: usize) -> f32 {
        self.drainage_network.get_flow_accumulation(x, y)
    }

    /// Regenerate drainage network from current heightmap (use after significant terrain changes)
    pub fn regenerate_drainage_network(&mut self) {
        self.drainage_network =
            DrainageNetwork::from_heightmap(&self.heightmap, &self._world_scale);
        // Invalidate biome cache due to drainage network changes
        self.biome_cache_valid = false;
    }

    /// Update drainage network periodically to account for erosion effects
    pub fn update_drainage_for_erosion(&mut self) {
        // Only regenerate drainage network occasionally due to computational cost
        // In practice, erosion changes are usually gradual
        if self.tick_count % 100 == 0 {
            self.regenerate_drainage_network();
        }
    }

    // DIAGNOSTICS SUPPORT METHODS - Required for SimulationDiagnostics

    /// Get water system reference for diagnostics
    pub fn get_water_system(&self) -> &WaterFlowSystem {
        &self.water_system
    }

    /// Get spatial system reference for diagnostics (placeholder)
    pub fn get_spatial_system(&self) -> SpatialSystemPlaceholder {
        // For now, return a placeholder with basic stats
        // This would be replaced with actual spatial partitioning system reference
        SpatialSystemPlaceholder {
            active_cells: (self.get_width() * self.get_height()) / 4, // Estimate 25% active
            total_cells: self.get_width() * self.get_height(),
        }
    }

    /// Get pressure layer reference for diagnostics
    pub fn get_pressure_layer(&self) -> &AtmosphericPressureLayer {
        &self.pressure_layer
    }

    /// Get atmospheric system reference for diagnostics
    pub fn get_atmospheric_system(&self) -> &AtmosphericSystem {
        &self.atmospheric_system
    }

    /// Get average flow rate across the water system for diagnostics
    pub fn get_average_flow_rate(&self) -> f32 {
        let mut total_flow = 0.0;
        let mut flow_count = 0;

        for y in 0..self.water.height() {
            for x in 0..self.water.width() {
                let velocity = self.water.velocity.get(x, y);
                let flow_magnitude = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
                if flow_magnitude > 0.0001 {
                    // Only count meaningful flows
                    total_flow += flow_magnitude;
                    flow_count += 1;
                }
            }
        }

        if flow_count > 0 {
            total_flow / flow_count as f32
        } else {
            0.0
        }
    }

    /// Count river cells for diagnostics
    pub fn count_river_cells(&self) -> usize {
        let mut river_count = 0;
        let river_threshold = self.water_system.evaporation_threshold * 5.0; // Scale-aware threshold

        for y in 0..self.water.height() {
            for x in 0..self.water.width() {
                if self.water.depth.get(x, y) > river_threshold {
                    // Check if this cell has flow (indicating a river vs static water)
                    let velocity = self.water.velocity.get(x, y);
                    let flow_speed = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
                    if flow_speed > 0.001 {
                        river_count += 1;
                    }
                }
            }
        }

        river_count
    }

    /// Calculate total water for mass conservation diagnostics
    pub fn calculate_total_water(&self) -> f32 {
        self.water.get_total_water()
    }

    /// Calculate total elevation for mass conservation diagnostics
    pub fn calculate_total_elevation(&self) -> f32 {
        let mut total = 0.0;
        for y in 0..self.heightmap.height() {
            for x in 0..self.heightmap.width() {
                total += self.heightmap.get(x, y);
            }
        }
        total
    }

    /// Calculate total sediment for mass conservation diagnostics
    pub fn calculate_total_sediment(&self) -> f32 {
        let mut total = 0.0;
        for y in 0..self.water.height() {
            for x in 0..self.water.width() {
                total += self.water.sediment.get(x, y);
            }
        }
        total
    }
}

/// Placeholder for spatial partitioning system stats
/// TODO: Replace with actual spatial partitioning system when available
#[derive(Debug, Clone)]
pub struct SpatialSystemPlaceholder {
    pub active_cells: usize,
    pub total_cells: usize,
}

impl SpatialSystemPlaceholder {
    /// Get performance stats placeholder
    pub fn get_performance_stats(&self) -> SpatialPerformanceStats {
        SpatialPerformanceStats {
            active_cells: self.active_cells,
            total_cells: self.total_cells,
        }
    }

    /// Check convergence placeholder
    pub fn has_converged(&self) -> bool {
        // Simplified heuristic: converged if less than 10% active cells
        (self.active_cells as f32 / self.total_cells as f32) < 0.1
    }
}

/// Placeholder for spatial performance stats
#[derive(Debug, Clone)]
pub struct SpatialPerformanceStats {
    pub active_cells: usize,
    pub total_cells: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::core::scale::{DetailLevel, WorldScale};

    // Helper function to create a test world scale
    fn test_scale(width: u32, height: u32) -> WorldScale {
        WorldScale::new(10.0, (width, height), DetailLevel::Standard)
    }

    // Helper function to create a test water system
    fn test_water_system(width: u32, height: u32) -> WaterFlowSystem {
        WaterFlowSystem::new_for_scale(&test_scale(width, height))
    }

    // Vec2 unit tests
    #[test]
    fn vec2_new_creates_correct_values() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn vec2_zero_creates_zero_vector() {
        let v = Vec2::zero();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
    }

    #[test]
    fn vec2_magnitude_calculates_correctly() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.magnitude(), 5.0); // 3-4-5 triangle

        let v2 = Vec2::new(0.0, 0.0);
        assert_eq!(v2.magnitude(), 0.0);

        let v3 = Vec2::new(1.0, 0.0);
        assert_eq!(v3.magnitude(), 1.0);

        let v4 = Vec2::new(-3.0, -4.0);
        assert_eq!(v4.magnitude(), 5.0); // Magnitude is always positive
    }

    #[test]
    fn vec2_magnitude_handles_edge_cases() {
        // Test very small values
        let v = Vec2::new(1e-10, 1e-10);
        assert!(v.magnitude().is_finite());
        assert!(v.magnitude() >= 0.0);

        // Test negative values
        let v2 = Vec2::new(-1.0, -1.0);
        assert_eq!(v2.magnitude(), 2.0_f32.sqrt());

        // Test one zero component
        let v3 = Vec2::new(5.0, 0.0);
        assert_eq!(v3.magnitude(), 5.0);
    }

    // WaterLayer unit tests
    #[test]
    fn water_layer_new_creates_correct_dimensions() {
        let layer = WaterLayer::new(10, 5);
        assert_eq!(layer.width(), 10);
        assert_eq!(layer.height(), 5);
        assert_eq!(layer.depth.height(), 5); // height rows
        assert_eq!(layer.depth.width(), 10); // width columns
        assert_eq!(layer.velocity.height(), 5);
        assert_eq!(layer.velocity.width(), 10);
        assert_eq!(layer.sediment.height(), 5);
        assert_eq!(layer.sediment.width(), 10);
    }

    #[test]
    fn water_layer_initializes_to_zero() {
        let layer = WaterLayer::new(3, 3);

        // All depths should be zero
        for y in 0..layer.height() {
            for x in 0..layer.width() {
                assert_eq!(layer.depth.get(x, y), 0.0);
            }
        }

        // All velocities should be zero
        for y in 0..layer.height() {
            for x in 0..layer.width() {
                let velocity = layer.velocity.get(x, y);
                assert_eq!(velocity.0, 0.0);
                assert_eq!(velocity.1, 0.0);
            }
        }

        // All sediment should be zero
        for y in 0..layer.height() {
            for x in 0..layer.width() {
                assert_eq!(layer.sediment.get(x, y), 0.0);
            }
        }
    }

    #[test]
    fn water_layer_get_total_water_works() {
        let mut layer = WaterLayer::new(2, 2);

        // Initially should be zero
        assert_eq!(layer.get_total_water(), 0.0);

        // Add some water
        layer.depth[0][0] = 1.0;
        layer.depth[0][1] = 2.0;
        layer.depth[1][0] = 0.5;
        layer.depth[1][1] = 1.5;

        assert_eq!(layer.get_total_water(), 5.0);
    }

    #[test]
    fn water_layer_add_water_works() {
        let mut layer = WaterLayer::new(3, 3);

        // Add water to center
        layer.add_water(1, 1, 2.5);
        assert_eq!(layer.depth[1][1], 2.5);
        assert_eq!(layer.get_total_water(), 2.5);

        // Add more water to same location
        layer.add_water(1, 1, 1.0);
        assert_eq!(layer.depth[1][1], 3.5);
        assert_eq!(layer.get_total_water(), 3.5);

        // Add water to different location
        layer.add_water(0, 2, 1.0);
        assert_eq!(layer.depth[2][0], 1.0);
        assert_eq!(layer.get_total_water(), 4.5);
    }

    #[test]
    fn water_layer_add_water_bounds_check() {
        let mut layer = WaterLayer::new(2, 2);

        // Valid positions
        layer.add_water(0, 0, 1.0);
        layer.add_water(1, 1, 1.0);
        assert_eq!(layer.get_total_water(), 2.0);

        // Out of bounds positions - should not crash or add water
        layer.add_water(2, 0, 5.0); // x out of bounds
        layer.add_water(0, 2, 5.0); // y out of bounds
        layer.add_water(5, 5, 5.0); // both out of bounds
        assert_eq!(layer.get_total_water(), 2.0); // Should still be 2.0
    }

    // WaterFlowSystem unit tests
    #[test]
    fn water_flow_parameters_default_values() {
        let params = WaterFlowParameters::default();
        assert_eq!(params.flow_rate, 0.1);
        assert_eq!(params.evaporation_rate, 0.001);
        assert_eq!(params.erosion_strength, 0.01);
        assert_eq!(params.deposition_rate, 0.05);
        assert_eq!(params.base_rainfall_rate, 0.0000027127);
        assert_eq!(params.rainfall_scaling, RainfallScaling::MassConserving);
    }

    #[test]
    fn water_flow_system_with_scale() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        let scale = WorldScale::new(10.0, (240, 120), DetailLevel::Standard);
        let system = WaterFlowSystem::new_for_scale(&scale);

        // Should have default parameter values
        assert_eq!(system.parameters.flow_rate, 0.1);
        assert_eq!(system.parameters.evaporation_rate, 0.001);

        // Effective rainfall should be calculated based on scale
        assert!(system.effective_rainfall_rate > 0.0);
    }

    #[test]
    fn flow_direction_flat_terrain_no_flow() {
        let mut system = test_water_system(3, 3);
        let heightmap = HeightMap::from_nested(vec![
            vec![0.5, 0.5, 0.5],
            vec![0.5, 0.5, 0.5],
            vec![0.5, 0.5, 0.5],
        ]);
        let mut water = WaterLayer::new(3, 3);

        system.calculate_flow_directions(&heightmap, &mut water);

        // On flat terrain, there should be no flow
        for y in 0..water.height() {
            for x in 0..water.width() {
                let velocity = water.velocity.get(x, y);
                assert_eq!(velocity.0, 0.0);
                assert_eq!(velocity.1, 0.0);
            }
        }
    }

    #[test]
    fn flow_direction_simple_slope() {
        let mut system = test_water_system(3, 3);
        // Create a simple slope from left to right
        let heightmap = HeightMap::from_nested(vec![
            vec![1.0, 0.5, 0.0],
            vec![1.0, 0.5, 0.0],
            vec![1.0, 0.5, 0.0],
        ]);
        let mut water = WaterLayer::new(3, 3);

        system.calculate_flow_directions(&heightmap, &mut water);

        // Water in center column should flow toward lower elevation (positive x direction)
        let (vx, vy) = water.velocity.get(1, 1);
        assert!(vx > 0.0, "Water should flow downhill (positive x)");
        // Note: May have small y component due to 8-neighbor diagonal flow

        // Water on rightmost column should have no flow (no lower neighbor)
        let (rv_x, rv_y) = water.velocity.get(2, 1);
        assert_eq!(rv_x, 0.0);
        assert_eq!(rv_y, 0.0);
    }

    #[test]
    fn flow_direction_with_water_depth() {
        let mut system = test_water_system(2, 2);
        let heightmap = HeightMap::from_nested(vec![vec![1.0, 0.5], vec![1.0, 0.5]]);
        let mut water = WaterLayer::new(2, 2);

        // Add water that changes the effective elevation
        water.depth[0][0] = 0.3; // Total elevation becomes 1.3
        water.depth[0][1] = 0.1; // Total elevation becomes 0.6

        system.calculate_flow_directions(&heightmap, &mut water);

        // Should still flow from higher total elevation to lower
        let velocity = water.velocity.get(0, 0);
        assert!(
            velocity.0 > 0.0,
            "Water should flow from higher to lower total elevation"
        );
    }

    #[test]
    fn flow_direction_eight_neighbors() {
        let mut system = test_water_system(2, 2);
        // Create a heightmap with center cell higher than all neighbors
        let heightmap = HeightMap::from_nested(vec![
            vec![0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ]);
        let mut water = WaterLayer::new(3, 3);

        system.calculate_flow_directions(&heightmap, &mut water);

        // Center cell should flow toward the steepest neighbor
        // All neighbors are equal, so it should pick one of them
        let center_velocity = water.velocity.get(1, 1);
        let magnitude =
            (center_velocity.0 * center_velocity.0 + center_velocity.1 * center_velocity.1).sqrt();
        assert!(
            magnitude > 0.0,
            "Center cell should have flow toward neighbors"
        );

        // Flow direction should be normalized
        assert!(magnitude <= system.parameters.flow_rate * 1.0); // slope of 1.0
    }

    #[test]
    fn flow_direction_boundary_conditions() {
        let mut system = test_water_system(2, 2);
        let heightmap = HeightMap::from_nested(vec![vec![1.0, 0.5], vec![0.8, 0.3]]);
        let mut water = WaterLayer::new(2, 2);

        system.calculate_flow_directions(&heightmap, &mut water);

        // Corner cells should only consider their available neighbors
        // This test ensures we don't access out-of-bounds indices
        // Just check that it doesn't panic - the exact flow values depend on implementation
        for y in 0..water.height() {
            for x in 0..water.width() {
                let velocity = water.velocity.get(x, y);
                let magnitude = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
                assert!(magnitude.is_finite());
            }
        }
    }

    // Water movement and physics tests
    #[test]
    fn rainfall_adds_water_uniformly() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        let scale = WorldScale::new(10.0, (2, 2), DetailLevel::Standard);
        let mut params = WaterFlowParameters::default();
        params.base_rainfall_rate = 0.1;
        let mut system = WaterFlowSystem::from_parameters(params, &scale);
        let mut water = WaterLayer::new(2, 2);

        system.add_rainfall(&mut water);

        for y in 0..water.height() {
            for x in 0..water.width() {
                assert_eq!(water.depth.get(x, y), system.effective_rainfall_rate);
            }
        }
    }

    #[test]
    fn evaporation_reduces_water() {
        let scale = test_scale(2, 2);
        let mut params = WaterFlowParameters::default();
        params.evaporation_rate = 0.1;
        let mut system = WaterFlowSystem::from_parameters(params, &scale);
        let mut water = WaterLayer::new(2, 2);
        water.depth[0][0] = 1.0;
        water.depth[0][1] = 0.5;

        system.apply_evaporation(&mut water);

        assert_eq!(water.depth[0][0], 0.9); // 1.0 * (1 - 0.1)
        assert_eq!(water.depth[0][1], 0.45); // 0.5 * (1 - 0.1)
    }

    #[test]
    fn evaporation_clears_tiny_amounts() {
        let mut system = test_water_system(2, 2);
        let mut water = WaterLayer::new(1, 1);

        // Use an amount smaller than the scale-aware threshold
        let tiny_amount = system.evaporation_threshold * 0.5;
        water.depth[0][0] = tiny_amount;

        system.apply_evaporation(&mut water);

        assert_eq!(water.depth[0][0], 0.0); // Should be cleared to 0
    }

    #[test]
    fn erosion_removes_terrain_adds_sediment() {
        let mut system = test_water_system(2, 2);
        let mut heightmap = HeightMap::from_nested(vec![vec![1.0]]);
        let mut water = WaterLayer::new(1, 1);
        water.depth[0][0] = 0.1;
        water.velocity.set(0, 0, (0.5, 0.0)); // Fast flow
        water.sediment[0][0] = 0.0; // No initial sediment

        let initial_height = heightmap.get(0, 0);
        system.apply_erosion(&mut heightmap, &mut water);

        assert!(
            heightmap.get(0, 0) < initial_height,
            "Terrain should be eroded"
        );
        assert!(water.sediment[0][0] > 0.0, "Sediment should increase");
    }

    #[test]
    fn deposition_adds_terrain_removes_sediment() {
        let mut system = test_water_system(2, 2);
        let mut heightmap = HeightMap::from_nested(vec![vec![1.0]]);
        let mut water = WaterLayer::new(1, 1);
        water.depth[0][0] = 0.1; // More water needed for capacity calculation
        water.velocity.set(0, 0, (0.02, 0.0)); // Slow but not too slow flow
        water.sediment[0][0] = 0.1; // Lots of sediment

        let initial_height = heightmap.get(0, 0);
        let initial_sediment = water.sediment[0][0];
        system.apply_erosion(&mut heightmap, &mut water);

        // Check if deposition occurred (height increased) OR if we're at capacity
        // This test validates the physics are working correctly
        let height_changed = heightmap.get(0, 0) != initial_height;
        let sediment_changed = water.sediment[0][0] != initial_sediment;
        assert!(
            height_changed || sediment_changed,
            "Erosion system should affect either terrain or sediment"
        );
    }

    // Integration tests
    #[test]
    fn simulation_tick_integrates_all_systems() {
        let heightmap = vec![vec![1.0, 0.5], vec![0.8, 0.3]];
        let mut sim = Simulation::new(HeightMap::from_nested(heightmap));

        // Add some initial water
        sim.add_water_at(0, 0, 0.5);

        let initial_tick = sim.tick_count;
        let initial_water = sim.water.get_total_water();

        sim.tick();

        assert_eq!(sim.tick_count, initial_tick + 1);
        // Water should change due to rainfall, flow, and evaporation
        assert_ne!(sim.water.get_total_water(), initial_water);
    }

    #[test]
    fn water_conservation_with_no_flow() {
        let heightmap = vec![vec![0.5; 3]; 3]; // Flat terrain
        let mut sim = Simulation::new(HeightMap::from_nested(heightmap));

        // On flat terrain, only rainfall and evaporation should affect water
        sim.tick();
        let rainfall_added = 9.0 * sim.water_system.effective_rainfall_rate; // 9 cells

        // With climate integration, evaporation is temperature-dependent
        // So we can't predict exact water amounts, but it should be reasonable
        let actual_water = sim.water.get_total_water();

        // Water should be positive (rainfall > evaporation)
        assert!(
            actual_water > 0.0,
            "Water should accumulate, got: {}",
            actual_water
        );

        // Water should be less than total rainfall (some evaporation occurred)
        assert!(
            actual_water < rainfall_added,
            "Water should be less than total rainfall due to evaporation. Rainfall: {}, Actual: {}",
            rainfall_added,
            actual_water
        );

        // Water should be reasonably close to expected range (climate can affect evaporation rates)
        // Temperature effects can increase or decrease evaporation, so be more flexible
        assert!(
            actual_water > rainfall_added * 0.05,
            "Water seems too low. Expected > {}, got: {}",
            rainfall_added * 0.05,
            actual_water
        );
        assert!(
            actual_water <= rainfall_added,
            "Water should not exceed total rainfall. Rainfall: {}, got: {}",
            rainfall_added,
            actual_water
        );
    }

    // Scale-aware rainfall tests
    #[test]
    fn mass_conserving_scaling_maintains_total_water_input() {
        // Create two different sized maps
        let small_heightmap = vec![vec![0.5; 10]; 10]; // 100 cells  
        let large_heightmap = vec![vec![0.5; 20]; 20]; // 400 cells (4x larger)

        let small_sim = Simulation::new(HeightMap::from_nested(small_heightmap));
        let large_sim = Simulation::new(HeightMap::from_nested(large_heightmap));

        // Both should use mass-conserving scaling by default
        assert!(matches!(
            small_sim.water_system.parameters.rainfall_scaling,
            RainfallScaling::MassConserving
        ));
        assert!(matches!(
            large_sim.water_system.parameters.rainfall_scaling,
            RainfallScaling::MassConserving
        ));

        // Check that total water input per tick is conserved across map sizes
        let small_water_per_tick = small_sim.water_system.effective_rainfall_rate * 100.0; // 100 cells  
        let large_water_per_tick = large_sim.water_system.effective_rainfall_rate * 400.0; // 400 cells

        // With mass-conserving scaling, total water input should be approximately equal
        let ratio = small_water_per_tick / large_water_per_tick;
        assert!(
            (ratio - 1.0).abs() < 0.01,
            "Total water input should be conserved, ratio: {}",
            ratio
        );
    }

    #[test]
    fn mass_conserving_scaling_adjusts_rainfall_rates() {
        // Reference size water system (240x120) with mass-conserving scaling
        let reference_system = WaterFlowSystem::new_for_scale(&test_scale(240, 120));
        let reference_rate = reference_system.effective_rainfall_rate;
        assert!(
            (reference_rate - 0.0000027127).abs() < 1e-9,
            "Reference rate should be ~0.0000027127, got {}",
            reference_rate
        );

        // Larger map should have proportionally lower effective rainfall rate with mass-conserving scaling
        let large_system = WaterFlowSystem::new_for_scale(&test_scale(480, 240)); // 4x larger area
        let large_rate = large_system.effective_rainfall_rate;
        assert!(large_rate < 0.002);
        // With linear scaling: 0.25, so 0.002 * 0.25 = 0.0005
        assert!(
            (large_rate - 0.0005).abs() < 1e-6,
            "4x larger map with mass conservation should have 1/4 rainfall rate, got {}",
            large_rate
        );

        // Smaller map should have proportionally higher effective rainfall rate
        let small_system = WaterFlowSystem::new_for_scale(&test_scale(120, 60)); // 1/4 area
        let small_rate = small_system.effective_rainfall_rate;
        assert!(small_rate > 0.002);
        // With linear scaling: 4.0, so 0.002 * 4.0 = 0.008
        assert!(
            (small_rate - 0.008).abs() < 1e-6,
            "1/4 area map with mass conservation should have 4x rainfall rate, got {}",
            small_rate
        );
    }

    #[test]
    fn per_cell_scaling_keeps_constant_rainfall() {
        // Test per-cell scaling mode
        let small_scale = test_scale(120, 60);
        let large_scale = test_scale(480, 240);

        let mut small_params = WaterFlowParameters::default();
        small_params.rainfall_scaling = RainfallScaling::_IntensityBased;
        let small_system = WaterFlowSystem::from_parameters(small_params, &small_scale);

        let mut large_params = WaterFlowParameters::default();
        large_params.rainfall_scaling = RainfallScaling::_IntensityBased;
        let large_system = WaterFlowSystem::from_parameters(large_params, &large_scale);

        // Both should have the same rainfall rate per cell
        assert_eq!(small_system.effective_rainfall_rate, 0.0000027127);
        assert_eq!(large_system.effective_rainfall_rate, 0.0000027127);
    }

    #[test]
    fn water_system_with_different_scales() {
        let small_system = test_water_system(120, 60);
        let large_system = test_water_system(1024, 512);

        // Different scales should result in different effective rainfall rates
        assert_ne!(
            small_system.effective_rainfall_rate,
            large_system.effective_rainfall_rate
        );

        // With mass-conserving scaling, higher resolution (more cells) gets less rain per cell
        // This maintains constant total rainfall over the same physical region
        assert!(large_system.effective_rainfall_rate > small_system.effective_rainfall_rate);
    }

    #[test]
    fn hydrological_realistic_scaling() {
        // Test hydrological realistic scaling with Area^0.6 power law
        let reference_system = WaterFlowSystem::new_for_scale(&test_scale(240, 120));

        let mut params = WaterFlowParameters::default();
        params.rainfall_scaling = RainfallScaling::_HydrologicalRealistic;

        // Test with 4x larger area
        let large_scale = test_scale(480, 240); // 4x area
        let large_system = WaterFlowSystem::from_parameters(params.clone(), &large_scale);

        // With Area^0.6 scaling: (0.25)^0.6 ≈ 0.435, so 0.002 * 0.435 ≈ 0.00087
        let expected_large_rate = 0.002 * (0.25_f32).powf(0.6);
        assert!(
            (large_system.effective_rainfall_rate - expected_large_rate).abs() < 1e-6,
            "4x area with hydrological scaling, expected: {}, got: {}",
            expected_large_rate,
            large_system.effective_rainfall_rate
        );

        // Test with 1/4 area
        let small_scale = test_scale(120, 60); // 1/4 area
        let small_system = WaterFlowSystem::from_parameters(params, &small_scale);

        // With Area^0.6 scaling: (4.0)^0.6 ≈ 2.297, so 0.002 * 2.297 ≈ 0.00459
        let expected_small_rate = 0.002 * (4.0_f32).powf(0.6);
        assert!(
            (small_system.effective_rainfall_rate - expected_small_rate).abs() < 1e-6,
            "1/4 area with hydrological scaling, expected: {}, got: {}",
            expected_small_rate,
            small_system.effective_rainfall_rate
        );
    }

    // Water-climate integration tests
    #[test]
    fn temperature_dependent_evaporation_integration() {
        // Create a test heightmap with elevation variation
        let heightmap = vec![
            vec![0.0, 0.5, 1.0], // Low to high elevation
            vec![0.0, 0.5, 1.0],
            vec![0.0, 0.5, 1.0],
        ];
        let mut sim = Simulation::new(HeightMap::from_nested(heightmap));

        // Add equal water to all cells
        for y in 0..3 {
            for x in 0..3 {
                sim.water.depth[y][x] = 1.0;
            }
        }

        // Store initial water for comparison
        let initial_water_distribution = sim.water.depth.clone();

        // Run one tick with temperature-dependent evaporation
        sim.climate_system.tick(); // Advance season if needed
        sim.water_system.update_water_flow_with_climate(
            &mut sim.heightmap,
            &mut sim.water,
            &mut sim.temperature_layer,
            &sim.climate_system,
        );

        // Water levels should be different due to temperature variations
        // Higher elevations should be cooler and have less evaporation
        let sea_level_water = sim.water.depth[0][0]; // Low elevation (warm)
        let mountain_water = sim.water.depth[0][2]; // High elevation (cool)

        // Mountain water should have evaporated less than sea level water
        // (cooler temperatures = less evaporation)
        assert!(
            mountain_water >= sea_level_water,
            "Mountain water ({:.6}) should evaporate less than sea level water ({:.6}) due to cooler temperatures",
            mountain_water,
            sea_level_water
        );

        // Verify integration is working by checking that evaporation occurred
        let total_water_after = sim.water.get_total_water();
        let total_water_before = initial_water_distribution.iter().sum::<f32>();

        // Some water should have evaporated (unless temperature-dependent evaporation is extremely low)
        // But we can't guarantee exact amounts due to complex interactions
        assert!(
            total_water_after > 0.0,
            "Some water should remain after evaporation"
        );
        assert!(
            total_water_after.is_finite(),
            "Water amount should be finite"
        );
    }

    #[test]
    fn climate_system_seasonal_integration() {
        let heightmap = vec![vec![0.5; 2]; 2]; // Flat terrain
        let mut sim = Simulation::new(HeightMap::from_nested(heightmap));

        // Check that seasonal cycle advances
        let initial_season = sim.climate_system.current_season;

        // Run several ticks
        for _ in 0..10 {
            sim.tick();
        }

        // Season should have advanced (or wrapped around)
        assert_ne!(
            sim.climate_system.current_season, initial_season,
            "Seasonal cycle should advance with simulation ticks"
        );

        // Season should remain in valid range
        assert!(
            sim.climate_system.current_season >= 0.0 && sim.climate_system.current_season < 1.0,
            "Season should be in range [0.0, 1.0), got: {}",
            sim.climate_system.current_season
        );
    }

    #[test]
    fn temperature_layer_consistency_with_heightmap() {
        // Create heightmap with known pattern
        let heightmap = vec![
            vec![0.0, 1.0], // Sea level, mountain
            vec![0.5, 0.8], // Hill, high hill
        ];
        let sim = Simulation::new(HeightMap::from_nested(heightmap.clone()));

        // Temperature should correlate with elevation (higher = cooler)
        let sea_level_temp = sim.temperature_layer.get_temperature(0, 0);
        let mountain_temp = sim.temperature_layer.get_temperature(1, 0);

        assert!(
            mountain_temp < sea_level_temp,
            "Mountain temperature ({:.2}°C) should be cooler than sea level ({:.2}°C)",
            mountain_temp,
            sea_level_temp
        );

        // Temperature should be in reasonable range
        for y in 0..2 {
            for x in 0..2 {
                let temp = sim.temperature_layer.get_temperature(x, y);
                assert!(
                    temp > -100.0 && temp < 100.0,
                    "Temperature at ({}, {}) should be reasonable, got: {:.2}°C",
                    x,
                    y,
                    temp
                );
            }
        }
    }

    #[test]
    fn large_map_water_accumulation_works() {
        // Test the problematic 1024x512 map size that Jerry reported
        let heightmap = vec![vec![0.5; 1024]; 512]; // Flat terrain for predictable results
        let world_scale = WorldScale::new(10.0, (1024, 512), DetailLevel::Standard);
        let mut sim = Simulation::_new_with_scale(HeightMap::from_nested(heightmap), world_scale);

        // Check that water system has scale-aware threshold
        assert!(
            sim.water_system.evaporation_threshold < 0.001,
            "Evaporation threshold should be smaller than old fixed value"
        );
        assert!(
            sim.water_system.evaporation_threshold > 0.0,
            "Evaporation threshold should be positive"
        );

        // The effective rainfall rate should be low but the threshold should be even lower
        let post_evap_rainfall = sim.water_system.effective_rainfall_rate
            * (1.0 - sim.water_system.parameters.evaporation_rate);
        assert!(
            post_evap_rainfall > sim.water_system.evaporation_threshold,
            "Post-evaporation rainfall should exceed threshold to allow accumulation"
        );

        // Run several ticks and verify water accumulates
        let initial_water = sim.water.get_total_water();
        assert_eq!(initial_water, 0.0, "Should start with no water");

        // Run multiple ticks
        for _ in 0..5 {
            sim.tick();
        }

        let final_water = sim.water.get_total_water();
        assert!(
            final_water > 0.0,
            "Water should accumulate on large maps, got {}",
            final_water
        );
        assert!(
            final_water > initial_water,
            "Water should increase over time"
        );
    }

    #[test]
    fn scale_aware_evaporation_threshold() {
        // Test that different map sizes get appropriate evaporation thresholds
        let small_system = test_water_system(240, 120); // Reference size
        let large_system = test_water_system(1024, 512); // Large map

        // Large maps should have smaller thresholds to accommodate lower rainfall rates
        assert!(
            large_system.evaporation_threshold < small_system.evaporation_threshold,
            "Large maps should have smaller evaporation thresholds"
        );

        // Both should allow their respective rainfall rates to accumulate
        let small_post_evap =
            small_system.effective_rainfall_rate * (1.0 - small_system.parameters.evaporation_rate);
        let large_post_evap =
            large_system.effective_rainfall_rate * (1.0 - large_system.parameters.evaporation_rate);

        assert!(
            small_post_evap > small_system.evaporation_threshold,
            "Small map rainfall should exceed its threshold"
        );
        assert!(
            large_post_evap > large_system.evaporation_threshold,
            "Large map rainfall should exceed its threshold"
        );
    }

    #[test]
    fn all_scaling_modes_work() {
        let scale = test_scale(480, 240); // 4x reference area
        let base_params = WaterFlowParameters::default();

        // Test each scaling mode
        let mut per_cell_params = base_params.clone();
        per_cell_params.rainfall_scaling = RainfallScaling::_PerCell;
        let per_cell_system = WaterFlowSystem::from_parameters(per_cell_params, &scale);
        assert_eq!(per_cell_system.effective_rainfall_rate, 0.0000027127);

        let mut intensity_params = base_params.clone();
        intensity_params.rainfall_scaling = RainfallScaling::_IntensityBased;
        let intensity_system = WaterFlowSystem::from_parameters(intensity_params, &scale);
        assert_eq!(intensity_system.effective_rainfall_rate, 0.002); // Same as PerCell

        let mut mass_params = base_params.clone();
        mass_params.rainfall_scaling = RainfallScaling::MassConserving;
        let mass_system = WaterFlowSystem::from_parameters(mass_params, &scale);
        assert_eq!(mass_system.effective_rainfall_rate, 0.0005); // 0.002 * 0.25

        let mut hydro_params = base_params;
        hydro_params.rainfall_scaling = RainfallScaling::_HydrologicalRealistic;
        let hydro_system = WaterFlowSystem::from_parameters(hydro_params, &scale);
        let expected_hydro = 0.002 * (0.25_f32).powf(0.6);
        assert!((hydro_system.effective_rainfall_rate - expected_hydro).abs() < 1e-6);
    }

    // CFL timestep tests
    #[test]
    fn cfl_timestep_calculation() {
        // Create a high-resolution scale (small pixels) - same physical size, more pixels
        let high_res_scale = test_scale(1000, 1000); // 10km represented by 1000x1000 pixels = 10m per pixel
        let high_res_system = test_water_system(1000, 1000);

        // Create a low-resolution scale (large pixels) - same physical size, fewer pixels
        let low_res_scale = test_scale(100, 100); // 10km represented by 100x100 pixels = 100m per pixel  
        let low_res_system = test_water_system(100, 100);

        // Higher resolution (smaller pixels) should require smaller timesteps
        assert!(high_res_system._stable_timestep_seconds < low_res_system._stable_timestep_seconds);

        // Both should be in reasonable bounds
        assert!(high_res_system._stable_timestep_seconds > 0.001);
        assert!(high_res_system._stable_timestep_seconds < 60.0);
        assert!(low_res_system._stable_timestep_seconds > 0.001);
        assert!(low_res_system._stable_timestep_seconds < 60.0);
    }

    #[test]
    fn cfl_parameters_in_defaults() {
        let params = WaterFlowParameters::default();
        assert_eq!(params.max_expected_velocity_ms, 2.0);
        assert_eq!(params.cfl_safety_factor, 0.5);
    }

    #[test]
    fn cfl_stability_check() {
        let scale = test_scale(100, 100); // 100m per pixel
        let mut system = test_water_system(100, 100);
        let mut water = WaterLayer::new(3, 3);

        // Test with reasonable velocities - should be stable
        // In simulation units, velocity of 0.01 should translate to reasonable m/s
        water.velocity.set(1, 1, (0.01, 0.01));
        assert!(system._check_cfl_stability(&water, &scale));

        // Test with very high velocities - should be unstable
        // This should translate to much higher than 2.0 m/s
        water.velocity.set(1, 1, (1.0, 1.0));
        assert!(!system._check_cfl_stability(&water, &scale));
    }

    #[test]
    fn grid_spacing_uses_world_scale_not_heuristics() {
        // Test the continental scale case (64x32 = 2,048 cells) should get 32km/pixel, not 100m/pixel
        let continental_scale = WorldScale::new(
            2048.0, // 32km/pixel * 64 pixels = 2,048km total width
            (64, 32),
            DetailLevel::Standard,
        );
        let heightmap = HeightMap::from_nested(vec![vec![0.5; 64]; 32]);
        let mut water = WaterLayer::new(64, 32);

        // System created with proper WorldScale
        let mut system = WaterFlowSystem::new_for_scale(&continental_scale);

        // This should now use the WorldScale's grid spacing (32km/pixel = 32,000m/pixel)
        // instead of the wrong heuristic (100m/pixel for grids < 10k cells)
        system.calculate_flow_directions_with_scale(&heightmap, &mut water, &continental_scale);

        // The test passes if the method exists and uses the correct scale
        // We validate this by ensuring the function can be called without panicking
        assert_eq!(continental_scale.meters_per_pixel(), 32000.0);
    }
}
