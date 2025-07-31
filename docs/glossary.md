# Project Glossary

ABOUTME: Standardized terminology and definitions for clear communication about the simulation project
ABOUTME: Distinguishes between different types of agents, systems, and technical concepts used throughout development

## Agent Types

### Sub-Agents (Development Tools)
**Sub-agents** are specialized AI assistants that help with development tasks. They are invoked using the `Task` tool and work autonomously on specific problems.

**Available Sub-Agents:**
- **general-purpose** - Research, search, multi-step tasks
- **test-specialist** - Comprehensive test coverage, TDD workflows  
- **debug-specialist** - Systematic bug investigation and root cause analysis
- **security-engineer** - Security assessment, vulnerability analysis, threat modeling
- **systems-architect** - Architecture guidance, system design, technology evaluation
- **ux-design-expert** - User experience design, interface optimization
- **senior-engineer** - Expert programming, algorithm design, performance optimization
- **qa-engineer** - Quality assurance validation, feature verification
- **performance-engineer** - Performance bottlenecks, system optimization
- **kernel-hacker** - Low-level systems, OS internals, hardware interfaces
- **code-reviewer** - Direct code feedback, architecture assessment
- **simulation-engineer** - Emergent behavior systems, simulation frameworks
- **cultural-mythology-engine** - Cultural evolution, belief systems, storytelling
- **social-systems-designer** - Multiplayer mechanics, social interactions
- **data-architect** - Data structure design, schema optimization
- **game-design-strategist** - Game mechanics, strategic gameplay
- **game-subsystem-engineer** - Modular game systems, component architecture
- **simulation-designer** - Complex emergent systems, computational modeling
- **rendering-engineer** - Visual representation, graphics optimization
- **world-generation-architect** - Procedural world generation, terrain systems

### Simulation Agents (Game Entities)
**Simulation agents** are entities within the game world that exhibit behaviors, interact with terrain, and participate in the simulation.

**Types of Simulation Agents:**
- **Player Avatar** - User-controlled character
- **NPCs** - Non-player characters with AI behaviors
- **Creatures** - Wildlife, monsters, ambient life
- **Environmental Agents** - Weather systems, geological processes
- **Economic Agents** - Traders, resource gatherers, settlement builders

## System Components

### Core Systems
- **Terrain Generation** - Procedural landscape creation using Diamond-Square and tectonic systems
- **Geological Evolution** - High-performance time-scale terrain aging with optimization
- **Climate System** - Temperature, precipitation, seasonal cycles
- **Water Flow System** - Fluid dynamics, erosion, sediment transport
- **Rendering System** - Multi-modal visualization (ASCII, TUI, Graphics)

### Performance Systems
- **FlatHeightmap** - Cache-friendly flat memory layout for terrain data
- **Spatial Partitioning** - Process only changing regions for 5-20x speedup
- **Intelligent Caching** - Temperature field caching with 85-95% hit rates
- **Convergence Detection** - Multi-criteria early termination for geological evolution

### Data Structures
- **WorldScale** - Scale-aware architecture for consistent physics across resolutions
- **TectonicSystem** - Voronoi-based continental plate simulation
- **TemperatureLayer** - 2D temperature field with thermal properties
- **WaterFlowSystem** - Conservation-based fluid dynamics

## Development Concepts

### Architecture Patterns
- **Trait-based Design** - Modular, swappable system implementations
- **Scale-aware Architecture** - Consistent physics across different world sizes  
- **Dimensional Analysis** - Physical unit validation and parameter scaling
- **Conservation-based Numerics** - Mass/energy conservation in simulations

### Performance Concepts
- **Cache Locality** - Memory access patterns optimized for CPU cache efficiency
- **Active Region Processing** - Only updating changing areas of simulation
- **Early Termination** - Stopping iterations when convergence is detected
- **Multi-tier Updates** - Different update frequencies for coupled systems

### Development Workflow
- **TDD** - Test-driven development with failing tests first
- **Atomic Commits** - Single logical changes per commit following Linux kernel style
- **Code-reviewer Approval** - All changes require review before committing
- **Agent Collaboration** - Using specialized sub-agents for domain expertise

## File Organization

### Documentation Structure
- **session-handoff.md** - Current implementation status and next steps
- **project-roadmap.md** - Implementation milestones and progress tracking
- **architecture-decisions.md** - Key design choices and rationale
- **deep-dive-*.md** - Comprehensive analysis of major systems

### Code Organization
- **src/main.rs** - Entry point and simulation orchestration
- **src/worldgen.rs** - Terrain generation and geological systems
- **src/sim.rs** - Simulation state management and game logic
- **src/graphics_render.rs** - Macroquad-based visual rendering
- **src/optimized_*.rs** - High-performance system implementations

## Common Abbreviations

- **TUI** - Terminal User Interface
- **ASCII** - American Standard Code for Information Interchange (text-based rendering)
- **NPC** - Non-Player Character
- **TDD** - Test-Driven Development
- **LRU** - Least Recently Used (caching strategy)
- **SIMD** - Single Instruction, Multiple Data (vectorization)
- **GPU** - Graphics Processing Unit
- **CPU** - Central Processing Unit

## Usage Guidelines

### When to Use Sub-Agents
- **Complex multi-step tasks** requiring specialized expertise
- **Domain-specific problems** where expert knowledge is needed
- **Quality assurance** before committing changes
- **Systematic investigation** of bugs or performance issues
- **Architecture decisions** requiring deep technical analysis

### Communication Standards
- Always specify **sub-agent** vs **simulation agent** when discussing agents
- Use **system** for major architectural components
- Use **optimization** for performance improvements
- Use **integration** for connecting systems together
- Use **experiment** for exploratory development work

## Examples

**Clear Communication:**
- ✅ "Let's use the debug-specialist sub-agent to investigate this memory leak"
- ✅ "The simulation agents need pathfinding to navigate terrain"
- ✅ "The geological evolution system completed Phase 3"

**Ambiguous Communication:**
- ❌ "The agents aren't working properly" (which agents?)
- ❌ "We need to optimize the system" (which system?)
- ❌ "The performance is bad" (need specific metrics)

This glossary will evolve as the project grows - suggest additions for any terms that cause confusion!