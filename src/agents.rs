// ABOUTME: Real-time agent system with high-performance structure-of-arrays storage
// ABOUTME: Supports NPCs, creatures, and player avatars with social dynamics and cultural evolution

use crate::climate::ClimateSystem;
use crate::heightmap::HeightMap;
use crate::scale::WorldScale;
use crate::sim::WaterLayer;
use macroquad::prelude::Vec2;

/// Agent system errors
#[derive(Debug)]
pub enum AgentError {
    InvalidSpawnPosition { position: Vec2, reason: String },
    AgentNotFound { agent_id: AgentId },
    SpatialBoundaryViolation { position: Vec2 },
    StateInconsistency { details: String },
}

/// Type-safe result types for agent operations
pub type AgentResult<T> = Result<T, AgentError>;
pub type SpawnResult = AgentResult<AgentId>;
pub type UpdateResult = AgentResult<()>;

/// Compile-time checked agent ID type with generational safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId {
    index: u32,      // Array index for O(1) access
    generation: u32, // ABA problem prevention
}

impl AgentId {
    /// Type-safe index access (no bounds checking in release)
    #[inline]
    pub fn index(self) -> usize {
        self.index as usize
    }

    /// Generation for validity checking
    #[inline]
    pub fn generation(self) -> u32 {
        self.generation
    }

    /// Create new agent ID (internal use only)
    pub(crate) fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }
}

/// Agent type enumeration for dispatch optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    NPC = 0,
    Creature = 1,
    Player = 2,
}

impl AgentType {
    /// Convert from u8 for packed storage
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(AgentType::NPC),
            1 => Some(AgentType::Creature),
            2 => Some(AgentType::Player),
            _ => None,
        }
    }

    /// Convert to u8 for packed storage
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// High-performance agent storage using SoA layout optimized for cache efficiency
pub struct AgentSystem {
    // Hot data - accessed every frame for rendering/collision (cache-friendly)
    positions: Vec<Vec2>,        // 8 bytes * n agents
    velocities: Vec<Vec2>,       // 8 bytes * n agents
    agent_types: Vec<AgentType>, // 1 byte * n agents (enum)
    bounds_radii: Vec<f32>,      // 4 bytes * n agents (simplified to radius)

    // Warm data - accessed during behavior updates
    health_values: Vec<f32>,  // 4 bytes * n agents
    energy_values: Vec<f32>,  // 4 bytes * n agents
    behavior_states: Vec<u8>, // 1 byte * n agents (state machine index)

    // Cold data - accessed occasionally
    agent_ids: Vec<AgentId>, // 8 bytes * n agents

    // Generation tracking for safe ID recycling
    generations: Vec<u32>,    // Generation counter per slot
    free_indices: Vec<usize>, // Recycled agent slots
    next_generation: u32,     // Global generation counter

    // Spatial indexing for efficient queries
    spatial_grid: SpatialGrid,

    // World boundaries for validation
    world_bounds: WorldBounds,
}

/// Spatial grid for O(1) agent neighbor queries
pub struct SpatialGrid {
    grid_size: usize,        // Grid dimensions (grid_size x grid_size)
    cell_size: f32,          // World units per grid cell
    cells: Vec<Vec<usize>>,  // Agent indices per cell
    agent_cells: Vec<usize>, // Current cell per agent
}

/// World boundary information
#[derive(Debug, Clone)]
pub struct WorldBounds {
    min: Vec2,
    max: Vec2,
}

impl WorldBounds {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, position: Vec2) -> bool {
        position.x >= self.min.x
            && position.x <= self.max.x
            && position.y >= self.min.y
            && position.y <= self.max.y
    }

    pub fn clamp(&self, position: Vec2) -> Vec2 {
        Vec2::new(
            position.x.clamp(self.min.x, self.max.x),
            position.y.clamp(self.min.y, self.max.y),
        )
    }
}

impl AgentSystem {
    /// Create new agent system with specified world bounds
    pub fn new(world_bounds: WorldBounds, initial_capacity: usize) -> Self {
        let grid_size = 32; // Start with 32x32 grid
        let world_size = world_bounds.max - world_bounds.min;
        let cell_size = world_size.x.max(world_size.y) / grid_size as f32;

        Self {
            positions: Vec::with_capacity(initial_capacity),
            velocities: Vec::with_capacity(initial_capacity),
            agent_types: Vec::with_capacity(initial_capacity),
            bounds_radii: Vec::with_capacity(initial_capacity),
            health_values: Vec::with_capacity(initial_capacity),
            energy_values: Vec::with_capacity(initial_capacity),
            behavior_states: Vec::with_capacity(initial_capacity),
            agent_ids: Vec::with_capacity(initial_capacity),
            generations: Vec::with_capacity(initial_capacity),
            free_indices: Vec::new(),
            next_generation: 0,
            spatial_grid: SpatialGrid {
                grid_size,
                cell_size,
                cells: vec![Vec::new(); grid_size * grid_size],
                agent_cells: Vec::with_capacity(initial_capacity),
            },
            world_bounds,
        }
    }

    /// Spawn new agent at specified position
    pub fn spawn_agent(
        &mut self,
        agent_type: AgentType,
        position: Vec2,
        radius: f32,
    ) -> SpawnResult {
        // Validate spawn position
        if !self.world_bounds.contains(position) {
            return Err(AgentError::InvalidSpawnPosition {
                position,
                reason: "Position outside world bounds".to_string(),
            });
        }

        // Get agent slot (reuse free slot or allocate new)
        let agent_index = if let Some(free_index) = self.free_indices.pop() {
            // Reuse free slot (generation already incremented in despawn)
            free_index
        } else {
            // Allocate new slot
            let new_index = self.positions.len();
            self.positions.push(Vec2::ZERO);
            self.velocities.push(Vec2::ZERO);
            self.agent_types.push(agent_type);
            self.bounds_radii.push(0.0);
            self.health_values.push(0.0);
            self.energy_values.push(0.0);
            self.behavior_states.push(0);
            self.agent_ids.push(AgentId::new(0, 0)); // Placeholder
            self.generations.push(self.next_generation);
            self.spatial_grid.agent_cells.push(0);
            new_index
        };

        // Create agent ID with current generation
        let agent_id = AgentId::new(agent_index as u32, self.generations[agent_index]);

        // Initialize agent data
        self.positions[agent_index] = position;
        self.velocities[agent_index] = Vec2::ZERO;
        self.agent_types[agent_index] = agent_type;
        self.bounds_radii[agent_index] = radius;
        self.health_values[agent_index] = 100.0; // Full health
        self.energy_values[agent_index] = 100.0; // Full energy
        self.behavior_states[agent_index] = 0; // Idle state
        self.agent_ids[agent_index] = agent_id;

        // Add to spatial grid
        self.add_to_spatial_grid(agent_index, position);

        // Increment global generation counter
        self.next_generation += 1;

        Ok(agent_id)
    }

    /// Remove agent from system
    pub fn despawn_agent(&mut self, agent_id: AgentId) -> AgentResult<()> {
        let agent_index = agent_id.index();

        // Validate agent exists and generation matches
        if agent_index >= self.agent_ids.len() {
            return Err(AgentError::AgentNotFound { agent_id });
        }

        if self.generations[agent_index] != agent_id.generation() {
            return Err(AgentError::AgentNotFound { agent_id });
        }

        // Remove from spatial grid
        self.remove_from_spatial_grid(agent_index);

        // Invalidate the agent by incrementing its generation
        self.generations[agent_index] += 1;

        // Mark slot as free for reuse
        self.free_indices.push(agent_index);

        Ok(())
    }

    /// Get agent position (hot path - frequently called)
    #[inline]
    pub fn get_position(&self, agent_id: AgentId) -> Option<Vec2> {
        let index = agent_id.index();
        if index < self.positions.len() && self.generations[index] == agent_id.generation() {
            Some(self.positions[index])
        } else {
            None
        }
    }

    /// Set agent position with spatial grid update
    pub fn set_position(&mut self, agent_id: AgentId, new_position: Vec2) -> AgentResult<()> {
        let index = agent_id.index();

        // Validate agent exists
        if index >= self.positions.len() || self.generations[index] != agent_id.generation() {
            return Err(AgentError::AgentNotFound { agent_id });
        }

        // Validate position is within world bounds
        if !self.world_bounds.contains(new_position) {
            return Err(AgentError::SpatialBoundaryViolation {
                position: new_position,
            });
        }

        // Update spatial grid if position changed significantly
        let old_position = self.positions[index];
        if (new_position - old_position).length_squared() > 0.01 {
            self.remove_from_spatial_grid(index);
            self.add_to_spatial_grid(index, new_position);
        }

        // Update position
        self.positions[index] = new_position;

        Ok(())
    }

    /// Get agents within radius of position (spatial query)
    pub fn agents_in_radius(&self, center: Vec2, radius: f32) -> Vec<AgentId> {
        let mut result = Vec::new();
        let radius_squared = radius * radius;

        // Calculate grid cell range to check
        let grid_x_min = ((center.x - radius) / self.spatial_grid.cell_size).floor() as i32;
        let grid_x_max = ((center.x + radius) / self.spatial_grid.cell_size).ceil() as i32;
        let grid_y_min = ((center.y - radius) / self.spatial_grid.cell_size).floor() as i32;
        let grid_y_max = ((center.y + radius) / self.spatial_grid.cell_size).ceil() as i32;

        // Check relevant grid cells
        for grid_y in grid_y_min..=grid_y_max {
            for grid_x in grid_x_min..=grid_x_max {
                if grid_x >= 0
                    && grid_x < self.spatial_grid.grid_size as i32
                    && grid_y >= 0
                    && grid_y < self.spatial_grid.grid_size as i32
                {
                    let cell_index =
                        (grid_y as usize) * self.spatial_grid.grid_size + (grid_x as usize);

                    // Check all agents in this cell
                    for &agent_index in &self.spatial_grid.cells[cell_index] {
                        let agent_position = self.positions[agent_index];
                        let distance_squared = (agent_position - center).length_squared();

                        if distance_squared <= radius_squared {
                            result.push(self.agent_ids[agent_index]);
                        }
                    }
                }
            }
        }

        result
    }

    /// Get total number of active agents
    pub fn agent_count(&self) -> usize {
        self.positions.len() - self.free_indices.len()
    }

    /// Validate agent ID is still valid
    pub fn is_valid_agent(&self, agent_id: AgentId) -> bool {
        let index = agent_id.index();
        index < self.generations.len() && self.generations[index] == agent_id.generation()
    }

    /// Internal: Add agent to spatial grid
    fn add_to_spatial_grid(&mut self, agent_index: usize, position: Vec2) {
        let grid_x = (position.x / self.spatial_grid.cell_size) as usize;
        let grid_y = (position.y / self.spatial_grid.cell_size) as usize;

        let grid_x = grid_x.min(self.spatial_grid.grid_size - 1);
        let grid_y = grid_y.min(self.spatial_grid.grid_size - 1);

        let cell_index = grid_y * self.spatial_grid.grid_size + grid_x;

        self.spatial_grid.cells[cell_index].push(agent_index);
        self.spatial_grid.agent_cells[agent_index] = cell_index;
    }

    /// Internal: Remove agent from spatial grid
    fn remove_from_spatial_grid(&mut self, agent_index: usize) {
        let cell_index = self.spatial_grid.agent_cells[agent_index];

        if let Some(pos) = self.spatial_grid.cells[cell_index]
            .iter()
            .position(|&x| x == agent_index)
        {
            self.spatial_grid.cells[cell_index].swap_remove(pos);
        }
    }
}

/// Extension trait for HeightMap integration
pub trait HeightMapAgentExtensions {
    /// Get elevation with agent-optimized interpolation
    fn agent_elevation(&self, world_pos: Vec2) -> f32;

    /// Check if position is navigable for agent movement
    fn is_navigable(&self, world_pos: Vec2, agent_type: AgentType) -> bool;

    /// Get movement cost for pathfinding integration
    fn movement_cost(&self, from: Vec2, to: Vec2, agent_type: AgentType) -> f32;
}

impl HeightMapAgentExtensions for HeightMap {
    #[inline]
    fn agent_elevation(&self, world_pos: Vec2) -> f32 {
        // Convert world position (0.0-1.0) to grid coordinates
        let grid_x = world_pos.x * (self.width() as f32 - 1.0);
        let grid_y = world_pos.y * (self.height() as f32 - 1.0);

        // Bilinear interpolation
        let x0 = grid_x.floor() as usize;
        let y0 = grid_y.floor() as usize;
        let x1 = (x0 + 1).min(self.width() - 1);
        let y1 = (y0 + 1).min(self.height() - 1);

        let fx = grid_x - x0 as f32;
        let fy = grid_y - y0 as f32;

        let v00 = self.get(x0, y0);
        let v01 = self.get(x0, y1);
        let v10 = self.get(x1, y0);
        let v11 = self.get(x1, y1);

        let v0 = v00 * (1.0 - fx) + v10 * fx;
        let v1 = v01 * (1.0 - fx) + v11 * fx;

        v0 * (1.0 - fy) + v1 * fy
    }

    fn is_navigable(&self, world_pos: Vec2, agent_type: AgentType) -> bool {
        let elevation = self.agent_elevation(world_pos);
        match agent_type {
            AgentType::Creature => elevation > 0.1 && elevation < 0.9, // Land creatures
            AgentType::NPC => elevation > 0.2 && elevation < 0.8,      // More restrictive
            AgentType::Player => elevation > 0.0,                      // Can go anywhere
        }
    }

    fn movement_cost(&self, from: Vec2, to: Vec2, agent_type: AgentType) -> f32 {
        let from_elevation = self.agent_elevation(from);
        let to_elevation = self.agent_elevation(to);

        // Base cost is distance
        let distance = (to - from).length();

        // Add elevation change cost
        let elevation_change = (to_elevation - from_elevation).abs();
        let elevation_cost = match agent_type {
            AgentType::Creature => elevation_change * 2.0, // Creatures struggle with elevation
            AgentType::NPC => elevation_change * 1.5,      // NPCs moderately affected
            AgentType::Player => elevation_change * 0.5,   // Players handle elevation well
        };

        distance + elevation_cost
    }
}

/// Context provider that integrates all simulation systems
pub struct SimulationContext<'a> {
    pub heightmap: &'a HeightMap,
    pub water_layer: &'a WaterLayer,
    pub climate_system: &'a ClimateSystem,
    pub world_scale: &'a WorldScale,
    pub tick_count: u64,
    pub time_delta: f32,
}

impl<'a> SimulationContext<'a> {
    /// Create context from simulation components
    pub fn new(
        heightmap: &'a HeightMap,
        water_layer: &'a WaterLayer,
        climate_system: &'a ClimateSystem,
        world_scale: &'a WorldScale,
        tick_count: u64,
        time_delta: f32,
    ) -> Self {
        Self {
            heightmap,
            water_layer,
            climate_system,
            world_scale,
            tick_count,
            time_delta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_system_creation() {
        let bounds = WorldBounds::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let agent_system = AgentSystem::new(bounds, 10);

        assert_eq!(agent_system.agent_count(), 0);
    }

    #[test]
    fn agent_spawn_and_despawn() {
        let bounds = WorldBounds::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let mut agent_system = AgentSystem::new(bounds, 10);

        // Spawn agent
        let agent_id = agent_system
            .spawn_agent(AgentType::NPC, Vec2::new(50.0, 50.0), 1.0)
            .unwrap();

        assert_eq!(agent_system.agent_count(), 1);
        assert!(agent_system.is_valid_agent(agent_id));

        // Check position
        let position = agent_system.get_position(agent_id).unwrap();
        assert_eq!(position, Vec2::new(50.0, 50.0));

        // Despawn agent
        agent_system.despawn_agent(agent_id).unwrap();
        assert_eq!(agent_system.agent_count(), 0);
        assert!(!agent_system.is_valid_agent(agent_id));
    }

    #[test]
    fn agent_spatial_queries() {
        let bounds = WorldBounds::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let mut agent_system = AgentSystem::new(bounds, 10);

        // Spawn agents at different positions
        let agent1 = agent_system
            .spawn_agent(AgentType::NPC, Vec2::new(10.0, 10.0), 1.0)
            .unwrap();
        let agent2 = agent_system
            .spawn_agent(AgentType::NPC, Vec2::new(12.0, 12.0), 1.0)
            .unwrap();
        let agent3 = agent_system
            .spawn_agent(AgentType::NPC, Vec2::new(50.0, 50.0), 1.0)
            .unwrap();

        // Query agents near (10, 10)
        let nearby_agents = agent_system.agents_in_radius(Vec2::new(10.0, 10.0), 5.0);

        // Should find agent1 and agent2, but not agent3
        assert_eq!(nearby_agents.len(), 2);
        assert!(nearby_agents.contains(&agent1));
        assert!(nearby_agents.contains(&agent2));
        assert!(!nearby_agents.contains(&agent3));
    }

    #[test]
    fn agent_position_validation() {
        let bounds = WorldBounds::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let mut agent_system = AgentSystem::new(bounds, 10);

        // Try to spawn agent outside bounds
        let result = agent_system.spawn_agent(
            AgentType::NPC,
            Vec2::new(150.0, 150.0), // Outside bounds
            1.0,
        );

        assert!(result.is_err());
        if let Err(AgentError::InvalidSpawnPosition { position, .. }) = result {
            assert_eq!(position, Vec2::new(150.0, 150.0));
        }
    }
}
