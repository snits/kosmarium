# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based simulation prototype that generates and visualizes terrain heightmaps. The project is in early development stage with a basic modular architecture.

## Development Commands

### Building and Running
- `cargo check` - Quick syntax and type checking
- `cargo build` - Build the project 
- `cargo run` - Build and run the simulation
- `cargo test` - Run tests (when implemented)

### Development Tools
- `cargo clippy` - Rust linter for code quality
- `cargo fmt` - Format code according to Rust standards
- `cargo clean` - Clean build artifacts

## Architecture Overview

The codebase follows a modular architecture with clear separation of concerns:

### Core Modules (`src/`)
- **main.rs** - Entry point orchestrating world generation → simulation setup → rendering
- **worldgen.rs** - Terrain generation using actual Diamond-Square algorithm with trait-based architecture
- **sim.rs** - Simulation state management (minimal implementation, prepared for agents/biomes)
- **render.rs** - ASCII visualization using crossterm for colored terminal output

### Data Flow
1. `DiamondSquareGenerator` creates heightmap with seeded randomization
2. `Simulation` wraps heightmap data structure (extensible for game state)
3. `ascii_render` visualizes terrain as colored ASCII characters based on elevation

### Key Dependencies
- **rand** (0.8) - Seeded random number generation for reproducible worlds
- **crossterm** (0.27) - Cross-platform terminal manipulation for colored output

## Development Context

### Educational Purpose
This project serves as a learning environment for both simulation concepts and agentic workflow experimentation. **Please provide detailed explanations for technical concepts, algorithmic decisions, and architectural choices.** The goal is to understand not just what the code does, but why specific approaches were chosen and how they work under the hood.

### Interactive Education Requirement
**When implementing changes, explain the mathematical concepts, physical principles, and algorithmic reasoning behind each modification as it's being made, providing real-time educational context alongside the development process.** This complements the post-completion deep-dive documentation by offering immediate learning opportunities during active development.

### Subsystem Documentation Requirement
**MANDATORY**: Upon completion of any major subsystem, create a comprehensive deep-dive document in `/docs/` that covers:

1. **Mathematical Foundations** - The underlying mathematical principles and equations
2. **Engineering Patterns** - Architectural patterns used and why they were chosen  
3. **Implementation Details** - How the theory translates to code
4. **System Extensions** - How the patterns extend to other systems
5. **Educational Insights** - What can be learned and applied elsewhere

**Examples:**
- `docs/deep-dive-scaling-foundations.md` - WorldScale architecture analysis
- `docs/deep-dive-water-physics.md` - Water flow system principles (when completed)
- `docs/deep-dive-terrain-generation.md` - Noise algorithms and procedural methods (when completed)

**Purpose**: These documents serve as both educational material and architectural reference, enabling deep understanding of design decisions and their broader applicability to other systems and domains.

### Current State  
- Actual Diamond-Square terrain generation implemented with trait-based architecture
- Colored ASCII rendering functional with realistic terrain patterns
- Simulation structure prepared for expansion (agents, biomes, game logic)
- Extensible TerrainGenerator trait system ready for algorithm experimentation

### Terrain Visualization
The renderer maps elevation values to colored symbols:
- `.` (blue) - Water/low elevation (< 0.2)
- `~` (cyan) - Coastline (0.2-0.4) 
- `^` (green) - Plains (0.4-0.6)
- `#` (yellow) - Hills (0.6-0.8)
- `@` (red) - Mountains (> 0.8)

### Extension Points
- TerrainGenerator trait architecture ready for additional algorithms (Perlin noise, stochastic diffusion)
- DiamondSquareConfig parameters enable terrain characteristic experimentation
- Simulation struct designed for adding agents, biome systems, time-based evolution
- Rendering system can be extended for different visualization modes and data layers

### Future Algorithm Experiments
- **Generalized Stochastic Diffusion** - Next planned terrain generation approach
- **Multi-layer Environmental Systems** - Temperature, precipitation, biome generation
- **Post-processing Pipeline** - Erosion simulation, river carving, climate modeling