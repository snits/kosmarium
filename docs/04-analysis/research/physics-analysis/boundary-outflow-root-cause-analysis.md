# Boundary Outflow Root Cause Analysis

## ABOUTME: Computational hydrologist analysis of why 0.39% water loss occurs with large velocities
## ABOUTME: Investigates velocity-outflow disconnect in continental-scale boundary flow simulation

## Key Discovery: Velocity-Outflow Paradox

The debug analysis reveals a fundamental contradiction:

### Measured Velocities (Too Large)
- **Simulation velocity**: 0.04 cells/tick
- **Manning's equation**: 0.00004886 cells/tick  
- **Velocity ratio**: 819× too large
- **Flow amounts**: 8mm vs expected 0.01mm

### Actual Boundary Outflow (Too Small)
- **Current result**: 0.39% water loss over 10 ticks
- **Expected result**: >1% water loss  
- **Manning's prediction**: ~34% loss over 10 ticks

**The paradox: Velocities are 819× too large, but boundary outflow is 26× too small!**

## Possible Root Causes

### 1. CFL Velocity Capping
```rust
let max_velocity = 0.5; // Conservative CFL condition
let flow_amount = water.depth.get(x, y) * velocity_mag.min(max_velocity);
```

**Analysis**: This limits effective flow to 0.04 vs 0.5, so CFL is NOT the limiting factor.

### 2. Flow Direction Issues
The debug output shows velocities are correctly calculated and directed toward lower elevations. Cell (1,1) has velocity (0.028284, -0.028284) indicating diagonal flow toward boundary.

### 3. Water Redistribution vs Boundary Loss
**Critical insight**: The simulation may be **redistributing water internally** rather than losing it at boundaries.

Let me examine what happens during the flow:
- Initial: 0.2m water on cells (0,0), (0,1), (0,2), (1,0), (1,1), (1,2)
- Total initial water: 6 × 0.2m = 1.2m³
- Expected final: <1.19m³ (>1% loss)
- Actual final: 1.195m³ (0.39% loss)

### 4. Bilinear Interpolation Boundary Handling
```rust
for (tx, ty, weight) in flow_cells {
    if tx >= 0 && tx < width && ty >= 0 && ty < height {
        // Add water to target cell
    } else {
        // Flow out of bounds = boundary outflow (lost water)
        // This is the critical fix: water that flows beyond boundaries is lost
    }
}
```

**The boundary handling looks correct** - water flowing out of bounds should be lost.

### 5. Rainfall Addition Masking Outflow
```rust
// Add rainfall
self.add_rainfall(water);

// Move water based on flow directions (now drainage-aware)
self.move_water_with_boundaries(water);
```

**Critical issue**: Rainfall is added BEFORE water movement. This could be masking boundary losses:

- Rainfall rate: calculated from debug as near-zero (0.00000000)
- This is not masking the effect

### 6. Evaporation Threshold Issues
```rust
if *depth < self.evaporation_threshold {
    *depth = 0.0;
}
```

- Evaporation threshold: 1e-8 (0.01mm)
- Flow amounts: 8mm (800× larger)
- Evaporation threshold is not the issue

## Hydraulic Reality Check

### Expected Physical Behavior
For a 5×3 domain with 0.2m water depth and 0.1 gradient per cell:

1. **Manning's Flow**: 0.098 m/s → 0.049 cells/tick
2. **Volume Flux**: 33.2 m³/s per 2km cell width  
3. **Boundary Loss Rate**: 4.15% per tick
4. **10-tick Loss**: ~34% total

### Simulation Calibration Error
The flow_rate parameter (0.1) appears to be dimensionally incorrect:

```rust
velocity = normalized_direction * slope * flow_rate
velocity = direction * 0.2 * 0.1 = direction * 0.02
```

But debug shows velocity of 0.04, suggesting the calculation includes other factors.

## Diagnostic Recommendations

### 1. Track Water During Individual Tick
Create detailed tracking of:
- Water before rainfall
- Water after rainfall  
- Water after flow movement
- Water after evaporation
- Exact boundary outflow amount per tick

### 2. Verify Bilinear Interpolation
Check if fractional flow calculations are correctly distributing water to boundary vs internal cells.

### 3. Test Without Rainfall/Evaporation
Run simulation with:
```rust
// Disable rainfall and evaporation to isolate flow effects
effective_rainfall_rate = 0.0;
evaporation_rate = 0.0;
```

### 4. Velocity Field Validation
Verify that velocity field is properly established and sustained across multiple ticks.

## Hypothesis

**Primary hypothesis**: The boundary outflow calculation may have a subtle bug where water that should flow out of bounds is instead being retained or redistributed within the domain.

**Secondary hypothesis**: The flow_rate parameter needs recalibration to match Manning's equation expectations, but this doesn't explain the velocity-outflow paradox.

## Implementation Fix Strategy

1. **Immediate**: Add detailed per-tick water accounting to boundary test
2. **Validation**: Disable rainfall/evaporation to isolate pure flow effects  
3. **Calibration**: Adjust flow_rate to match Manning's equation (reduce by 819×)
4. **Verification**: Confirm >1% boundary outflow with realistic velocities

The root cause appears to be in the boundary flow implementation, not the threshold values.