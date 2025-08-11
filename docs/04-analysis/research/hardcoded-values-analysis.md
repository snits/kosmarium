# Hardcoded Values Analysis for Scale-Aware Simulation

## Executive Summary

This comprehensive analysis examines all hardcoded values found in the simulation that may require scale-aware treatment. The simulation currently operates at 512x256 resolution with 8km/pixel, creating a 4096km × 2048km domain. Many hardcoded thresholds and limits were designed for smaller reference scales (~240x120, 1km/pixel) and are causing **water accumulation blocking**, **unrealistic flow speeds**, and **biome generation issues** at continental scales.

**Critical Issues Identified:**
- Water flow thresholds too large for high-resolution grids
- CFL timestep bounds inappropriate for 8km/pixel resolution  
- Atmospheric pressure clamping preventing realistic pressure gradients
- Erosion thresholds blocking sediment transport at large scales

**Priority Actions Required:**
1. **CRITICAL**: Fix water flow and convergence thresholds (causing flow blocking)
2. **CRITICAL**: Implement scale-aware CFL timestep bounds
3. **IMPORTANT**: Remove atmospheric pressure hard limits for continental domains
4. **IMPORTANT**: Scale erosion and sediment thresholds

---

## Detailed Analysis by Category

### 1. CRITICAL - Water Flow Thresholds (Causing Simulation Bugs)

#### Issue: Flow Amount Thresholds
**Locations:**
- `sim.rs:409, 627`: `flow_amount > 0.001`
- `sim.rs:443`: `water_depth > 0.001`
- `spatial_partitioning.rs:413`: `flow_speed > 0.01 && water_depth > 0.001`

**Physical Meaning:** These thresholds prevent movement of very small water amounts to avoid numerical instability.

**Scale Dependency:** **CRITICAL** - At 8km/pixel, the effective rainfall rate is ~0.0005 (mass-conserving scaling), but the flow threshold is 0.001. This means **water cannot flow** because rainfall never exceeds the movement threshold.

**Current Impact:** This is the primary cause of Jerry's water accumulation issues. Water accumulates but cannot flow because rainfall amounts are below hardcoded thresholds.

**Recommended Fix:**
```rust
// Replace hardcoded thresholds with scale-aware values
let flow_threshold = self.water_system.evaporation_threshold * 10.0; // 10x evaporation threshold
if flow_amount > flow_threshold {
    // ... flow logic
}
```

**Priority:** **CRITICAL** - Fix immediately

---

#### Issue: Convergence Thresholds  
**Locations:**
- `spatial_partitioning.rs:267, 273`: `elevation_change > 0.001`, `water_change > 0.001`
- `spatial_partitioning.rs:288`: `has_converged(0.001)`
- `spatial_partitioning.rs:622`: `water_change > 0.001 || elevation_change > 0.001`

**Physical Meaning:** Determine when simulation changes are significant enough to continue processing.

**Scale Dependency:** **CRITICAL** - At high resolution with low rainfall rates, legitimate changes are below 0.001, causing premature convergence.

**Current Impact:** System thinks it has converged when there should still be water movement.

**Recommended Fix:**
```rust
// Scale convergence thresholds to rainfall rate
let convergence_threshold = self.water_system.effective_rainfall_rate * 0.1; // 10% of rainfall
if water_change > convergence_threshold {
    // ... continue processing
}
```

**Priority:** **CRITICAL** - Fix immediately

---

### 2. CRITICAL - CFL Timestep Bounds

#### Issue: Timestep Clamping
**Location:** `sim.rs:146`: `cfl_timestep.max(0.001).min(60.0)`

**Physical Meaning:** CFL (Courant-Friedrichs-Lewy) condition ensures numerical stability by limiting timestep based on grid spacing and velocity.

**Scale Dependency:** **CRITICAL** - At 8km/pixel (8000m grid spacing), the appropriate timestep should be much larger than the 0.001s minimum. The current bounds force unrealistically small timesteps.

**Current Impact:** Forces extremely small timesteps that slow simulation unnecessarily and may cause numerical issues.

**Recommended Fix:**
```rust
// Scale bounds based on grid resolution
let min_timestep = (scale.meters_per_pixel() as f32 / 100000.0).max(0.001).min(10.0);
let max_timestep = (scale.meters_per_pixel() as f32 / 100.0).max(60.0).min(3600.0);
cfl_timestep.max(min_timestep).min(max_timestep)
```

**Priority:** **CRITICAL** - Fix immediately

---

### 3. IMPORTANT - Atmospheric Pressure Limits (Blocking Realistic Gradients)

#### Issue: Pressure Clamping
**Locations:**
- `climate.rs:613, 661, 799, 958, 1027`: `pressure.max(50000.0).min(110000.0)`

**Physical Meaning:** Clamp atmospheric pressure between 500-1100 hPa (reasonable for sea level to mountain peaks).

**Scale Dependency:** **IMPORTANT** - On continental scales (4000km domains), natural pressure variations can exceed these limits due to large-scale weather systems.

**Current Impact:** Prevents realistic pressure gradients that drive continental-scale wind patterns, resulting in artificial uniform pressure fields.

**Recommended Fix:**
```rust
// Use scale-aware pressure bounds for continental domains
let (min_pressure, max_pressure) = if scale.physical_size_km > 1000.0 {
    (30000.0, 120000.0) // Wider range for continental scales
} else {
    (50000.0, 110000.0) // Original range for regional scales
};
pressure = pressure.max(min_pressure).min(max_pressure);
```

**Priority:** **IMPORTANT** - Fix after critical water flow issues

---

### 4. IMPORTANT - Erosion and Sediment Thresholds

#### Issue: Erosion Amount Limits
**Locations:**
- `spatial_partitioning.rs:421`: `erosion_amount.min(0.001)`
- `sim.rs:451`: `erosion_amount.min(0.001)`
- `geological_evolution.rs:236`: `sediment_amount > 0.01`

**Physical Meaning:** Limit erosion per timestep to prevent unrealistic terrain destruction. Threshold for sediment deposition.

**Scale Dependency:** **IMPORTANT** - At 8km/pixel, each cell represents 64 km² area. Erosion amounts should scale with cell area for realistic geological processes.

**Current Impact:** Erosion rates too small for continental-scale terrain evolution. Sediment thresholds may block realistic sediment transport.

**Recommended Fix:**
```rust
// Scale erosion limits based on cell area
let cell_area_km2 = (scale.meters_per_pixel() / 1000.0).powi(2);
let max_erosion = 0.001 * cell_area_km2.sqrt(); // Scale with cell size
let erosion_amount = (erosion_capacity - current_sediment).min(max_erosion);

// Scale sediment threshold similarly
let sediment_threshold = 0.01 * cell_area_km2.sqrt();
if sediment_amount > sediment_threshold {
    // ... deposition logic
}
```

**Priority:** **IMPORTANT** - Implement after water flow fixes

---

### 5. PHYSICAL CONSTANTS - Correct as Hardcoded

#### Atmospheric Constants
**Locations:**
- `atmosphere.rs:26`: `earth_rotation_rate: 7.27e-5` (rad/s)
- `atmosphere.rs:27`: `air_density_sea_level: 1.225` (kg/m³)

**Physical Meaning:** Universal physical constants.

**Scale Dependency:** **NONE** - These are fundamental physical constants that don't change with domain size.

**Recommendation:** Keep as hardcoded values.

**Priority:** No action needed

---

### 6. CONFIGURATION - Could Be Made Configurable

#### Resolution Scaling Factors
**Locations:**
- `atmospheric_moisture.rs:45`: `resolution_scale = (meters_per_pixel / 1000.0).sqrt().min(2.0)`
- `climate.rs:286, 299`: `domain_scaling = (physical_extent_km / 100.0).min(3.0)`
- `atmosphere.rs:64`: `* ((scale.meters_per_pixel() / 1000.0).min(1.0) as f32)`

**Physical Meaning:** Scaling factors to adjust parameters based on resolution and domain size.

**Scale Dependency:** These are already scale-aware but use hardcoded scaling relationships.

**Current Impact:** Generally working well, but scaling relationships could be tuned for specific simulation scenarios.

**Recommended Action:** Consider making scaling exponents and limits configurable parameters rather than hardcoded constants.

**Priority:** Low - optimize after core issues resolved

---

#### Rendering Constants  
**Locations:**
- `graphics_render.rs:122`: `alpha = (water_depth * 255.0).min(200.0)`
- `graphics_render.rs:176`: `sample_rate = (cell_size / 10.0).max(1.0)`

**Physical Meaning:** Visualization parameters for rendering water transparency and wind vector sampling.

**Scale Dependency:** Moderate - affects visual quality but not simulation physics.

**Current Impact:** May cause rendering artifacts at extreme zoom levels or resolutions.

**Recommended Action:** Make configurable or add auto-scaling based on viewport size.

**Priority:** Low - cosmetic issues only

---

### 7. DRAINAGE SYSTEM CONSTANTS

#### River and Flow Thresholds
**Locations:**
- `drainage.rs:336`: `river_accumulation_threshold: 100.0`
- `drainage.rs:339`: `concentration_factor: 10.0`
- `drainage.rs:340`: `permanent_water_threshold: 0.01`

**Physical Meaning:** Thresholds for river formation, water concentration, and permanent water body identification.

**Scale Dependency:** **IMPORTANT** - At 8km/pixel, 100 cells upstream represents massive drainage areas (64,000 km²). This threshold is too high for realistic river formation.

**Current Impact:** May prevent river formation in areas where rivers should exist, affecting biome generation.

**Recommended Fix:**
```rust
// Scale river threshold based on cell area
let cells_per_km2 = 1.0 / (scale.meters_per_pixel() / 1000.0).powi(2);
let river_threshold = (50.0 * cells_per_km2.sqrt()).max(10.0); // Adaptive threshold
```

**Priority:** **IMPORTANT** - Affects biome realism

---

## Implementation Roadmap

### Phase 1: Critical Water Flow Fixes (Immediate)
1. **Replace hardcoded flow thresholds** with scale-aware calculations based on `evaporation_threshold`
2. **Fix convergence thresholds** to use percentage of rainfall rate
3. **Implement scale-aware CFL bounds** based on grid resolution
4. **Test with 512x256, 8km/pixel domain** to verify water accumulation and flow

### Phase 2: Atmospheric and Pressure Systems (Week 2)
1. **Remove pressure hard limits** for continental domains (>1000km)
2. **Scale erosion and sediment thresholds** based on cell area
3. **Adjust drainage system thresholds** for high-resolution grids
4. **Validate atmospheric pressure gradients** drive realistic wind patterns

### Phase 3: Configuration and Optimization (Week 3-4)
1. **Make scaling factors configurable** through parameter files
2. **Add rendering auto-scaling** based on viewport and domain size
3. **Implement adaptive algorithm switching** based on domain characteristics
4. **Performance testing and optimization** for large domains

---

## Code Examples for Critical Fixes

### Scale-Aware Flow Thresholds
```rust
impl WaterFlowSystem {
    fn get_scale_aware_flow_threshold(&self) -> f32 {
        // Flow threshold should be fraction of effective rainfall to allow accumulation
        self.evaporation_threshold * 5.0 // 5x safety margin above evaporation clearing
    }
    
    fn get_scale_aware_convergence_threshold(&self) -> f32 {
        // Convergence when changes are <1% of rainfall rate
        self.effective_rainfall_rate * 0.01
    }
}
```

### Scale-Aware CFL Bounds
```rust
fn calculate_cfl_bounds(scale: &WorldScale) -> (f32, f32) {
    let grid_spacing_m = scale.meters_per_pixel() as f32;
    
    // Minimum timestep: fine enough for stability
    let min_dt = (grid_spacing_m / 100000.0).max(0.001).min(1.0);
    
    // Maximum timestep: coarse enough for efficiency  
    let max_dt = (grid_spacing_m / 10.0).max(60.0).min(3600.0);
    
    (min_dt, max_dt)
}
```

### Continental Pressure Bounds
```rust
fn get_pressure_bounds(scale: &WorldScale) -> (f32, f32) {
    if scale.physical_size_km > 1000.0 {
        // Continental scale: wider pressure range for weather systems
        (30000.0, 120000.0) // 300-1200 hPa
    } else {
        // Regional scale: standard atmospheric range
        (50000.0, 110000.0) // 500-1100 hPa  
    }
}
```

---

## Impact Assessment on Current Issues

### Water Accumulation Problems
**Root Cause:** Hardcoded flow thresholds (0.001) exceed effective rainfall rate (0.0005) at 8km/pixel resolution.

**Fix Impact:** Implementing scale-aware thresholds will immediately resolve water accumulation blocking and restore proper water flow dynamics.

### Biome Generation Issues  
**Root Cause:** 
1. Water not flowing to create realistic water distributions
2. Drainage thresholds inappropriate for high-resolution grids
3. Pressure limits preventing continental-scale weather patterns

**Fix Impact:** Scale-aware fixes will restore realistic water distribution, river formation, and atmospheric pressure gradients needed for accurate biome classification.

### Simulation Performance
**Root Cause:** CFL timestep bounds force unnecessarily small timesteps for large grid spacing.

**Fix Impact:** Appropriate timestep bounds will improve performance while maintaining numerical stability.

---

This analysis provides a complete roadmap for resolving the scale-dependency issues in Jerry's simulation. The critical water flow threshold fixes should be implemented immediately as they are the primary cause of the reported simulation problems.