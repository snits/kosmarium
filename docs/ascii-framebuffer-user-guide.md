# ASCII Framebuffer User Guide

## ABOUTME: User documentation for the --ascii-frames command-line interface
## ABOUTME: Comprehensive guide for researchers monitoring continental-scale atmospheric physics simulations

## Overview

The ASCII Framebuffer interface provides real-time visualization of continental-scale atmospheric physics simulations through terminal-based multi-layer monitoring. This system eliminates graphics rendering overhead while delivering scientifically accurate visualization of complex atmospheric dynamics, water flow, and climate systems.

**Key Capabilities:**
- Real-time monitoring of 8 different physical data layers
- Continental-scale resolution (240x120 @ 200km scale typical)
- Physics-validated atmospheric pressure patterns [91.2, 102.0] kPa
- Temporal buffering for change detection and analysis
- Scale-aware diagnostic tools with quantitative metrics

**Target Applications:**
- Continental atmospheric physics research
- Climate system validation and debugging
- Educational demonstration of weather pattern formation
- Long-duration simulation monitoring
- Physics violation detection and troubleshooting

---

## Quick Start Guide

### Basic Usage

Start monitoring a continental-scale simulation with default scientific layers:

```bash
cargo run --bin sim-protoype -- --ascii-frames
```

This launches with:
- **Default layers**: elevation, water, biomes
- **Continental scale**: 200km physical domain
- **Buffer size**: 5 frames for temporal analysis
- **Update interval**: Every 10 simulation ticks

### Common Scientific Workflows

**Climate Analysis** (Temperature-biome relationships):
```bash
cargo run --bin sim-protoype -- --ascii-frames --preset climate-analysis
```

**Storm Tracking** (Atmospheric pressure and circulation):
```bash
cargo run --bin sim-protoype -- --ascii-frames --preset storm-tracking
```

**Comprehensive Monitoring** (All atmospheric layers):
```bash
cargo run --bin sim-protoype -- --ascii-frames --layers elevation,water,temperature,pressure,wind,flow,biomes,sediment
```

**High-Resolution Regional Analysis**:
```bash
cargo run --bin sim-protoype -- --ascii-frames --zoom local --frame-width 80 --frame-height 40
```

---

## Data Layer Reference

The ASCII framebuffer displays 8 different physical data layers, each representing distinct aspects of the continental-scale simulation:

### 1. Elevation Layer
**Scientific Significance**: Terrain topography drives atmospheric flow patterns and precipitation
**ASCII Symbols**:
- `~` Deep water/ocean (elevation < -0.5)
- `.` Shallow water/sea level (elevation < 0.0)
- `,` Beach/coastal zones (elevation 0.0-0.2)
- `^` Low hills/plains (elevation 0.2-0.4)
- `#` Hills/uplands (elevation 0.4-0.6)
- `@` Mountains (elevation 0.6-0.8)
- `%` High peaks (elevation > 0.8)

**Interpretation**: Look for mountain ranges that create rain shadows, coastal plains where temperature gradients drive circulation, and valley systems that channel atmospheric flow.

### 2. Water Layer
**Scientific Significance**: Surface water distribution affects evaporation, latent heat cooling, and local humidity
**ASCII Symbols**:
- `.` Dry terrain (depth < threshold)
- `:` Trace water/soil moisture (depth < 5× threshold)
- `~` Shallow surface water (depth < 20× threshold)
- `#` Deep water bodies/lakes (depth < 50× threshold)
- `@` Very deep water/major lakes (depth > 50× threshold)

**Interpretation**: Track evaporation hotspots, seasonal water accumulation, and drainage patterns that influence local climate through latent heat effects.

### 3. Temperature Layer
**Scientific Significance**: Temperature drives atmospheric pressure gradients and circulation patterns
**ASCII Symbols**:
- `■` Very cold (< -10°C) - Arctic conditions
- `▓` Cold (< 0°C) - Winter/high altitude
- `▒` Cool (< 10°C) - Temperate conditions
- `░` Mild (< 20°C) - Pleasant temperatures
- `.` Warm (< 30°C) - Summer conditions
- `+` Hot (< 40°C) - Desert/tropical
- `#` Very hot (> 40°C) - Extreme heat

**Interpretation**: Temperature gradients drive pressure differences. Warm areas (`.` `+`) create low pressure, cool areas (`▒` `▓`) create high pressure. Look for thermal circulation patterns.

### 4. Pressure Layer
**Scientific Significance**: Atmospheric pressure gradients drive wind patterns and storm formation
**ASCII Symbols**:
- `-` Low pressure systems (< 20th percentile) - Storm centers
- `.` Below average pressure (20-40th percentile)
- `0` Average pressure (40-60th percentile) - Neutral zones
- `+` Above average pressure (60-80th percentile)
- `#` High pressure systems (> 80th percentile) - Clear weather

**Physics Context**: Realistic pressure range [91.2, 102.0] kPa achieved through proper thermal circulation physics (not random generation). Pressure patterns should correlate with temperature - warm areas show `-` `.`, cool areas show `+` `#`.

### 5. Wind Layer
**Scientific Significance**: Wind vectors reveal atmospheric circulation and pressure-driven flow
**ASCII Symbols**:
- `.` Calm conditions (speed < 1.0 m/s)
- `→` East wind
- `↗` Northeast wind
- `↑` North wind
- `↖` Northwest wind
- `←` West wind
- `↙` Southwest wind
- `↓` South wind
- `↘` Southeast wind

**Interpretation**: Wind flows from high pressure (`#` `+`) to low pressure (`-` `.`). Look for circulation patterns around pressure systems and topographic wind channeling through valleys.

### 6. Flow Layer (Water Velocity)
**Scientific Significance**: Surface water flow reveals drainage patterns and erosion potential
**ASCII Symbols**: Same directional arrows as wind layer
- `.` No flow (velocity < 0.001 m/s)
- Directional arrows show water movement direction

**Interpretation**: Flow patterns should follow elevation gradients, showing drainage from high to low elevations. Strong flows indicate erosion potential and sediment transport.

### 7. Biomes Layer
**Scientific Significance**: Biome distribution reflects climate-terrain interactions and ecosystem patterns
**ASCII Symbols**:
- `~` Ocean - Marine environments
- `=` Lake - Freshwater bodies
- `-` River - Flowing water systems
- `*` Wetland - High moisture areas
- `G` Grassland - Temperate grasslands
- `S` Savanna - Tropical grasslands
- `s` Shrubland - Arid shrub communities
- `F` Temperate Forest - Deciduous forests
- `T` Tundra - Cold climate vegetation
- `D` Desert - Arid landscapes
- `R` Rain Forest - Tropical forests
- `B` Boreal Forest - Coniferous forests
- `A` Alpine - High elevation vegetation
- `I` Ice - Permanent ice/snow

**Interpretation**: Biome patterns should logically follow temperature and moisture gradients. Misplaced biomes may indicate climate simulation issues.

### 8. Sediment Layer
**Scientific Significance**: Sediment transport reveals erosion patterns and deposition zones
**ASCII Symbols**:
- `.` No sediment transport
- `:` Light sediment load
- `+` Medium sediment transport
- `#` Heavy sediment movement
- `@` Very heavy sediment load

**Interpretation**: High sediment transport occurs in areas with strong water flow and steep terrain. Deposition zones appear downstream of erosion areas.

---

## Command Line Interface

### Core Options

**`--ascii-frames`**
Enables the ASCII framebuffer interface instead of the default TUI or graphics modes.

**`--layers <LAYERS>`**
Specify which data layers to display (comma-separated):
```bash
--layers elevation,water,temperature,pressure
--layers wind,flow,sediment
--layers temperature,pressure,wind  # Storm tracking
```
Available layers: `elevation`, `water`, `biomes`, `temperature`, `pressure`, `wind`, `flow`, `sediment`

**`--zoom <LEVEL>`**
Set visualization zoom level:
- `continental` - Broad overview (2× base resolution)
- `regional` - Regional detail (3× base resolution)
- `local` - High detail (4× base resolution)

**`--buffer-size <SIZE>`**
Frame buffer size for temporal analysis (default: 5)
```bash
--buffer-size 10  # More frames for change detection
--buffer-size 3   # Minimal buffering for performance
```

### Scientific Workflow Presets

**`--preset climate-analysis`**
- Layers: temperature, biomes, elevation
- Zoom: continental
- Purpose: Temperature-biome relationship analysis

**`--preset storm-tracking`**
- Layers: pressure, wind, temperature
- Zoom: regional
- Purpose: Atmospheric circulation monitoring

**`--preset change-detection`**
- Layers: pressure, temperature, water, changes
- Zoom: continental
- Buffer: 10 frames
- Purpose: Temporal evolution analysis

**`--preset regional-deep-dive`**
- Layers: elevation, water, temperature, pressure, biomes, wind
- Zoom: local
- Purpose: Comprehensive local analysis

### Frame Display Configuration

**`--frame-width <WIDTH>` / `--frame-height <HEIGHT>`**
Override automatic frame sizing:
```bash
--frame-width 80 --frame-height 40  # Explicit dimensions
--frame-width 0 --frame-height 0    # Auto-size (default)
```

**`--interval <TICKS>`**
Update interval in simulation ticks (default: 10):
```bash
--interval 1   # Every tick (high frequency)
--interval 50  # Every 50 ticks (low frequency)
```

### Workspace Management

**`--load-config <FILE>`**
Load complete workspace configuration from YAML:
```bash
--load-config my-climate-study.yaml
```

**`--save-config <FILE>`**
Save current configuration for reproducible analysis:
```bash
--save-config storm-analysis-workspace.yaml --author "Dr. Smith"
```

### Simulation Parameters

**Physical Scale**:
```bash
--scale-km 200.0    # Continental domain (default)
--scale-km 50.0     # Regional domain (disables Coriolis)
--scale-km 1000.0   # Large continental domain
```

**Domain Resolution**:
```bash
--width 240 --height 120  # Default resolution
--width 480 --height 240  # Higher resolution (slower)
```

**Terrain Generation**:
```bash
--seed 12345          # Reproducible terrain
--roughness 0.7       # Terrain complexity (0.0-1.0)
--persistence 0.6     # Detail persistence (0.0-1.0)
```

---

## Scientific Interpretation Guide

### Understanding Atmospheric Physics Patterns

**Healthy Atmospheric Circulation Indicators:**
- **Temperature-Pressure Correlation**: Warm areas (`.` `+` in temperature) should show low pressure (`-` `.` in pressure)
- **Wind-Pressure Relationship**: Wind arrows should point from high pressure (`#` `+`) toward low pressure (`-` `.`)
- **Realistic Pressure Range**: [91.2, 102.0] kPa indicates proper thermal circulation physics
- **Coherent Circulation**: Pressure systems should drive organized wind patterns, not random vectors

**Physics Violation Warning Signs:**
- **Random Pressure Patterns**: Scattered `#` `-` symbols with no temperature correlation
- **Extreme Pressure Values**: Values outside [85, 115] kPa range may indicate numerical issues
- **Disconnected Wind Fields**: Wind arrows showing no relationship to pressure gradients
- **Impossible Temperature Gradients**: Sudden temperature jumps without elevation changes

### Continental-Scale Circulation Patterns

**Thermal Circulation (Expected Pattern)**:
1. **Mountain Heating**: During day, mountain slopes (`@` `%`) warm faster than valleys (`.` `,`)
2. **Pressure Response**: Warm areas develop low pressure (`-` `.`), cool areas high pressure (`+` `#`)
3. **Wind Generation**: Air flows from high to low pressure, creating upslope winds (`↑` `↗` `↖`)
4. **Return Circulation**: Compensating flow at altitude creates circulation cells

**Topographic Effects**:
- **Orographic Lifting**: Wind hitting mountains (`→` hitting `@`) creates upslope flow (`↗` `↑`)
- **Valley Channeling**: Winds follow valley orientation, creating strong directional flow
- **Rain Shadow**: Downslope winds (`↓` `↘`) create dry conditions on leeward sides

### Water Cycle Monitoring

**Evaporation-Temperature Coupling**:
- High water areas (`.` `~` `#` in water layer) with warm temperatures (`.` `+`) should show evaporation
- **Latent Heat Cooling**: Evaporation should reduce local temperature (proper physics implementation)
- **Humidity Effects**: Evaporation zones affect local atmospheric moisture and pressure

**Drainage Pattern Analysis**:
- Flow arrows (`→` `↓` etc. in flow layer) should follow elevation gradients
- **Convergent Flow**: Multiple flow arrows converging indicates stream formation
- **Sediment Transport**: High flow areas should show sediment movement (`+` `#` `@`)

### Biome-Climate Validation

**Climate-Biome Consistency**:
- **Temperature Zones**: Arctic (`■` `▓`) → Tundra (`T`), Temperate (`▒` `░`) → Forest (`F`), Hot (`+` `#`) → Desert (`D`)
- **Moisture Effects**: High water + warm temperature should support forests (`F` `R`), not deserts (`D`)
- **Elevation Effects**: Mountains (`@` `%`) should show alpine biomes (`A`) or ice (`I`)

**Problematic Patterns**:
- Deserts (`D`) in high-moisture, cool areas
- Tropical forests (`R`) in cold, dry regions
- Ice biomes (`I`) at low elevations with warm temperatures

### Long-Duration Monitoring

**System Stability Indicators**:
- **Mass Conservation**: Water distribution should remain stable over time
- **Energy Balance**: Temperature patterns should reach quasi-equilibrium
- **Circulation Persistence**: Pressure systems should show coherent, persistent patterns

**Temporal Analysis Using Buffer**:
- **Change Detection**: Compare current frame with previous frames in buffer
- **Trend Analysis**: Monitor system evolution over multiple buffer cycles
- **Instability Detection**: Rapid oscillations or growing perturbations indicate numerical issues

---

## Troubleshooting Guide

### Common Issues and Solutions

**Problem**: Random, scattered pressure patterns with no correlation to temperature
**Cause**: Physics violation - random pressure generation instead of thermal circulation
**Solution**: This indicates fundamental atmospheric physics issues requiring code fixes

**Problem**: Wind arrows showing no relationship to pressure gradients
**Cause**: Wind-pressure coupling disabled or incorrect pressure calculation
**Solution**: Verify pressure-driven wind generation is properly implemented

**Problem**: Extreme pressure values outside [85, 115] kPa range
**Cause**: Numerical instability or incorrect physical parameters
**Solution**: Check domain scale, timestep limits, and boundary conditions

**Problem**: Impossible biome distributions (deserts in cold, wet areas)
**Cause**: Climate-biome coupling issues or incorrect moisture transport
**Solution**: Verify temperature and moisture fields are physically reasonable

**Problem**: Interface freezing or very slow updates
**Cause**: Large domain size or high update frequency
**Solutions**:
- Reduce resolution: `--width 120 --height 60`
- Increase interval: `--interval 50`
- Use continental zoom: `--zoom continental`

**Problem**: ASCII symbols appear garbled or incorrect
**Cause**: Terminal encoding issues or font compatibility
**Solution**: Ensure UTF-8 terminal encoding and Unicode arrow character support

### Performance Optimization

**For Large Domains**:
```bash
# Reduce resolution
--width 160 --height 80 --zoom continental

# Lower update frequency
--interval 20

# Fewer layers
--layers elevation,pressure,temperature
```

**For High-Frequency Monitoring**:
```bash
# Single critical layer
--layers pressure

# Minimal buffering
--buffer-size 3

# Small frame size
--frame-width 40 --frame-height 20
```

### Validation Workflows

**Physics Validation Checklist**:
1. **Temperature-Pressure Correlation**: Warm areas should correlate with low pressure
2. **Wind-Pressure Gradient**: Wind should flow from high to low pressure
3. **Pressure Range Realism**: Values should stay within [90, 105] kPa for continental domains
4. **Mass Conservation**: Total water should remain constant over time
5. **Biome-Climate Consistency**: Biomes should match local temperature and moisture conditions

**Data Quality Assessment**:
```bash
# Run with stats for quantitative validation
cargo run --bin sim-protoype -- --stats --interval 10

# Monitor specific layers for physics validation
cargo run --bin sim-protoype -- --ascii-frames --layers temperature,pressure,wind --interval 5
```

---

## Advanced Usage

### Custom Workspace Configuration

Create a YAML workspace file for reproducible analysis:

```yaml
metadata:
  name: "Storm System Analysis"
  author: "Dr. Smith"
  team: "Atmospheric Physics Lab"
  description: "High-resolution storm formation monitoring"
  version: "1.0"

defaults:
  seed: 42
  scale_km: 200.0
  roughness: 0.7
  persistence: 0.6
  dimensions: [240, 120]
  interval: 5

layout:
  buffer_size: 10
  layers: ["pressure", "wind", "temperature"]
  zoom: "regional"
  frame_size: [80, 40]
  show_timestamps: true
  highlight_changes: false
  subsample_rate: 1
```

Load workspace:
```bash
cargo run --bin sim-protoype -- --ascii-frames --load-config storm-analysis.yaml
```

### Research Team Collaboration

**Standardized Configurations**:
```bash
# Team lead creates workspace template
cargo run --bin sim-protoype -- --ascii-frames --preset climate-analysis --save-config team-climate-template.yaml --author "Team Lead"

# Team members load shared configuration
cargo run --bin sim-protoype -- --ascii-frames --load-config team-climate-template.yaml
```

**Reproducible Analysis**:
```bash
# Fixed seed ensures identical terrain across team
cargo run --bin sim-protoype -- --ascii-frames --seed 12345 --load-config shared-workspace.yaml
```

### Long-Duration Studies

**Continuous Monitoring Setup**:
```bash
# Low-frequency updates for long simulations
cargo run --bin sim-protoype -- --ascii-frames --interval 100 --buffer-size 20 --layers pressure,temperature

# Log output for offline analysis
cargo run --bin sim-protoype -- --ascii-frames --interval 50 > simulation-log.txt 2>&1
```

**Change Detection Analysis**:
```bash
# Large buffer for temporal analysis
cargo run --bin sim-protoype -- --ascii-frames --preset change-detection --buffer-size 50 --interval 25
```

---

## Educational Applications

### Classroom Demonstrations

**Basic Atmospheric Circulation**:
```bash
# Simple temperature-pressure relationship
cargo run --bin sim-protoype -- --ascii-frames --layers temperature,pressure --zoom regional --interval 5
```

**Weather Pattern Formation**:
```bash
# Complete atmospheric system
cargo run --bin sim-protoype -- --ascii-frames --preset storm-tracking --interval 10
```

**Water Cycle Demonstration**:
```bash
# Water-temperature coupling
cargo run --bin sim-protoype -- --ascii-frames --layers water,temperature,flow --zoom local
```

### Learning Objectives

Students can observe:
- How temperature gradients create pressure differences
- How pressure differences drive wind patterns
- How topography influences atmospheric circulation
- How water evaporation affects local temperature
- How different scales affect circulation patterns

### Guided Exploration Questions

1. **Temperature-Pressure Relationship**: "Where do you see warm areas? What happens to pressure in those locations?"
2. **Wind Generation**: "Look at the pressure map - where do the wind arrows point? Why?"
3. **Topographic Effects**: "How do mountains affect wind patterns? What happens on different sides?"
4. **Scale Effects**: "Compare continental vs local zoom - what patterns change?"
5. **System Coupling**: "How does evaporation from lakes affect local temperature?"

---

## Technical Implementation Notes

### System Requirements

**Terminal Compatibility**:
- UTF-8 encoding support required for directional arrows
- Minimum 80×40 character display for reasonable visibility
- Color terminal support recommended (not required)

**Performance Characteristics**:
- Continental scale (240×120 @ 200km): ~10-50ms per frame
- Regional scale (480×240 @ 100km): ~50-200ms per frame
- Update frequency: 1-100 simulation ticks per display update

### Data Sampling and Accuracy

**Spatial Sampling**:
The ASCII framebuffer samples the full simulation grid to the display resolution. For a 240×120 simulation displayed in 80×40 ASCII, each character represents a 3×3 grid cell average.

**Temporal Accuracy**:
Display updates reflect the exact simulation state at the specified interval. No temporal interpolation is performed between frames.

**Numerical Precision**:
All physical quantities maintain full floating-point precision internally. ASCII symbol mapping uses physically meaningful thresholds based on the variable range and scientific significance.

### Integration with Core Physics

The ASCII framebuffer directly accesses validated physics modules:
- **Temperature Layer**: Full thermal diffusion with realistic heat transfer
- **Pressure Layer**: Thermodynamic pressure calculation with proper equation of state
- **Wind Layer**: Geostrophic balance with Coriolis effects for domains > 100km
- **Water System**: Mass-conserving flow with scale-aware erosion physics
- **Biome Classification**: Climate-driven ecosystem modeling with moisture-temperature coupling

This ensures the visualization represents genuine physical processes, not simplified approximations.

---

## Conclusion

The ASCII Framebuffer interface provides a powerful, scientifically accurate tool for monitoring continental-scale atmospheric physics simulations. By combining real-time visualization with quantitative validation capabilities, it enables researchers to understand complex atmospheric dynamics, validate physics implementations, and conduct reproducible climate research.

The interface bridges sophisticated computational physics with accessible terminal-based visualization, making it suitable for both advanced research applications and educational demonstrations of atmospheric science principles.

For additional technical details, see the complementary documentation:
- `docs/session-handoff.md` - Current implementation status
- `docs/physics-correct-atmospheric-implementation-specification.md` - Physics implementation details
- `docs/collaborative-scientific-physics-specification.md` - Scientific validation framework