# Scale Behavior Analysis: Continental-Lock Problem

## Summary
Analysis of claimed "scale-invariant" terrain generation reveals the system is fundamentally locked at continental scale, unable to properly zoom from local patches to planetary views.

## Code Evidence Analysis

### Scale Infrastructure (scale.rs)
- `WorldScale` struct provides coordinate mapping (`meters_per_pixel()`, `physical_size_km`)
- **Problem**: Core terrain algorithms don't use these scale metrics
- Diamond-Square generates identical fractal patterns regardless of claimed physical scale

### TectonicConfig Scale Adaptation (worldgen.rs:288-327)
**Attempted Scaling:**
- Plate count: `(area * plates_per_million_km²)` with 4-20 plate limits
- Coastal blending: Converts 100km real-world distance to pixels
- Roughness: Minor parameter adjustments based on resolution

**Fundamental Flaws:**
- 1km local patch still gets minimum 4 tectonic plates (physically impossible)
- Assumes continental-scale tectonic features at all scales
- No mechanism to switch from tectonic to local geology models

### Missing Scale Validation
**No Evidence Found:**
- Testing across different physical scales (100m to 100km claims)
- Validation of geological appropriateness at different scales
- Tests of scale-invariant behavior
- Only basic finite-value tests exist

## Physical Scale Requirements

**Local Scale (100m-1km):**
- Surface geology patterns
- Outcrop formations  
- Local topographic features
- **Current System**: Applies tectonic plates (wrong model)

**Regional Scale (1km-100km):**
- Watershed systems
- Local mountain ranges
- Valley networks
- **Current System**: Might work with proper parameter tuning

**Continental Scale (100km+):**
- Tectonic plate boundaries
- Major mountain ranges
- Ocean basin formation
- **Current System**: Designed for this scale

## Conclusion
The "continental-lock" problem is real. The system:
1. Has scaling infrastructure that algorithms don't use
2. Applies continental-scale physical models regardless of claimed scale  
3. Cannot zoom from local geology to planetary tectonics
4. Lacks scale-appropriate physical modeling

The 100m-100km scale-invariant claims are unsupported by the actual code behavior.