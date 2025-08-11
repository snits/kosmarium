# Atmospheric Physics System Redesign - TDD & Agile Plan

**ABOUTME: Comprehensive TDD task structure for fixing impossible 135 m/s uniform winds**
**ABOUTME: Physics-first approach to replace artificial wind generation with realistic atmospheric dynamics**

## Executive Summary

Based on the atmospheric wind band assessment, the root cause is **artificial uniform wind generation (135 m/s)** that violates basic atmospheric physics. This plan systematically redesigns the wind generation algorithm using Test-Driven Development and agile methodology to implement realistic pressure-driven atmospheric dynamics.

**Root Issue**: `generate_geostrophic_winds()` creates uniform 135 m/s winds independent of pressure gradients
**Target Solution**: Implement proper geostrophic balance `f × v ≈ -(1/ρ)∇P` with realistic 5-25 m/s speeds

---

## Phase 0: SageMath Mathematical Validation (Epic 0)
**Objective**: Validate atmospheric physics mathematics before Rust implementation
**Duration**: 1 implementation session
**Physics Focus**: Mathematical correctness and numerical stability analysis

### User Story 0.1: SageMath Atmospheric Physics Validation
**As a** climate scientist **I want** to validate atmospheric physics equations in SageMath **so that** I can catch mathematical errors before Rust implementation.

**Acceptance Criteria**:
- [ ] Implement geostrophic balance `f × v = -(1/ρ)∇P` in SageMath
- [ ] Test numerical stability near equator where f→0
- [ ] Validate coordinate system consistency (NH cyclones counterclockwise)
- [ ] Test realistic pressure gradients generate realistic wind speeds
- [ ] Validate Rossby number calculations and physics regime boundaries
- [ ] Document parameter bounds and safety thresholds
- [ ] Create analytical test cases with known solutions

**Expected Deliverables**:
- `atmospheric_physics_validation.sage` - Complete mathematical validation
- Parameter bounds documentation (pressure gradients, Coriolis thresholds)
- Numerical stability analysis and mitigation strategies
- Analytical test cases for Rust implementation validation

**Expected Commit**: `math: validate atmospheric physics equations in SageMath before implementation`

### User Story 0.2: Multi-Physics System SageMath Validation Plan
**As a** systems architect **I want** SageMath validation for all physics systems **so that** I can prevent mathematical errors across the entire simulation.

**Acceptance Criteria**:
- [ ] Create SageMath validation plan for water flow physics
- [ ] Design erosion & sediment transport mathematical validation
- [ ] Plan climate system coupling mathematics validation
- [ ] Design geological evolution time-scale validation
- [ ] Plan agent spatial indexing mathematical validation
- [ ] Document validation workflow for future physics systems

**Expected Deliverables**:
- `multi_physics_sagemath_validation_plan.md` - Comprehensive plan
- Template for physics system mathematical validation
- Integration plan with existing TDD workflow

**Expected Commit**: `plan: add SageMath validation framework for all physics systems`

---

## Phase 1: Diagnostic Foundation (Epic 1)
**Objective**: Establish physics validation framework before redesign
**Duration**: 1-2 implementation sessions
**Physics Focus**: Create testable validation for atmospheric physics principles

### User Story 1.1: Geostrophic Balance Validation Framework
**As a** climate scientist **I want** to validate geostrophic balance across the domain **so that** I can detect physics violations before implementing fixes.

**Acceptance Criteria**:
- [ ] Implement `validate_geostrophic_balance()` function that checks `f × v ≈ -(1/ρ)∇P`
- [ ] Create test for detecting current 135 m/s artificial winds
- [ ] Measure pressure-wind coupling correlation (should be near zero currently)
- [ ] Generate diagnostic report showing physics violations
- [ ] All tests pass with current broken system (documents current state)

**Test Requirements**:
```rust
#[test]
fn test_current_geostrophic_balance_is_violated() {
    // This test SHOULD FAIL with current system - documents the problem
    let validation = system.validate_geostrophic_balance(&pressure, &wind);
    assert!(validation.balance_error > 100.0); // Massive violation
    assert!(validation.coupling_correlation < 0.1); // Near zero correlation
}
```

**Expected Commit**: `test: add geostrophic balance validation framework detecting current physics violations`

### User Story 1.2: Atmospheric Scale Analysis Framework
**As a** climate scientist **I want** to validate atmospheric scaling relationships **so that** I can ensure domain-appropriate physics.

**Acceptance Criteria**:
- [ ] Implement Rossby number calculation: `Ro = U/(fL)`
- [ ] Add scale-appropriate wind speed validation (5-25 m/s for continental domains)
- [ ] Create dimensional analysis validation for atmospheric equations
- [ ] Test framework detects current scale violations (uniform 135 m/s regardless of domain size)
- [ ] Quality gates: All diagnostic tests pass

**Test Requirements**:
```rust
#[test]
fn test_atmospheric_scaling_relationships() {
    let diagnostics = system.analyze_atmospheric_scaling(&scale);
    assert!(diagnostics.rossby_number_realistic()); 
    assert!(diagnostics.wind_speeds_scale_appropriate());
    // Current system should fail these tests
}
```

**Expected Commit**: `test: add atmospheric scaling diagnostics framework`

### User Story 1.3: Mass Conservation Diagnostic Enhancement
**As a** climate scientist **I want** detailed mass conservation diagnostics **so that** I can track momentum accumulation sources.

**Acceptance Criteria**:
- [ ] Enhance existing `calculate_total_momentum()` with spatial analysis
- [ ] Add momentum flux boundary analysis: `∫(ρv·n)dA` around domain boundary
- [ ] Implement continuity equation validation: `∂ρ/∂t + ∇·(ρv) = 0`
- [ ] Create momentum conservation test framework
- [ ] Document current violations (should show ~13,582 m/s total momentum)

**Test Requirements**:
```rust
#[test]
fn test_mass_conservation_detailed_diagnostics() {
    let conservation = system.analyze_mass_conservation(&wind);
    assert!(conservation.total_momentum_magnitude < scale.acceptable_threshold());
    assert!(conservation.boundary_flux_balanced());
    // Current system should fail with ~13,582 m/s momentum
}
```

**Expected Commit**: `test: enhance mass conservation diagnostics with spatial analysis`

---

## Phase 2: Realistic Pressure Field Generation (Epic 2)
**Objective**: Create physically realistic pressure fields that drive proper wind patterns
**Duration**: 2-3 implementation sessions
**Physics Focus**: Replace arbitrary pressure generation with atmospheric dynamics

### User Story 2.1: Realistic Pressure Gradient Generator
**As a** climate scientist **I want** realistic pressure gradients **so that** I can drive proper geostrophic winds.

**Acceptance Criteria**:
- [ ] Replace current pressure generation with scale-appropriate gradients (0.1-2.0 Pa/m)
- [ ] Implement smooth pressure field without boundary step functions
- [ ] Add synoptic-scale pressure patterns (1000-3000 km wavelengths)
- [ ] Ensure pressure gradients match realistic atmospheric values
- [ ] Test validates pressure field continuity across boundaries

**Test Requirements**:
```rust
#[test]
fn test_realistic_pressure_gradients() {
    let pressure = generator.create_realistic_pressure_field(&scale);
    let gradients = pressure.get_all_gradients();
    
    // Realistic continental pressure gradients: 0.1-2.0 Pa/m
    assert!(gradients.iter().all(|g| g.magnitude() >= 0.1 && g.magnitude() <= 2.0));
    assert!(!pressure.has_boundary_discontinuities());
    assert!(pressure.has_synoptic_scale_patterns(&scale));
}
```

**Expected Commit**: `feat: implement realistic atmospheric pressure gradient generator`

### User Story 2.2: Pressure-Elevation Coupling
**As a** climate scientist **I want** pressure fields coupled to terrain elevation **so that** I can create realistic orographic effects.

**Acceptance Criteria**:
- [ ] Implement hydrostatic pressure variation: `P(z) = P₀ * exp(-z/H)`
- [ ] Add orographic pressure perturbations over mountains
- [ ] Ensure pressure field responds to terrain barriers
- [ ] Create lee wave and windward/leeward pressure effects
- [ ] Test validates hydrostatic balance with terrain

**Test Requirements**:
```rust
#[test]
fn test_pressure_elevation_coupling() {
    let pressure = system.generate_pressure_with_terrain(&heightmap, &scale);
    
    // Higher elevation should have lower pressure (hydrostatic)
    assert!(pressure.validates_hydrostatic_balance(&heightmap));
    assert!(pressure.has_orographic_effects());
}
```

**Expected Commit**: `feat: add pressure-elevation coupling with hydrostatic balance`

### User Story 2.3: Synoptic Weather Pattern Integration
**As a** climate scientist **I want** realistic high/low pressure systems **so that** I can generate natural weather patterns.

**Acceptance Criteria**:
- [ ] Add pressure system generator (cyclones, anticyclones)
- [ ] Implement realistic pressure amplitudes (±200-500 Pa from mean)
- [ ] Create traveling pressure systems with proper spatial scales
- [ ] Ensure pressure systems drive realistic wind patterns
- [ ] Test validates weather pattern pressure signatures

**Test Requirements**:
```rust
#[test]
fn test_synoptic_pressure_patterns() {
    let weather_systems = generator.create_weather_patterns(&scale);
    
    assert!(weather_systems.has_realistic_pressure_amplitudes());
    assert!(weather_systems.spatial_scales_appropriate(&scale));
    assert!(weather_systems.pressure_gradients_drive_realistic_winds());
}
```

**Expected Commit**: `feat: add synoptic weather pattern pressure generation`

---

## Phase 3: Geostrophic Wind Algorithm Redesign (Epic 3)
**Objective**: Replace artificial 135 m/s wind generation with pressure-driven geostrophic balance
**Duration**: 3-4 implementation sessions (CRITICAL PHASE)
**Physics Focus**: Core atmospheric dynamics implementation

### User Story 3.1: Replace Artificial Wind Generation
**As a** climate scientist **I want** to eliminate artificial uniform wind generation **so that** I can implement physics-based winds.

**Acceptance Criteria**:
- [ ] Remove code generating uniform 135 m/s winds
- [ ] Replace with null wind field (zero initial conditions)
- [ ] Ensure wind generation only from pressure gradients
- [ ] Add debug logging showing wind generation source
- [ ] Test confirms no artificial wind speeds exist

**Test Requirements**:
```rust
#[test]
fn test_no_artificial_wind_generation() {
    let wind = system.generate_winds_from_pressure(&pressure, &scale);
    
    // No uniform artificial speeds should exist
    assert!(!wind.has_uniform_speeds());
    assert!(wind.max_speed() < 50.0); // Reasonable upper bound
    assert!(wind.all_speeds_pressure_derived(&pressure));
}
```

**Expected Commit**: `refactor: remove artificial uniform wind generation algorithm`

### User Story 3.2: Implement True Geostrophic Balance
**As a** climate scientist **I want** proper geostrophic balance implementation **so that** winds follow pressure gradients correctly.

**Acceptance Criteria**:
- [ ] Implement `f × v = -(1/ρ)∇P` with proper vector cross product
- [ ] Add latitude-dependent Coriolis parameter calculation
- [ ] Handle equatorial and polar cases appropriately
- [ ] Ensure numerical stability for small Coriolis parameters
- [ ] Test validates geostrophic balance equation

**Test Requirements**:
```rust
#[test]
fn test_true_geostrophic_balance_implementation() {
    let wind = system.calculate_geostrophic_winds(&pressure, &scale);
    
    // Validate geostrophic balance at each cell
    for (x, y) in domain.all_cells() {
        let balance_error = validate_geostrophic_balance_at_cell(x, y, &wind, &pressure);
        assert!(balance_error < 0.01); // 1% tolerance
    }
    
    assert!(wind.wind_speeds_realistic()); // 5-25 m/s range
    assert!(wind.follows_pressure_gradients(&pressure));
}
```

**Expected Commit**: `feat: implement true geostrophic balance wind calculation`

### User Story 3.3: Scale-Appropriate Wind Speed Validation
**As a** climate scientist **I want** realistic wind speeds for domain scale **so that** atmospheric dynamics match physical expectations.

**Acceptance Criteria**:
- [ ] Implement wind speed validation: 5-25 m/s for continental domains
- [ ] Add scale-dependent maximum wind limits
- [ ] Create wind speed clamping with physical justification
- [ ] Ensure winds scale properly with domain size
- [ ] Test validates wind speeds match atmospheric observations

**Test Requirements**:
```rust
#[test]
fn test_scale_appropriate_wind_speeds() {
    let wind = system.generate_realistic_winds(&pressure, &scale);
    
    // Continental domains: typical winds 5-25 m/s
    assert!(wind.mean_speed() >= 5.0 && wind.mean_speed() <= 25.0);
    assert!(wind.max_speed() <= scale.maximum_realistic_wind_speed());
    assert!(wind.speed_distribution_realistic());
}
```

**Expected Commit**: `feat: implement scale-appropriate wind speed validation and limits`

### User Story 3.4: Gradient Wind Implementation for Enhanced Realism
**As a** climate scientist **I want** gradient wind effects **so that** I can include curvature effects in strong pressure systems.

**Acceptance Criteria**:
- [ ] Implement gradient wind balance: `v²/R + fv = (1/ρ)|∇P|`
- [ ] Add centripetal acceleration effects in curved flow
- [ ] Handle cyclonic vs anticyclonic curvature appropriately
- [ ] Ensure gradient winds reduce to geostrophic winds for straight flow
- [ ] Test validates gradient wind physics

**Test Requirements**:
```rust
#[test]
fn test_gradient_wind_implementation() {
    let pressure = create_curved_pressure_system(&scale);
    let wind = system.calculate_gradient_winds(&pressure, &scale);
    
    assert!(wind.includes_curvature_effects());
    assert!(wind.cyclonic_anticyclonic_asymmetry_correct());
    assert!(wind.reduces_to_geostrophic_for_straight_flow());
}
```

**Expected Commit**: `feat: add gradient wind effects for curved flow patterns`

---

## Phase 4: Natural Boundary Condition Integration (Epic 4)
**Objective**: Replace forcing zero-velocity boundaries with physics-based outflow
**Duration**: 2-3 implementation sessions
**Physics Focus**: Mass-conserving atmospheric boundaries

### User Story 4.1: Pressure-Based Boundary Conditions
**As a** climate scientist **I want** boundary conditions based on pressure gradients **so that** I can eliminate artificial velocity forcing.

**Acceptance Criteria**:
- [ ] Replace zero-velocity boundary forcing with pressure-based outflow
- [ ] Implement natural atmospheric boundary conditions
- [ ] Ensure outflow preserves pressure gradient relationships
- [ ] Add inflow/outflow determination from pressure field
- [ ] Test validates natural boundary behavior

**Test Requirements**:
```rust
#[test]
fn test_pressure_based_boundary_conditions() {
    let (pressure, wind) = system.solve_with_natural_boundaries(&initial_pressure, &scale);
    
    assert!(!wind.has_forced_zero_boundaries());
    assert!(wind.boundaries_follow_pressure_gradients(&pressure));
    assert!(wind.natural_inflow_outflow_patterns());
}
```

**Expected Commit**: `feat: implement pressure-based natural boundary conditions`

### User Story 4.2: Mass-Conserving Outflow Implementation
**As a** climate scientist **I want** mass-conserving outflow boundaries **so that** momentum doesn't accumulate artificially.

**Acceptance Criteria**:
- [ ] Implement `∫(ρv·n)dA = 0` around domain boundary
- [ ] Add outflow velocity calculation maintaining mass balance
- [ ] Ensure no artificial momentum sources at boundaries
- [ ] Create adaptive outflow damping based on local conditions
- [ ] Test validates mass conservation across boundaries

**Test Requirements**:
```rust
#[test]
fn test_mass_conserving_outflow() {
    let wind = system.apply_mass_conserving_boundaries(&initial_wind, &scale);
    
    let boundary_flux = wind.calculate_boundary_mass_flux();
    assert!(boundary_flux.magnitude() < scale.conservation_threshold());
    assert!(wind.total_momentum_magnitude() < scale.acceptable_momentum());
}
```

**Expected Commit**: `feat: implement mass-conserving atmospheric outflow boundaries`

### User Story 4.3: Adaptive Sponge Layer Replacement
**As a** climate scientist **I want** physics-based atmospheric damping **so that** boundaries don't fight the pressure-driven flow.

**Acceptance Criteria**:
- [ ] Replace current sponge layer with physics-based damping
- [ ] Implement atmospheric boundary layer effects
- [ ] Add scale-appropriate damping that preserves geostrophic balance
- [ ] Ensure damping doesn't create artificial pressure gradients
- [ ] Test validates boundary stability without physics violations

**Test Requirements**:
```rust
#[test]
fn test_physics_based_boundary_damping() {
    let wind = system.apply_physics_damping(&pressure_driven_wind, &scale);
    
    assert!(wind.preserves_geostrophic_balance_near_boundaries());
    assert!(wind.boundary_damping_physically_justified());
    assert!(!wind.creates_artificial_pressure_effects());
}
```

**Expected Commit**: `feat: replace sponge layer with physics-based atmospheric damping`

---

## Phase 5: System Integration and Validation (Epic 5)
**Objective**: Integrate all components and validate complete atmospheric physics
**Duration**: 2-3 implementation sessions
**Physics Focus**: End-to-end atmospheric dynamics validation

### User Story 5.1: Complete Atmospheric Physics Integration
**As a** climate scientist **I want** integrated atmospheric physics system **so that** all components work together correctly.

**Acceptance Criteria**:
- [ ] Integrate pressure generation + geostrophic winds + natural boundaries
- [ ] Ensure all atmospheric physics principles are satisfied
- [ ] Add complete system validation framework
- [ ] Create end-to-end atmospheric simulation test
- [ ] Validate system produces realistic wind patterns

**Test Requirements**:
```rust
#[test]
fn test_complete_atmospheric_integration() {
    let atmosphere = AtmosphericSystem::new_with_realistic_physics(&scale);
    let (pressure, wind) = atmosphere.simulate_atmospheric_dynamics(100); // 100 time steps
    
    assert!(atmosphere.geostrophic_balance_satisfied(&pressure, &wind));
    assert!(atmosphere.mass_conserved(&wind));
    assert!(atmosphere.realistic_wind_speeds(&wind));
    assert!(atmosphere.natural_boundary_behavior(&wind));
}
```

**Expected Commit**: `feat: integrate complete realistic atmospheric physics system`

### User Story 5.2: Atmospheric Pattern Validation
**As a** climate scientist **I want** validation against real atmospheric patterns **so that** I can confirm realistic behavior.

**Acceptance Criteria**:
- [ ] Compare generated patterns to real atmospheric observations
- [ ] Validate weather system characteristics (size, intensity, motion)
- [ ] Ensure cyclonic rotation directions correct (NH: counterclockwise)
- [ ] Add statistical validation of wind speed distributions
- [ ] Test atmospheric patterns match theoretical expectations

**Test Requirements**:
```rust
#[test]
fn test_atmospheric_pattern_realism() {
    let patterns = system.generate_weather_patterns(&realistic_pressure, &scale);
    
    assert!(patterns.cyclonic_rotation_correct_for_hemisphere());
    assert!(patterns.weather_system_scales_realistic());
    assert!(patterns.wind_speed_statistics_match_observations());
}
```

**Expected Commit**: `feat: validate atmospheric patterns against real-world observations`

### User Story 5.3: Performance and Stability Validation
**As a** climate scientist **I want** stable long-term atmospheric simulation **so that** I can run extended climate models.

**Acceptance Criteria**:
- [ ] Validate system stability over 1000+ time steps
- [ ] Ensure no numerical instabilities in geostrophic calculation
- [ ] Add performance benchmarks for new atmospheric system
- [ ] Verify memory usage remains reasonable
- [ ] Test system handles various domain sizes correctly

**Test Requirements**:
```rust
#[test]
fn test_atmospheric_system_stability() {
    let results = system.run_extended_simulation(1000, &scale);
    
    assert!(results.numerically_stable());
    assert!(results.performance_acceptable());
    assert!(results.memory_usage_reasonable());
    assert!(results.works_across_all_scales());
}
```

**Expected Commit**: `feat: validate atmospheric system performance and long-term stability`

---

## Quality Gates and Validation Steps

### Pre-Implementation Quality Gates:
- [ ] All diagnostic tests implemented and documented
- [ ] Physics equations validated against literature
- [ ] Code-reviewer approval for each user story
- [ ] Test coverage > 90% for atmospheric physics

### Implementation Quality Gates (Per User Story):
- [ ] All acceptance criteria tests pass
- [ ] TDD cycle: Red → Green → Refactor completed
- [ ] Code-reviewer approval before commit
- [ ] No regression in existing atmospheric system functionality
- [ ] Physics validation confirms improvement

### Phase Completion Gates:
- [ ] Epic validation tests pass
- [ ] Integration tests confirm phase objectives met
- [ ] Performance benchmarks show no degradation
- [ ] Documentation updated with physics principles
- [ ] Code-reviewer final approval for phase

### Final System Validation:
- [ ] **ROOT CAUSE RESOLVED**: No more 135 m/s artificial winds
- [ ] **PHYSICS VALIDATED**: Geostrophic balance `f × v ≈ -(1/ρ)∇P` satisfied
- [ ] **REALISTIC SPEEDS**: Wind speeds 5-25 m/s for continental domains  
- [ ] **MASS CONSERVED**: Total momentum < threshold for all scales
- [ ] **NATURAL BOUNDARIES**: No forced zero-velocity conditions
- [ ] **SYSTEM STABLE**: Long-term simulation stability confirmed

---

## Expected Commit Sequence (17 Commits Total):

### Phase 0 (2 commits):
1. `math: validate atmospheric physics equations in SageMath before implementation`
2. `plan: add SageMath validation framework for all physics systems`

### Phase 1 (3 commits):
3. `test: add geostrophic balance validation framework detecting current physics violations`
4. `test: add atmospheric scaling diagnostics framework`
5. `test: enhance mass conservation diagnostics with spatial analysis`

### Phase 2 (3 commits):
6. `feat: implement realistic atmospheric pressure gradient generator`
7. `feat: add pressure-elevation coupling with hydrostatic balance`
8. `feat: add synoptic weather pattern pressure generation`

### Phase 3 (4 commits):
9. `refactor: remove artificial uniform wind generation algorithm`
10. `feat: implement true geostrophic balance wind calculation`
11. `feat: implement scale-appropriate wind speed validation and limits`
12. `feat: add gradient wind effects for curved flow patterns`

### Phase 4 (3 commits):
13. `feat: implement pressure-based natural boundary conditions`
14. `feat: implement mass-conserving atmospheric outflow boundaries`
15. `feat: replace sponge layer with physics-based atmospheric damping`

### Phase 5 (2 commits):
16. `feat: integrate complete realistic atmospheric physics system`
17. `feat: validate atmospheric system performance and long-term stability`

---

## Risk Analysis and Potential Issues

### High-Risk Areas:
1. **Numerical Stability**: Geostrophic calculation `v = -(1/fρ)∇P × k` can become unstable near equator where `f → 0`
   - **Mitigation**: Add numerical stability limits and handle equatorial case specially
   - **Test**: Validate stability across all latitudes including near-equatorial regions

2. **Scale Transition**: Ensuring smooth behavior across domain sizes (100 km to 10,000 km)
   - **Mitigation**: Use continuous scaling functions instead of threshold-based parameters
   - **Test**: Validate physics consistency across scale transitions

3. **Boundary Condition Stability**: Replacing forced boundaries with natural outflow may cause initial instability
   - **Mitigation**: Phase implementation gradually, validate mass conservation at each step
   - **Test**: Long-term stability tests with various pressure configurations

### Medium-Risk Areas:
1. **Performance Regression**: New physics calculations may slow simulation
   - **Mitigation**: Profile critical paths, optimize geostrophic calculation loops
   - **Test**: Performance benchmarks comparing old vs new system

2. **Integration Complexity**: Multiple atmospheric physics components must work together
   - **Mitigation**: Incremental integration with validation at each step
   - **Test**: End-to-end integration tests with realistic scenarios

### Low-Risk Areas:
1. **Code Architecture**: Existing `AtmosphericSystem` structure can accommodate new physics
2. **Testing Framework**: Existing test infrastructure suitable for physics validation
3. **Scaling Parameters**: `ScaleAware` system ready for new atmospheric parameters

---

## Success Metrics

### Quantitative Success Criteria:
- **Wind Speed Realism**: 95% of domain has winds 5-25 m/s (vs current 135 m/s uniform)
- **Geostrophic Balance**: Balance error < 5% across domain (vs current complete decoupling)  
- **Mass Conservation**: Total momentum < 100 m/s (vs current ~13,582 m/s)
- **Pressure Gradients**: 0.1-2.0 Pa/m range (vs current boundary discontinuities)
- **System Stability**: 1000+ time step stability (vs current instability)

### Qualitative Success Criteria:
- **Physics Realism**: Atmospheric patterns match theoretical expectations
- **Boundary Behavior**: Natural wind flow at boundaries without artificial forcing
- **Weather Patterns**: Realistic cyclones/anticyclones with proper rotation
- **Scaling Consistency**: Appropriate atmospheric behavior across all domain sizes
- **Code Quality**: Clean, testable, maintainable atmospheric physics implementation

---

## Implementation Notes

### Key Physics Equations to Implement:
1. **Geostrophic Balance**: `f × v = -(1/ρ)∇P`
2. **Gradient Wind**: `v²/R + fv = (1/ρ)|∇P|`
3. **Continuity Equation**: `∂ρ/∂t + ∇·(ρv) = 0`
4. **Hydrostatic Balance**: `∂P/∂z = -ρg`

### Critical Implementation Details:
- Handle `f = 2Ω sin(φ) → 0` at equator with numerical stability limits
- Implement proper vector cross product for Coriolis force
- Use central differences for pressure gradients with boundary treatments
- Add physical wind speed limits to prevent numerical blow-up

### Testing Strategy:
- **Unit Tests**: Each physics calculation (geostrophic balance, pressure gradients)
- **Integration Tests**: Complete atmospheric system behavior
- **Physics Validation**: Comparison with theoretical expectations
- **Regression Tests**: Ensure no performance degradation
- **Scale Tests**: Validate behavior across domain sizes

This comprehensive plan addresses the root cause (artificial wind generation) through systematic TDD implementation, ensuring each step builds proper atmospheric physics that will eliminate the persistent 135 m/s wind band artifacts and create realistic atmospheric dynamics.