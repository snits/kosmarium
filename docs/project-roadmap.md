# Experiment Roadmap

ABOUTME: Ideas to explore, concepts to test, and interesting experiments for the simulation prototype
ABOUTME: Tracks what's been tried, what worked, and what to investigate next

## Current Exploration

**Theme**: Experimenting with procedural world generation and simulation concepts

### Active Experiments

#### 🔧 Basic Infrastructure (Getting things working)
- [ ] Fix compilation issue (reserved keyword error)
- [ ] Get basic Diamond-Square algorithm working (currently just random noise)
- [ ] Make map size configurable via command line

#### 🌍 World Generation Experiments
- [ ] Compare different noise algorithms (Diamond-Square vs Perlin vs Simplex)
- [ ] Experiment with terrain post-processing (erosion simulation, river carving)
- [ ] Try procedural biome placement based on elevation/temperature
- [ ] Generate realistic coastlines and island shapes

#### 🎨 Visualization Ideas to Try
- [ ] Different ASCII symbol sets for terrain types
- [ ] Heat map visualization modes (elevation, moisture, temperature)
- [ ] Animation of world generation process
- [ ] Export to image files for better visualization

## Future Concepts to Explore

### Simulation Mechanics
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

- [ ] Add more terrain elevation bands with different colors/symbols
- [ ] Implement simple temperature map based on latitude/elevation
- [ ] Add command-line seed parameter for reproducible worlds
- [ ] Try different map aspect ratios (square vs rectangular vs long strips)
- [ ] Experiment with edge effects (wrapping vs boundaries vs infinite generation)

## Rabbit Holes to Avoid (For Now)

- Complex GUI frameworks (terminal is fine for experiments)
- Over-optimizing performance before understanding what's slow
- Building comprehensive save/load systems before core mechanics work
- Getting distracted by graphics when gameplay concepts are more interesting

*Note: This is a personal experiment - follow curiosity and don't worry about "finishing" everything*