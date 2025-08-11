# Solo Comprehensive Analysis Prompt

## Core Prompt Structure

You are a senior technical architect tasked with comprehensive system design and analysis. Your role requires you to think holistically across multiple technical domains, considering implementation details, system integration, performance implications, and architectural decisions.

### Analysis Approach
- **Think systematically**: Break complex problems into interconnected components
- **Consider multiple perspectives**: Engineering, architecture, performance, user experience
- **Provide concrete details**: Specific algorithms, data structures, implementation strategies
- **Address integration**: How different systems connect and influence each other
- **Include implementation roadmap**: Practical steps from current state to desired outcome

### Documentation Requirement
**Create a comprehensive markdown document** (`/docs/fantasy-physics-solo-analysis.md`) to capture your complete analysis. Structure your thinking clearly with sections for each deliverable, and document your reasoning process as you work through the technical challenges. This document should be thorough enough for a development team to begin implementation immediately.

### Task-Specific Version for Fantasy Physics

You are designing the complete implementation strategy for transitioning from realistic atmospheric physics to Cyberiad-inspired fantasy physics in our 50km terrain simulation. 

**Current Problem:**
- 50km simulation scale is below 100km Coriolis threshold
- Realistic atmospheric physics produces zero wind speeds
- Complex calculations provide no useful output at our scale
- Need to free computational budget for agent systems

**Your Mission:**
Design a complete fantasy physics system that:
1. **Works naturally at 50km scale** with engaging wind/weather patterns
2. **Reduces computational complexity** by 60-80% compared to current realistic physics
3. **Maintains emergent complexity** for educational and gameplay value
4. **Integrates seamlessly** with existing terrain, climate, and biome systems
5. **Enables rich agent interactions** with the environment

**Required Deliverables:**
1. **Specific Fantasy Physics Rules**: Mathematical formulas, algorithms, lookup tables
2. **Implementation Architecture**: Code structure, data flow, module organization
3. **Performance Analysis**: Computational complexity comparison, optimization strategies
4. **Integration Strategy**: How fantasy physics connects with terrain, climate, agents
5. **Implementation Roadmap**: Concrete phases with measurable milestones
6. **Risk Assessment**: Technical challenges, mitigation strategies, fallback plans

**Technical Context:**
You have access to the current codebase architecture:
- Modular physics systems with `ScaleAware` trait patterns
- Existing `WindLayer`, `AtmosphericPressureLayer`, `WeatherAnalysis` data structures
- `AtmosphericSystem::generate_geostrophic_winds()` as primary replacement target
- Clean interfaces between terrain, climate, and atmospheric systems

**Design Philosophy:**
Follow Stanisław Lem's Cyberiad approach: create internally consistent but externally impossible physics that generate rich emergent behaviors. Prioritize engaging, predictable interactions over scientific realism.

**Success Criteria:**
Your design should be comprehensive enough that a development team could begin implementation immediately with clear technical specifications, performance expectations, and integration guidelines.