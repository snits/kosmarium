# Agent System Architecture Design

ABOUTME: Comprehensive multi-agent design for real-time agent systems with social dynamics and cultural evolution
ABOUTME: Documents collaborative architecture from systems-architect, social-systems-designer, cultural-mythology-engine, and rust-specialist

## Executive Summary

This document captures the collaborative design for a real-time agent system that serves as the foundation for NPCs, creatures, player avatars, and future game mechanics. The architecture builds on our existing high-performance HeightMap optimization (2-3x speedup) using structure-of-arrays patterns while enabling rich social dynamics and emergent cultural storytelling.

**Key Design Principles:**
- **Performance First**: Structure-of-arrays memory layout targeting 60fps with hundreds of agents
- **Social Emergence**: Relationship dynamics that create natural cooperation without forced mechanics
- **Cultural Evolution**: Historical memory and storytelling systems that create world depth over time
- **Rust Idiomaticity**: Zero-cost abstractions with type safety preventing common agent system bugs

## Systems-Architect: Technical Foundation

### Core Architecture Philosophy

The agent system follows a **cache-optimized storage pattern** similar to our successful HeightMap conversion, using structure-of-arrays where agent positions, types, and bounds are stored in separate contiguous vectors for 2-3x performance improvements.

### Agent Trait Hierarchy

```rust
pub trait Agent: Send + Sync {
    type State: AgentState;
    type Behavior: AgentBehavior;
    // Core lifecycle and behavior interface
}

pub trait AgentBehavior {
    fn update(&mut self, agent_state: &dyn AgentState, context: &BehaviorContext) -> ActionSet;
    fn priority(&self) -> f32;
}

pub trait AgentState: Send + Sync {
    fn position(&self) -> Vec2;
    fn set_position(&mut self, pos: Vec2);
    fn agent_type(&self) -> AgentType;
}
```

### Spatial Indexing Integration

Built on existing spatial partitioning concepts using grid-based indexing for O(1) neighbor queries. Critical for hundreds of agents interacting in real-time.

### Clean Simulation Integration

```rust
pub struct SimulationContext<'a> {
    pub heightmap: &'a HeightMap,
    pub water_layer: &'a WaterLayer,
    pub climate_system: &'a ClimateSystem,
    // Provides clean access to all simulation systems
}
```

### Performance Considerations

**Real-time Optimizations:**
- **Type-batched updates** - NPCs, creatures, players updated separately for cache locality
- **SIMD-friendly layouts** - Vec<Vec2> positions enable vectorized operations
- **Spatial query efficiency** - Grid cells prevent O(n²) agent interactions
- **Hot/cold data separation** - Frequently accessed data stored contiguously

**HeightMap Integration:**
- Bilinear interpolation for smooth agent movement
- Slope calculations for realistic physics
- Navigability queries for pathfinding
- All leveraging high-performance flat memory layout

### Extensibility Framework

**Behavior Components:**
Modular behavior system allowing complex AI through composition:
```rust
pub trait BehaviorComponent {
    fn update(&mut self, agent_state: &dyn AgentState, context: &BehaviorContext) -> ActionSet;
    fn priority(&self) -> f32;
}
```

**Agent Type Specialization:**
- **NPCs**: Complex state (health, inventory, knowledge) with social behaviors
- **Creatures**: Instinctual behaviors, pack dynamics, environmental responses  
- **Player Avatars**: Input handling with AI assistance, immediate response requirements

### Integration Points

Clean separation while enabling tight integration:

1. **Elevation queries** through HeightMap interpolation
2. **Environmental effects** via climate and water systems
3. **Collision detection** using spatial indexing
4. **Rendering pipeline** with frustum culling support
5. **Event system** for agent communication and world interaction

### Future Extension Support

- Resource gathering and settlement building
- Pack/group behaviors for creatures
- Dynamic spawning based on environmental conditions
- Pathfinding integration (A* or flow fields)
- Networking support for multiplayer
- Modular behavior loading for extensibility

## Social-Systems-Designer: Relationship Dynamics

### Core Social Philosophy

Following Dan Bunten's principles, social systems should feel **natural and emergent** rather than mechanical. Simple rules that produce complex social behaviors, where cooperation becomes genuinely beneficial rather than forced.

### Agent Relationship Dynamics

**Structure-of-arrays relationship storage for cache efficiency:**

```rust
pub struct RelationshipSystem {
    // Core relationship data (hot path for decision making)
    relationship_values: Vec<f32>,     // -1.0 to 1.0 (enemy to ally)
    trust_levels: Vec<f32>,           // 0.0 to 1.0 (reliability)
    shared_history: Vec<u32>,         // Interaction count for weight
    
    // Relationship metadata (cooler data)
    relationship_types: Vec<RelationshipType>,
    last_interaction: Vec<u64>,       // Tick count
    emotional_memory: Vec<EmotionalState>,
    
    // Efficient lookup structures
    agent_to_index: HashMap<(AgentId, AgentId), usize>,
    agent_relationships: HashMap<AgentId, Vec<usize>>,
}

#[derive(Clone, Debug)]
pub enum RelationshipType {
    Stranger, Acquaintance, Ally, Rival, Enemy, Friend,
    Mentor, Student, TradePartner,
}

#[derive(Clone, Debug)]
pub struct EmotionalState {
    pub gratitude: f32,      // For received help
    pub resentment: f32,     // For perceived wrongs
    pub respect: f32,        // For demonstrated competence
    pub fear: f32,           // For perceived threat
}
```

**Key Design Insight**: Relationships have **momentum and memory**. A single negative interaction doesn't destroy a friendship, but repeated patterns create lasting change.

### Cooperative Mechanics That Feel Natural

The secret to natural cooperation is making it **emergently beneficial** rather than mechanically required:

```rust
pub struct ResourceCooperationSystem {
    // Building on HeightMap resource layers
    resource_extraction_efficiency: HashMap<(AgentId, AgentId), f32>,
    shared_knowledge_bonuses: HashMap<AgentId, KnowledgeBonus>,
    reputation_modifiers: HashMap<AgentId, ReputationEffect>,
}
```

**Cooperation Design Principles:**
- **Specialization rewards**: Different agent types become more effective together
- **Risk mitigation**: Dangerous areas become accessible to groups
- **Knowledge amplification**: Shared discoveries benefit the group more than individuals
- **Reputation networks**: Well-regarded groups get better trade terms

### Emergent Social Behaviors Through Simple Rules

```rust
pub struct SocialBehaviorEngine {
    interaction_rules: Vec<Box<dyn InteractionRule>>,
    cultural_traits: CulturalSystem,
    reputation_network: ReputationSystem,
}

/// Example: Reciprocity Rule
pub struct ReciprocityRule {
    memory_decay: f32,
    reciprocity_threshold: f32,
}

impl InteractionRule for ReciprocityRule {
    fn evaluate(&self, context: &SocialContext) -> Vec<SocialAction> {
        // Check for unreciprocated favors and generate impulse to return favor
        // Creates natural give-and-take relationship dynamics
    }
}

/// Example: Social Learning Rule  
pub struct SocialLearningRule {
    observation_range: f32,
    success_threshold: f32,
}
```

**Emergent Behaviors This Creates:**
- **Reputation cascades**: Success breeds observation and imitation
- **Cultural clusters**: Successful strategies spread through social networks
- **Innovation diffusion**: New techniques propagate based on social trust
- **Social stratification**: Respected agents become informal leaders

### Communication Systems

```rust
pub struct InformationSystem {
    agent_knowledge: HashMap<AgentId, KnowledgeBase>,
    information_value: HashMap<InfoType, f32>,
    communication_networks: Vec<CommunicationNetwork>,
    rumor_propagation: RumorSystem,
}

#[derive(Clone, Debug)]
pub struct KnowledgeBase {
    // Practical knowledge
    resource_locations: HashMap<ResourceType, Vec<LocationInfo>>,
    danger_warnings: HashMap<Vec2, DangerInfo>,
    trade_opportunities: Vec<TradeInfo>,
    
    // Social knowledge  
    agent_reputations: HashMap<AgentId, ReputationInfo>,
    group_dynamics: HashMap<GroupId, GroupInfo>,
    cultural_norms: Vec<CulturalNorm>,
}
```

**Communication Design Features:**
- **Information as currency**: Valuable knowledge becomes a tradeable resource
- **Trust networks**: Information flows more freely between trusted agents
- **Verification systems**: Information accuracy affects sharer's reputation
- **Cultural transmission**: Norms and practices spread through communication

### Cultural Transmission and Learning

```rust
pub struct CulturalSystem {
    cultural_traits: Vec<CulturalTrait>,
    trait_adoption_rates: HashMap<AgentId, Vec<f32>>,
    cultural_pressure: SpatialLayer<f32>,  // Like HeightMap but for culture
    innovation_events: Vec<Innovation>,
}

#[derive(Clone, Debug)]
pub struct CulturalTrait {
    pub trait_id: TraitId,
    pub effectiveness: f32,        // How well it works
    pub adoption_cost: f32,        // How hard to learn
    pub social_pressure: f32,      // How much others expect it
    pub prerequisites: Vec<TraitId>, // What you need to know first
}
```

**Cultural Evolution Features:**
- **Spatial diffusion**: Ideas spread geographically like water flow system
- **Social validation**: Traits spread faster among trusted social networks  
- **Innovation pressure**: Environmental challenges drive cultural adaptation
- **Cultural clustering**: Different regions develop distinct cultural profiles

### Conflict Resolution Without Relationship Destruction

```rust
pub struct ConflictResolutionSystem {
    active_disputes: Vec<Dispute>,
    resolution_mechanisms: Vec<Box<dyn ResolutionMechanism>>,
    reputation_arbiters: ReputationArbiters,
}

#[derive(Clone, Debug)]
pub enum DisputeType {
    ResourceAccess,
    TerritorialBoundary,
    TradeDisagreement,
    ReputationChallenge,
    CulturalNormViolation,
}
```

**Conflict Resolution Principles:**
- **Reputation stakes**: Conflicts affect social standing rather than physical resources
- **Mediator systems**: Trusted third parties facilitate resolution
- **Challenge alternatives**: Competition through skill demonstrations rather than destruction
- **Relationship repair**: Mechanisms for rebuilding trust after conflicts

### Integration with Technical Architecture

```rust
/// Extending AgentBehavior trait with social awareness
pub trait SocialBehavior: AgentBehavior {
    fn evaluate_social_context(&mut self, 
        social_context: &SocialContext,
        behavior_context: &BehaviorContext
    ) -> SocialActionSet;
    
    fn handle_social_event(&mut self, event: &SocialEvent);
    fn social_motivations(&self) -> Vec<SocialMotivation>;
}

/// Extending SimulationContext with social layers
pub struct SocialContext<'a> {
    pub base_context: &'a SimulationContext<'a>,
    pub relationships: &'a RelationshipSystem,
    pub cultural_system: &'a CulturalSystem,
    pub information_system: &'a InformationSystem,
    pub reputation_network: &'a ReputationSystem,
    pub current_disputes: &'a [Dispute],
}
```

### Emergent Gameplay Examples

**Resource Cooperation Scenario:**
1. Agent A discovers rich mineral deposit in dangerous mountain area
2. Agent A's relationship network determines who they share information with
3. Trusted agents form expedition group, each contributing specialized skills
4. Success strengthens all relationships and improves group reputation
5. Other agents observe and try to form similar cooperative groups
6. Cultural norm emerges: "dangerous extraction requires group cooperation"

**Cultural Innovation Scenario:**
1. Agent B develops new building technique during resource shortage
2. Technique spreads to B's trusted network first  
3. Early adopters benefit, improving their reputation
4. Social pressure builds for others to adopt the innovation
5. Cultural split emerges between adopters and traditionalists
6. Resolution comes through demonstration challenges or mediation

## Cultural-Mythology-Engine: Storytelling Systems

### Historical Memory Systems

Agents create, maintain, and share stories about important events through structured narrative systems:

```rust
pub struct CulturalMemorySystem {
    // Event tracking and significance assessment
    historical_events: Vec<HistoricalEvent>,
    event_significance_tracker: SignificanceTracker,
    
    // Story creation and evolution
    cultural_narratives: Vec<CulturalNarrative>,
    story_propagation_network: StoryNetwork,
    
    // Belief system integration
    cultural_beliefs: BeliefSystem,
    ritual_practices: RitualSystem,
    sacred_locations: SacredLocationRegistry,
}

#[derive(Clone, Debug)]
pub struct HistoricalEvent {
    pub event_id: EventId,
    pub event_type: EventType,
    pub participants: Vec<AgentId>,
    pub location: WorldPos,
    pub timestamp: u64,
    pub impact_metrics: ImpactMetrics,
    pub cultural_significance: f32,
}

#[derive(Clone, Debug)]
pub enum EventType {
    HeroicDiscovery { resource_type: ResourceType, value: f32 },
    SuccessfulMediation { dispute_resolved: DisputeId },
    CulturalInnovation { innovation: Innovation },
    TragedicLoss { agents_lost: Vec<AgentId>, cause: LossCause },
    FoundingAchievement { settlement_established: SettlementId },
}
```

### Legend Formation

Significant agent actions become stories that spread through the population:

```rust
pub struct LegendFormationSystem {
    legend_templates: Vec<LegendTemplate>,
    hero_archetypes: Vec<HeroArchetype>,
    legend_propagation_rules: Vec<PropagationRule>,
}

#[derive(Clone, Debug)]
pub struct LegendTemplate {
    pub archetype: HeroArchetype,
    pub required_achievements: Vec<Achievement>,
    pub story_elements: Vec<StoryElement>,
    pub cultural_impact: CulturalImpact,
}

#[derive(Clone, Debug)]
pub enum HeroArchetype {
    Discoverer,      // Finds valuable resources or new territories
    Peacemaker,      // Resolves conflicts and builds cooperation
    Innovator,       // Creates new techniques or cultural practices
    Protector,       // Defends against dangers or threats
    UnityBuilder,    // Brings different groups together
}
```

### Cultural Narrative Evolution

Stories change as they pass through different social networks and regions:

```rust
pub struct NarrativeEvolutionSystem {
    mutation_rules: Vec<NarrativeMutation>,
    cultural_filters: Vec<CulturalFilter>,
    geographic_variations: HashMap<RegionId, RegionalVariation>,
}

impl NarrativeEvolutionSystem {
    /// Stories evolve during transmission based on cultural context
    pub fn transmit_story(
        &mut self,
        story: &CulturalNarrative,
        from_agent: AgentId,
        to_agent: AgentId,
        cultural_context: &CulturalContext
    ) -> CulturalNarrative {
        let mut evolved_story = story.clone();
        
        // Apply cultural filters based on receiving agent's beliefs
        for filter in &self.cultural_filters {
            if filter.applies_to_context(cultural_context) {
                evolved_story = filter.transform_narrative(evolved_story);
            }
        }
        
        // Geographic variation based on regional cultural differences
        if let Some(regional_variation) = self.geographic_variations.get(&cultural_context.region_id) {
            evolved_story = regional_variation.apply_regional_flavor(evolved_story);
        }
        
        // Random mutation for cultural drift
        for mutation_rule in &self.mutation_rules {
            if mutation_rule.should_apply(&evolved_story, cultural_context) {
                evolved_story = mutation_rule.mutate_story(evolved_story);
            }
        }
        
        evolved_story
    }
}
```

### Belief System Architecture

Shared beliefs about the world emerge from collective storytelling:

```rust
pub struct BeliefSystem {
    // Hierarchical belief structure
    core_beliefs: Vec<CoreBelief>,          // Fundamental worldview (resistant to change)
    derived_beliefs: Vec<DerivedBelief>,    // From stories and experiences (moderate flexibility)
    practical_beliefs: Vec<PracticalBelief>, // Context-dependent (adapt readily)
    
    // Belief relationships and coherence
    belief_coherence_network: CoherenceNetwork,
    belief_update_mechanisms: Vec<BeliefUpdateRule>,
}

#[derive(Clone, Debug)]
pub struct CoreBelief {
    pub belief_id: BeliefId,
    pub belief_content: BeliefContent,
    pub resistance_to_change: f32,      // How strongly held this belief is
    pub cultural_foundation: CulturalFoundation,
}

#[derive(Clone, Debug)]
pub enum BeliefContent {
    DangerousTerritory { location: WorldPos, danger_type: DangerType },
    SuccessfulTechnique { technique: Technique, effectiveness: f32 },
    SocialNorm { behavior: SocialBehavior, enforcement_level: f32 },
    SacredLocation { location: WorldPos, significance: SacredSignificance },
    GroupCharacteristics { group: GroupId, traits: Vec<GroupTrait> },
}
```

### Mythological Influence on Behavior

Cultural stories and beliefs influence agent decision-making:

```rust
pub struct CulturalInfluenceSystem {
    behavior_modifiers: HashMap<BeliefId, BehaviorModifier>,
    decision_biases: Vec<CulturalBias>,
    cultural_behavior_patterns: HashMap<CulturalGroup, Vec<BehaviorPattern>>,
}

impl CulturalInfluenceSystem {
    /// Apply cultural beliefs to agent decision making
    pub fn apply_cultural_influence(
        &self,
        agent_id: AgentId,
        possible_actions: &[Action],
        cultural_context: &CulturalContext
    ) -> Vec<WeightedAction> {
        let agent_beliefs = cultural_context.get_agent_beliefs(agent_id);
        let cultural_group = cultural_context.get_agent_cultural_group(agent_id);
        
        possible_actions.iter().map(|action| {
            let mut weight = 1.0; // Base weight
            
            // Apply belief-based modifiers
            for belief in agent_beliefs {
                if let Some(modifier) = self.behavior_modifiers.get(&belief.belief_id) {
                    weight *= modifier.apply_to_action(action, belief);
                }
            }
            
            // Apply cultural behavior patterns
            if let Some(patterns) = self.cultural_behavior_patterns.get(&cultural_group) {
                for pattern in patterns {
                    weight *= pattern.evaluate_action_compatibility(action);
                }
            }
            
            // Apply cultural biases
            for bias in &self.decision_biases {
                weight *= bias.apply_bias(action, cultural_context);
            }
            
            WeightedAction {
                action: action.clone(),
                weight,
                cultural_reasoning: self.generate_cultural_reasoning(action, agent_beliefs),
            }
        }).collect()
    }
}
```

### Ritual and Tradition Systems

Commemorative behaviors emerge around significant events or locations:

```rust
pub struct RitualSystem {
    established_rituals: Vec<EstablishedRitual>,
    ritual_emergence_conditions: Vec<EmergenceCondition>,
    sacred_calendar: SacredCalendar,
    ritual_participation_tracking: ParticipationTracker,
}

#[derive(Clone, Debug)]
pub struct EstablishedRitual {
    pub ritual_id: RitualId,
    pub ritual_type: RitualType,
    pub triggering_conditions: Vec<TriggerCondition>,
    pub required_participants: ParticipantRequirements,
    pub ritual_actions: Vec<RitualAction>,
    pub cultural_significance: f32,
    pub social_bonding_effect: f32,
}

#[derive(Clone, Debug)]
pub enum RitualType {
    Commemorative { commemorated_event: EventId },
    Seasonal { season: Season, agricultural_significance: f32 },
    LifeCycle { life_event: LifeEvent },
    Protection { protected_against: DangerType },
    Prosperity { prosperity_domain: ProsperityDomain },
}
```

### Technical Integration

The cultural mythology system integrates with existing architecture through:

**Performance Optimizations:**
- Leverages existing SpatialIndex for efficient cultural transmission
- Uses batch processing for cultural updates
- Shared cultural elements reduce memory overhead

**Agent Architecture Integration:**
- Extends AgentEventSystem with cultural event types
- Modifies agent behavior through cultural influence systems
- Uses existing trait-based behavior system for cultural pattern application

**Emergent Complexity:**
- Simple rules create rich cultural evolution
- Geographic barriers naturally create cultural variation
- Social networks enable realistic story propagation patterns

## Rust-Specialist: Implementation Patterns

### Memory Layout & Performance: Structure-of-Arrays Optimization

Building on the HeightMap's flat `Vec<f32>` pattern, optimal agent data layout:

```rust
/// High-performance agent storage using SoA layout optimized for cache efficiency
pub struct AgentSystem {
    // Hot data - accessed every frame for rendering/collision
    positions: Vec<Vec2>,           // 8 bytes * n agents
    velocities: Vec<Vec2>,          // 8 bytes * n agents  
    agent_types: Vec<AgentType>,    // 1 byte * n agents (enum)
    bounds_radii: Vec<f32>,         // 4 bytes * n agents (simplified to radius)
    
    // Warm data - accessed during behavior updates
    health_values: Vec<f32>,        // 4 bytes * n agents
    energy_values: Vec<f32>,        // 4 bytes * n agents
    behavior_states: Vec<u8>,       // 1 byte * n agents (state machine index)
    
    // Cold data - accessed occasionally
    agent_ids: Vec<AgentId>,        // 8 bytes * n agents
    cultural_group_ids: Vec<CulturalGroupId>, // 4 bytes * n agents
    
    // Type-specific data stores (separate allocations)
    npc_states: SlotMap<AgentId, NPCState>,
    creature_states: SlotMap<AgentId, CreatureState>, 
    player_states: SlotMap<AgentId, PlayerState>,
    
    // Spatial indexing (reuses existing pattern)
    spatial_grid: SpatialGrid,
    
    // Efficient ID recycling
    free_indices: Vec<usize>,
    generation_counter: u64,
}
```

**Why this layout excels:**
- **Cache line efficiency**: Hot data fits in fewer cache lines during update loops
- **SIMD readiness**: Contiguous float arrays enable vectorized operations  
- **Memory predictability**: Allocation patterns match HeightMap performance characteristics
- **Separate cold storage**: Type-specific data doesn't pollute hot paths

### Trait System Design: Zero-Cost Abstractions

Trait hierarchy that compiles to efficient assembly while maintaining extensibility:

```rust
/// Core agent interface - designed for monomorphization efficiency
pub trait Agent: Send + Sync + 'static {
    type State: AgentState;
    type Behavior: AgentBehavior<Self::State>;
    
    /// Agent's unique identifier (no heap allocation)
    fn id(&self) -> AgentId;
    
    /// Update agent - hot path optimized
    #[inline]
    fn update(&mut self, dt: f32, context: &SimulationContext) -> UpdateResult;
    
    /// Agent type for dispatch optimization
    fn agent_type() -> AgentType where Self: Sized;
    
    /// Memory layout for SoA integration
    fn extract_hot_data(&self) -> AgentHotData;
    fn restore_from_hot_data(&mut self, data: &AgentHotData);
}

/// Compile-time behavior dispatch
pub trait AgentBehavior<S: AgentState>: Send + Sync {
    /// Behavior update - inlined for zero overhead
    #[inline]
    fn update(&mut self, state: &S, context: &BehaviorContext) -> ActionSet;
    
    /// Behavior priority for switching logic
    fn priority(&self) -> u8; // u8 instead of f32 for cache efficiency
}

/// Marker trait for agent state - enables type safety
pub trait AgentState: Send + Sync + Clone + 'static {
    /// Validate state consistency (debug builds only)
    #[cfg(debug_assertions)]
    fn validate(&self) -> Result<(), StateError>;
    
    /// Serialize for networking (feature-gated)
    #[cfg(feature = "networking")]
    fn serialize(&self) -> Vec<u8>;
}
```

**Rust-specific optimizations:**
- **Monomorphization**: Each agent type gets specialized code, no vtable overhead
- **Inline hints**: Critical paths inline for optimal assembly generation
- **Associated types**: Zero-cost type relationships without trait objects
- **Feature gates**: Optional functionality doesn't bloat release builds

### Ownership & Borrowing: Mutable Access Patterns

For agents updating each other's state, use this borrowing pattern:

```rust
/// Safe mutable access to agent data during updates
pub struct AgentUpdateContext<'a> {
    // Immutable world data
    heightmap: &'a HeightMap,
    water_layer: &'a WaterLayer,
    climate_system: &'a ClimateSystem,
    
    // Mutable agent access through controlled borrowing
    agent_queries: &'a mut AgentQuerySystem,
    spatial_events: &'a mut SpatialEventBuffer,
}

/// Query system that prevents conflicting borrows
pub struct AgentQuerySystem {
    agent_positions: Vec<Vec2>,      // Read-only during updates
    pending_moves: Vec<(AgentId, Vec2)>, // Write-only buffer
    interaction_requests: Vec<InteractionRequest>, // Deferred processing
}
```

**Borrowing pattern benefits:**
- **No RefCell overhead**: Compile-time borrow checking, not runtime
- **Clear data flow**: Request/response pattern makes dependencies explicit
- **Batch processing**: Deferred updates improve cache efficiency
- **Deadlock prevention**: No circular borrowing possible

### Error Handling: Integration with thiserror

Building on existing error handling patterns:

```rust
use thiserror::Error;

/// Agent system errors that integrate with existing error types
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Invalid spawn position: {position:?} (reason: {reason})")]
    InvalidSpawnPosition { position: Vec2, reason: String },
    
    #[error("Agent {agent_id:?} not found")]
    AgentNotFound { agent_id: AgentId },
    
    #[error("Spatial boundary violation: {position:?} outside world bounds")]
    SpatialBoundaryViolation { position: Vec2 },
    
    #[error("Agent state inconsistent: {details}")]
    StateInconsistency { details: String },
    
    #[error("Cultural system error: {0}")]
    CulturalSystem(#[from] CulturalError),
    
    #[error("Spatial partitioning error: {0}")]
    SpatialPartitioning(#[from] SpatialError),
}

/// Type-safe result types for agent operations
pub type AgentResult<T> = Result<T, AgentError>;
pub type SpawnResult = AgentResult<AgentId>;
pub type UpdateResult = AgentResult<AgentUpdateInfo>;
```

### Concurrency Readiness: Spatial Partitioning Structure

Prepare for future parallelization without current overhead:

```rust
/// Spatial partitioning designed for future parallel processing
pub struct ConcurrencyReadySpatialGrid {
    // Partition data for parallel processing
    partitions: Vec<SpatialPartition>,
    partition_size: Vec2,
    partition_dependencies: Vec<Vec<usize>>, // Which partitions affect others
    
    // Current serial processing (parallel feature-gated)
    #[cfg(not(feature = "parallel"))]
    _parallel_disabled: (),
}

impl ConcurrencyReadySpatialGrid {
    /// Update agents with optional parallelization
    pub fn update_agents(&mut self, agents: &mut [AgentData], dt: f32) {
        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            self.partitions.par_iter_mut()
                .for_each(|partition| partition.update_agents_in_partition(agents, dt));
        }
        
        #[cfg(not(feature = "parallel"))]
        {
            for partition in &mut self.partitions {
                partition.update_agents_in_partition(agents, dt);
            }
        }
    }
}
```

### Integration Patterns: Clean Architecture

Integrate with existing systems without tight coupling:

```rust
/// Extension trait for HeightMap integration
pub trait HeightMapAgentExtensions {
    /// Get elevation with agent-optimized interpolation
    fn agent_elevation(&self, world_pos: Vec2) -> f32;
    
    /// Check if position is navigable for agent movement
    fn is_navigable(&self, world_pos: Vec2, agent_type: AgentType) -> bool;
    
    /// Get movement cost for pathfinding integration
    fn movement_cost(&self, from: Vec2, to: Vec2, agent_type: AgentType) -> f32;
}

impl HeightMapAgentExtensions for HeightMap {
    #[inline]
    fn agent_elevation(&self, world_pos: Vec2) -> f32 {
        // Reuse existing interpolation logic
        self.interpolate_at(
            world_pos.x * (self.width() as f32 - 1.0),
            world_pos.y * (self.height() as f32 - 1.0)
        )
    }
    
    fn is_navigable(&self, world_pos: Vec2, agent_type: AgentType) -> bool {
        let elevation = self.agent_elevation(world_pos);
        match agent_type {
            AgentType::Creature => elevation > 0.1 && elevation < 0.9, // Land creatures
            AgentType::NPC => elevation > 0.2 && elevation < 0.8,      // More restrictive
            AgentType::Player => elevation > 0.0,                      // Can go anywhere
        }
    }
}
```

### Type Safety: Preventing Common Agent System Bugs

Use Rust's type system to prevent typical agent system issues:

```rust
/// Newtype wrappers for spatial coordinates to prevent mixing coordinate systems
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldPos(Vec2);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridPos { x: u32, y: u32 }

/// Compile-time checked agent state transitions
#[derive(Debug, Clone)]
pub enum NPCBehaviorState {
    Idle { energy: f32 },
    Moving { target: WorldPos, progress: f32 },
    Interacting { target_agent: AgentId, interaction_type: InteractionType },
    Dead, // Terminal state
}

impl NPCBehaviorState {
    /// Type-safe state transitions
    pub fn transition_to_moving(self, target: WorldPos) -> Result<NPCBehaviorState, StateTransitionError> {
        match self {
            NPCBehaviorState::Idle { energy } if energy > 0.1 => {
                Ok(NPCBehaviorState::Moving { target, progress: 0.0 })
            }
            NPCBehaviorState::Dead => Err(StateTransitionError::DeadAgentAction),
            _ => Err(StateTransitionError::InvalidTransition),
        }
    }
}

/// Bounded numeric types for agent properties
use bounded_integer::BoundedU8;

pub type Health = BoundedU8<0, 100>;      // 0-100 health, no invalid values
pub type Energy = BoundedU8<0, 100>;      // 0-100 energy, enforced at compile time
pub type BehaviorPriority = BoundedU8<1, 10>; // 1-10 priority, never zero
```

### Cultural System Memory Layout Recommendations

For the cultural mythology engine, use similar SoA patterns:

```rust
/// High-performance cultural data storage
pub struct CulturalMemorySystem {
    // Story data (hot path for propagation)
    story_positions: Vec<WorldPos>,        // Where stories originated
    story_influences: Vec<f32>,            // Current influence strength  
    story_agent_counts: Vec<u32>,          // How many agents know each story
    
    // Story content (cold storage)
    story_narratives: SlotMap<StoryId, StoryNarrative>,
    story_metadata: SlotMap<StoryId, StoryCulturalData>,
    
    // Agent cultural knowledge (sparse storage)
    agent_cultural_data: HashMap<AgentId, AgentCulturalKnowledge>,
    
    // Social network (compressed adjacency representation)
    social_network: CompressedSocialGraph,
}
```

## Integration Summary

This architecture maintains the high-performance characteristics achieved with HeightMap while enabling rich social and cultural systems. The key is leveraging Rust's zero-cost abstractions and ownership model to prevent common agent system bugs while preserving cache efficiency and real-time performance.

**Performance Foundation:**
- Structure-of-arrays patterns for 2-3x speedup potential
- Cache-friendly memory layouts following HeightMap success
- SIMD-ready data structures for future optimization

**Social Dynamics:**
- Natural cooperation through emergent benefit rather than forced mechanics
- Relationship systems with momentum and emotional memory
- Cultural transmission following social networks and geographic patterns

**Cultural Evolution:**
- Historical memory creating world depth over time
- Story evolution through transmission across social networks
- Belief systems that influence agent decision-making
- Ritual emergence around significant events and locations

**Rust Implementation:**
- Zero-cost abstractions with compile-time optimization
- Type safety preventing common agent system bugs
- Clean integration with existing simulation systems
- Concurrency readiness for future parallelization

The architecture serves as a solid foundation for rapid iteration on agent behaviors while maintaining the 60fps performance target with hundreds of agents.