# Experiment Roadmap

ABOUTME: Ideas to explore, concepts to test, and interesting experiments for the simulation prototype
ABOUTME: Tracks what's been tried, what worked, and what to investigate next

## Current Exploration

**Theme**: Professional-grade multi-physics simulation with scale-aware architecture and dimensional analysis foundations

### Active Experiments

#### 🔧 Basic Infrastructure (Getting things working)
- [x] Fix compilation issue (reserved keyword error)
- [x] Get basic Diamond-Square algorithm working (currently just random noise)
- [x] Make map size configurable via command line

#### 🌍 World Generation Experiments
- [ ] Compare different noise algorithms (Diamond-Square vs Perlin vs Simplex)
- [ ] Experiment with terrain post-processing (erosion simulation, river carving)
- [ ] Try procedural biome placement based on elevation/temperature
- [ ] Generate realistic coastlines and island shapes

#### 🎨 TUI Interface Enhancements (Based on UX Design Expert Recommendations)

**High-Value Quick Wins:**
- [x] Mini-map in corner (20x10 character overview showing position within larger terrain)
- [x] Elevation legend/scale (compact reference showing symbol meanings: ~ sea, ^ hills, ▲ peaks)
- [x] Zoom levels (1:1, 1:2, 1:4 scale via sampling existing terrain data)
- [x] Multi-panel status bar (coordinates, elevation at cursor, terrain type, active overlays)

**Navigation & Discovery Features:**
- [ ] Bookmark system (B to bookmark, 1-0 to jump to bookmarks)
- [ ] Search/goto coordinates (G to jump to specific coordinates)
- [ ] Trail breadcrumbs (option to show recent path with fading dots)
- [ ] Inspection mode (I key for detailed cell information without moving viewport)
- [ ] Measurement tools (M to measure distances between points)

**UX Design Expert Priority Recommendations:**

**Phase 1 - Critical UX Issues (Immediate):**
- [ ] Split status bar into logical chunks (reduce cognitive overload)
- [ ] Add interactive help overlay (? key) for better feature discoverability
- [ ] Implement zoom-adaptive movement (navigation feels consistent across zoom levels)
- [ ] Add breadcrumb trail for better spatial orientation

**Phase 2 - Enhanced Exploration:**
- [ ] Clickable mini-map for direct navigation
- [ ] Elevation profile views and terrain statistics (P key for cross-sections)
- [ ] Bookmark system for saving interesting locations
- [ ] Compass indicator and coordinate grid overlay (G key)

**Phase 3 - Advanced Features:**
- [ ] Measurement tools (click-and-drag distances)
- [ ] Contextual information density control (I key to cycle detail levels)
- [ ] Enhanced cursor system (crosshair mode, elevation contours)
- [ ] Accessibility improvements (pattern overlays, high contrast mode)

**Future Overlay System Architecture:**
- [ ] Toggle-based overlays (1-9 keys for different data layers)
- [ ] Layered visualization system design:
  - `1` Terrain (base layer, always visible)
  - `2` Biomes (background color changes)
  - `3` Agent positions (animated characters)
  - `4` Belief zones (pattern overlays using Unicode box-drawing)
  - `5` Culture territories (border indicators)
- [ ] Context-sensitive detail levels (more detail when stationary, less when scrolling)
- [ ] Smart aggregation for zoomed-out views

**Performance & Polish:**
- [ ] Selective rendering (only update changed screen regions)
- [ ] LOD rendering (fewer details when scrolling quickly)
- [ ] Configuration profiles (Explorer/Observer/Analyst interface modes)

#### 🎨 Other Visualization Ideas to Try
- [ ] Different ASCII symbol sets for terrain types
- [ ] Heat map visualization modes (elevation, moisture, temperature)
- [ ] Animation of world generation process
- [ ] Export to image files for better visualization

## Future Concepts to Explore

### Simulation Mechanics

**Professional Multi-Physics Simulation Roadmap:**

**Phase 1: Physics Foundation (COMPLETED)**
1. **✅ Dimensional Analysis Framework** - Scale-aware architecture with WorldScale trait system
2. **✅ Grid Convergence Testing** - Numerical accuracy validation for spatial discretization
3. **✅ Water Flow Physics** - Conservation-based flow with proper dimensionless numbers
4. **✅ Erosion/Sediment Transport** - Mass conservation with scale-dependent parameters

**Phase 2A: Environmental Systems (COMPLETED)**
5. **✅ Temperature Field Dynamics** - Heat diffusion with realistic thermal properties
6. **✅ Climate System Integration** - Temperature-precipitation coupling with seasonal variation
7. **✅ Water-Climate Coupling** - Evaporation rates driven by temperature gradients
8. **✅ Professional TUI Visualization** - Multi-panel interface with real-time data overlays

**Phase 2B: Large-Scale Flow Effects (COMPLETED)**
9. **✅ Atmospheric Pressure Systems** - Pressure gradients driving wind patterns
10. **✅ Coriolis Force Integration** - Geostrophic flow and cyclonic circulation
11. **✅ Weather Pattern Formation** - Storm systems and precipitation modeling
12. **✅ Macroquad Graphics Migration** - Real-time 2D visualization with 6 display modes

**Phase 2C: System Integration & Polish (COMPLETED)**
13. **✅ Graphics Mode Threading**: Fixed macroquad async integration using proper `Window::from_config()` pattern
14. **✅ Terrain Artifact Resolution**: (0,0) mountain artifact eliminated from Diamond-Square generation
15. **✅ Build System Optimization**: Streamlined cargo build process with `default-run` configuration
16. **✅ Journal System Integration**: 774 searchable entries with semantic search and multi-agent context

**Phase 3: Geological Systems (COMPLETED)**
17. **✅ High-Performance Geological Evolution** - Optimized geological time-scale terrain aging system with 10x-100x performance improvements
18. **✅ Spatial Partitioning System** - Process only changing terrain regions (5-20% of cells) for dramatic speedup
19. **✅ Intelligent Caching** - Temperature field caching with 85-95% hit rates reducing expensive calculations by 90%
20. **✅ Multi-Criteria Convergence Detection** - Early termination saves 30-70% of iterations through robust stopping criteria
21. **✅ Integrated Performance Architecture** - Complete optimization suite with comprehensive analysis and documentation

**Phase 3B: Rust Performance Optimization (HIGH PRIORITY)**
22. **📊 PNG Export Diagnostic Mode** - Add --export-png flag for visual validation of optimizations and terrain analysis
23. **🚀 Memory Layout Optimization** - Replace Vec<Vec<T>> with flat Vec<T> + indexing (2-3x performance improvement)
24. **🚀 Error Handling Implementation** - Add comprehensive Result types with thiserror throughout codebase
25. **🚀 Double Buffering System** - Eliminate expensive clones in water simulation with dual buffer approach
26. **🚀 SIMD Operations** - Add vectorized operations for bulk mathematical computations
27. **🚀 Memory Pool Architecture** - Implement reusable memory pools for temporary allocations

**Phase 4A: Real-Time Gameplay Systems (READY TO BEGIN)**
28. **🎮 Agent Systems** - NPCs, creatures, player avatar with intelligent behaviors using optimized simulation foundation
29. **🎮 Game Mechanics** - Resource gathering, exploration, settlement building on high-performance terrain
30. **🎮 Interactive Elements** - Landing sequences, terrain interaction, survival mechanics with real-time responsiveness
31. **🎮 Roguelike Features** - Procedural events, exploration rewards, character progression in persistent worlds

**Phase 4B: Advanced Simulation Features (READY TO BEGIN)**
32. **🔬 Biome Evolution** - Dynamic ecosystem development on high-performance geological terrain
33. **🔬 Weather Systems** - Real-time weather patterns affecting gameplay with optimized climate integration
34. **🔬 Seasonal Cycles** - Long-term environmental changes and adaptation using convergence-detected equilibrium
35. **🔬 Ecological Networks** - Species interactions, food webs, population dynamics with spatial partitioning efficiency

**Legacy Ideas:**
- **Agents/Creatures**: Simple creatures that move around, eat, reproduce
- **Weather Systems**: Rain, temperature affecting terrain and agents
- **Resource Management**: Food sources, water availability
- **Evolution**: Agent traits changing over time
- **Ecosystems**: Predator-prey relationships, population dynamics

### Technical Experiments
- **Performance**: How large can maps get before it's too slow?
- **Real-time Updates**: Live simulation with continuous terrain changes
- **Networking**: Multiple connected worlds or shared simulations
- **Save/Load**: Persistent world states, time-lapse replays
- **Procedural Storytelling**: Generate narratives from simulation events

## What's Been Learned

### Terrain Generation Insights
- Random noise creates unrealistic scattered patterns
- Elevation-based color mapping works well for quick visualization
- Terminal rendering is surprisingly effective for prototyping

### Architecture Decisions That Work
- **Scale-aware design patterns** enable consistent physics across resolution scales
- **Dimensional analysis framework** ensures physical realism and parameter validation
- **Trait-based architecture** allows seamless swapping of physics implementations
- **Conservation-based numerics** provide stable, physically-meaningful simulations
- **Professional TUI design** delivers real-time visualization of complex multi-field data
- **Modular separation** (worldgen/sim/render) enables independent development of physics systems

## Interesting Questions to Investigate

**Physics & Scale Questions:**
- How do Coriolis effects manifest differently across various planetary scales?
- What dimensionless numbers govern the transition between different flow regimes?
- How can grid convergence be maintained while scaling to continental-size domains?
- What are the computational trade-offs between implicit vs explicit time integration?

**Multi-Physics Coupling Questions:**
- How do atmospheric pressure gradients couple with topographic wind channeling?
- What feedback mechanisms emerge between erosion rates and climate patterns?
- How do geological timescales interact with surface process equilibration?
- Can realistic storm formation emerge from first-principles atmospheric modeling?

**Simulation Architecture Questions:**
- How can trait-based physics enable runtime switching between different model complexities?
- What visualization techniques best reveal multi-scale coupling mechanisms?
- How do we maintain numerical stability across coupled multi-physics systems?

## Quick Win Ideas (Easy experiments to try)

**Phase 2B Implementation Quick Wins:**
- [ ] Simple pressure gradient visualization in existing TUI panels
- [ ] Coriolis parameter sensitivity analysis with different planetary rotation rates
- [ ] Wind vector overlay showing pressure-driven flow patterns
- [ ] Storm formation detection using pressure gradient thresholds

**Physics Validation Quick Wins:**
- [ ] Dimensional analysis verification tool for new physics modules
- [ ] Grid convergence testing framework for atmospheric models
- [ ] Scale-aware parameter validation across different world sizes
- [ ] Conservation law checking for coupled multi-physics systems

**Architecture Exploration Quick Wins:**
- [ ] Trait-based atmospheric physics similar to existing water system
- [ ] Runtime physics model switching (simple → full → research-grade)
- [ ] Performance profiling of coupled vs decoupled physics integration
- [ ] Memory usage optimization for large-scale continental simulations

## Rabbit Holes to Avoid (For Now)

**Technical Rabbit Holes:**
- Advanced numerical methods (finite element, spectral) before finite difference mastery
- GPU acceleration before CPU implementation is fully validated and optimized
- Complex visualizations before core physics coupling is thoroughly understood
- Real-time 3D rendering when 2D TUI effectively demonstrates the physics

**Scope Creep Rabbit Holes:**
- Agent-based modeling before environmental foundation is solid
- Full Earth-scale simulations before regional models are validated
- Comprehensive save/load systems before physics implementations stabilize
- Publication-quality documentation before experimental phase concludes

**Physics Rabbit Holes:**
- Detailed atmospheric chemistry before fluid dynamics fundamentals work
- Turbulence modeling before laminar flow regimes are properly implemented
- Non-hydrostatic effects before hydrostatic balance is established

*Note: Maintain experimental spirit while building professional-grade foundations - follow curiosity within established physics principles*