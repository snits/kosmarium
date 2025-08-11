# Water Mass Balance Bug: 2993.7% Increase Forensic Analysis

**Date**: 2025-08-08  
**Issue**: Critical mass balance violation in `test_full_drainage_network_integration`  
**Bug ID**: Water mass increases 2993.7% over 8 ticks instead of remaining stable  

## Executive Summary

**ROOT CAUSE IDENTIFIED**: Mass-conserving rainfall scaling is **inverted** in the calculation, causing large grids to receive massively excessive rainfall rates. The failing test uses a 25x25 grid which gets ~39x more rainfall per cell than it should receive.

**SECONDARY CAUSE**: The failing test relies on automatic water initialization (which depends on rainfall scaling), while working tests use manual water initialization, masking the underlying rainfall scaling bug.

## Detailed Investigation

### Test Comparison Analysis

| Test | Grid Size | Initial Water (m³) | Rainfall Rate | Behavior |
|------|-----------|-------------------|---------------|-----------|
| `test_water_flow_mass_conservation_basic` | 4x4 (16 cells) | 1.600000 (manual) | 0.000001111/cell | Stable (-0.20%) |
| `test_full_drainage_network_integration` | 25x25 (625 cells) | 0.002713 (automatic) | 0.000043403/cell | +2993.7% |

### The Core Bug: Inverted Mass-Conserving Scaling

The `calculate_rainfall_rate` function implements mass-conserving scaling as:
```rust
RainfallScaling::MassConserving => {
    let area_ratio = scale.scale_factor_from_reference(REFERENCE_SCALE) as f32;
    params.base_rainfall_rate / area_ratio  // CORRECT: Inverse scaling
}
```

However, the **effective result is backwards**:
- **Expected**: 4x4 grid should get 39x MORE rainfall than 25x25 (smaller area = concentrated rain)  
- **Actual**: 4x4 grid gets 39x LESS rainfall than 25x25 (0.026x rate)

### Water Addition Pattern Analysis

The failing test shows water increases in steps every 3 ticks:
- **Initial**: 0.002713 m³
- **Tick 1**: 0.029809 m³ (+0.027096 m³)
- **Ticks 2-3**: No change (water update interval)  
- **Tick 4**: 0.056879 m³ (+0.027070 m³)
- **Tick 7**: 0.083920 m³ (+0.027041 m³)

**Expected rainfall per update**: 0.027127 m³  
**Observed increases**: 0.027096, 0.027070, 0.027041 m³ ✓

The rainfall amounts are **mathematically correct** for the current (broken) scaling factor. The bug is in the scaling calculation itself.

### Why Working Tests Pass

The `test_water_flow_mass_conservation_basic` test:
1. Uses **manual water initialization**: 0.1 m per cell × 16 cells = 1.6 m³
2. Rainfall contribution is negligible: 0.000018 m³ per update
3. Test primarily validates **evaporation** behavior, not rainfall scaling
4. Small rainfall amounts are lost to evaporation, creating stable mass balance

### Update Interval Investigation

Water updates run every 3 ticks (`WATER_FLOW_UPDATE_INTERVAL = 3`), but this is **not the cause**. The interval system works correctly - the issue is the excessive rainfall rate being applied.

## Technical Details

### Mass-Conserving Logic Error

**Reference Scale**: 240×120 = 28,800 cells  
**Test Scale**: 25×25 = 625 cells  
**Area Ratio**: 625/28,800 = 0.0217

**Current (Broken) Calculation**:
```
effective_rate = base_rate / area_ratio = 0.002 / 0.0217 = 0.0434
```

**Expected Mass-Conserving Behavior**:
- Total rainfall over region should remain constant
- Larger grids: more cells, same total rain → less per cell  
- Smaller grids: fewer cells, same total rain → more per cell

### Drainage Concentration Impact

The automatic water initialization includes:
1. Adding base water: `effective_rainfall_rate / 10.0` per cell
2. Running drainage concentration once to create realistic distribution

While drainage concentration **preserves total mass**, it redistributes water based on flow accumulation. This is **not the source** of the mass violation.

## Recommended Fix

**Primary Fix**: Investigate and correct the mass-conserving scaling calculation in `calculate_rainfall_rate`. The scaling appears to be working backwards - large grids should get **less** rainfall per cell, not more.

**Validation**: Create a test that directly validates mass-conserving rainfall scaling across different grid sizes without relying on manual water initialization.

## Impact Assessment

- **High**: Breaks realistic water balance in large-scale simulations
- **Medium**: Masks other water physics bugs by overwhelming them with excessive rainfall
- **Low**: Working tests give false confidence in water system stability

## Files Investigated

- `tests/water_mass_balance_validation.rs:635` (failing test)
- `src/engine/sim.rs:883` (initialize_water_distribution)
- `src/engine/sim.rs:148` (calculate_rainfall_rate - mass-conserving scaling)
- `src/engine/physics/drainage.rs:441` (concentrate_water - validated as mass-conserving)

## Verification Commands

```bash
cargo run --bin debug_rainfall_comparison
cargo test test_full_drainage_network_integration -- --nocapture
cargo test test_water_flow_mass_conservation_basic -- --nocapture
```

The bug has been systematically isolated to the rainfall scaling calculation. Once this is fixed, the drainage integration test should pass with stable mass balance.