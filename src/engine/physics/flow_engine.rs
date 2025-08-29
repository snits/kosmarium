// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Unified flow calculation engine consolidating 5 duplicate implementations
// ABOUTME: Provides consistent physics algorithms with pluggable approaches for different contexts

use crate::engine::core::{heightmap::HeightMap, math::Vec2, scale::WorldScale};
use crate::engine::physics::{drainage::DrainageNetwork, water::WaterLayer};

/// Flow calculation algorithms optimized for different physics contexts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlowAlgorithm {
    /// Gradient-based steepest descent for interactive/real-time simulation
    /// Fast approximation: v = gradient × flow_rate
    Gradient,

    /// Conservation-based shallow water physics with momentum equations
    /// Accurate: ∂v/∂t = -g∇h - v·∇v + friction_terms
    Conservation,

    /// Change-tracking optimization processing only active cells  
    /// High performance: selective update based on change detection
    Spatial,

    /// Static topological analysis using flow accumulation
    /// Network analysis: Kahn's algorithm for drainage patterns
    Drainage,
}

/// Unified velocity field representation using Phase 2.1 Vec2 foundation
#[derive(Debug, Clone)]
pub struct VelocityField {
    /// Velocity vectors at each grid cell
    pub velocities: Vec<Vec<Vec2>>,

    /// Grid dimensions
    pub width: usize,
    pub height: usize,

    /// Physical units (m/s) with WorldScale integration
    pub meters_per_pixel: f64,
}

impl VelocityField {
    /// Create new velocity field with specified dimensions
    pub fn new(width: usize, height: usize, scale: &WorldScale) -> Self {
        Self {
            velocities: vec![vec![Vec2::zero(); height]; width],
            width,
            height,
            meters_per_pixel: scale.meters_per_pixel(),
        }
    }

    /// Get velocity at specified coordinates
    pub fn get_velocity(&self, x: usize, y: usize) -> Vec2 {
        if x < self.width && y < self.height {
            self.velocities[x][y]
        } else {
            Vec2::zero()
        }
    }

    /// Set velocity at specified coordinates
    pub fn set_velocity(&mut self, x: usize, y: usize, velocity: Vec2) {
        if x < self.width && y < self.height {
            self.velocities[x][y] = velocity;
        }
    }

    /// Get maximum velocity magnitude in the field
    pub fn max_velocity_magnitude(&self) -> f32 {
        let mut max_mag = 0.0f32;
        for x in 0..self.width {
            for y in 0..self.height {
                max_mag = max_mag.max(self.velocities[x][y].magnitude());
            }
        }
        max_mag
    }

    /// Calculate total kinetic energy in the field
    pub fn total_kinetic_energy(&self) -> f64 {
        let mut total = 0.0f64;
        for x in 0..self.width {
            for y in 0..self.height {
                let mag_sq = self.velocities[x][y].magnitude_squared() as f64;
                total += mag_sq; // Kinetic energy ∝ v²
            }
        }
        total * 0.5 // Factor of 1/2 for kinetic energy
    }
}

/// Flow calculation parameters for different physics contexts
#[derive(Debug, Clone)]
pub struct FlowParameters {
    /// Gravitational acceleration (m/s²)
    pub gravity: f32,

    /// Surface roughness coefficient (Manning's n)
    pub roughness: f32,

    /// Minimum water depth for flow calculations (m)
    pub min_depth: f32,

    /// Flow concentration scaling factor for drainage networks
    pub concentration_factor: f32,

    /// Numerical stability factor for CFL condition
    pub cfl_safety: f32,

    /// Time step for explicit integration (seconds)  
    pub dt: f32,
}

impl Default for FlowParameters {
    fn default() -> Self {
        Self {
            gravity: 9.81,                // Standard Earth gravity
            roughness: 0.03,              // Typical natural channel
            min_depth: 1e-6,              // 1 micrometer minimum
            concentration_factor: 5000.0, // From Phase 1 continental drainage solution
            cfl_safety: 0.5,              // Conservative stability
            dt: 1.0,                      // 1 second timestep
        }
    }
}

/// Factory methods for creating optimized flow parameters
impl FlowParameters {
    /// Parameters optimized for climate system integration
    pub fn for_climate() -> Self {
        Self {
            concentration_factor: 1000.0, // Less aggressive for climate coupling
            dt: 10.0,                     // Longer timestep for climate processes
            ..Default::default()
        }
    }

    /// Parameters optimized for geological-scale processes
    pub fn for_geological() -> Self {
        Self {
            concentration_factor: 10000.0, // High concentration for erosion
            dt: 100.0,                     // Much longer geological timesteps
            roughness: 0.05,               // Higher roughness for geological surfaces
            ..Default::default()
        }
    }

    /// Parameters optimized for interactive/real-time simulation
    pub fn for_interactive() -> Self {
        Self {
            dt: 0.1,         // Short timestep for responsiveness
            cfl_safety: 0.3, // More conservative for stability
            ..Default::default()
        }
    }

    /// Parameters optimized for large-scale simulation performance
    pub fn for_large_scale(grid_size: usize) -> Self {
        let concentration = if grid_size > 1000 {
            8000.0 // Higher concentration for continental scales
        } else {
            3000.0 // Moderate for regional scales
        };

        Self {
            concentration_factor: concentration,
            dt: grid_size as f32 * 0.01, // Scale timestep with domain size
            ..Default::default()
        }
    }
}

/// Core unified flow calculation engine
#[derive(Debug)]
pub struct FlowEngine {
    /// Selected flow algorithm
    pub algorithm: FlowAlgorithm,

    /// Physics parameters
    pub parameters: FlowParameters,

    /// Current velocity field state
    pub velocity_field: VelocityField,
}

impl FlowEngine {
    /// Create flow engine with specified algorithm and scale
    pub fn new(algorithm: FlowAlgorithm, width: usize, height: usize, scale: &WorldScale) -> Self {
        let parameters = match algorithm {
            FlowAlgorithm::Gradient => FlowParameters::for_interactive(),
            FlowAlgorithm::Conservation => FlowParameters::default(),
            FlowAlgorithm::Spatial => FlowParameters::for_large_scale(width * height),
            FlowAlgorithm::Drainage => FlowParameters::for_geological(),
        };

        Self {
            algorithm,
            parameters,
            velocity_field: VelocityField::new(width, height, scale),
        }
    }

    /// Factory method for climate system integration
    pub fn for_climate(width: usize, height: usize, scale: &WorldScale) -> Self {
        Self {
            algorithm: FlowAlgorithm::Conservation, // Conservation physics for climate coupling
            parameters: FlowParameters::for_climate(),
            velocity_field: VelocityField::new(width, height, scale),
        }
    }

    /// Factory method for geological processes
    pub fn for_geology(width: usize, height: usize, scale: &WorldScale) -> Self {
        Self {
            algorithm: FlowAlgorithm::Drainage, // Network analysis for geological evolution
            parameters: FlowParameters::for_geological(),
            velocity_field: VelocityField::new(width, height, scale),
        }
    }

    /// Factory method for high-performance large-scale simulation
    pub fn for_performance(width: usize, height: usize, scale: &WorldScale) -> Self {
        Self {
            algorithm: FlowAlgorithm::Spatial, // Change-tracking optimization
            parameters: FlowParameters::for_large_scale(width * height),
            velocity_field: VelocityField::new(width, height, scale),
        }
    }

    /// Main flow calculation dispatch to appropriate algorithm
    pub fn calculate_flow(
        &mut self,
        heightmap: &HeightMap,
        water: &mut WaterLayer,
        drainage: Option<&DrainageNetwork>,
        scale: &WorldScale,
    ) {
        // Extract temporal scaling factor for unified physics scaling
        let temporal_factor = scale.temporal_scale.temporal_factor() as f32;

        // Ensure velocity field matches current scale
        self.update_scale_if_needed(scale);

        match self.algorithm {
            FlowAlgorithm::Gradient => self.calculate_gradient_flow_scaled(heightmap, water, scale, temporal_factor),
            FlowAlgorithm::Conservation => {
                self.calculate_conservation_flow_scaled(heightmap, water, scale, temporal_factor)
            }
            FlowAlgorithm::Spatial => self.calculate_spatial_flow_scaled(heightmap, water, scale, temporal_factor),
            FlowAlgorithm::Drainage => {
                if let Some(drainage_net) = drainage {
                    self.calculate_drainage_flow_scaled(heightmap, water, drainage_net, scale, temporal_factor)
                } else {
                    // Fallback to gradient method if no drainage network provided
                    self.calculate_gradient_flow_scaled(heightmap, water, scale, temporal_factor)
                }
            }
        }

        // Update water layer velocities from unified field
        self.update_water_layer_velocities(water);
    }

    /// Update scale parameters if WorldScale has changed
    fn update_scale_if_needed(&mut self, scale: &WorldScale) {
        let current_scale = self.velocity_field.meters_per_pixel;
        let new_scale = scale.meters_per_pixel();

        if (current_scale - new_scale).abs() > 1e-6 {
            self.velocity_field.meters_per_pixel = new_scale;
            // Adjust time step based on new scale for CFL stability
            let scale_ratio = (new_scale / current_scale) as f32;
            self.parameters.dt *= scale_ratio.sqrt(); // CFL ∝ √(dx)
        }
    }

    /// Gradient-based flow calculation (from sim.rs implementation)
    fn calculate_gradient_flow(
        &mut self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        scale: &WorldScale,
    ) {
        let grid_spacing_m = scale.meters_per_pixel() as f32;

        for x in 0..heightmap.width() {
            for y in 0..heightmap.height() {
                let velocity =
                    self.compute_gradient_velocity(heightmap, water, x, y, grid_spacing_m);
                self.velocity_field.set_velocity(x, y, velocity);
            }
        }
    }

    /// Gradient-based flow calculation with temporal scaling for unified physics consistency
    fn calculate_gradient_flow_scaled(
        &mut self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        scale: &WorldScale,
        temporal_factor: f32,
    ) {
        let grid_spacing_m = scale.meters_per_pixel() as f32;

        for x in 0..heightmap.width() {
            for y in 0..heightmap.height() {
                let mut velocity =
                    self.compute_gradient_velocity(heightmap, water, x, y, grid_spacing_m);
                
                // CRITICAL: Scale velocity with temporal factor
                velocity = velocity * temporal_factor;
                
                self.velocity_field.set_velocity(x, y, velocity);
            }
        }
    }

    /// Conservation-based flow calculation (from corrected_water_flow.rs)
    fn calculate_conservation_flow(
        &mut self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        scale: &WorldScale,
    ) {
        let grid_spacing_m = scale.meters_per_pixel() as f32;

        for x in 0..heightmap.width() {
            for y in 0..heightmap.height() {
                let velocity =
                    self.compute_conservation_velocity(heightmap, water, x, y, grid_spacing_m);
                self.velocity_field.set_velocity(x, y, velocity);
            }
        }
    }

    /// Conservation-based flow calculation with temporal scaling for unified physics consistency
    fn calculate_conservation_flow_scaled(
        &mut self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        scale: &WorldScale,
        temporal_factor: f32,
    ) {
        let grid_spacing_m = scale.meters_per_pixel() as f32;

        for x in 0..heightmap.width() {
            for y in 0..heightmap.height() {
                let mut velocity = 
                    self.compute_conservation_velocity(heightmap, water, x, y, grid_spacing_m);
                
                // CRITICAL: Scale velocity with temporal factor
                velocity = velocity * temporal_factor;
                
                self.velocity_field.set_velocity(x, y, velocity);
            }
        }
    }

    /// Spatial optimization flow calculation (from spatial_partitioning.rs)
    fn calculate_spatial_flow(
        &mut self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        scale: &WorldScale,
    ) {
        let grid_spacing_m = scale.meters_per_pixel() as f32;

        // Only process cells that have changed since last update
        for x in 0..heightmap.width() {
            for y in 0..heightmap.height() {
                if self.should_update_cell(water, x, y) {
                    let velocity =
                        self.compute_gradient_velocity(heightmap, water, x, y, grid_spacing_m);
                    self.velocity_field.set_velocity(x, y, velocity);
                }
            }
        }
    }

    /// Spatial optimization flow calculation with temporal scaling for unified physics consistency
    fn calculate_spatial_flow_scaled(
        &mut self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        scale: &WorldScale,
        temporal_factor: f32,
    ) {
        let grid_spacing_m = scale.meters_per_pixel() as f32;

        // Only process cells that have changed since last update
        for x in 0..heightmap.width() {
            for y in 0..heightmap.height() {
                if self.should_update_cell(water, x, y) {
                    let mut velocity =
                        self.compute_gradient_velocity(heightmap, water, x, y, grid_spacing_m);
                    
                    // CRITICAL: Scale velocity with temporal factor
                    velocity = velocity * temporal_factor;
                    
                    self.velocity_field.set_velocity(x, y, velocity);
                }
            }
        }
    }

    /// Drainage network flow calculation (from drainage.rs)
    fn calculate_drainage_flow(
        &mut self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        drainage: &DrainageNetwork,
        scale: &WorldScale,
    ) {
        let grid_spacing_m = scale.meters_per_pixel() as f32;

        for x in 0..heightmap.width() {
            for y in 0..heightmap.height() {
                let flow_accumulation = drainage.get_flow_accumulation(x, y);
                let velocity = self.compute_drainage_enhanced_velocity(
                    heightmap,
                    water,
                    x,
                    y,
                    grid_spacing_m,
                    flow_accumulation,
                );
                self.velocity_field.set_velocity(x, y, velocity);
            }
        }
    }

    /// Drainage network flow calculation with temporal scaling for unified physics consistency
    fn calculate_drainage_flow_scaled(
        &mut self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        drainage: &DrainageNetwork,
        scale: &WorldScale,
        temporal_factor: f32,
    ) {
        let grid_spacing_m = scale.meters_per_pixel() as f32;

        for x in 0..heightmap.width() {
            for y in 0..heightmap.height() {
                let flow_accumulation = drainage.get_flow_accumulation(x, y);
                let mut velocity = self.compute_drainage_enhanced_velocity(
                    heightmap,
                    water,
                    x,
                    y,
                    grid_spacing_m,
                    flow_accumulation,
                );
                
                // CRITICAL: Scale velocity with temporal factor
                velocity = velocity * temporal_factor;
                
                self.velocity_field.set_velocity(x, y, velocity);
            }
        }
    }

    /// Compute gradient-based velocity for a single cell
    fn compute_gradient_velocity(
        &self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        x: usize,
        y: usize,
        grid_spacing_m: f32,
    ) -> Vec2 {
        let water_surface_elevation = heightmap.get(x, y) + water.get_water_depth(x, y);

        // Calculate gradients to 8 neighbors
        let mut best_velocity = Vec2::zero();
        let mut steepest_gradient = 0.0f32;

        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;

                if nx < heightmap.width() && ny < heightmap.height() {
                    let neighbor_elevation = heightmap.get(nx, ny) + water.get_water_depth(nx, ny);
                    let elevation_diff = water_surface_elevation - neighbor_elevation;

                    if elevation_diff > 0.0 {
                        let distance = if dx.abs() + dy.abs() == 2 {
                            grid_spacing_m * 1.414213 // Diagonal distance
                        } else {
                            grid_spacing_m
                        };

                        let gradient = elevation_diff / distance;
                        if gradient > steepest_gradient {
                            steepest_gradient = gradient;
                            let flow_speed = (self.parameters.gravity * gradient).sqrt();
                            best_velocity =
                                Vec2::new(dx as f32 * flow_speed, dy as f32 * flow_speed);
                        }
                    }
                }
            }
        }

        best_velocity
    }

    /// Compute conservation-based velocity using shallow water equations  
    fn compute_conservation_velocity(
        &self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        x: usize,
        y: usize,
        grid_spacing_m: f32,
    ) -> Vec2 {
        let depth = water.get_water_depth(x, y).max(self.parameters.min_depth);
        let _elevation = heightmap.get(x, y); // Available for future conservation equations

        // Pressure gradient force: -g * ∇h
        let grad_x = self.compute_surface_gradient_x(heightmap, water, x, y, grid_spacing_m);
        let grad_y = self.compute_surface_gradient_y(heightmap, water, x, y, grid_spacing_m);

        // Manning's equation for friction
        let current_velocity = water.velocity.get(x, y);
        let velocity_magnitude = (current_velocity.0 * current_velocity.0
            + current_velocity.1 * current_velocity.1)
            .sqrt();

        let manning_coefficient = self.parameters.roughness;
        let hydraulic_radius = depth; // Approximation for wide shallow flow
        let friction_factor =
            (manning_coefficient * velocity_magnitude) / (hydraulic_radius.powf(2.0 / 3.0));

        // Conservation momentum equation: ∂v/∂t = -g∇h - friction_terms
        let acceleration_x =
            -self.parameters.gravity * grad_x - friction_factor * current_velocity.0;
        let acceleration_y =
            -self.parameters.gravity * grad_y - friction_factor * current_velocity.1;

        // Explicit time integration
        let new_velocity_x = current_velocity.0 + acceleration_x * self.parameters.dt;
        let new_velocity_y = current_velocity.1 + acceleration_y * self.parameters.dt;

        Vec2::new(new_velocity_x, new_velocity_y)
    }

    /// Compute drainage-enhanced velocity with flow concentration
    fn compute_drainage_enhanced_velocity(
        &self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        x: usize,
        y: usize,
        grid_spacing_m: f32,
        flow_accumulation: f32,
    ) -> Vec2 {
        // Start with base gradient velocity
        let base_velocity = self.compute_gradient_velocity(heightmap, water, x, y, grid_spacing_m);

        // Apply drainage concentration factor (from Phase 1 solution)
        let pixel_area = (grid_spacing_m * grid_spacing_m) as f64;
        let concentration = 1.0
            + (flow_accumulation as f64 / pixel_area).sqrt()
                * self.parameters.concentration_factor as f64;

        base_velocity * concentration as f32
    }

    /// Helper: compute surface gradient in X direction
    fn compute_surface_gradient_x(
        &self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        x: usize,
        y: usize,
        grid_spacing_m: f32,
    ) -> f32 {
        let x_left = if x > 0 { x - 1 } else { x };
        let x_right = if x < heightmap.width() - 1 { x + 1 } else { x };

        let left_surface = heightmap.get(x_left, y) + water.get_water_depth(x_left, y);
        let right_surface = heightmap.get(x_right, y) + water.get_water_depth(x_right, y);

        let distance = if x_left != x_right {
            2.0 * grid_spacing_m
        } else {
            grid_spacing_m
        };

        (right_surface - left_surface) / distance
    }

    /// Helper: compute surface gradient in Y direction  
    fn compute_surface_gradient_y(
        &self,
        heightmap: &HeightMap,
        water: &WaterLayer,
        x: usize,
        y: usize,
        grid_spacing_m: f32,
    ) -> f32 {
        let y_bottom = if y > 0 { y - 1 } else { y };
        let y_top = if y < heightmap.height() - 1 { y + 1 } else { y };

        let bottom_surface = heightmap.get(x, y_bottom) + water.get_water_depth(x, y_bottom);
        let top_surface = heightmap.get(x, y_top) + water.get_water_depth(x, y_top);

        let distance = if y_bottom != y_top {
            2.0 * grid_spacing_m
        } else {
            grid_spacing_m
        };

        (top_surface - bottom_surface) / distance
    }

    /// Check if cell should be updated (for spatial optimization)
    fn should_update_cell(&self, water: &WaterLayer, x: usize, y: usize) -> bool {
        // Simple heuristic: update if water depth is significant
        water.get_water_depth(x, y) > self.parameters.min_depth * 10.0
    }

    /// Update WaterLayer velocity field from unified representation
    fn update_water_layer_velocities(&self, water: &mut WaterLayer) {
        for x in 0..self.velocity_field.width {
            for y in 0..self.velocity_field.height {
                let velocity = self.velocity_field.get_velocity(x, y);
                water.velocity.set(x, y, (velocity.x, velocity.y));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::core::scale::DetailLevel;

    fn create_test_scale() -> WorldScale {
        WorldScale::new(1000.0, (10, 10), DetailLevel::Standard)
    }

    #[test]
    fn test_velocity_field_basic_operations() {
        let scale = create_test_scale();
        let mut field = VelocityField::new(10, 10, &scale);

        let test_velocity = Vec2::new(1.5, -0.8);
        field.set_velocity(5, 3, test_velocity);

        let retrieved = field.get_velocity(5, 3);
        assert_eq!(retrieved, test_velocity);

        let zero_velocity = field.get_velocity(2, 7);
        assert_eq!(zero_velocity, Vec2::zero());
    }

    #[test]
    fn test_flow_engine_factory_methods() {
        let scale = create_test_scale();

        let climate_engine = FlowEngine::for_climate(10, 10, &scale);
        assert_eq!(climate_engine.algorithm, FlowAlgorithm::Conservation);

        let geology_engine = FlowEngine::for_geology(10, 10, &scale);
        assert_eq!(geology_engine.algorithm, FlowAlgorithm::Drainage);

        let performance_engine = FlowEngine::for_performance(10, 10, &scale);
        assert_eq!(performance_engine.algorithm, FlowAlgorithm::Spatial);
    }

    #[test]
    fn test_flow_parameters_scaling() {
        let interactive = FlowParameters::for_interactive();
        let large_scale = FlowParameters::for_large_scale(10000);

        // Interactive should have shorter timestep
        assert!(interactive.dt < large_scale.dt);

        // Large scale should have higher concentration factor
        assert!(large_scale.concentration_factor > interactive.concentration_factor);
    }
}
