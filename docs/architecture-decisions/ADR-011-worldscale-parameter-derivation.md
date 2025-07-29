# ADR-011: WorldScale Parameter Derivation Architecture

**Status:** Accepted  
**Date:** 2025-01-29  
**Deciders:** Jerry, Claude  

## Context

The simulation system needed to work consistently across vastly different map sizes (64x64 to 4096x4096) while maintaining realistic behavior. The initial implementation had fixed per-cell parameters that caused larger maps to accumulate disproportionately more water, creating unrealistic results.

### Problem Statement
- **Scale Dependency**: Parameters tuned for 240x120 maps produced unrealistic results at other scales
- **Total vs Per-Cell Effects**: Fixed rainfall per cell meant larger maps got proportionally more total water input
- **Physical vs Resolution Scale**: Same resolution could represent vastly different real-world areas
- **Library Usability**: Users shouldn't need to manually retune dozens of parameters when changing map sizes

### Expert Consultation
Consulted simulation-engineer and world-generation-architect specialists who confirmed this is a fundamental scaling challenge in procedural generation. Professional systems solve this through:
- Physical unit consistency
- Area-normalized parameters  
- Multi-scale modeling approaches
- Grid independence testing

## Decision

Implement a **WorldScale Parameter Derivation** architecture with three core components:

### 1. WorldScale Context
```rust  
pub struct WorldScale {
    pub physical_size_km: f64,     // Real-world size represented
    pub resolution: (u32, u32),    // Output detail level
    pub detail_level: DetailLevel, // Quality vs performance trade-off
}
```

Separates **what** you're modeling from **how detailed** the output is.

### 2. Raw vs Derived Parameter Separation
```rust
// Scale-independent base values
pub struct WaterFlowParameters {
    pub base_rainfall_rate: f32,
    pub rainfall_scaling: RainfallScaling,
    // ... other raw parameters
}

// Scale-aware system with effective values
pub struct WaterFlowSystem {
    pub parameters: WaterFlowParameters,
    pub effective_rainfall_rate: f32,  // Computed for current scale
}
```

### 3. ScaleAware Trait Pattern
```rust
pub trait ScaleAware {
    fn derive_parameters(&self, scale: &WorldScale) -> Self;
}
```

Creates transformation pipeline: Raw Parameters → Scale Context → Effective Parameters

## Alternatives Considered

### Alternative 1: Manual Parameter Scaling
- **Approach**: Document parameter relationships, let users calculate scaling factors
- **Rejected**: Places burden on library users, error-prone, not composable

### Alternative 2: Fixed Scale Modes  
- **Approach**: Predefined "small/medium/large" parameter sets
- **Rejected**: Not flexible enough, arbitrary boundaries, doesn't handle arbitrary scales

### Alternative 3: Distributed Scaling Logic
- **Approach**: Each system handles its own scaling independently
- **Rejected**: No coordination between systems, harder to test, coupling issues

### Alternative 4: ECS Component Architecture
- **Approach**: Separate Raw and Effective parameter components with transformation systems
- **Considered**: Clean separation of concerns, very testable
- **Deferred**: Would require ECS refactor, can migrate to this pattern later

## Rationale

### Primary Benefits

**Scale Independence**: Systems work consistently from prototype (64x64) to production (4096x4096) scales without parameter retuning.

**Separation of Concerns**: 
- Raw parameters represent algorithm behavior
- Scale context represents world characteristics  
- Derivation logic handles the transformation
- Each can be tested independently

**Composability**: Multiple systems (water, terrain, climate) can all use the same scaling context while implementing domain-specific derivation logic.

**Library Usability**: Users specify their world scale once, all systems automatically derive appropriate parameters.

**Debuggability**: Both raw and effective parameters are inspectable, making it easy to understand why a system behaves a certain way at a given scale.

### Implementation Strategy

**Reference Scale Approach**: All parameters calibrated for 240x120 reference resolution, with scaling factors computed relative to this baseline.

**Configurable Scaling Modes**:
- `PerCell`: Same parameter value regardless of map size (simple, predictable)
- `Density`: Scale to maintain consistent behavior across map sizes (recommended default)
- Future: `Physical` mode for real-world unit specifications

## Consequences

### Positive
- **Eliminates scale-dependent parameter tuning** - major usability win for library users
- **Testable architecture** - can test raw parameters, scaling logic, and effective results separately  
- **Extensible pattern** - template for making any generation system scale-aware
- **Professional approach** - follows patterns used in commercial generation tools
- **Educational value** - demonstrates separation of concerns and context-aware design

### Negative
- **Added complexity** - more types and indirection than simple parameter structs
- **Migration effort** - existing code needs updates to use new API
- **Learning curve** - users need to understand scale vs parameter distinction

### Neutral
- **Performance impact** - minimal (one-time parameter derivation per system)
- **Memory overhead** - small (storing both raw and effective parameters)

## Compliance

### Validation Requirements
- **Grid independence tests**: Results should converge as resolution increases for same physical scale
- **Scale consistency tests**: Different resolutions of same physical area should produce similar total effects
- **Parameter documentation**: Clear specification of what each raw parameter represents

### Migration Path
1. ✅ Implement core WorldScale types and ScaleAware trait
2. ✅ Refactor WaterFlowSystem to use new architecture  
3. ✅ Update all tests to use scale-aware constructors
4. 🚧 Add comprehensive scaling validation tests
5. ⏳ Document scaling guidelines for library users
6. ⏳ Extend pattern to terrain generation systems
7. ⏳ Add physical unit support for scientific applications

## Notes

This architecture emerged from collaborative problem-solving with domain experts, demonstrating the value of specialist consultation in architectural decisions. The pattern is inspired by ECS thinking (separation of raw data, derived data, and transformation logic) but implemented in a simpler, non-ECS context.

The decision validates the educational approach of problem-driven architecture discovery rather than premature pattern application.

## References
- Expert consultation with simulation-engineer on professional scaling approaches
- Expert consultation with world-generation-architect on multi-scale generation patterns
- Analysis of commercial tools (World Machine, Gaea) parameter scaling strategies
- `/docs/scaling-guidelines.md` - User-facing documentation of scaling system