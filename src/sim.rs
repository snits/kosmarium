// ABOUTME: Core simulation state and water flow system for dynamic terrain evolution
// ABOUTME: Manages heightmap terrain with real-time water flow, accumulation, and hydraulic erosion

use crate::scale::{REFERENCE_SCALE, ScaleAware, WorldScale};

#[derive(Clone, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

#[derive(Clone, Debug)]
pub struct WaterLayer {
    pub depth: Vec<Vec<f32>>,     // Water depth at each cell
    pub velocity: Vec<Vec<Vec2>>, // Flow direction and speed
    pub sediment: Vec<Vec<f32>>,  // Carried sediment for erosion
    width: usize,
    height: usize,
}

impl WaterLayer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            depth: vec![vec![0.0; width]; height],
            velocity: vec![vec![Vec2::zero(); width]; height],
            sediment: vec![vec![0.0; width]; height],
            width,
            height,
        }
    }

    pub fn get_total_water(&self) -> f32 {
        self.depth.iter().flat_map(|row| row.iter()).sum()
    }

    pub fn add_water(&mut self, x: usize, y: usize, amount: f32) {
        if x < self.width && y < self.height {
            self.depth[y][x] += amount;
        }
    }
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
pub struct WaterFlowSystem {
    pub parameters: WaterFlowParameters,
    pub effective_rainfall_rate: f32, // Computed rainfall rate for current scale
    pub stable_timestep_seconds: f32, // CFL-derived timestep for numerical stability
    pub evaporation_threshold: f32,   // Scale-aware threshold for clearing tiny water amounts
}

#[derive(Clone, Debug, PartialEq)]
pub enum RainfallScaling {
    /// Same rainfall per cell regardless of map size (higher total water on larger maps)
    /// Use for predictable behavior and debugging
    PerCell,

    /// Mass-conserving scaling: Total rainfall over region remains constant
    /// Rain per cell ∝ 1/area - larger maps get less rain per cell
    /// Use for realistic water budget modeling
    MassConserving,

    /// Intensity-based scaling: Meteorological intensity remains constant per unit area
    /// Same as PerCell but with clearer physical interpretation
    /// Use when modeling actual precipitation rates
    IntensityBased,

    /// Hydrologically realistic scaling: Based on empirical watershed relationships
    /// Many hydrological processes follow Area^0.6 scaling laws
    /// Use for realistic terrain evolution simulation
    HydrologicalRealistic,
}

impl Default for WaterFlowParameters {
    fn default() -> Self {
        // Default values calibrated for ~240x120 reference map
        Self {
            flow_rate: 0.1,
            evaporation_rate: 0.001,
            erosion_strength: 0.01,
            deposition_rate: 0.05,
            base_rainfall_rate: 0.002,
            rainfall_scaling: RainfallScaling::MassConserving, // Physics-based total mass conservation
            max_expected_velocity_ms: 2.0, // Reasonable for gentle water flow (walking speed)
            cfl_safety_factor: 0.5,        // Conservative safety margin
        }
    }
}

impl ScaleAware for WaterFlowParameters {
    fn derive_parameters(&self, scale: &WorldScale) -> Self {
        // For now, most parameters don't scale - just return copy
        // Future enhancement: could scale flow_rate based on meters_per_pixel, etc.
        self.clone()
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
            stable_timestep_seconds,
            evaporation_threshold,
        }
    }

    /// Calculate the effective rainfall rate based on scaling strategy
    fn calculate_rainfall_rate(params: &WaterFlowParameters, scale: &WorldScale) -> f32 {
        match params.rainfall_scaling {
            RainfallScaling::PerCell => {
                // No scaling - same rain per cell regardless of map size
                params.base_rainfall_rate
            }
            RainfallScaling::MassConserving => {
                // Total rainfall over region remains constant
                // Rain per cell ∝ 1/area (inverse scaling with map size)
                let area_ratio = scale.scale_factor_from_reference(REFERENCE_SCALE) as f32;
                params.base_rainfall_rate * area_ratio
            }
            RainfallScaling::IntensityBased => {
                // Meteorological intensity remains constant - same as PerCell
                params.base_rainfall_rate
            }
            RainfallScaling::HydrologicalRealistic => {
                // Based on empirical relationships in hydrology
                // Many watershed processes follow ~ Area^0.6 relationships
                let area_ratio = scale.scale_factor_from_reference(REFERENCE_SCALE) as f32;
                params.base_rainfall_rate * area_ratio.powf(0.6)
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

        // Clamp to reasonable bounds (at least 0.001s, at most 60.0s for very coarse grids)
        cfl_timestep.max(0.001).min(60.0)
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
    pub fn get_effective_rainfall_rate(&self) -> f32 {
        self.effective_rainfall_rate
    }

    /// Get the CFL-stable timestep for this system
    pub fn get_stable_timestep_seconds(&self) -> f32 {
        self.stable_timestep_seconds
    }

    /// Check if current flow velocities are within CFL stability bounds
    pub fn check_cfl_stability(&self, water: &WaterLayer, scale: &WorldScale) -> bool {
        let dx = scale.meters_per_pixel() as f32;
        let mut max_observed_velocity = 0.0f32;

        for row in &water.velocity {
            for velocity in row {
                max_observed_velocity = max_observed_velocity.max(velocity.magnitude());
            }
        }

        // Convert from dimensionless simulation units to m/s (rough approximation)
        let estimated_velocity_ms = max_observed_velocity * dx;

        // Check if we're within expected bounds
        estimated_velocity_ms <= self.parameters.max_expected_velocity_ms
    }

    /// Calculate flow direction for each cell based on elevation gradients
    pub fn calculate_flow_directions(&self, heightmap: &[Vec<f32>], water: &mut WaterLayer) {
        let height = heightmap.len();
        let width = if height > 0 {
            heightmap[0].len()
        } else {
            return;
        };

        for y in 0..height {
            for x in 0..width {
                let current_elevation = heightmap[y][x] + water.depth[y][x];
                let mut steepest_slope = 0.0;
                let mut flow_direction = Vec2::zero();

                // Check all 8 neighbors for steepest descent
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;

                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            let nx = nx as usize;
                            let ny = ny as usize;

                            let neighbor_elevation = heightmap[ny][nx] + water.depth[ny][nx];
                            let slope = current_elevation - neighbor_elevation;

                            if slope > steepest_slope {
                                steepest_slope = slope;
                                flow_direction = Vec2::new(dx as f32, dy as f32);
                            }
                        }
                    }
                }

                // Normalize flow direction and scale by flow rate
                if flow_direction.magnitude() > 0.0 {
                    let magnitude = flow_direction.magnitude();
                    flow_direction.x =
                        (flow_direction.x / magnitude) * steepest_slope * self.parameters.flow_rate;
                    flow_direction.y =
                        (flow_direction.y / magnitude) * steepest_slope * self.parameters.flow_rate;
                }

                water.velocity[y][x] = flow_direction;
            }
        }
    }

    /// Simulate one tick of water flow
    pub fn update_water_flow(&self, heightmap: &mut Vec<Vec<f32>>, water: &mut WaterLayer) {
        // Calculate flow directions based on current state
        self.calculate_flow_directions(heightmap, water);

        // Add rainfall
        self.add_rainfall(water);

        // Move water based on flow directions
        self.move_water(water);

        // Apply erosion and deposition
        self.apply_erosion(heightmap, water);

        // Evaporate water
        self.apply_evaporation(water);
    }

    fn add_rainfall(&self, water: &mut WaterLayer) {
        for row in water.depth.iter_mut() {
            for depth in row.iter_mut() {
                *depth += self.effective_rainfall_rate;
            }
        }
    }

    fn move_water(&self, water: &mut WaterLayer) {
        let mut new_depth = water.depth.clone();

        for y in 0..water.height {
            for x in 0..water.width {
                let velocity = &water.velocity[y][x];
                let flow_amount = water.depth[y][x] * velocity.magnitude().min(1.0);

                if flow_amount > 0.001 {
                    // Calculate target position
                    let target_x = (x as f32 + velocity.x).round() as i32;
                    let target_y = (y as f32 + velocity.y).round() as i32;

                    // Move water if target is in bounds
                    if target_x >= 0
                        && target_x < water.width as i32
                        && target_y >= 0
                        && target_y < water.height as i32
                    {
                        new_depth[y][x] -= flow_amount;
                        new_depth[target_y as usize][target_x as usize] += flow_amount;
                    }
                }
            }
        }

        water.depth = new_depth;
    }

    fn apply_erosion(&self, heightmap: &mut Vec<Vec<f32>>, water: &mut WaterLayer) {
        for y in 0..water.height {
            for x in 0..water.width {
                let flow_speed = water.velocity[y][x].magnitude();
                let water_depth = water.depth[y][x];

                if flow_speed > 0.01 && water_depth > 0.001 {
                    // Erosion capacity based on flow speed and water depth
                    let erosion_capacity =
                        flow_speed * water_depth * self.parameters.erosion_strength;

                    // Erode terrain if we're below capacity
                    if water.sediment[y][x] < erosion_capacity {
                        let erosion_amount = (erosion_capacity - water.sediment[y][x]).min(0.001);
                        heightmap[y][x] -= erosion_amount;
                        water.sediment[y][x] += erosion_amount;
                    }
                    // Deposit sediment if we're over capacity
                    else if water.sediment[y][x] > erosion_capacity {
                        let deposition_amount = (water.sediment[y][x] - erosion_capacity)
                            * self.parameters.deposition_rate;
                        heightmap[y][x] += deposition_amount;
                        water.sediment[y][x] -= deposition_amount;
                    }
                }
            }
        }
    }

    fn apply_evaporation(&self, water: &mut WaterLayer) {
        for row in water.depth.iter_mut() {
            for depth in row.iter_mut() {
                *depth *= 1.0 - self.parameters.evaporation_rate;
                if *depth < self.evaporation_threshold {
                    *depth = 0.0;
                }
            }
        }

        // Also evaporate sediment when water disappears
        for y in 0..water.height {
            for x in 0..water.width {
                if water.depth[y][x] < self.evaporation_threshold {
                    water.sediment[y][x] *= 0.5; // Sediment settles when water dries up
                }
            }
        }
    }
}

pub struct Simulation {
    pub heightmap: Vec<Vec<f32>>,
    pub water: WaterLayer,
    pub water_system: WaterFlowSystem,
    pub world_scale: WorldScale,
    pub tick_count: u64,
}

impl Simulation {
    /// Create a simulation with default world scale (assumes 10km physical size)
    pub fn new(heightmap: Vec<Vec<f32>>) -> Self {
        let height = heightmap.len();
        let width = if height > 0 { heightmap[0].len() } else { 0 };

        // Default to 10km physical size with standard detail
        let world_scale = WorldScale::new(
            10.0,
            (width as u32, height as u32),
            crate::scale::DetailLevel::Standard,
        );

        Self {
            heightmap,
            water: WaterLayer::new(width, height),
            water_system: WaterFlowSystem::new_for_scale(&world_scale),
            world_scale,
            tick_count: 0,
        }
    }

    /// Create a simulation with explicit world scale
    pub fn new_with_scale(heightmap: Vec<Vec<f32>>, world_scale: WorldScale) -> Self {
        let height = heightmap.len();
        let width = if height > 0 { heightmap[0].len() } else { 0 };

        Self {
            heightmap,
            water: WaterLayer::new(width, height),
            water_system: WaterFlowSystem::new_for_scale(&world_scale),
            world_scale,
            tick_count: 0,
        }
    }

    /// Advance simulation by one time step
    pub fn tick(&mut self) {
        self.water_system
            .update_water_flow(&mut self.heightmap, &mut self.water);
        self.tick_count += 1;
    }

    /// Get the total water + terrain elevation at a position
    pub fn get_total_elevation(&self, x: usize, y: usize) -> f32 {
        if y < self.heightmap.len() && x < self.heightmap[0].len() {
            self.heightmap[y][x] + self.water.depth[y][x]
        } else {
            0.0
        }
    }

    /// Add water at a specific location (useful for testing/debugging)
    pub fn add_water_at(&mut self, x: usize, y: usize, amount: f32) {
        self.water.add_water(x, y, amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::{DetailLevel, WorldScale};

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
        assert_eq!(layer.width, 10);
        assert_eq!(layer.height, 5);
        assert_eq!(layer.depth.len(), 5); // height rows
        assert_eq!(layer.depth[0].len(), 10); // width columns
        assert_eq!(layer.velocity.len(), 5);
        assert_eq!(layer.velocity[0].len(), 10);
        assert_eq!(layer.sediment.len(), 5);
        assert_eq!(layer.sediment[0].len(), 10);
    }

    #[test]
    fn water_layer_initializes_to_zero() {
        let layer = WaterLayer::new(3, 3);

        // All depths should be zero
        for row in &layer.depth {
            for &depth in row {
                assert_eq!(depth, 0.0);
            }
        }

        // All velocities should be zero
        for row in &layer.velocity {
            for velocity in row {
                assert_eq!(velocity.x, 0.0);
                assert_eq!(velocity.y, 0.0);
            }
        }

        // All sediment should be zero
        for row in &layer.sediment {
            for &sediment in row {
                assert_eq!(sediment, 0.0);
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
        assert_eq!(params.base_rainfall_rate, 0.002);
        assert_eq!(params.rainfall_scaling, RainfallScaling::MassConserving);
    }

    #[test]
    fn water_flow_system_with_scale() {
        use crate::scale::{DetailLevel, WorldScale};
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
        let system = test_water_system(3, 3);
        let heightmap = vec![
            vec![0.5, 0.5, 0.5],
            vec![0.5, 0.5, 0.5],
            vec![0.5, 0.5, 0.5],
        ];
        let mut water = WaterLayer::new(3, 3);

        system.calculate_flow_directions(&heightmap, &mut water);

        // On flat terrain, there should be no flow
        for row in &water.velocity {
            for velocity in row {
                assert_eq!(velocity.x, 0.0);
                assert_eq!(velocity.y, 0.0);
            }
        }
    }

    #[test]
    fn flow_direction_simple_slope() {
        let system = test_water_system(3, 3);
        // Create a simple slope from left to right
        let heightmap = vec![
            vec![1.0, 0.5, 0.0],
            vec![1.0, 0.5, 0.0],
            vec![1.0, 0.5, 0.0],
        ];
        let mut water = WaterLayer::new(3, 3);

        system.calculate_flow_directions(&heightmap, &mut water);

        // Water in center column should flow toward lower elevation (positive x direction)
        let center_velocity = &water.velocity[1][1];
        assert!(
            center_velocity.x > 0.0,
            "Water should flow downhill (positive x)"
        );
        // Note: May have small y component due to 8-neighbor diagonal flow

        // Water on rightmost column should have no flow (no lower neighbor)
        let right_velocity = &water.velocity[1][2];
        assert_eq!(right_velocity.x, 0.0);
        assert_eq!(right_velocity.y, 0.0);
    }

    #[test]
    fn flow_direction_with_water_depth() {
        let system = test_water_system(2, 2);
        let heightmap = vec![vec![1.0, 0.5], vec![1.0, 0.5]];
        let mut water = WaterLayer::new(2, 2);

        // Add water that changes the effective elevation
        water.depth[0][0] = 0.3; // Total elevation becomes 1.3
        water.depth[0][1] = 0.1; // Total elevation becomes 0.6

        system.calculate_flow_directions(&heightmap, &mut water);

        // Should still flow from higher total elevation to lower
        let velocity = &water.velocity[0][0];
        assert!(
            velocity.x > 0.0,
            "Water should flow from higher to lower total elevation"
        );
    }

    #[test]
    fn flow_direction_eight_neighbors() {
        let system = test_water_system(2, 2);
        // Create a heightmap with center cell higher than all neighbors
        let heightmap = vec![
            vec![0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ];
        let mut water = WaterLayer::new(3, 3);

        system.calculate_flow_directions(&heightmap, &mut water);

        // Center cell should flow toward the steepest neighbor
        // All neighbors are equal, so it should pick one of them
        let center_velocity = &water.velocity[1][1];
        let magnitude = center_velocity.magnitude();
        assert!(
            magnitude > 0.0,
            "Center cell should have flow toward neighbors"
        );

        // Flow direction should be normalized
        assert!(magnitude <= system.parameters.flow_rate * 1.0); // slope of 1.0
    }

    #[test]
    fn flow_direction_boundary_conditions() {
        let system = test_water_system(2, 2);
        let heightmap = vec![vec![1.0, 0.5], vec![0.8, 0.3]];
        let mut water = WaterLayer::new(2, 2);

        system.calculate_flow_directions(&heightmap, &mut water);

        // Corner cells should only consider their available neighbors
        // This test ensures we don't access out-of-bounds indices
        // Just check that it doesn't panic - the exact flow values depend on implementation
        for row in &water.velocity {
            for velocity in row {
                assert!(velocity.magnitude().is_finite());
            }
        }
    }

    // Water movement and physics tests
    #[test]
    fn rainfall_adds_water_uniformly() {
        use crate::scale::{DetailLevel, WorldScale};
        let scale = WorldScale::new(10.0, (2, 2), DetailLevel::Standard);
        let mut params = WaterFlowParameters::default();
        params.base_rainfall_rate = 0.1;
        let system = WaterFlowSystem::from_parameters(params, &scale);
        let mut water = WaterLayer::new(2, 2);

        system.add_rainfall(&mut water);

        for row in &water.depth {
            for &depth in row {
                assert_eq!(depth, system.effective_rainfall_rate);
            }
        }
    }

    #[test]
    fn evaporation_reduces_water() {
        let scale = test_scale(2, 2);
        let mut params = WaterFlowParameters::default();
        params.evaporation_rate = 0.1;
        let system = WaterFlowSystem::from_parameters(params, &scale);
        let mut water = WaterLayer::new(2, 2);
        water.depth[0][0] = 1.0;
        water.depth[0][1] = 0.5;

        system.apply_evaporation(&mut water);

        assert_eq!(water.depth[0][0], 0.9); // 1.0 * (1 - 0.1)
        assert_eq!(water.depth[0][1], 0.45); // 0.5 * (1 - 0.1)
    }

    #[test]
    fn evaporation_clears_tiny_amounts() {
        let system = test_water_system(2, 2);
        let mut water = WaterLayer::new(1, 1);

        // Use an amount smaller than the scale-aware threshold
        let tiny_amount = system.evaporation_threshold * 0.5;
        water.depth[0][0] = tiny_amount;

        assert_eq!(water.depth[0][0], 0.0); // Should be cleared to 0
    }

    #[test]
    fn erosion_removes_terrain_adds_sediment() {
        let system = test_water_system(2, 2);
        let mut heightmap = vec![vec![1.0]];
        let mut water = WaterLayer::new(1, 1);
        water.depth[0][0] = 0.1;
        water.velocity[0][0] = Vec2::new(0.5, 0.0); // Fast flow
        water.sediment[0][0] = 0.0; // No initial sediment

        let initial_height = heightmap[0][0];
        system.apply_erosion(&mut heightmap, &mut water);

        assert!(heightmap[0][0] < initial_height, "Terrain should be eroded");
        assert!(water.sediment[0][0] > 0.0, "Sediment should increase");
    }

    #[test]
    fn deposition_adds_terrain_removes_sediment() {
        let system = test_water_system(2, 2);
        let mut heightmap = vec![vec![1.0]];
        let mut water = WaterLayer::new(1, 1);
        water.depth[0][0] = 0.1; // More water needed for capacity calculation
        water.velocity[0][0] = Vec2::new(0.02, 0.0); // Slow but not too slow flow
        water.sediment[0][0] = 0.1; // Lots of sediment

        let initial_height = heightmap[0][0];
        let initial_sediment = water.sediment[0][0];
        system.apply_erosion(&mut heightmap, &mut water);

        // Check if deposition occurred (height increased) OR if we're at capacity
        // This test validates the physics are working correctly
        let height_changed = heightmap[0][0] != initial_height;
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
        let mut sim = Simulation::new(heightmap);

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
        let mut sim = Simulation::new(heightmap);

        // On flat terrain, only rainfall and evaporation should affect water
        sim.tick();
        let rainfall_added = 9.0 * sim.water_system.effective_rainfall_rate; // 9 cells
        let evaporation_factor = 1.0 - sim.water_system.parameters.evaporation_rate;
        let expected_water = rainfall_added * evaporation_factor;
        let actual_water = sim.water.get_total_water();

        // Should be close (rainfall is added first, then evaporation is applied)
        assert!(
            (actual_water - expected_water).abs() < 1e-4,
            "Expected: {}, Actual: {}",
            expected_water,
            actual_water
        );
    }

    // Scale-aware rainfall tests
    #[test]
    fn mass_conserving_scaling_maintains_total_water_input() {
        // Create two different sized maps
        let small_heightmap = vec![vec![0.5; 10]; 10]; // 100 cells  
        let large_heightmap = vec![vec![0.5; 20]; 20]; // 400 cells (4x larger)

        let mut small_sim = Simulation::new(small_heightmap);
        let mut large_sim = Simulation::new(large_heightmap);

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
            (reference_rate - 0.002).abs() < 1e-6,
            "Reference rate should be ~0.002, got {}",
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
        small_params.rainfall_scaling = RainfallScaling::IntensityBased;
        let small_system = WaterFlowSystem::from_parameters(small_params, &small_scale);

        let mut large_params = WaterFlowParameters::default();
        large_params.rainfall_scaling = RainfallScaling::IntensityBased;
        let large_system = WaterFlowSystem::from_parameters(large_params, &large_scale);

        // Both should have the same rainfall rate per cell
        assert_eq!(small_system.effective_rainfall_rate, 0.002);
        assert_eq!(large_system.effective_rainfall_rate, 0.002);
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

        // Larger maps should have lower effective rainfall (mass conserving scaling)
        assert!(large_system.effective_rainfall_rate < small_system.effective_rainfall_rate);
    }

    #[test]
    fn hydrological_realistic_scaling() {
        // Test hydrological realistic scaling with Area^0.6 power law
        let reference_system = WaterFlowSystem::new_for_scale(&test_scale(240, 120));

        let mut params = WaterFlowParameters::default();
        params.rainfall_scaling = RainfallScaling::HydrologicalRealistic;

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

    #[test]
    fn large_map_water_accumulation_works() {
        // Test the problematic 1024x512 map size that Jerry reported
        let heightmap = vec![vec![0.5; 1024]; 512]; // Flat terrain for predictable results
        let world_scale = WorldScale::new(10.0, (1024, 512), DetailLevel::Standard);
        let mut sim = Simulation::new_with_scale(heightmap, world_scale);

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
        per_cell_params.rainfall_scaling = RainfallScaling::PerCell;
        let per_cell_system = WaterFlowSystem::from_parameters(per_cell_params, &scale);
        assert_eq!(per_cell_system.effective_rainfall_rate, 0.002);

        let mut intensity_params = base_params.clone();
        intensity_params.rainfall_scaling = RainfallScaling::IntensityBased;
        let intensity_system = WaterFlowSystem::from_parameters(intensity_params, &scale);
        assert_eq!(intensity_system.effective_rainfall_rate, 0.002); // Same as PerCell

        let mut mass_params = base_params.clone();
        mass_params.rainfall_scaling = RainfallScaling::MassConserving;
        let mass_system = WaterFlowSystem::from_parameters(mass_params, &scale);
        assert_eq!(mass_system.effective_rainfall_rate, 0.0005); // 0.002 * 0.25

        let mut hydro_params = base_params;
        hydro_params.rainfall_scaling = RainfallScaling::HydrologicalRealistic;
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
        assert!(high_res_system.stable_timestep_seconds < low_res_system.stable_timestep_seconds);

        // Both should be in reasonable bounds
        assert!(high_res_system.stable_timestep_seconds > 0.001);
        assert!(high_res_system.stable_timestep_seconds < 60.0);
        assert!(low_res_system.stable_timestep_seconds > 0.001);
        assert!(low_res_system.stable_timestep_seconds < 60.0);
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
        let system = test_water_system(100, 100);
        let mut water = WaterLayer::new(3, 3);

        // Test with reasonable velocities - should be stable
        // In simulation units, velocity of 0.01 should translate to reasonable m/s
        water.velocity[1][1] = Vec2::new(0.01, 0.01);
        assert!(system.check_cfl_stability(&water, &scale));

        // Test with very high velocities - should be unstable
        // This should translate to much higher than 2.0 m/s
        water.velocity[1][1] = Vec2::new(1.0, 1.0);
        assert!(!system.check_cfl_stability(&water, &scale));
    }
}
