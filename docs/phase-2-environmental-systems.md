# Phase 2: Environmental Foundation Systems

ABOUTME: Next major development phase focusing on temperature/climate systems and large-scale flow effects
ABOUTME: Builds on the completed water flow physics foundation to add environmental complexity

## Overview

After completing the water flow physics foundation with scale-aware parameters, CFL stability, and physics-based scaling laws, the next natural progression is to add environmental systems that interact with and enhance the water simulation.

## Phase 2A: Temperature/Climate Layer (Priority 1)

### Core Implementation
```rust
// New module: src/climate.rs
pub struct TemperatureLayer {
    pub temperature: Vec<Vec<f32>>,         // Celsius at each cell
    pub seasonal_variation: Vec<Vec<f32>>,  // Annual temperature swing
    pub elevation_gradient: f32,            // Temperature drops with altitude
}

pub struct ClimateParameters {
    pub base_temperature_c: f32,            // Sea level base temperature
    pub elevation_lapse_rate: f32,          // °C per meter elevation
    pub seasonal_amplitude: f32,            // Seasonal variation range
    pub latitude_gradient: f32,             // Temperature change per degree latitude
}

pub struct ClimateSystem {
    pub parameters: ClimateParameters,
    pub effective_temp_gradient: f32,       // Scale-aware temperature gradients
    pub current_season: f32,                // 0.0-1.0 annual cycle
}
```

### Integration with Water System
```rust
impl WaterFlowSystem {
    pub fn update_with_climate(&mut self, climate: &ClimateSystem, temp: &TemperatureLayer) {
        // Temperature affects evaporation rates
        // Higher temperature = faster evaporation
        // Lower temperature = slower evaporation, potential snow/ice
        
        // Temperature affects precipitation type
        // Below freezing = snow accumulation
        // Above freezing = liquid precipitation
    }
}
```

### Scale-Aware Architecture Integration
```rust
impl ScaleAware for ClimateParameters {
    fn derive_parameters(&self, scale: &WorldScale) -> Self {
        let meters_per_pixel = scale.meters_per_pixel() as f32;
        
        Self {
            // Base temperature remains constant (intensive property)
            base_temperature_c: self.base_temperature_c,
            
            // Lapse rate scales with pixel resolution
            elevation_lapse_rate: self.elevation_lapse_rate,
            
            // Seasonal amplitude scales with map coverage area
            seasonal_amplitude: self.seasonal_amplitude,
            
            // Latitude gradient scales with physical map extent
            latitude_gradient: self.latitude_gradient * scale.physical_size_km as f32,
        }
    }
}
```

### Implementation Approach

**Pre-computed Base Layer:**
- Generate base temperature from elevation + latitude during world generation
- Use heightmap to calculate elevation-based temperature gradients
- Apply latitude-based temperature zones

**Dynamic Seasonal Effects:**
- Real-time seasonal temperature cycling
- Weather pattern simulation (optional advanced feature)
- Day/night temperature variations

**Integration Points:**
- **Evaporation**: Temperature modifies `WaterFlowSystem.evaporation_rate`
- **Precipitation**: Temperature determines rain vs snow
- **Flow Dynamics**: Temperature affects water viscosity (advanced)
- **Visualization**: Temperature affects terrain color schemes

### Benefits
- More realistic water cycle behavior
- Foundation for weather patterns
- Enhanced visual feedback
- Educational value for climate system understanding

## Phase 2B: Large-Scale Flow Effects (Priority 2)

### Core Implementation
```rust
// Extension to existing water system
pub struct LargeScaleFlow {
    pub coriolis_parameter: f32,            // Varies with latitude
    pub pressure_gradients: Vec<Vec<Vec2>>, // Atmospheric pressure effects
    pub geostrophic_flow: Vec<Vec<Vec2>>,   // Large-scale circulation patterns
}

pub struct AtmosphericSystem {
    pub pressure_field: Vec<Vec<f32>>,      // Atmospheric pressure at each cell
    pub wind_patterns: Vec<Vec<Vec2>>,      // Wind velocity vectors
    pub coriolis_strength: f32,             // Planet rotation effect
}
```

### When to Implement
- **After temperature system** - requires temperature gradients to drive pressure differences
- **For large maps** - Coriolis effects only meaningful at scales >100km
- **Advanced feature** - requires understanding of rotating reference frames and geophysical fluid dynamics

### Mathematical Complexity
- Rotating reference frame mathematics
- Geostrophic balance equations
- Pressure gradient force calculations
- Integration with existing flow solver

## Implementation Timeline

### Phase 2A: Temperature/Climate (Weeks 1-2)
1. **Week 1**: Core temperature layer implementation
   - `src/climate.rs` module creation
   - Basic temperature field generation from elevation
   - Integration with existing `WorldScale` architecture
   - Scale-aware parameter derivation

2. **Week 2**: Water system integration
   - Temperature-dependent evaporation rates
   - Precipitation type determination (rain/snow)
   - Seasonal cycling system
   - Comprehensive testing and validation

### Phase 2B: Large-Scale Effects (Weeks 3-4, Optional)
1. **Week 3**: Pressure and wind systems
   - Atmospheric pressure field calculation
   - Wind pattern generation from pressure gradients
   - Basic circulation pattern simulation

2. **Week 4**: Coriolis integration
   - Coriolis force implementation
   - Large-scale flow pattern modification
   - Validation against known geophysical patterns

## Architecture Benefits

### Builds on Existing Foundation
- Uses same `WorldScale` context system
- Follows same `ScaleAware` trait pattern
- Integrates with existing parameter derivation
- Maintains modular, testable architecture

### Educational Value
- Demonstrates climate system interactions
- Shows how physical systems couple together
- Provides foundation for understanding weather patterns
- Illustrates scale-dependent physical phenomena

### Technical Progression
- Natural evolution from water physics to environmental physics
- Maintains focus on realistic physical relationships
- Provides basis for more complex simulations (weather, ecology)
- Demonstrates professional simulation architecture patterns

## Next Steps After Phase 2

With temperature and optional large-scale flow systems complete, natural next phases include:

- **Phase 3**: Biome/Vegetation Systems
- **Phase 4**: Agent-Based Systems (fauna, human settlements)
- **Phase 5**: Economic/Resource Systems
- **Phase 6**: Advanced Rendering and Visualization

## References

### Climate System Implementation
- Atmospheric physics textbooks for temperature gradient calculations
- Meteorology references for pressure-wind relationships
- Climate modeling papers for seasonal cycling approaches

### Large-Scale Flow Systems
- Geophysical fluid dynamics textbooks for Coriolis effects
- Oceanography references for large-scale circulation patterns
- CFD literature for pressure-driven flow implementation