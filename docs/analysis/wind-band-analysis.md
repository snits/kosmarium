# Atmospheric Physics Analysis: Blue Wind Vector Band at 4096km Scale

## Executive Summary

Jerry has observed a consistent horizontal band of blue wind vectors toward the top of the atmospheric simulation at 4096km scale. From an atmospheric physics perspective, this pattern likely represents **a boundary condition artifact** rather than natural atmospheric physics, with potential contributions from **inadequate Coriolis force modeling** at this scale.

## Detailed Atmospheric Physics Analysis

### 1. Scale Context: 4096km Domain
At 4096km scale, we are dealing with **continental to sub-continental scale dynamics** where several atmospheric physics principles become critical:

- **Rossby Deformation Radius**: ~1000-2000km at mid-latitudes
- **Synoptic Scale Motions**: Weather systems typically 1000-3000km
- **Geostrophic Balance**: Should dominate at scales >100km
- **Coriolis Parameter Variation**: β-plane effects become significant

### 2. Boundary Condition Analysis

#### Physical Expectation vs. Observed Pattern
**Natural Atmospheric Behavior**: At 4096km scale, winds should show:
- Gradual transitions near domain edges
- Realistic outflow patterns 
- No sharp horizontal banding
- Smooth geostrophic adjustment

**Observed Artifact**: A distinct horizontal band suggests:
- **Zero-gradient boundary extrapolation** creating artificial wind accumulation
- **Insufficient sponge layer damping** in the upper boundary region
- **Latitude-dependent Coriolis effects** creating artificial wind patterns

#### Boundary Condition Implementation Issues

From the code analysis, the current boundary conditions use:
```rust
// North boundary (y = 0): extrapolate from y = 1
for x in 0..width {
    if height > 1 {
        let velocity = self.velocity.get(x, 1).clone();
        self.velocity.set(x, 0, velocity);
    }
}
```

**Problem**: This simple extrapolation **does not account for**:
1. **Geostrophic adjustment** near boundaries
2. **Mass conservation constraints** at domain edges
3. **Realistic atmospheric outflow physics**

### 3. Coriolis Force and Latitude Mapping Issues

#### Latitude Assignment Problem
At 4096km scale, the current latitude mapping assigns:
```rust
if self.world_scale.physical_size_km <= CONTINENTAL_THRESHOLD_KM {
    // Uses ±2.5° variation around 45°N (42.5°N to 47.5°N range)
} else {
    // Maps to full latitude range [-90°, +90°]
}
```

**Critical Issue**: 4096km significantly exceeds the 1000km threshold, triggering **global-scale latitude mapping** which creates:
- **Extreme Coriolis parameter variation** across the domain
- **Unrealistic polar effects** in the northern region
- **Artificial wind band formation** due to excessive f-parameter gradients

#### Geostrophic Wind Generation Problem
The geostrophic wind calculation:
```rust
let latitude_rad = self.grid_y_to_latitude(y, height);
let f = self.coriolis_parameter_at_latitude(latitude_rad);
```

At the upper boundary (y=0, representing "North"), this creates:
- **Very high Coriolis parameter** values (approaching polar regions)
- **Excessive geostrophic wind speeds** due to f being in denominator
- **Wind speed clamping** to artificial limits, creating uniform "blue band"

### 4. Physical vs. Numerical Diagnosis

#### This is Likely a **Numerical Artifact** because:

1. **Scale Mismatch**: 4096km is being treated as global-scale when it should be continental
2. **Inappropriate Coriolis Scaling**: Using full ±90° latitude range for continental domain
3. **Boundary Condition Inadequacy**: Simple extrapolation cannot handle geostrophic flows
4. **Wind Speed Clamping**: Artificial limits create visual banding

#### **NOT** Natural Atmospheric Physics because:

1. **Real atmospheres show gradual transitions** near domain boundaries
2. **Synoptic systems at this scale** don't create sharp horizontal bands
3. **Geostrophic balance** should produce smooth, curved flow patterns
4. **Mass conservation** would prevent persistent accumulation at boundaries

### 5. Specific Recommendations for Physics Corrections

#### A. Fix Latitude Mapping for Continental Domains
```rust
// Recommend changing CONTINENTAL_THRESHOLD_KM from 1000.0 to 5000.0
const CONTINENTAL_THRESHOLD_KM: f64 = 5000.0; 

// For 4096km domains, use continental-scale latitude variation (~±5°)
// instead of global-scale mapping
```

#### B. Implement Proper Geostrophic Boundary Conditions
Replace simple extrapolation with:
- **Characteristic-based outflow** conditions
- **Geostrophic adjustment** near boundaries  
- **Mass flux conservation** constraints

#### C. Add Beta-Plane Coriolis Effects
For domains >1000km, implement:
- **β-plane approximation**: f = f₀ + β*y
- **Rossby wave dynamics** for realistic large-scale flow
- **Proper planetary vorticity** gradient effects

#### D. Enhance Sponge Layer Implementation
Current sponge layer (2-8 cells) is insufficient for 4096km domains:
- **Increase sponge width** to 10-20 cells for large domains
- **Improve damping profile** with better exponential decay
- **Add momentum conservation** checks in sponge region

### 6. Validation Tests Needed

After implementing fixes:
1. **Boundary Stability**: Check that total momentum is conserved
2. **Geostrophic Balance**: Verify f × v = -∇P/ρ relationship holds
3. **Scale Consistency**: Test that 1000km and 5000km domains behave similarly
4. **Visual Inspection**: Confirm elimination of artificial banding

## Conclusion

The blue wind vector band is **almost certainly a numerical artifact** caused by inappropriate latitude scaling and inadequate boundary conditions for continental-scale domains. The 4096km domain is being treated with global-scale physics when it should use continental-scale approximations.

**Priority Fix**: Adjust `CONTINENTAL_THRESHOLD_KM` to 5000km and implement proper continental-scale Coriolis parameter mapping to eliminate the artificial wind band.

## Technical Implementation Priority

1. **Immediate (Critical)**: Fix latitude mapping threshold
2. **Short-term (High)**: Improve boundary conditions  
3. **Medium-term (Important)**: Add β-plane effects
4. **Long-term (Enhancement)**: Full primitive equation implementation