# Computational Hydrologist: Boundary Flow Threshold Analysis

## ABOUTME: Detailed hydrological analysis of flow threshold blocking boundary outflow
## ABOUTME: Calculates realistic thresholds for 5x3 test domain with 0.2m water depth

## Test Case Configuration

### Physical Setup
- **Domain**: 5×3 grid representing continental-scale cells
- **Default World Scale**: 10km physical size → 2km per cell
- **Topographic Gradient**: 1.0 → 0.2 elevation (0.8 units over 4 cells = 0.2 gradient per cell)
- **Initial Water Depth**: 0.2m on left side (x=0,1)
- **Expected Flow Direction**: Left to right toward boundary

### Current System Parameters
```rust
// From WaterFlowParameters::default()
base_rainfall_rate: 0.0000027127  // Optimized for 240×120 reference
evaporation_rate: 0.001
flow_rate: 0.1
rainfall_scaling: MassConserving  // Scales with 1/area
```

### Scale-Derived Parameters
For 5×3 domain using default Simulation::new():
```rust
// WorldScale defaults to 10km physical size
cell_size = 10km / 5 = 2km per cell
reference_area = 240 × 120 = 28,800 cells
test_area = 5 × 3 = 15 cells
scaling_factor = 28,800 / 15 = 1,920

// Mass-conserving rainfall scaling
effective_rainfall_rate = 0.0000027127 × 1,920 = 0.00521 m/tick

// Evaporation threshold calculation
post_evap_rainfall = 0.00521 × (1 - 0.001) = 0.00520
evaporation_threshold = 0.00520 × 0.01 = 0.0000520 m
```

## Hydrological Problem Analysis

### Root Cause: Missing Flow Threshold in Current Code

The simulation uses **1e-8** as flow threshold in `move_water()`, but diagnostic tools reference `evaporation_threshold × 10.0` which would be:

```
diagnostic_flow_threshold = 0.0000520 × 10 = 0.000520 m
```

This threshold (0.52mm) is **26 times larger** than the computational precision threshold (1e-8 ≈ 0.01mm).

### Manning's Equation Analysis

For the test case (0.2m depth, 0.2 gradient per 2km cell):

```
Manning's Equation: v = (1/n) × R^(2/3) × S^(1/2)

Where:
- n = 0.035 (natural channels)
- R = hydraulic radius ≈ depth = 0.2m for sheet flow
- S = slope = 0.2 / 2000m = 0.0001 (per meter)

v = (1/0.035) × (0.2)^(2/3) × (0.0001)^(1/2)
v = 28.57 × 0.292 × 0.01
v = 0.083 m/s = 83 mm/s

Flow rate per unit width:
q = v × d = 0.083 × 0.2 = 0.0166 m²/s per meter width
```

### Expected Boundary Outflow

For 2km cell width with continuous flow:
```
Volume flux = 0.0166 m²/s × 2000m = 33.2 m³/s per cell
Time per tick ≈ 1 simulation second (assumed)
Volume per tick = 33.2 m³

Initial water volume per cell = 0.2m × (2000m)² = 800,000 m³
Expected outflow rate = 33.2 / 800,000 = 0.0000415 = 4.15% per tick

Over 10 ticks: 1 - (1-0.0415)^10 ≈ 34% total loss
```

**Expected Result**: ~34% water loss over 10 ticks
**Current Result**: 0.39% water loss (85× too small)

## Threshold Analysis

### Current Thresholds Block Flow

1. **CFL Velocity Limit**: max_velocity = 0.5 cells/timestep
   - Physical velocity = 0.5 × 2000m / 1s = 1000 m/s (unrealistic)
   - Manning velocity = 0.083 m/s → 0.0000415 cells/tick
   - **CFL is not the limiting factor**

2. **Flow Amount Calculation**:
   ```rust
   flow_amount = water_depth × velocity_magnitude × cfl_limit
   flow_amount = 0.2 × 0.0000415 × 0.5 = 0.00000415 m
   ```

3. **Flow Threshold Comparison**:
   - Current: 1e-8 = 0.00001 mm ✓ (allows flow)
   - Missing: 0.000520 m = 0.52 mm ✗ (blocks flow)

**The diagnostic flow threshold would block all realistic flow!**

## Recommended Solution

### Fix 1: Remove Excessive Flow Threshold

Replace any `evaporation_threshold × 10.0` flow thresholds with computational precision:

```rust
// Current problematic threshold (if used):
let flow_threshold = self.evaporation_threshold * 10.0; // 0.52mm - blocks everything

// Recommended fix:
let flow_threshold = 1e-8; // 0.01mm - computational precision only
```

### Fix 2: Scale-Aware Flow Validation

For erosion and other processes that need significant flow:

```rust
// Erosion should require substantial flow
let erosion_flow_threshold = self.evaporation_threshold * 20.0; // Keep this
let erosion_depth_threshold = self.evaporation_threshold * 5.0;  // Keep this

// But basic water movement should use minimal threshold
let movement_threshold = 1e-8; // Computational precision
```

### Fix 3: Verify Velocity Calculation

Check that `water.velocity` is being set correctly by the flow direction algorithm. The gradient should produce:

```
Expected velocity ∝ √(gradient × gravity × depth)
Expected velocity ∝ √(0.0001 × 9.81 × 0.2) = 0.044 m/s
```

Close to Manning's equation result (0.083 m/s).

## Implementation Guidance

### Code Changes Needed

1. **Verify no 10× threshold in move_water()**:
   ```bash
   grep -n "evaporation_threshold.*10" src/engine/sim.rs
   ```

2. **Check velocity calculation accuracy**:
   ```rust
   // In calculate_velocities() or equivalent
   let gradient = (neighbor_elevation - current_elevation) / cell_size_m;
   let velocity_magnitude = flow_rate * sqrt(gradient * gravity * water_depth);
   ```

3. **Test with reduced CFL limit**:
   ```rust
   // Temporary test: reduce CFL to match realistic velocities
   let max_velocity = 0.00005; // Match Manning's equation scale
   ```

## Validation Metrics

### Success Criteria
- **Boundary Outflow**: >1% water loss (currently 0.39%)
- **Physical Realism**: Flow velocities 0.05-0.1 m/s for given gradient
- **Mass Conservation**: Total outflow matches hydraulic expectations
- **Numerical Stability**: No oscillations or negative water depths

### Expected Results After Fix
- **10-tick simulation**: 10-30% water loss
- **Monotonic decrease**: Most ticks show water loss
- **Realistic velocities**: 50-100 mm/s flow speeds
- **Smooth flow patterns**: Water concentrates in channels toward boundary

## Conclusion

The boundary outflow issue is caused by missing or excessive flow thresholds that block realistic water movement. The current 1e-8 threshold in `move_water()` is appropriate, but any 10× evaporation threshold would be hydrologicallyunrealistic.

**Primary fix**: Ensure flow thresholds don't exceed computational precision limits.
**Secondary fix**: Verify velocity calculations match Manning's equation expectations.
**Validation**: Test should achieve >1% boundary outflow with realistic flow patterns.