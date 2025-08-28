# Temporal Scaling Architecture Implementation Plan

ABOUTME: Comprehensive plan for implementing multi-rate temporal architecture to fix 3,650x ecological scaling violation
ABOUTME: Generated from zen planner analysis - addresses temporal scaling while preserving ecosystem dynamics

## Executive Summary

**Problem**: Ecosystem dynamics run 3,650x too fast (10.0 kg/m²/day instead of realistic 1-3 kg/m²/year)  
**Solution**: Multi-rate temporal architecture with configurable scaling modes  
**Approach**: Temporal abstraction layer preserving current dynamics while enabling scientific realism

---

## Implementation Phases

```
Phase 1: Foundation        Phase 2: Realistic Mode      Phase 3: Research & Polish
=================         =====================        ======================
Architecture Analysis  -> Scientific Rate Calc     -> Configurable Scaling
Framework Design       -> Ecosystem Integration     -> Extension Architecture  
Demo Mode Preservation -> Cross-System Validation   -> Performance Optimization
      |                        |                           |
   [GATE]                   [GATE]                     [GATE]
```

---

## Phase 1: Foundation Analysis & Framework Design

### Step 1: Current Code Architecture Analysis
- **Primary Focus**: `/src/engine/physics/ecosystem_feedback.rs:272`
- **Secondary Analysis**: 
  - Configuration system patterns in `/src/engine/config/`
  - Simulation timestep management in main loop
  - Current temporal coupling between physics systems

### Step 2: Temporal Framework Architecture
- **Core Components**:
  ```rust
  pub enum TemporalMode {
      Demo,      // Current 3,650x speed - observable changes
      Realistic, // Proper ecological timescales  
      Research,  // Custom scaling for studies
  }
  
  pub struct TemporalScaling {
      mode: TemporalMode,
      scaling_factors: HashMap<String, f64>,
  }
  ```
- **Integration Strategy**: Use existing scale-aware architecture patterns
- **System Boundaries**: Additive framework - no core physics modifications

### Step 3: Demo Mode Preservation Implementation
- **Critical Requirement**: Bit-identical behavior to current implementation
- **Validation Method**: Ecosystem state comparison after 1000 simulation ticks  
- **Success Gate**: Must achieve perfect preservation before proceeding

---

## Phase 2: Realistic Mode Implementation

### Step 4: Scientific Rate Calculation  
- **Target Rates**: 1-3 kg/m²/year (realistic ecological growth)
- **Scaling Factor**: 0.000274 (1/3650 of current Demo rate)
- **Documentation**: Scientific rationale for ecological timescale selection

### Step 5: Ecosystem Integration
- **Modification Scope**: Replace hardcoded rates with temporally-scaled versions
- **Key Changes**: 
  - `growth_rate` calculations
  - `decay_rate` adjustments  
  - `stress_response` temporal scaling
- **Preservation**: Maintain exact mathematical relationships between processes

### Step 6: Cross-System Validation
- **Validation Criteria**:
  - Realistic mode produces 1-3 kg/m²/year growth rates
  - Ecological processes remain coherent (drought stress, recovery patterns)
  - Performance impact <5% (maintain >333 ticks/10s)
- **Quality Assurance**: Mathematical validation using existing Metis framework

---

## Phase 3: Research Mode & Polish  

### Step 7: Research Mode Implementation
- **Configurable Scaling**: User-defined temporal multipliers (0.1x to 100x)
- **Use Cases**:
  - Accelerated research (10x current rates)
  - Slow-motion analysis (0.1x rates)
  - Custom research scenarios
- **Interface**: Runtime configuration for temporal mode switching

### Step 8: Extension Architecture & Future-Proofing
- **Extension Points**: Framework for non-ecosystem temporal processes
- **Architecture**: Clean interfaces supporting future temporal complexity
- **Documentation**: Guidelines for extending temporal scaling to new systems

---

## Critical Success Criteria

### Validation Gates
```
Gate 1: Architecture Understanding
├─ Complete temporal rate structure mapping
├─ Configuration integration path identified  
└─ System dependencies documented

Gate 2: Demo Mode Preservation  
├─ Bit-identical ecosystem behavior (1000 tick test)
├─ No disruption to existing physics systems
└─ Performance maintained (>333 ticks/10s)

Gate 3: Realistic Mode Validation
├─ Growth rates within 1-3 kg/m²/year range
├─ Ecological processes remain coherent
└─ Scientific accuracy confirmed
```

### Performance Requirements
- **Baseline**: >350 ticks/10s current performance
- **Target**: >333 ticks/10s (95% of baseline)
- **Maximum Overhead**: <5% from temporal calculations

---

## Risk Mitigation Strategies

### Primary Risks & Rollback Options

**Risk 1: Demo Mode Can't Be Preserved Exactly**
- **Rollback**: Simple rate constant replacement approach  
- **Alternative**: Configuration-only solution with external rate files

**Risk 2: Performance Degrades >5%**
- **Optimization**: Cache scaling factor calculations
- **Fallback**: Pre-compute temporal multipliers at initialization  

**Risk 3: Integration Proves Too Complex**  
- **Scope Reduction**: Implement ecosystem-only solution initially
- **Future Extension**: Add temporal scaling to other systems later

---

## Implementation Execution Plan

### Immediate Actions (First Session)

**Action 1: Architecture Analysis** [30 minutes]
- Read `/src/engine/physics/ecosystem_feedback.rs` completely
- Identify all temporal rate constants and their current values  
- Examine `/src/engine/config/` to understand configuration patterns
- Review main simulation loop in `/src/main.rs` or `/src/sim.rs`
- Document current temporal architecture and dependencies

**Action 2: Framework Design** [45 minutes]  
- Create detailed `TemporalMode` enum specification  
- Design `TemporalScaling` struct with rate calculation methods
- Define configuration integration approach using existing patterns
- Create architectural diagrams showing system boundaries
- Specify exact interfaces that preserve existing system isolation

**Action 3: Demo Mode Foundation** [60 minutes]
- Implement basic temporal framework with Demo mode only
- Create comprehensive behavioral preservation tests
- Verify exact current behavior reproduction
- Establish performance baseline measurements
- **CRITICAL GATE: Must achieve bit-identical Demo mode before proceeding**

### Follow-Up Sessions Plan

**Session 2: Realistic Mode Implementation** [2-3 hours]
- Calculate precise ecological scaling factors (1/3650 = 0.000274)
- Implement Realistic mode with scientific growth rates  
- Modify ecosystem_feedback.rs to use temporal scaling
- Scientific validation of ecological behavior coherence
- Performance validation and optimization if needed

**Session 3: Research Mode & Polish** [2 hours]  
- Add configurable scaling factor system
- Implement runtime temporal mode switching
- Create usage examples and documentation
- Extension architecture for future temporal processes
- Final performance optimization and benchmarking

---

## Expected Outcome

**Result**: Temporal scaling violation resolved while preserving excellent ecosystem dynamics  
**User Benefit**: Choice between observable Demo mode and scientifically accurate Realistic mode  
**Foundation**: Clean temporal architecture ready for future extensions

---

## Status

- **Plan Status**: Complete and validated
- **Ready for Execution**: Yes - all dependencies identified and success criteria defined
- **First Action**: Begin architecture analysis of ecosystem_feedback.rs
- **Critical Success Factor**: Demo mode preservation gates all progress
- **Primary Risk**: Framework complexity - mitigated by incremental approach

---

**Generated**: August 28, 2025  
**Planning Method**: zen planner with comprehensive strategic analysis  
**Validation**: Requirements verified, dependencies mapped, risks mitigated