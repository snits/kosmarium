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

**Decision**: Use Diamond-Square algorithm for heightmap generation with trait-based architecture

**Context**: Need realistic-looking terrain with controllable parameters and extensibility

**Rationale**:
- **Realism**: Produces natural-looking terrain with realistic features
- **Performance**: O(n²) complexity, suitable for real-time generation
- **Deterministic**: Seeded randomization allows reproducible worlds
- **Well-understood**: Established algorithm with known characteristics
- **Extensible**: Trait-based TerrainGenerator allows additional algorithms

**Alternatives Considered**:
- Perlin/Simplex noise: More computationally expensive, less terrain-like results
- Voronoi diagrams: Good for biomes but poor for elevation
- Random noise: Too chaotic, no realistic terrain features

**Trade-offs**:
- ✅ Produces realistic terrain patterns
- ✅ Fast enough for interactive use
- ✅ Trait architecture supports future algorithms (GSD, Perlin, etc.)
- ❌ Can produce artifacts at boundaries
- ❌ Limited control over specific terrain features

**Status**: ✅ Implemented with configurable parameters (roughness, persistence, corners)

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
- ❌ Compilation error with `gen` reserved keyword (resolved)
- ❌ Potential compatibility issues with older dependencies

**Status**: ✅ Implemented and working with all current dependencies

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

## ADR-007: TUI Interface with Ratatui

**Decision**: Use ratatui for interactive terminal user interface with viewport navigation

**Context**: Need interactive exploration of large generated terrains beyond static ASCII output

**Rationale**:
- **Interactive exploration**: WASD navigation, zoom levels, mini-map for spatial orientation
- **Rich visualization**: Status bars, legends, multiple information layers
- **Terminal compatibility**: Works in standard terminal environments without GUI dependencies
- **Performance**: Efficient rendering with smart redraw logic for responsive navigation

**Implementation details**:
- Viewport system for navigating large maps with bounds checking
- Mini-map with viewport highlighting and cursor position indicators
- Zoom levels (1:1, 1:2, 1:4) via render sampling
- Multi-panel layout: main terrain view + sidebar (mini-map + legend)
- Status bar with position, terrain analysis, and control hints

**Trade-offs**:
- ✅ Excellent user experience for terrain exploration
- ✅ Professional visualization quality in terminal
- ✅ Extensible for additional simulation layers (agents, beliefs, etc.)
- ❌ More complex than simple ASCII output
- ❌ Additional dependency (ratatui, tokio)

**Status**: ✅ Implemented with comprehensive navigation and visualization features

## ADR-008: Command-Line Parameter System

**Decision**: Use clap for command-line argument parsing to enable terrain generation experimentation

**Context**: Need easy parameter experimentation without code recompilation

**Rationale**:
- **Rapid experimentation**: Change seed, roughness, persistence, dimensions via CLI
- **Reproducible results**: Seed parameter enables sharing specific terrain configurations
- **Educational tool**: Parameter exploration helps understand algorithm behavior
- **User-friendly**: Clear parameter descriptions and defaults

**Parameters implemented**:
- `--seed`: Random seed for reproducible terrain generation
- `--roughness`: Terrain chaos level (0.0-1.0)  
- `--persistence`: Detail persistence across scales (0.0-1.0)
- `--width/--height`: Map dimensions in cells
- `--ascii`: Legacy ASCII mode vs default TUI

**Trade-offs**:
- ✅ Enables rapid terrain generation experimentation
- ✅ No recompilation needed for parameter changes
- ✅ Professional CLI interface with help system
- ❌ Additional dependency (clap)
- ❌ Parameter validation complexity

**Status**: ✅ Implemented with comprehensive parameter set for Diamond-Square experimentation

## ADR-009: Trait-Based Terrain Generator Architecture

**Decision**: Use trait-based architecture for terrain generation algorithms

**Context**: Need extensible system supporting multiple terrain generation approaches

**Rationale**:
- **Algorithm diversity**: Support Diamond-Square, GSD, Perlin, Simplex noise
- **Experimentation**: Easy to swap algorithms for comparison
- **Testability**: Each generator can be unit tested independently
- **Future-proofing**: New algorithms can be added without breaking existing code

**Architecture pattern**:
```rust
pub trait TerrainGenerator {
    type Config: Clone + Default;
    fn generate(&self, width: usize, height: usize, config: &Self::Config) -> Vec<Vec<f32>>;
    fn name(&self) -> &'static str;
    fn supports_arbitrary_dimensions(&self) -> bool;
}
```

**Trade-offs**:
- ✅ Highly extensible and maintainable
- ✅ Clean separation between algorithms and application logic
- ✅ Easy to benchmark and compare different approaches
- ❌ Slight abstraction overhead
- ❌ More complex than single-algorithm approach

**Status**: ✅ Implemented with Diamond-Square generator, ready for additional algorithms

## ADR-010: TUI-First, Migration-Ready Rendering Strategy

**Decision**: Continue with TUI as primary interface while architecting for eventual graphics migration

**Context**: Complex multi-layer, real-time simulation will eventually exceed TUI visualization capabilities

**Analysis from rendering-engineer**:
- **TUI → 2D sprites**: Medium effort (2-3 weeks), good viewport system compatibility
- **TUI → 3D**: High effort (4-6 weeks), requires complete coordinate system redesign
- **Multi-backend support**: Very high effort (6-8 weeks), exponential maintenance complexity

**Analysis from senior-engineer**:
- **Multi-backend maintenance cost**: Exponential testing complexity, abstraction layer poison, crushing cognitive load
- **Tech debt assessment**: 3x codebase size, dependency hell, 60-70% development time on multi-backend testing
- **Recommendation**: Don't maintain multiple backends simultaneously - focus on simulation, migrate cleanly when needed

**Analysis from ux-design-expert**:
- **TUI breaking point**: When visualization complexity exceeds cognitive benefits (3-4 layers + real-time + spatial relationships)
- **Current sweet spot**: Single layer focus with educational value
- **Migration triggers**: Multiple interactive layers, real-time dynamics requiring visual continuity

**Rationale**:
- **TUI skills have broad value**: Roguelikes, rapid prototyping, developer tools, system administration
- **Educational focus**: Terminal interface forces focus on core mechanics over presentation
- **Migration readiness**: Visualization-agnostic data structures enable clean transition
- **Complexity management**: Avoid premature optimization while preparing for inevitable needs

**Implementation Strategy**:
```rust
// Visualization-agnostic simulation data
trait SimulationLayer {
    fn get_data_at(&self, x: usize, y: usize) -> LayerData;
    fn get_region(&self, bounds: Rect) -> RegionData;
    fn get_changes_since(&self, tick: u64) -> ChangeSet;
}

// Clean renderer abstraction
trait SimulationRenderer {
    fn render_frame(&mut self, layers: &[Box<dyn SimulationLayer>]);
    fn handle_input(&mut self) -> Option<UserAction>;
    fn supports_layer_composition(&self) -> bool;
    fn supports_animation(&self) -> bool;
}
```

**TUI Enhancement Plan**:
- Add layer selection hotkeys (show terrain OR water OR agents individually)
- Implement visualization-agnostic data structures
- Maintain clear separation between simulation logic and rendering

**Migration Criteria**:
- **Stay with TUI while**: Single primary layer focus, turn-based/slow simulation, educational emphasis
- **Plan migration when**: 3+ interactive layers essential, real-time dynamics need continuity, spatial relationships become core

**Trade-offs**:
- ✅ Builds broadly applicable TUI development skills
- ✅ Forces focus on simulation mechanics over graphics
- ✅ Cross-platform compatibility with minimal dependencies
- ✅ Clean migration path without multi-backend complexity
- ❌ Will eventually hit visualization limits for complex systems
- ❌ ASCII representation limits spatial relationship clarity

**Status**: ✅ Strategy decided, TUI development continues with migration-ready architecture

## Pending Decisions

### PD-001: Simulation Engine Architecture (Expert Guidance Available)
**Context**: Need core tick loop and modular system architecture for dynamic simulation
**Expert recommendation (simulation-engineer)**:
- **System trait architecture**: Modular components with dependency tracking
- **Event-driven communication**: Loose coupling between simulation layers
- **Layer-based world state**: Terrain, water, climate, agents, culture layers
- **Double buffering**: For systems needing previous + current state

### PD-002: Multi-Layer Environmental System (Expert Guidance Available)  
**Context**: Need climate, biome, and environmental data layers
**Expert recommendation (world-generation-architect)**:
- **Environmental data layers**: Temperature, precipitation, humidity, wind velocity
- **Climate simulation pipeline**: Realistic temperature/precipitation modeling
- **Biome assignment**: Whittaker classification based on environmental conditions
- **Dynamic environmental changes**: Support for seasonal variation and climate events

### PD-003: Advanced Terrain Generation Evolution (Expert Guidance Available)
**Context**: Evolution beyond Diamond-Square to geological realism
**Expert recommendation (world-generation-architect)**:
- **Generalized Stochastic Diffusion (GSD)**: Phase 1 priority for geological realism
- **Hybrid pipeline**: Layered erosion, uplift, wind/rain post-processors  
- **Performance architecture**: Hierarchical LOD, streaming, spatial partitioning
- **Dynamic terrain modification**: Real-time erosion and agent terraforming support

### PD-004: Agent System Architecture (Expert Guidance Available)
**Context**: Agent movement, settlement, cultural behaviors for Phase 3-4
**Expert recommendations**:
- **SlotMap storage** (simulation-engineer): Stable IDs with packed memory layout
- **Spatial partitioning** (both experts): O(1) neighbor queries, performance scaling
- **Relationship-driven systems** (social-systems-designer): Cultural influence through personal relationships
- **Strategic resource management** (game-design-strategist): Faith points, belief scarcity, meaningful tradeoffs

### PD-005: Cultural/Belief System Implementation (Expert Guidance Available)
**Context**: Phase 4 cultural memory, belief propagation, myth creation systems
**Expert recommendations**:
- **Event-driven myth generation** (social-systems-designer): Major events become myth raw material
- **Strategic belief resources** (game-design-strategist): Faith points, theological coherence costs
- **Influence maps** (simulation-engineer): Spatial tracking of belief strength with SmallVec optimization
- **Bottom-up emergence**: Individual agent relationships create cultural patterns