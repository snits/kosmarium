# Architecture Decisions

ABOUTME: Key design choices, rationale, and architectural patterns for the simulation prototype  
ABOUTME: Documents why specific technical approaches were chosen and their trade-offs

## ADR-001: Modular Architecture with Clear Separation

**Decision**: Split codebase into distinct modules (worldgen, sim, render) with single responsibilities

**Context**: Early prototype needs structure that can scale as complexity grows

**Rationale**:
- **Maintainability**: Each module has clear boundaries and responsibilities
- **Testability**: Modules can be unit tested in isolation  
- **Extensibility**: New features can be added without touching unrelated code
- **Reusability**: Components can be swapped (e.g., different rendering backends)

**Trade-offs**:
- ✅ Clear separation of concerns
- ✅ Easy to reason about data flow
- ❌ Slight overhead in module boundaries
- ❌ More files to navigate initially

**Status**: Implemented and working well

## ADR-002: Diamond-Square Algorithm for Terrain Generation

**Decision**: Use Diamond-Square algorithm for heightmap generation (placeholder implementation currently)

**Context**: Need realistic-looking terrain with controllable parameters

**Rationale**:
- **Realism**: Produces natural-looking terrain with realistic features
- **Performance**: O(n²) complexity, suitable for real-time generation
- **Deterministic**: Seeded randomization allows reproducible worlds
- **Well-understood**: Established algorithm with known characteristics

**Alternatives Considered**:
- Perlin/Simplex noise: More computationally expensive, less terrain-like results
- Voronoi diagrams: Good for biomes but poor for elevation
- Random noise: Too chaotic, no realistic terrain features

**Trade-offs**:
- ✅ Produces realistic terrain patterns
- ✅ Fast enough for interactive use
- ❌ Can produce artifacts at boundaries
- ❌ Limited control over specific terrain features

**Status**: Architecture in place, algorithm implementation pending

## ADR-003: Crossterm for Terminal Rendering

**Decision**: Use crossterm crate for colored ASCII terminal output

**Context**: Need cross-platform visualization that works without additional dependencies

**Rationale**:
- **Cross-platform**: Works on Windows, macOS, Linux consistently
- **Lightweight**: Minimal dependency footprint
- **Feature-rich**: Supports colors, cursor movement, input handling
- **Active maintenance**: Well-maintained crate with good documentation

**Alternatives Considered**:
- termion: Unix-only, limits cross-platform compatibility
- Raw ANSI codes: Platform-specific issues, harder to maintain
- GUI frameworks (egui, iced): Overkill for prototype, adds complexity

**Trade-offs**:
- ✅ Works everywhere without platform-specific code
- ✅ Fast rendering for text-based output
- ❌ Limited to terminal resolution
- ❌ ASCII art has visualization constraints

**Status**: Implemented and functional

## ADR-004: Rust 2024 Edition

**Decision**: Use Rust 2024 edition for the project

**Context**: New project with opportunity to use latest language features

**Rationale**:
- **Modern syntax**: Access to latest language improvements
- **Future-proofing**: Ensures compatibility with ecosystem direction
- **Performance**: Potential optimizations in newer editions
- **Developer experience**: Better error messages and tooling

**Trade-offs**:
- ✅ Access to latest Rust features and improvements
- ✅ Better alignment with current ecosystem
- ❌ Compilation error with `gen` reserved keyword
- ❌ Potential compatibility issues with older dependencies

**Status**: Implemented but causing compilation issues (needs resolution)

## ADR-005: Single-threaded Architecture (Current)

**Decision**: Keep initial implementation single-threaded

**Context**: Prototype phase focusing on correctness over performance

**Rationale**:
- **Simplicity**: Easier to debug and reason about
- **Development velocity**: No synchronization complexity
- **Sufficient performance**: Terminal rendering is not CPU-intensive
- **Premature optimization**: Complexity not justified at prototype stage

**Future considerations**:
- Parallel worldgen for large maps
- Async rendering for better responsiveness
- Multi-threaded simulation updates

**Trade-offs**:
- ✅ Simple mental model, easier debugging
- ✅ Faster development iteration
- ❌ Won't scale to very large simulations
- ❌ UI blocking during generation

**Status**: Appropriate for current phase, revisit in Phase 2

## ADR-006: Elevation-based Terrain Visualization

**Decision**: Map elevation values to colored ASCII symbols

**Context**: Need intuitive visual representation of heightmap data

**Rationale**:
- **Intuitive mapping**: Lower elevations = water/blue, higher = mountains/red
- **Good contrast**: Color scheme provides clear terrain distinction
- **Extensible**: Easy to add more elevation bands or different color schemes
- **Accessible**: Works in standard terminal environments

**Mapping chosen**:
```
< 0.2: '.' (blue) - Water/sea level
0.2-0.4: '~' (cyan) - Coastline/beaches  
0.4-0.6: '^' (green) - Plains/grassland
0.6-0.8: '#' (yellow) - Hills/forests
> 0.8: '@' (red) - Mountains/peaks
```

**Trade-offs**:
- ✅ Immediately recognizable terrain patterns
- ✅ Good visual hierarchy
- ❌ Limited to 5 elevation bands
- ❌ Color-blind accessibility not considered

**Status**: Implemented and effective for visualization

## Pending Decisions

### PD-001: Agent/Entity System Architecture
**Context**: Phase 2 will require adding mobile entities to the simulation
**Options under consideration**:
- Entity-Component-System (ECS) pattern
- Simple struct-based agents with behavior traits
- Actor model with message passing

### PD-002: Persistence/Serialization Strategy  
**Context**: Need to save/load simulation states
**Options under consideration**:
- JSON for human readability
- Binary format for performance
- Database for complex queries

### PD-003: Performance Optimization Approach
**Context**: Will need to handle larger maps efficiently
**Options under consideration**:
- Spatial partitioning for agent updates
- Level-of-detail for rendering
- Streaming/chunked world generation