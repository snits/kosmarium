# Phase 4A: Single-Scale Agent Specification

ABOUTME: Defines 300 individual agent representation for biome integration
ABOUTME: Single-scale implementation plan for Phase 4A terrain-aware agent behaviors

## Agent Definition

**300 Individual Creatures/People** at consistent terrain scale with basic biome-aware behaviors.

### Core Agent Representation
```rust
struct Agent {
    // Hot data (frequently accessed)
    position: Vec2,
    energy: f32,
    current_goal: AgentGoal,
    
    // Warm data (cached biome properties)
    cached_movement_cost: f32,
    cached_resource_density: f32,
    cached_visibility: f32,
    
    // Cold data (infrequently accessed)
    memory: BasicMemory,
}

enum AgentGoal {
    SeekFood,
    SeekWater,
    Explore,
    Rest,
}
```

## Biome Integration Points

### Movement System
- **API**: `BiomeMap::movement_cost(position)` 
- **Behavior**: Avoid high-cost terrain, prefer paths through favorable biomes
- **Implementation**: A* pathfinding with biome-aware costs

### Resource-Seeking System  
- **API**: `BiomeMap::resource_density(position)`
- **Behavior**: Move toward areas with higher resource availability when energy low
- **Implementation**: Gradient following with spatial memory

### Line-of-Sight System
- **API**: `BiomeMap::visibility_multiplier(position)`
- **Behavior**: Detection ranges affected by terrain (forests reduce visibility, plains increase it)
- **Implementation**: Ray-casting with biome-modified detection distances

## Behavioral Rules (Simple State Machine)

1. **Energy > 80%**: Explore (random walk with biome preferences)
2. **Energy 40-80%**: Continue current activity or seek better terrain
3. **Energy 20-40%**: Seek food using `resource_density()` gradients
4. **Energy < 20%**: Emergency resource seeking, ignore movement costs
5. **Energy restored**: Return to exploration behavior

## Performance Targets

- **Total Agent Processing**: <5ms per 10Hz tick (500μs per agent budget)
- **Memory Layout**: Structure-of-Arrays for cache efficiency
- **Spatial Queries**: O(1) neighbor detection using existing spatial grid
- **Biome Queries**: Cached per agent, updated only on cell boundary crossings

## Implementation Strategy

### Phase 4A.1: Basic Movement (Week 1)
- Integrate `movement_cost()` with existing spatial system
- Implement simple A* pathfinding with biome costs
- Validate 300 agents moving without performance degradation

### Phase 4A.2: Resource Behaviors (Week 2)  
- Add energy system and resource-seeking using `resource_density()`
- Implement gradient-following resource location
- Add basic state machine (energy thresholds → behavior changes)

### Phase 4A.3: Visibility & Polish (Week 3)
- Integrate `visibility_multiplier()` for detection/interaction ranges
- Performance optimization and comprehensive testing
- Validate all biome integration APIs working with agent behaviors

## Success Criteria

- [ ] 300 agents navigating terrain using biome movement costs
- [ ] Agents seeking resources in biome-appropriate locations  
- [ ] Visibility-based interactions working across different biomes
- [ ] Performance budget maintained (<5ms total agent processing)
- [ ] All biome integration APIs validated through agent usage

## Deferred to Phase 4C

**Multi-Scale Agent Architecture**: Individual → Tribal → National scale representations archived for future implementation after Phase 4A validation complete.

---

**Approved by**: project-scope-guardian, technical-feasibility-assessor, simulation-designer  
**Implementation Ready**: 2025-08-01  
**Next Phase**: Begin Phase 4A.1 Basic Movement implementation