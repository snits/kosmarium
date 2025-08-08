# Technical Implementation Roadmaps

ABOUTME: Detailed technical guidance from expert agents for implementing advanced simulation systems
ABOUTME: Comprehensive roadmaps for terrain generation evolution and simulation architecture

## World Generation Architect Roadmap

### Phase 1: Generalized Stochastic Subdivision (GSS) Implementation (2-3 weeks)

**Core GSS Algorithm Architecture:**
```rust
pub struct GSSGenerator {
    seed: u64,
    tectonic_config: TectonicConfig,
    erosion_config: ErosionConfig,
}

pub struct GSSConfig {
    pub iterations: usize,              // Simulation time steps
    pub initial_uplift: f32,            // Starting tectonic energy
    pub subdivision_rate: f32,          // Stochastic subdivision granularity
    pub precipitation_map: Option<PrecipitationField>,
    pub thermal_erosion_angle: f32,     // Angle of repose for sediment
    pub hydraulic_erosion_strength: f32, // Water-based erosion intensity
}
```

**Implementation Steps:**
1. **Tectonic Uplift Phase**: Random uplift events based on geological hotspots
2. **Thermal Erosion Phase**: Simulate slope-based material movement using angle of repose
3. **Hydraulic Erosion Phase**: Water flow simulation with sediment transport
4. **Subdivision Refinement Phase**: Apply stochastic subdivision at multiple resolution levels

**Key Algorithm Components:**
- **Uplift System**: Probabilistic elevation increases at tectonic hotspots
- **Flow Field Calculation**: Water accumulation and flow direction mapping
- **Sediment Transport**: Capacity-based erosion and deposition
- **Thermal Stability**: Slope-angle based material redistribution

### Phase 2: Multi-Layer Environmental System (1-2 months)

**Environmental Data Structure:**
```rust
pub struct EnvironmentalLayers {
    pub elevation: Grid2D<f32>,
    pub temperature: Grid2D<f32>,
    pub precipitation: Grid2D<f32>,
    pub humidity: Grid2D<f32>,
    pub wind_velocity: Grid2D<Vector2>,
    pub soil_composition: Grid2D<SoilType>,
    pub drainage: Grid2D<f32>,
    pub biome: Grid2D<BiomeType>,
}
```

**Climate Simulation Models:**
```rust
pub struct TemperatureModel {
    pub latitude_gradient: f32,         // Temperature change per latitude degree
    pub elevation_lapse_rate: f32,      // Temperature decrease per elevation unit
    pub ocean_moderation: f32,          // Coastal temperature moderation
    pub seasonal_amplitude: f32,        // Seasonal temperature variation
}

pub struct PrecipitationModel {
    pub prevailing_winds: Vector2,      // Primary wind direction
    pub orographic_factor: f32,         // Mountain-induced precipitation
    pub rain_shadow_coefficient: f32,   // Leeward precipitation reduction
    pub evaporation_rate: f32,          // Water body evaporation
}
```

**Implementation Sequence:**
1. **Base Temperature**: Calculate from latitude and elevation using realistic lapse rates
2. **Wind Patterns**: Simulate prevailing winds based on pressure differentials and Coriolis effect
3. **Precipitation Calculation**: Model orographic effects, rain shadows, and coastal influences
4. **Biome Assignment**: Use Whittaker biome classification based on temperature/precipitation combinations

### Phase 3: Dynamic Terrain Modification (3-4 months)

**Real-Time Erosion System:**
```rust
pub struct DynamicErosion {
    active_flows: Vec<WaterFlow>,
    erosion_events: VecDeque<ErosionEvent>,
    sediment_deposits: HashMap<(usize, usize), SedimentLayer>,
}

pub enum ErosionEvent {
    RainfallEvent { location: Point2D, intensity: f32, duration: f32 },
    FloodEvent { river_system: RiverNetwork, magnitude: f32 },
    LandslideEvent { trigger_location: Point2D, material_volume: f32 },
}
```

**Performance Architecture:**
```rust
pub struct HierarchicalTerrain {
    pub levels: Vec<TerrainLevel>,      // Multiple resolution levels
    pub active_level: usize,            // Current detail level for processing
    pub transition_zones: Vec<Region>,  // Smooth transitions between levels
}
```

**Multi-Scale Processing:**
- **Continental Scale** (1km/cell): Tectonic processes, major climate patterns
- **Regional Scale** (100m/cell): River systems, biome transitions, weather systems  
- **Local Scale** (10m/cell): Detailed erosion, agent interactions, local hydrology
- **Micro Scale** (1m/cell): Construction sites, detailed resource deposits

### Phase 4: Advanced Features (6+ months)

**Agent-Terrain Integration:**
```rust
pub trait TerrainQuery {
    fn elevation_at(&self, location: Point2D) -> f32;
    fn slope_at(&self, location: Point2D) -> f32;
    fn biome_at(&self, location: Point2D) -> BiomeType;
    fn water_access(&self, location: Point2D, radius: f32) -> Vec<WaterSource>;
    fn resource_availability(&self, location: Point2D) -> ResourceMap;
    fn traversability(&self, from: Point2D, to: Point2D, agent_type: AgentType) -> PathCost;
}
```

## Simulation Engineer Roadmap

### Phase 1: Core Simulation Architecture (Immediate - 2-3 weeks)

**Tick System Design:**
```rust
pub struct SimulationEngine {
    world: World,
    systems: Vec<Box<dyn System>>,
    tick_counter: u64,
    delta_time: f32,
    paused: bool,
}

pub trait System: Send + Sync {
    fn name(&self) -> &'static str;
    fn dependencies(&self) -> Vec<&'static str>;
    fn update(&mut self, world: &mut World, delta_time: f32) -> Result<(), SimError>;
    fn reset(&mut self);
}
```

**World State Architecture:**
```rust
pub struct World {
    pub terrain: TerrainLayer,
    pub water: WaterLayer,
    pub climate: ClimateLayer,
    pub agents: AgentLayer,
    pub culture: CultureLayer,
    pub events: EventQueue,
    dimensions: (usize, usize),
}
```

**Key Design Decisions:**
- **Fixed timestep with delta time**: Ensures deterministic simulation
- **System ordering**: Systems execute in dependency order (water → erosion → climate → agents)
- **Layer-based separation**: Each simulation aspect gets its own data structure
- **Event queue**: Centralized communication between systems

### Phase 2: Environmental Systems (1-2 months)

**Water Flow System Implementation:**
```rust
pub struct WaterFlowSystem {
    flow_rate: f32,
    evaporation_rate: f32,
}

pub struct WaterLayer {
    depth: Grid2D<f32>,
    velocity: Grid2D<Vec2>,    // Flow direction and speed
    sediment: Grid2D<f32>,     // Carried sediment for erosion
}
```

**Climate System Implementation:**
```rust
pub struct ClimateLayer {
    temperature: Grid2D<f32>,
    precipitation: Grid2D<f32>,
    humidity: Grid2D<f32>,
    wind: Grid2D<Vec2>,
}
```

### Phase 3: Agent Foundation (Months 4-5)

**Agent System Architecture:**
```rust
pub struct AgentLayer {
    agents: SlotMap<AgentId, Agent>,    // Stable IDs, packed storage
    spatial_index: SpatialGrid<AgentId>,
    population_by_region: Grid2D<u32>,  // Fast population queries
}

pub struct Agent {
    position: (f32, f32),               // Sub-cell precision
    velocity: Vec2,
    culture: CultureProfile,
    needs: ResourceNeeds,
    memory: AgentMemory,                // Beliefs, experiences
}
```

**Spatial Partitioning:**
```rust
pub struct SpatialGrid<T> {
    cells: Vec<Vec<Vec<T>>>,    // Grid of entity lists
    cell_size: f32,
    dimensions: (usize, usize),
}
```

### Phase 4: Cultural Systems (Months 6+)

**Cultural Data Structures:**
```rust
pub struct CultureLayer {
    beliefs: HashMap<BeliefId, Belief>,
    influence_map: Grid2D<CultureInfluence>,
    sacred_sites: Vec<SacredSite>,
    trade_routes: Graph<TradeRoute>,
}

pub struct CultureInfluence {
    dominant_beliefs: SmallVec<[BeliefId; 4]>,  // Most regions have 1-4 beliefs
    influence_strength: SmallVec<[f32; 4]>,
}
```

**Event System Design:**
```rust
pub enum Event {
    // Environmental events
    WaterLevelChanged { pos: (usize, usize), old: f32, new: f32 },
    ErosionOccurred { pos: (usize, usize), amount: f32 },
    ClimateShift { region: Region, temp_delta: f32 },
    
    // Agent events  
    AgentMoved { id: AgentId, from: (usize, usize), to: (usize, usize) },
    SettlementFounded { pos: (usize, usize), founder: AgentId },
    
    // Cultural events
    BeliefSpread { from: (usize, usize), to: (usize, usize), belief: BeliefId },
    
    // System events
    TickComplete(u64),
}
```

## Performance Optimization Strategy

### Memory Management:
```rust
pub struct Grid2D<T> {
    data: Vec<T>,
    width: usize,
    height: usize,
    // Store as single Vec for cache locality
}

// Sparse environmental data for efficient storage
pub struct SparseEnvironmentalLayer<T> {
    default_value: T,
    sparse_data: HashMap<(usize, usize), T>,
    chunk_size: usize,
}
```

### Computational Budgeting:
```rust
pub struct ComputationBudget {
    pub max_terrain_updates_per_frame: usize,
    pub erosion_calculation_budget: Duration,
    pub climate_simulation_frequency: Duration,
    pub agent_terrain_query_limit: usize,
}
```

## Integration Strategy

### TUI Integration:
```rust
pub enum RenderLayer {
    Terrain,
    Water,
    Climate,
    Agents,
    Culture,
    Combined,
}

impl SimulationRenderer {
    pub fn render_to_terminal(&self, world: &World, viewport: ViewPort) {
        match self.current_layer {
            RenderLayer::Terrain => self.render_heightmap(&world.terrain, viewport),
            RenderLayer::Water => self.render_water_layer(&world.water, viewport),
            RenderLayer::Agents => self.render_agents(&world.agents, viewport),
            // ... etc
        }
    }
}
```

### Backwards Compatibility:
```rust
impl Simulation {
    pub fn new(heightmap: Vec<Vec<f32>>) -> Self {
        let terrain = TerrainLayer::from_heightmap(heightmap);
        let world = World::new(terrain);
        let engine = SimulationEngine::new(world);
        
        Self {
            engine,
            legacy_heightmap: None, // Keep for backwards compat
        }
    }
}
```

## Implementation Priority Summary

**Immediate (2-3 weeks)**: Core tick loop + water system architecture
**Short-term (1-2 months)**: Environmental systems + basic climate modeling  
**Medium-term (3-4 months)**: Agent systems + spatial partitioning
**Long-term (6+ months)**: Cultural systems + belief propagation

This technical roadmap provides concrete implementation guidance for evolving from static terrain generation to complex dynamic simulation systems with emergent cultural behaviors.