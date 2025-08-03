# Experiment Roadmap

ABOUTME: Ideas to explore, concepts to test, and interesting experiments for the simulation prototype
ABOUTME: Tracks what's been tried, what worked, and what to investigate next

## Current Exploration

**Theme**: Experimenting with procedural world generation and simulation concepts

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

**Simulation-Designer Roadmap (6+ Month Development Plan):**

**Phase 1: Environmental Foundation (Months 1-2)**
1. **Water Flow System** - Water flows to lowest adjacent cell, evaporates based on temperature
2. **Temperature/Climate Layer** - Enables realistic evaporation patterns
3. **Basic Erosion/Sediment** - Sediment transport proportional to velocity, deposition when flow slows

**Phase 2: Biome Systems (Month 3)**  
4. **Precipitation Patterns** - Driven by terrain and temperature
5. **Vegetation/Biome Layer** - Responds to water availability and climate
6. **Resource Distribution** - Minerals, fertile soil, etc.

**Phase 3: Agent Foundation (Months 4-5)**
7. **Simple Agent Movement** - Pathfinding influenced by terrain
8. **Settlement Patterns** - Agents prefer river valleys, avoid mountains
9. **Basic Resource Gathering** - Agents interact with environment

**Phase 4: Cultural Systems (Months 6+)**
10. **Belief Propagation** - Pantheon and myth systems
11. **Trade Networks** - Following natural geographic routes
12. **Cultural Memory** - Stories tied to geographic features

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
- Modular separation (worldgen/sim/render) makes experimentation easier
- Crossterm provides good cross-platform terminal features
- Seeded generation allows reproducible experiments

## Interesting Questions to Investigate

- How do different noise algorithms affect the "feel" of generated worlds?
- What's the minimum complexity needed for emergent behavior in agents?
- Can simple rules create realistic ecosystem dynamics?
- How does map size affect simulation performance and behavior patterns?
- What visualization approaches best reveal simulation patterns?

## Quick Win Ideas (Easy experiments to try)

**TUI Interface Quick Wins (High Impact, Low Effort):**
- [x] Mini-map in corner (simple downsampled view)
- [x] Elevation legend (3-line reference guide)
- [x] Zoom levels via render sampling (1:1, 1:2, 1:4)
- [x] Fast movement with Shift+WASD (already implemented)

**Terrain Generation Quick Wins:**
- [ ] Add more terrain elevation bands with different colors/symbols
- [ ] Implement simple temperature map based on latitude/elevation
- [x] Add command-line seed parameter for reproducible worlds
- [ ] Try different map aspect ratios (square vs rectangular vs long strips)
- [ ] Experiment with edge effects (wrapping vs boundaries vs infinite generation)

## Rabbit Holes to Avoid (For Now)

- Complex GUI frameworks (terminal is fine for experiments)
- Over-optimizing performance before understanding what's slow
- Building comprehensive save/load systems before core mechanics work
- Getting distracted by graphics when gameplay concepts are more interesting

*Note: This is a personal experiment - follow curiosity and don't worry about "finishing" everything*