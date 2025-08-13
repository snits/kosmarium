# Flow Engine Integration Specification

## Executive Summary

This document defines the integration points between the unified FlowEngine and existing systems, ensuring seamless operation while enabling the missing physics couplings identified in Phase 2.2. The integration leverages the Vec2 foundation from Phase 2.1 and WorldScale architecture to provide consistent cross-system data sharing.

## Core Integration Architecture

### FlowEngine as Physics Service
The FlowEngine serves as a centralized physics service accessed by multiple simulation systems:

```
┌─────────────────────────────────────────────────────────┐
│                    FlowEngine                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │  Gradient   │  │Conservation │  │  Spatial    │      │
│  │   Based     │  │   Based     │  │Partitioned  │      │
│  └─────────────┘  └─────────────┘  └─────────────┘      │
└─────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼────────┐ ┌────────▼────────┐ ┌────────▼────────┐
│ Climate System │ │ Geological Evol │ │ Real-time Sim   │
│ (Conservation) │ │ (Accelerated)   │ │ (Gradient)      │
└────────────────┘ └─────────────────┘ └─────────────────┘
```

## WorldScale Integration

### Current Scaling Inconsistencies
The 5 duplicate implementations handle metric conversion differently:

1. **sim.rs**: `estimate_grid_spacing_from_context()` heuristic (error-prone)
2. **corrected_water_flow.rs**: Direct `world_scale.meters_per_pixel()` access (correct)
3. **spatial_partitioning.rs**: Uses internal water system scaling (indirect)
4. **drainage.rs**: No explicit scaling (assumes pixel coordinates)
5. **geological_evolution.rs**: Inherits scaling from wrapped system

### FlowEngine Solution
```rust
pub struct FlowEngine {
    /// Authoritative world scale for all metric conversions
    world_scale: WorldScale,
    // ... other fields
}

impl FlowEngine {
    /// All distance calculations use consistent metric conversion
    fn get_grid_spacing_meters(&self) -> f32 {
        self.world_scale.meters_per_pixel() as f32
    }
    
    /// Diagonal distance calculation for 8-neighbor flow
    fn get_diagonal_distance_meters(&self) -> f32 {
        self.get_grid_spacing_meters() * 1.414213562
    }
}
```

### Integration Points
- **Climate System**: Passes WorldScale from climate grid configuration
- **Geological Evolution**: Uses WorldScale from terrain generation parameters  
- **Real-time Simulation**: Gets WorldScale from viewport scaling settings
- **Drainage Networks**: Inherits WorldScale for channel depth calculations

## Vec2 Velocity Integration

### Current Velocity Storage Inconsistencies
Different systems store velocities in incompatible formats:

1. **sim.rs**: Uses unified `Vec2` from `math.rs` (Phase 2.1 standard)
2. **corrected_water_flow.rs**: Custom Vec2 struct with duplicate methods
3. **spatial_partitioning.rs**: Raw `(f32, f32)` tuples in flat arrays
4. **drainage.rs**: No velocity storage (static analysis only)

### Unified VelocityField Implementation
```rust
pub struct VelocityField {
    /// Unified Vec2 storage for cross-system compatibility
    velocities: Vec<Vec2>,
    width: usize,
    height: usize,
}

impl VelocityField {
    /// Efficient access using array indexing
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Vec2 {
        self.velocities[y * self.width + x]
    }
    
    /// Direct memory layout compatible with GPU processing
    pub fn as_raw_slice(&self) -> &[f32] {
        unsafe { 
            std::slice::from_raw_parts(
                self.velocities.as_ptr() as *const f32,
                self.velocities.len() * 2
            )
        }
    }
}
```

### Cross-System Velocity Sharing
```rust
// Example: Climate system reading flow velocities for heat transport
let velocity = flow_engine.get_velocity_field().get(x, y);
climate_system.transport_heat(velocity, temperature_gradient);

// Example: Biome system reading flow for seed dispersal
let flow_magnitude = velocity.magnitude();
biome_system.disperse_seeds(flow_magnitude, species_mobility);
```

## Drainage Network Integration

### Current Drainage Coupling
Only some systems integrate with drainage networks:

1. **sim.rs**: Has `calculate_flow_directions_with_drainage()` method
2. **corrected_water_flow.rs**: Includes drainage channel depth in surface gradients
3. **spatial_partitioning.rs**: No drainage integration
4. **drainage.rs**: Provides static flow accumulation data
5. **geological_evolution.rs**: No direct drainage coupling

### Unified Drainage Enhancement
```rust
impl FlowEngine {
    /// Consistent drainage enhancement across all algorithms
    fn calculate_drainage_enhancement(
        &self,
        drainage_network: Option<&DrainageNetwork>,
        x: usize,
        y: usize,
    ) -> f32 {
        if let Some(drainage) = drainage_network {
            let accumulation = drainage.get_flow_accumulation(x, y);
            let stats = drainage.get_statistics();
            
            // Normalized accumulation ratio [0, 1]
            let accumulation_ratio = if stats.max_accumulation > 0.0 {
                accumulation / stats.max_accumulation
            } else {
                0.0
            };
            
            // Enhancement factor [1.0, 3.0] for realistic flow concentration
            1.0 + 2.0 * accumulation_ratio
        } else {
            1.0 // No enhancement without drainage data
        }
    }
    
    /// Channel depth calculation for surface gradient modification
    fn calculate_channel_depth(
        &self,
        drainage_network: Option<&DrainageNetwork>,
        x: usize,
        y: usize,
    ) -> f32 {
        if let Some(drainage) = drainage_network {
            let flow_accumulation = drainage.get_flow_accumulation(x, y);
            let pixel_area = (self.get_grid_spacing_meters() * self.get_grid_spacing_meters()) as f64;
            
            // Physically-based channel depth scaling: 1cm per pixel of accumulation
            let depth_scale = 0.01; 
            (flow_accumulation as f64 * depth_scale / pixel_area.sqrt()) as f32
        } else {
            0.0
        }
    }
}
```

### Dynamic Drainage-Flow Coupling
```rust
// Enable future bidirectional coupling: flow modifies drainage networks
pub trait DrainageNetworkEvolution {
    /// Update drainage network based on actual flow patterns
    fn evolve_from_flow_patterns(
        &mut self, 
        flow_velocities: &VelocityField,
        flow_accumulation_threshold: f32
    );
    
    /// Create new channels where persistent flow exceeds threshold
    fn carve_new_channels(&mut self, persistent_flow_map: &FlowAccumulationMap);
}
```

## Algorithm Selection Strategy

### Physics-Driven Algorithm Selection
Different simulation contexts require different flow physics:

```rust
impl FlowEngine {
    /// Factory method for climate system integration
    pub fn for_climate_coupling(world_scale: WorldScale) -> Self {
        // Climate systems need mass conservation for energy balance
        let algorithm = FlowAlgorithm::ConservationBased {
            h_min_threshold: 0.001,  // 1mm minimum depth
            cfl_factor: 0.5,         // Stable for climate timesteps
            include_advection: true, // Full physics for accuracy
        };
        
        let mut parameters = FlowParameters::default();
        parameters.evaporation_rate = 0.0; // Climate system handles evaporation
        
        Self::new(world_scale, algorithm, parameters)
    }
    
    /// Factory method for geological evolution
    pub fn for_geological_timescales(world_scale: WorldScale) -> Self {
        // Geological systems need speed over perfect accuracy
        let algorithm = FlowAlgorithm::GradientBased { 
            flow_rate: 0.2  // Slower flow for geological realism
        };
        
        let mut parameters = FlowParameters::default();
        parameters.erosion_rate = 0.05;     // Enhanced erosion
        parameters.flow_rate = 0.2;         // Consistent with algorithm
        
        Self::new(world_scale, algorithm, parameters)
    }
    
    /// Factory method for real-time interactive simulation
    pub fn for_interactive_simulation(world_scale: WorldScale) -> Self {
        // Interactive systems need responsive performance
        let base_algorithm = Box::new(FlowAlgorithm::GradientBased { flow_rate: 1.0 });
        let algorithm = FlowAlgorithm::SpatialPartitioned {
            change_threshold: 0.001,
            base_algorithm,
        };
        
        let parameters = FlowParameters::default();
        let mut engine = Self::new(world_scale, algorithm, parameters);
        engine.optimization.use_spatial_partitioning = true;
        engine.optimization.enable_parallel = true;
        
        engine
    }
}
```

## Missing Physics Coupling Enablement

### Biome-Hydrology Coupling
The unified FlowEngine enables seamless biome-hydrology integration:

```rust
// Previously impossible due to incompatible velocity formats
pub fn couple_vegetation_flow_resistance(
    flow_engine: &mut FlowEngine,
    biome_system: &BiomeSystem,
    vegetation_density: &HeightMap,
) {
    // Modify flow parameters based on vegetation density
    let base_parameters = flow_engine.parameters.clone();
    
    for y in 0..vegetation_density.height() {
        for x in 0..vegetation_density.width() {
            let density = vegetation_density.get(x, y);
            let resistance_factor = 1.0 + 2.0 * density; // Denser vegetation = more resistance
            
            // This coupling was impossible with 5 separate implementations
            flow_engine.set_local_resistance(x, y, resistance_factor);
        }
    }
}
```

### Maritime Climate Coupling  
Ocean-land water exchange through unified interface:

```rust
pub fn couple_maritime_climate(
    flow_engine: &FlowEngine,
    climate_system: &mut ClimateSystem,
    coastal_cells: &[(usize, usize)],
) {
    for &(x, y) in coastal_cells {
        // Read land-to-sea flow from FlowEngine
        let land_velocity = flow_engine.get_velocity_field().get(x, y);
        let outflow_rate = land_velocity.magnitude();
        
        // Couple to climate system's ocean model
        climate_system.add_freshwater_flux(x, y, outflow_rate);
        
        // Bidirectional: ocean evaporation affects land precipitation
        let ocean_evaporation = climate_system.get_ocean_evaporation(x, y);
        // This creates the missing maritime climate feedback loop
    }
}
```

### Atmospheric-Surface Flow Coupling
Direct precipitation-runoff integration:

```rust
pub fn couple_atmospheric_surface_flow(
    flow_engine: &mut FlowEngine,
    atmosphere: &AtmosphericSystem,
    surface_water: &mut WaterLayer,
) {
    // Atmospheric system provides spatially-variable precipitation
    let precipitation_field = atmosphere.get_precipitation_field();
    
    // FlowEngine handles surface runoff with proper physics
    for y in 0..precipitation_field.height() {
        for x in 0..precipitation_field.width() {
            let precipitation = precipitation_field.get(x, y);
            surface_water.depth.add(x, y, precipitation * flow_engine.parameters.rainfall_rate);
        }
    }
    
    // Flow engine calculates runoff with conservation physics
    let flow_result = flow_engine.calculate_flow(
        &heightmap, surface_water, Some(&drainage_network), dt
    );
    
    // Feed surface evaporation back to atmosphere
    let surface_evaporation = flow_engine.calculate_surface_evaporation(surface_water);
    atmosphere.add_moisture_source(surface_evaporation);
}
```

## System Integration Examples

### Climate System Integration
```rust
pub struct ClimateSystem {
    flow_engine: FlowEngine,
    // ... other climate fields
}

impl ClimateSystem {
    pub fn new(world_scale: WorldScale) -> Self {
        Self {
            flow_engine: FlowEngine::for_climate_coupling(world_scale),
            // ... initialize other fields
        }
    }
    
    pub fn update_hydrology(&mut self, dt: f32) {
        // Use conservation physics for accurate energy balance
        let flow_result = self.flow_engine.calculate_flow(
            &self.terrain,
            &mut self.surface_water,
            Some(&self.drainage_network),
            dt
        );
        
        // Validate mass conservation for energy balance calculations
        if flow_result.mass_conservation_ratio < 0.99 || flow_result.mass_conservation_ratio > 1.01 {
            eprintln!("Warning: Mass conservation violated in climate coupling: {:.3}", 
                     flow_result.mass_conservation_ratio);
        }
        
        // Use velocity field for heat transport
        self.transport_heat_with_flow(&self.flow_engine.get_velocity_field());
    }
}
```

### Geological Evolution Integration
```rust
pub struct GeologicalEvolution {
    flow_engine: FlowEngine,
    // ... other geological fields
}

impl GeologicalEvolution {
    pub fn evolve_terrain(&mut self, iterations: u64) -> EvolutionResult {
        for _i in 0..iterations {
            // Accelerated flow for geological timescales
            let flow_result = self.flow_engine.calculate_flow(
                &mut self.heightmap,
                &mut self.water_layer,
                Some(&self.drainage_network),
                1.0 // Geological timestep
            );
            
            // Apply erosion based on flow result
            self.apply_geological_erosion(&flow_result);
            
            // Update drainage network based on new terrain
            self.drainage_network.evolve_from_terrain(&self.heightmap);
        }
        
        // This coupling enables realistic landscape evolution
        EvolutionResult::new(self.calculate_terrain_statistics())
    }
}
```

## Performance Integration Strategy

### Memory Layout Optimization
```rust
// Ensure cache-friendly memory access patterns
impl VelocityField {
    /// Structure-of-arrays layout for SIMD operations
    pub fn as_soa(&self) -> (Vec<f32>, Vec<f32>) {
        let mut x_components = Vec::with_capacity(self.velocities.len());
        let mut y_components = Vec::with_capacity(self.velocities.len());
        
        for velocity in &self.velocities {
            x_components.push(velocity.x);
            y_components.push(velocity.y);
        }
        
        (x_components, y_components)
    }
}
```

### Parallel Processing Integration
```rust
impl FlowEngine {
    /// Multi-threaded processing for large grids
    fn calculate_flow_parallel(
        &mut self,
        heightmap: &HeightMap,
        water: &mut WaterLayer,
        drainage_network: Option<&DrainageNetwork>,
        dt: f32,
    ) -> FlowResult {
        use rayon::prelude::*;
        
        let width = heightmap.width();
        let height = heightmap.height();
        let chunk_size = (width * height) / num_cpus::get();
        
        // Process chunks in parallel, avoiding data races
        let results: Vec<FlowResult> = (0..height)
            .into_par_iter()
            .chunks(chunk_size)
            .map(|chunk| self.process_chunk(chunk, heightmap, water, drainage_network, dt))
            .collect();
            
        // Combine results from parallel chunks
        self.combine_flow_results(results)
    }
}
```

## Testing Integration Requirements

### Cross-System Validation
```rust
#[test]
fn test_climate_flow_coupling() {
    let world_scale = WorldScale::new(1000.0, 1000.0, 100, 100);
    let mut climate_system = ClimateSystem::new(world_scale);
    let mut standalone_flow = FlowEngine::for_climate_coupling(world_scale);
    
    // Validate that climate-coupled flow produces identical results
    // to standalone conservation-based flow
    for _iteration in 0..100 {
        let climate_result = climate_system.update_hydrology(1.0);
        let standalone_result = standalone_flow.calculate_flow(/*...*/);
        
        assert_flow_results_equivalent(&climate_result, &standalone_result, 1e-6);
    }
}

#[test]
fn test_cross_system_velocity_sharing() {
    // Validate that velocity data can be shared between systems
    let flow_engine = FlowEngine::for_interactive_simulation(world_scale);
    let velocity_field = flow_engine.get_velocity_field();
    
    // Multiple systems should be able to read the same velocity data
    let biome_flow_effect = biome_system.calculate_seed_dispersal(velocity_field);
    let climate_heat_transport = climate_system.calculate_heat_flux(velocity_field);
    let erosion_sediment_transport = erosion_system.calculate_sediment_flux(velocity_field);
    
    // All systems use consistent velocity representation
    assert!(biome_flow_effect.is_finite());
    assert!(climate_heat_transport.is_finite());
    assert!(erosion_sediment_transport.is_finite());
}
```

## API Backward Compatibility

### Wrapper Implementations
```rust
// Maintain existing APIs during migration period
impl WaterSystem {
    /// Legacy method maintained for compatibility
    #[deprecated(note = "Use FlowEngine::calculate_flow instead")]
    pub fn calculate_flow_directions(&self, heightmap: &HeightMap, water: &mut WaterLayer) {
        // Delegate to FlowEngine with appropriate algorithm
        let mut flow_engine = FlowEngine::for_realtime_simulation(self.world_scale);
        flow_engine.calculate_flow(heightmap, water, None, 1.0);
    }
    
    /// Legacy method maintained for compatibility  
    #[deprecated(note = "Use FlowEngine::for_geological_evolution instead")]
    pub fn update_water_flow_with_climate(&mut self, /* ... */) {
        // Internal migration to FlowEngine while maintaining exact API
        if self.flow_engine.is_none() {
            self.flow_engine = Some(FlowEngine::for_geological_evolution(self.world_scale));
        }
        
        // Delegate to unified implementation
        self.flow_engine.as_mut().unwrap().calculate_flow(/* ... */);
    }
}
```

## Future Extension Points

### Advanced Physics Models
```rust
pub enum FlowAlgorithm {
    // Existing variants...
    
    /// Full 3D Navier-Stokes for complex terrain
    NavierStokes3D {
        viscosity: f32,
        turbulence_model: TurbulenceModel,
    },
    
    /// Multi-phase flow with sediment suspension
    MultiPhase {
        sediment_density: f32,
        suspension_threshold: f32,
    },
    
    /// Machine learning accelerated flow approximation
    MLAccelerated {
        model_path: String,
        fallback_algorithm: Box<FlowAlgorithm>,
    },
}
```

### GPU Acceleration Integration
```rust
impl FlowEngine {
    /// GPU-accelerated computation for massive grids
    #[cfg(feature = "gpu")]
    pub fn calculate_flow_gpu(
        &mut self,
        heightmap_gpu: &GPUHeightMap,
        water_gpu: &mut GPUWaterLayer,
        compute_context: &GPUContext,
    ) -> FlowResult {
        // CUDA/OpenCL implementation for 10000x10000+ grids
        compute_context.launch_flow_kernel(self.algorithm.as_gpu_params())
    }
}
```

## Conclusion

The FlowEngine integration specification provides a comprehensive foundation for unifying the 5 duplicate flow implementations while enabling the missing physics couplings essential for a complete desert island simulation. Key integration benefits include:

1. **Consistent Scaling**: WorldScale integration eliminates metric conversion inconsistencies
2. **Cross-System Data Sharing**: Unified Vec2 velocities enable seamless system coupling  
3. **Physics Accuracy**: Algorithm selection ensures appropriate physics for each context
4. **Performance Optimization**: Spatial partitioning and parallel processing for large-scale simulations
5. **Extensibility**: Clear extension points for advanced physics and GPU acceleration

The integration maintains backward compatibility during migration while providing a clean foundation for implementing the biome-hydrology, maritime climate, and atmospheric-surface couplings identified as missing in Phase 2.2.