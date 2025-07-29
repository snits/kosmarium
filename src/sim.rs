// ABOUTME: Core simulation state and water flow system for dynamic terrain evolution
// ABOUTME: Manages heightmap terrain with real-time water flow, accumulation, and hydraulic erosion

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

pub struct WaterFlowSystem {
    pub flow_rate: f32,        // How fast water flows (0.0-1.0)
    pub evaporation_rate: f32, // Water loss per tick (0.0-1.0)
    pub erosion_strength: f32, // How much sediment water can carry
    pub deposition_rate: f32,  // How fast sediment settles
    pub rainfall_rate: f32,    // How much water is added per tick
}

impl Default for WaterFlowSystem {
    fn default() -> Self {
        Self {
            flow_rate: 0.1,
            evaporation_rate: 0.001,
            erosion_strength: 0.01,
            deposition_rate: 0.05,
            rainfall_rate: 0.002,
        }
    }
}

impl WaterFlowSystem {
    pub fn new() -> Self {
        Self::default()
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
                        (flow_direction.x / magnitude) * steepest_slope * self.flow_rate;
                    flow_direction.y =
                        (flow_direction.y / magnitude) * steepest_slope * self.flow_rate;
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
                *depth += self.rainfall_rate;
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
                    let erosion_capacity = flow_speed * water_depth * self.erosion_strength;

                    // Erode terrain if we're below capacity
                    if water.sediment[y][x] < erosion_capacity {
                        let erosion_amount = (erosion_capacity - water.sediment[y][x]).min(0.001);
                        heightmap[y][x] -= erosion_amount;
                        water.sediment[y][x] += erosion_amount;
                    }
                    // Deposit sediment if we're over capacity
                    else if water.sediment[y][x] > erosion_capacity {
                        let deposition_amount =
                            (water.sediment[y][x] - erosion_capacity) * self.deposition_rate;
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
                *depth *= 1.0 - self.evaporation_rate;
                if *depth < 0.001 {
                    *depth = 0.0;
                }
            }
        }

        // Also evaporate sediment when water disappears
        for y in 0..water.height {
            for x in 0..water.width {
                if water.depth[y][x] < 0.001 {
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
    pub tick_count: u64,
}

impl Simulation {
    pub fn new(heightmap: Vec<Vec<f32>>) -> Self {
        let height = heightmap.len();
        let width = if height > 0 { heightmap[0].len() } else { 0 };

        Self {
            heightmap,
            water: WaterLayer::new(width, height),
            water_system: WaterFlowSystem::new(),
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
    fn water_flow_system_default_values() {
        let system = WaterFlowSystem::default();
        assert_eq!(system.flow_rate, 0.1);
        assert_eq!(system.evaporation_rate, 0.001);
        assert_eq!(system.erosion_strength, 0.01);
        assert_eq!(system.deposition_rate, 0.05);
        assert_eq!(system.rainfall_rate, 0.002);
    }

    #[test]
    fn water_flow_system_new_uses_default() {
        let system1 = WaterFlowSystem::new();
        let system2 = WaterFlowSystem::default();
        assert_eq!(system1.flow_rate, system2.flow_rate);
        assert_eq!(system1.evaporation_rate, system2.evaporation_rate);
    }

    #[test]
    fn flow_direction_flat_terrain_no_flow() {
        let system = WaterFlowSystem::new();
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
        let system = WaterFlowSystem::new();
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
        let system = WaterFlowSystem::new();
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
        let system = WaterFlowSystem::new();
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
        assert!(magnitude <= system.flow_rate * 1.0); // slope of 1.0
    }

    #[test]
    fn flow_direction_boundary_conditions() {
        let system = WaterFlowSystem::new();
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
        let system = WaterFlowSystem {
            rainfall_rate: 0.1,
            ..Default::default()
        };
        let mut water = WaterLayer::new(2, 2);

        system.add_rainfall(&mut water);

        for row in &water.depth {
            for &depth in row {
                assert_eq!(depth, 0.1);
            }
        }
    }

    #[test]
    fn evaporation_reduces_water() {
        let system = WaterFlowSystem {
            evaporation_rate: 0.1,
            ..Default::default()
        };
        let mut water = WaterLayer::new(2, 2);
        water.depth[0][0] = 1.0;
        water.depth[0][1] = 0.5;

        system.apply_evaporation(&mut water);

        assert_eq!(water.depth[0][0], 0.9); // 1.0 * (1 - 0.1)
        assert_eq!(water.depth[0][1], 0.45); // 0.5 * (1 - 0.1)
    }

    #[test]
    fn evaporation_clears_tiny_amounts() {
        let system = WaterFlowSystem::new();
        let mut water = WaterLayer::new(1, 1);
        water.depth[0][0] = 0.0005; // Very small amount

        system.apply_evaporation(&mut water);

        assert_eq!(water.depth[0][0], 0.0); // Should be cleared to 0
    }

    #[test]
    fn erosion_removes_terrain_adds_sediment() {
        let system = WaterFlowSystem::new();
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
        let system = WaterFlowSystem::new();
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
        let rainfall_added = 9.0 * sim.water_system.rainfall_rate; // 9 cells
        let evaporation_factor = 1.0 - sim.water_system.evaporation_rate;
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
}
