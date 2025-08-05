# Experiment Roadmap

ABOUTME: Ideas to explore, concepts to test, and interesting experiments for the simulation prototype
ABOUTME: Tracks what's been tried, what worked, and what to investigate next

## Current Exploration

**Theme**: Continental-scale simulation debugging with ASCII monitoring tools and scale-aware physics

### Recently Completed - Scale-Aware Simulation Infrastructure
- [x] **ASCII Framebuffer System**: Multi-layer real-time visualization (elevation, water, biomes, temperature, pressure, wind, flow, sediment)
- [x] **Hardcoded Values Analysis**: Comprehensive 327-line audit identifying scale-dependency issues
- [x] **Scale-Aware Water/Erosion Thresholds**: Replaced hardcoded values with domain-scaled calculations
- [x] **Diagnostic Infrastructure**: --stats mode with quantitative scale metrics and threshold validation
- [x] **Terminal-Based Monitoring**: Efficient debugging without graphics rendering overhead

### Active Priority - Complete Phase 1 Scale-Aware Fixes
- [ ] **Remove atmospheric pressure clamping** (50-110 kPa → 30-120 kPa for continental domains)
- [ ] **Implement scale-aware CFL timestep bounds** (fix hardcoded 0.001-60.0 second limits)  
- [ ] **Scale drainage system thresholds** (river_accumulation_threshold: 100.0 too high for 8km/pixel)
- [ ] **Validate fixes eliminate horizontal blue wind bands** using ASCII framebuffer monitoring

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

## Educational Deep Dive Sessions

### Code Architecture & Design Walkthroughs
**Goal**: Comprehensive understanding of system designs and implementation patterns from foundational concepts through advanced agent systems

**✅ COMPLETED SYSTEMS - Available for Educational Sessions:**

#### ✅ Phase 1: Foundation Systems (IMPLEMENTED)

**✅ Session 1: Scale-Aware Architecture & Dimensional Analysis** 
*Files: `src/scale.rs`, `src/dimensional.rs`, `docs/deep-dive-scaling-foundations.md`, `examples/dimensional_analysis_demo.rs`*
- [x] WorldScale trait system and why it enables consistent physics across resolutions
- [x] Dimensional analysis framework - ensuring physical realism in simulation parameters  
- [x] Grid convergence testing and numerical accuracy validation
- [x] Conservation-based flow equations and their discrete implementations
- [x] CFL stability conditions and timestep calculations
- [x] Physical unit system (PhysicalUnit, PhysicalQuantity) 
- [x] Unit conversion and validation systems

**✅ Session 2: Water Flow Physics Deep Dive**
*Files: `src/water.rs`, `src/drainage.rs`, `examples/water_climate_demo.rs`*
- [x] Mass conservation and water flow direction algorithms
- [x] Scale-aware water flow parameters and rainfall scaling
- [x] Drainage network formation and flow accumulation
- [x] Water-climate coupling implementation
- [x] Performance optimization (O(n²) → O(n) flow accumulation)

**✅ Session 3: Climate System Integration**
*Files: `src/climate.rs`, `src/atmosphere.rs`, `src/atmospheric_moisture.rs`*
- [x] Temperature field dynamics and elevation-based lapse rates
- [x] Seasonal variation and climate parameter scaling
- [x] Water-climate coupling and evaporation rate modeling
- [x] Atmospheric moisture separation and global coverage
- [x] Scale-aware climate effects and continental behavior

#### ✅ Phase 2: Environmental Systems (PARTIALLY IMPLEMENTED)

**✅ Session 4: TUI Architecture & User Experience**
*Files: `src/tui.rs`, `src/graphics_render.rs`, `src/render.rs`*
- [x] Professional TUI design patterns using crossterm and ratatui
- [x] Multi-panel interface architecture and state management
- [x] Real-time data overlay systems and display mode switching  
- [x] User interaction patterns and responsive interface design
- [x] Macroquad graphics integration and real-time visualization
- [x] Biome overlay mode and multiple display modes

**⚠️ Session 5: Atmospheric Pressure & Wind Systems** *(PLANNED - Fantasy Physics)*
*Status: Design completed in `docs/fantasy-physics-solo-analysis.md`, implementation pending*
- [ ] Geostrophic wind calculations and Coriolis force integration  
- [ ] Pressure field generation from temperature gradients
- [ ] Storm formation mechanics and weather pattern emergence
- [ ] 79% computational reduction through Cyberiad-inspired physics

#### ✅ Phase 3: Performance & Optimization (MOSTLY IMPLEMENTED)

**✅ Session 6: HeightMap Performance Revolution**
*Files: `src/heightmap.rs`, performance analysis in docs/*
- [x] Cache performance analysis and memory layout optimization
- [x] Structure-of-arrays conversion patterns and Vec2Map implementation
- [x] Performance measurement and validation of 2-3x improvements
- [x] Flat memory layout foundation for all systems

**✅ Session 7: Geological Systems & Performance**
*Files: `src/geological_evolution.rs`, `src/spatial_partitioning.rs`, `src/convergence.rs`*
- [x] Geological evolution performance analysis and time-scale acceleration
- [x] Multi-criteria convergence detection and early termination
- [x] Intelligent caching systems and temperature field optimization
- [x] Drainage network optimization (240x120 map: 2.22ms vs 15+ seconds)
- ⚠️ Spatial partitioning (claims 5-20% active cells but processes 100% - debug needed)
- ⚠️ Tectonic parameter fixes needed (extreme base elevations causing issues)

**✅ Session 8: Rust Performance Patterns & Memory Management**
*Files: Throughout codebase, documented in rust analysis docs*
- [x] Zero-cost abstractions and trait monomorphization in practice
- [x] Memory layout strategies for cache efficiency
- [x] SIMD readiness and vectorization preparation
- [x] Error handling patterns and type safety

#### ✅ Phase 4: Agent Systems & Social Dynamics (FOUNDATION IMPLEMENTED)

**✅ Session 9: Agent System Architecture Deep Dive**
*Files: `src/agents.rs`, `docs/deep-dive-agent-systems.md`*
- [x] SoA vs ECS vs hybrid approaches - trade-offs and performance implications
- [x] Generational safety patterns and why they prevent use-after-free bugs
- [x] Spatial indexing mathematics and O(1) neighbor query implementation
- [x] Hot/warm/cold data separation and cache optimization strategies
- [x] High-performance structure-of-arrays memory layout
- [x] AgentId type safety with generation counters
- [x] Comprehensive testing framework (4/4 agent tests passing)

**⚠️ Session 10: Social Systems Design** *(DESIGN PHASE)*
*Status: Collaborative design documents exist, implementation not started*
- [ ] Relationship graph mathematics and trust propagation algorithms
- [ ] Emergent cooperation mechanics - Dan Bunten's design philosophy in practice
- [ ] Information networks and reputation systems implementation
- [ ] Cultural trait diffusion and social learning rule engines

**⚠️ Session 11: Cultural Evolution & Mythology** *(DESIGN PHASE)*
*Status: `docs/cultural-mythology-engine.md` contains detailed specifications*
- [ ] Story propagation networks and narrative mutation during transmission
- [ ] Belief system hierarchies and coherence checking algorithms
- [ ] Historical memory formation and significance assessment systems
- [ ] Ritual emergence patterns and cultural selection pressures

**✅ Session 12: Multi-Agent Design Collaboration Analysis**
*Files: `docs/agent-collaboration-experiment-handoff.md`, collaboration analysis documents*
- [x] How specialist agents contributed complementary expertise across all phases
- [x] Integration challenges between performance, social, and cultural requirements
- [x] Design pattern emergence from collaborative architecture discussions
- [x] Cognitive architecture patterns (solo vs multi-agent processing)
- [x] 9-agent collaborative evaluation experiment completed

#### ✅ Cross-Phase Integration & Lessons (READY FOR ANALYSIS)

**✅ Session 13: Performance Foundation Impact Study**
*Status: Implementation complete, ready for educational analysis*
- [x] How Phase 1 scale-aware design enabled Phase 2 environmental systems
- [x] How Phase 2 TUI patterns supported Phase 3 optimization visualization  
- [x] How Phase 3 HeightMap optimization enabled Phase 4 agent performance
- [x] Compound performance gains and system interdependencies
- [x] Cross-system validation and testing patterns

**✅ Session 14: Architectural Evolution & Design Patterns**
*Status: Implementation complete, ready for educational analysis*
- [x] Trait-based design evolution from water systems to agent systems
- [x] Modular architecture patterns that supported incremental complexity
- [x] Error handling evolution and type safety improvements
- [x] Documentation patterns and deep-dive analysis methodology
- [x] ScaleAware trait as universal scaling framework

## **🎓 UPDATED EDUCATIONAL APPROACH**

### **Current Status: Critical Physics Architecture Issue - 10/14 Sessions Ready for Education**

**✅ IMMEDIATELY AVAILABLE (Complete implementations):**
- Session 1: Scale-Aware Architecture & Dimensional Analysis
- Session 2: Water Flow Physics Deep Dive  
- Session 3: Climate System Integration
- Session 4: TUI Architecture & User Experience
- Session 6: HeightMap Performance Revolution
- Session 7: Geological Systems & Performance
- Session 8: Rust Performance Patterns & Memory Management
- Session 9: Agent System Architecture Deep Dive
- Session 13: Performance Foundation Impact Study
- Session 14: Architectural Evolution & Design Patterns

**🚨 CRITICAL ARCHITECTURE ISSUE IDENTIFIED:**
- Session 5: Atmospheric Pressure & Wind Systems *(PHYSICS FLAW: Uses random noise instead of atmospheric dynamics)*

**⚠️ DESIGN PHASE (Specifications exist, implementation pending):**
- Session 10: Social Systems Design *(Collaborative specs exist)*
- Session 11: Cultural Evolution & Mythology *(Detailed design docs exist)*
- Session 12: Multi-Agent Design Collaboration Analysis *(Research complete)*

### **Recommended Education Starting Points:**

**🚀 START HERE: Session 1** - Scale-Aware Architecture & Dimensional Analysis  
*Foundation for understanding all other systems*

**🎯 HIGH VALUE: Session 9** - Agent System Architecture Deep Dive  
*Most recent and complete implementation with comprehensive testing*

**📊 PERFORMANCE FOCUS: Session 6** - HeightMap Performance Revolution  
*Demonstrates concrete 2-3x performance improvements with measurable results*

*Note: Each session includes hands-on code walkthroughs, mathematical concept explanations, and practical implementation examples*

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

**Phase 3B: Rust Performance Optimization (✅ COMPLETED - COMMITTED 5f7f13113630)**
22. **✅ Memory Layout Optimization** - Replaced Vec<Vec<T>> with flat Vec<T> HeightMap (2-3x performance improvement foundation)
23. **✅ Structure-of-Arrays Pattern** - Vec2Map implemented for velocity fields with SIMD readiness
24. **✅ Performance Foundation** - Cache-friendly memory layout established across all core modules
25. **✅ Test Suite Integration** - 102/102 tests passing with complete HeightMap API migration
26. **✅ Quality Gates** - All build, test, format, and code-reviewer requirements met
27. **✅ Production Ready** - Multi-agent workflow validation with proper quality enforcement

**Phase 4A: Engine Library Architecture (READY TO BEGIN)**
28. **📚 Engine/Application Separation** - Reorganize codebase to separate pure engine from application instances
29. **📚 Library API Design** - Clean public API for engine components and ScaleAware systems
30. **📚 Multiple Application Examples** - Terrain explorer, fantasy world, scientific simulation instances
31. **📚 Cultural Scaling Framework** - Extend ScaleAware to social/cultural behaviors for multi-scale entity emergence

**Phase 4B: Real-Time Gameplay Systems (READY TO BEGIN)**
32. **🎮 Agent Systems** - NPCs, creatures, player avatar with intelligent behaviors using optimized simulation foundation
33. **🎮 Game Mechanics** - Resource gathering, exploration, settlement building on high-performance terrain
34. **🎮 Interactive Elements** - Landing sequences, terrain interaction, survival mechanics with real-time responsiveness
35. **🎮 Roguelike Features** - Procedural events, exploration rewards, character progression in persistent worlds

**Phase 4C: Advanced Simulation Features (READY TO BEGIN)**
36. **🔬 Biome Evolution** - Dynamic ecosystem development on high-performance geological terrain
37. **🔬 Weather Systems** - Real-time weather patterns affecting gameplay with optimized climate integration
38. **🔬 Seasonal Cycles** - Long-term environmental changes and adaptation using convergence-detected equilibrium
39. **🔬 Ecological Networks** - Species interactions, food webs, population dynamics with spatial partitioning efficiency

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

**Phase 4A Engine Architecture Quick Wins:**
- [ ] Create `src/engine/` directory and move core modules
- [ ] Implement `applications/terrain_explorer.rs` from current main.rs logic
- [ ] Design clean library API in `lib.rs` with proper re-exports
- [ ] Add `applications/fantasy_world.rs` demonstrating cultural scaling
- [ ] Create `CulturalConfig` with ScaleAware implementation
- [ ] Test multi-scale agent behavior (individual → institution → civilization)

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

**Engine Architecture Rabbit Holes:**
- Complex plugin systems before basic engine/application separation works
- Advanced trait abstractions before ScaleAware cultural scaling is proven
- Multiple library crates before single-crate organization is validated
- Backwards compatibility before API is stabilized

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