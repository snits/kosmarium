# Water System Bug Fix - Large Maps

## Problem
Jerry reported seeing 0.0 water levels on 1024x512 maps despite MassConserving rainfall scaling that should maintain appropriate water levels.

## Root Cause
The evaporation system had a hard-coded threshold of 0.001 that cleared any water below that amount. For large maps using MassConserving scaling:

- 1024x512 map effective rainfall: ~0.00011 per cell per tick
- After evaporation (multiply by 0.999): ~0.0001097 per cell
- **This was BELOW the 0.001 threshold**
- Result: All water cleared to 0.0 every single tick

## Solution
Implemented scale-aware evaporation threshold:

1. **Added `evaporation_threshold` field** to `WaterFlowSystem`
2. **Created `calculate_evaporation_threshold()`** that sets threshold to 1% of post-evaporation rainfall
3. **Updated evaporation logic** to use dynamic threshold instead of hard-coded 0.001
4. **Added safety bounds** (1e-8 to 1e-4) to prevent floating-point issues

## Technical Details

### Before Fix
```rust
if *depth < 0.001 {  // Hard-coded threshold
    *depth = 0.0;
}
```

### After Fix  
```rust
if *depth < self.evaporation_threshold {  // Scale-aware threshold
    *depth = 0.0;
}
```

### Threshold Calculation
```rust
let post_evaporation_rainfall = effective_rainfall_rate * (1.0 - evaporation_rate);
let threshold = (post_evaporation_rainfall * 0.01).max(1e-8).min(1e-4);
```

## Results

### Map Size Comparison
| Map Size | Effective Rainfall | Old Threshold | New Threshold | Water Accumulates? |
|----------|-------------------|---------------|---------------|-------------------|
| 240x120  | 0.002000         | 0.001         | ~0.00002      | ✅ Before & After |
| 480x240  | 0.000500         | 0.001         | ~0.000005     | ❌ Before, ✅ After |
| 1024x512 | 0.000110         | 0.001         | ~0.000001     | ❌ Before, ✅ After |

### Validation
- ✅ All 36 existing tests pass
- ✅ New test `large_map_water_accumulation_works()` validates 1024x512 behavior
- ✅ New test `scale_aware_evaporation_threshold()` validates threshold scaling
- ✅ Simulation now shows water symbols on large maps instead of 0.0 levels

## Files Modified
- `src/sim.rs` - Added scale-aware threshold system
- Added unit tests for large map validation

Jerry should now see appropriate water levels on 1024x512 maps with MassConserving scaling working as intended.