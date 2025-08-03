# 🌍 Simulation Project Design Overview

## 🗺️ Map Generation Systems

### ✅ Current Approach
- **Prototype heightmap via random noise** (placeholder for Diamond-Square)
- ASCII rendering with elevation thresholds and color bands

### 🧱 Planned / Discussed Systems

#### 1. Diamond-Square
- Fast, easy to implement
- Good for prototyping
- Grid artifacts, not geologically plausible

#### 2. Generalized Stochastic Diffusion (GSD)
- Simulates tectonics, erosion, and diffusion processes
- Supports natural terrain shaping and hydrology
- Ideal for realistic terrain + simulation input

#### 3. Hybrid Pipeline (Recommended)
- Start with Diamond-Square or noise
- Layer in erosion, uplift, wind/rain post-processors
- Modular stages for:
  - Base heightmap
  - Tectonic/erosion shaping
  - Biome/rainfall/temperature tagging

---

## 📖 Narrative & Belief Systems

### ✅ Simulation Concepts

#### 1. Belief Spread System
- Region-based or tile-based cultural vectors
- Competing deities, myths, philosophies
- Agents or events affect propagation

#### 2. Pantheon Simulation
- Deities generated with domains, personalities, symbolic meaning
- Myth creation system (e.g., grammar-based or symbolic logic)
- Myths interact with geography, history, and agent memory
- Could influence gameplay systems (magic, social cohesion, conflict)

#### 3. Cultural Memory
- Simulate belief as historical memory or evolving narrative
- Legends may shift over time or mutate between cultures
- Priests, bards, or events could carry/alter these stories

---

## 🔁 Simulation Engine Goals

### Core Features
- Modular tick-based loop
- World state includes:
  - Heightmap
  - Biomes
  - Agents / factions
  - Resource overlays (optional)

### Agent Behaviors
- Autonomous expansion
- Cultural influence
- Migration, conflict, diplomacy

### Output & Feedback
- ASCII/PNG/GIF renders per tick
- Logging and snapshot systems
- Debug overlays for agent or myth influence

---

## 🤖 Agent Roles / Subsystems

| Agent | Focus |
|-------|-------|
| `sid-designer` | Strategic depth, clarity, meaningful decisions |
| `will-designer` | Systems thinking, emergent simulation, visualization |
| `bunten-designer` | Social dynamics, empathy, emergent stories |
| `simulation-engineer` | Core tick loop, state transitions, modularity |
| `game-subsystem-engineer` | Belief, migration, political models |
| `render-engineer` | Terminal or PNG output layers |
| `worldgen-architect` | Tectonics, erosion, rainfall, map pipeline |
| `data-modeler` | Core structs, serialization, memory layout |
| `story-weaver` | Myths, legends, symbolic systems, narrative vectors |

---

## 🔜 Next Steps

- [ ] Normalize and polish ASCII renderer
- [ ] Add procedural noise or tectonic worldgen
- [ ] Define core `Tile`, `Agent`, and `Myth` types
- [ ] Implement simple belief spread simulation
- [ ] Add visual overlays (belief zones, biome maps)

---
