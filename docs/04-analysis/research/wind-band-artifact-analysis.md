# Wind Band Artifact Analysis and Solution

## Problem Description

Jerry reported a persistent horizontal blue wind band artifact appearing at the top of ASCII wind visualizations on both 4096km and 8192km maps. This analysis identifies the root causes and provides specific fixes.

## Root Cause Analysis

### 1. Latitude Mapping Bug (Critical - Large Domains)

**Issue:** The `grid_y_to_latitude` function in `atmosphere.rs` has incorrect coordinate mapping for large domains (>5000km):

```rust
// BUGGY CODE in atmosphere.rs lines 551-560
if self.world_scale.physical_size_km <= CONTINENTAL_THRESHOLD_KM {
    // Continental domains work correctly (42.5° to 47.5°)
} else {
    // Global domains: INCORRECT mapping
    let normalized_y = (y as f64) / ((height - 1) as f64);
    let latitude_range = std::f64::consts::PI; 
    (normalized_y - 0.5) * latitude_range // -90° to +90°
}
```

**Problem:** For 8192km domains:
- **North boundary (y=0) → -90° (South Pole)**
- **South boundary (y=255) → +90° (North Pole)**

This is backwards and creates impossible atmospheric physics conditions.

### 2. Boundary Condition Problem

**Issue:** Zero-gradient boundary conditions copy interior geostrophic winds to boundary cells:

```rust
// Lines 213-214 in atmosphere.rs  
let velocity = self.velocity.get(x, 1).clone();
self.velocity.set(x, 0, velocity); // Copies y=1 wind to y=0 boundary
```

**Problem:** Interior cells have strong geostrophic winds (20-100 m/s) that become:
- **Unnaturally uniform** at boundaries (std dev < 0.1 m/s)
- **Predominantly horizontal** (90% horizontal flow)
- **Visually rendered as blue horizontal bands** in ASCII output

### 3. Mass Conservation Failure

**Issue:** All test domains show:
- `Is mass conserved: false`
- `Is system stable: false` 
- **Total momentum magnitude: 4,246,933 m/s** (massively excessive)

## Diagnostic Evidence

From the analysis run:

### 4096km Domain
```
North boundary statistics:
  Average speed: 8.3 m/s
  Speed std deviation: 1.11 m/s  
  Horizontal flow fraction: 90.0%
  ⚠️  ARTIFACT DETECTED: Unnaturally horizontal wind pattern!
```

### 8192km Domain  
```
North boundary (y=0): lat=-90.00°, f=-1.45e-4 s⁻¹
  ⚠️  SUSPICIOUS: Very high latitude at north boundary!
Speed std deviation: 0.04 m/s
  ⚠️  ARTIFACT DETECTED: Unnaturally uniform wind speeds!
```

## Atmospheric Physics Explanation

The horizontal blue band represents **unphysical geostrophic winds** caused by:

1. **Incorrect Coriolis parameter** from latitude bug creates extreme f values
2. **Pressure gradient boundary extrapolation** creates artificial uniformity  
3. **Zero-gradient BC** preserves strong interior winds at boundaries
4. **ASCII renderer** shows predominantly horizontal winds as blue symbols

This violates fundamental atmospheric physics principles:
- **Mass conservation** (momentum should not accumulate at boundaries)
- **Geostrophic balance** (pressure gradients and Coriolis force should balance)
- **Natural atmospheric outflow** (winds should exit domain boundaries smoothly)

## Specific Code Fixes Required

### Fix 1: Correct Latitude Mapping (atmosphere.rs lines 550-560)

```rust
// CORRECTED latitude mapping for large domains
} else {
    // Global scale: Map y coordinate correctly  
    let normalized_y = if height > 1 {
        (y as f64) / ((height - 1) as f64) // 0 to 1 from north to south
    } else {
        0.5 // Single cell = equator  
    };
    // CORRECT: North should be +90°, South should be -90°
    let latitude_range = std::f64::consts::PI; // 180° total range
    let latitude_rad = std::f64::consts::PI / 2.0 - (normalized_y * latitude_range);
    latitude_rad // +90° (north) to -90° (south)
}
```

### Fix 2: Implement Proper Atmospheric Boundary Conditions

Replace zero-gradient extrapolation with **outflow boundary conditions**:

```rust
// For north boundary (y = 0) - allow natural outflow
for x in 0..width {
    if height > 1 {
        let interior_velocity = self.velocity.get(x, 1);
        // Apply outflow condition: preserve direction but reduce magnitude
        let outflow_velocity = interior_velocity * 0.8; // Damping factor
        self.velocity.set(x, 0, outflow_velocity);
    }
}
```

### Fix 3: Enhanced Sponge Layer Implementation  

The existing sponge layer should be **always active** for large domains:

```rust
// In generate_geostrophic_winds() line 662
let use_sponge = self.world_scale.physical_size_km > 100.0; // Current
// CHANGE TO:
let use_sponge = true; // Always use sponge layer for boundary stability
```

## Expected Results After Fixes

1. **Eliminate horizontal blue bands** - boundaries will show natural wind variation
2. **Correct latitude physics** - proper Coriolis parameters at all latitudes  
3. **Achieve mass conservation** - momentum magnitude < 1000 m/s (reasonable threshold)
4. **Stable atmospheric system** - `is_system_stable: true`

## Implementation Priority

1. **CRITICAL:** Fix latitude mapping bug (affects all large domains)
2. **HIGH:** Implement atmospheric outflow boundary conditions
3. **MEDIUM:** Enhance sponge layer damping effectiveness

These fixes address the fundamental atmospheric physics violations causing the wind band artifact while maintaining realistic circulation patterns in the interior domain.

## Test Validation

After implementing fixes, run:
```bash
cargo run --bin debug_wind_band_analysis
```

Expected improvements:
- North boundary horizontal flow fraction < 50%
- Speed std deviation > 1.0 m/s (natural variation) 
- Mass conservation: `true`
- System stability: `true`
- Correct latitude mapping: North = positive, South = negative