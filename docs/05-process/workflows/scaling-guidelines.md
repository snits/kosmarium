# Map Size Scaling Guidelines

ABOUTME: Guidelines for using the simulation library across different map sizes
ABOUTME: Explains parameter scaling strategies and how to achieve consistent behavior

## The Map Scaling Problem

When using procedural world generation across different map sizes, you encounter a fundamental scaling challenge: parameters that work well at one resolution may produce unrealistic results at another.

For example:
- A 240x120 map with 0.002 rainfall per cell per tick produces reasonable water levels
- A 1024x512 map with the same parameters gets 18x more total rainfall input (524,288 vs 28,800 cells)
- This causes larger maps to have disproportionately higher water levels

## Water Flow System Scaling

The `WaterFlowSystem` provides configurable scaling through the `RainfallScaling` enum:

### Density Scaling (Default)

```rust
let mut system = WaterFlowSystem::new_for_map(width, height);
// Uses RainfallScaling::Density by default
```

**How it works:**
- Rainfall rate scales inversely with map area: `rainfall_rate * (reference_cells / current_cells)`
- Reference size is 240x120 (28,800 cells)
- Larger maps get proportionally less rainfall per cell
- Maintains consistent average water depth across map sizes

**Best for:**
- Library users who want consistent water behavior regardless of map size
- Scientific/engineering applications requiring scale independence
- Games where larger maps shouldn't be "wetter" than smaller ones

### Per-Cell Scaling

```rust
let mut system = WaterFlowSystem::new_for_map(width, height);
system.rainfall_scaling = RainfallScaling::PerCell;
```

**How it works:**
- Same rainfall amount per cell regardless of map size
- Larger maps accumulate proportionally more total water
- Simple and intuitive for manual parameter tuning

**Best for:**
- Situations where you want to manually control water levels
- When larger worlds should have more total water resources
- Prototyping and experimentation

## Usage Examples

### Consistent Water Levels Across Scales

```rust
// Both maps will have similar water behavior
let small_sim = Simulation::new(create_terrain(240, 120));
let large_sim = Simulation::new(create_terrain(1024, 512));

// Run simulations - water levels will be proportional to map size
for _ in 0..100 {
    small_sim.tick();
    large_sim.tick();
}
```

### Manual Parameter Control

```rust
let mut sim = Simulation::new(heightmap);
sim.water_system.rainfall_scaling = RainfallScaling::PerCell;
sim.water_system.rainfall_rate = 0.001; // Manually tuned for this specific map
```

## Parameter Reference

### Scale-Aware Parameters

| Parameter | Scaling Behavior | Notes |
|-----------|------------------|-------|
| `rainfall_rate` | Scales with map area (Density mode) | Base rate calibrated for 240x120 reference |
| `evaporation_rate` | Per-cell percentage (no scaling) | Same evaporation behavior at all scales |
| `flow_rate` | Fixed (no scaling) | Flow physics remain consistent |
| `erosion_strength` | Fixed (no scaling) | Erosion intensity stays constant |

### Default Values

```rust
// Default WaterFlowSystem parameters (calibrated for 240x120)
WaterFlowSystem {
    flow_rate: 0.1,              // 10% of water flows each tick
    evaporation_rate: 0.001,     // 0.1% evaporation per tick
    erosion_strength: 0.01,      // Low erosion strength
    deposition_rate: 0.05,       // 5% deposition rate
    rainfall_rate: 0.002,        // Base rainfall amount
    rainfall_scaling: RainfallScaling::Density, // Scale-aware by default
}
```

## Testing Scale Independence

To verify your parameters work across scales:

```rust
#[test]
fn test_scale_independence() {
    let sizes = [(60, 30), (240, 120), (480, 240), (960, 480)];
    let mut equilibrium_levels = Vec::new();
    
    for (w, h) in sizes {
        let mut sim = Simulation::new(create_flat_terrain(w, h));
        
        // Run to equilibrium
        for _ in 0..1000 { 
            sim.tick(); 
        }
        
        let avg_depth = sim.water.get_total_water() / (w * h) as f32;
        equilibrium_levels.push(avg_depth);
    }
    
    // With proper scaling, equilibrium levels should be similar
    let max_variation = equilibrium_levels.iter().fold(0.0, |acc, &x| acc.max(x)) 
                      / equilibrium_levels.iter().fold(f32::INFINITY, |acc, &x| acc.min(x));
    
    assert!(max_variation < 1.2, "Scale independence test failed: {}", max_variation);
}
```

## Future Enhancements

The scaling system is designed for extension:

### Physical Unit Support
```rust
// Planned enhancement
pub enum RainfallScaling {
    PerCell,
    Density,
    Physical { 
        cell_size_meters: f32,
        rainfall_mm_per_hour: f32 
    }
}
```

### Time Step Scaling
For very high-resolution maps, smaller time steps may be needed for numerical stability.

### Evaporation Scaling
Currently evaporation is percentage-based. Future versions may offer area-based evaporation scaling.

## Recommendations for Library Users

1. **Use Density scaling by default** - It provides the most consistent behavior
2. **Test across multiple scales** - Verify your results work at different resolutions
3. **Start with default parameters** - They're calibrated for reasonable behavior
4. **Document your scale choices** - Make it clear what map sizes your parameters target
5. **Consider physical meaning** - Think about what real-world phenomena you're modeling

## Common Pitfalls

- **Mixing scaling strategies** - Don't manually tune parameters for one scale then change scaling mode
- **Ignoring equilibrium time** - Larger maps may take longer to reach steady state
- **Over-tuning parameters** - Default values work well for most use cases
- **Scale-dependent validation** - Don't assume behavior that works at 256x256 will work at 2048x2048

The scaling system ensures your procedural worlds behave consistently regardless of output resolution, making your library robust for users working at different scales.