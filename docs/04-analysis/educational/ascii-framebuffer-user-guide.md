# ASCII Framebuffer User Guide

## ABOUTME: User documentation for the --ascii-frames command-line interface
## ABOUTME: Comprehensive guide for researchers monitoring continental-scale atmospheric physics simulations

## Overview

The ASCII Framebuffer interface provides real-time visualization of continental-scale atmospheric physics simulations through colorized terminal-based multi-layer monitoring. This system eliminates graphics rendering overhead while delivering scientifically accurate visualization of complex atmospheric dynamics, water flow, and climate systems through rich ANSI color coding.

**Key Capabilities:**
- **Colorized real-time monitoring** of 8 different physical data layers with semantic color schemes
- Continental-scale resolution (240x120 @ 200km scale typical)
- Physics-validated atmospheric pressure patterns [91.2, 102.0] kPa
- **Advanced wind visualization** combining speed intensity with directional hue modulation
- Temporal buffering for change detection and analysis
- Scale-aware diagnostic tools with quantitative metrics
- **ANSI color consistency** with graphics frontend for cross-platform pattern analysis

**Target Applications:**
- Continental atmospheric physics research with colorized pattern recognition
- Climate system validation and debugging through semantic color schemes
- Educational demonstration of weather pattern formation with intuitive color coding
- Long-duration simulation monitoring with rich visual feedback
- Physics violation detection and troubleshooting via color-coded anomaly identification
- **AI agent integration** - colorized ASCII provides rich visual data for text-based AI analysis

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
**ASCII Symbols & Colors**:
- `~` **Deep water/ocean** (elevation < -0.5) - *Plain ASCII*
- `.` **Shallow water/sea level** (elevation < 0.0) - *Plain ASCII*
- `,` **Beach/coastal zones** (elevation 0.0-0.2) - *Plain ASCII*
- `^` **Low hills/plains** (elevation 0.2-0.4) - *Plain ASCII*
- `#` **Hills/uplands** (elevation 0.4-0.6) - *Plain ASCII*
- `@` **Mountains** (elevation 0.6-0.8) - *Plain ASCII*
- `%` **High peaks** (elevation > 0.8) - *Plain ASCII*

**Colorization Scheme** (when using colorized output):
- **Blue**: Water and very low elevations (< 0.2) - Ocean and coastal areas
- **Cyan**: Coastline and transitional zones (0.2-0.4) - Beaches and low plains
- **Green**: Plains and grasslands (0.4-0.6) - Fertile lowlands
- **Yellow**: Hills and uplands (0.6-0.8) - Elevated terrain
- **Red**: Mountains and peaks (> 0.8) - High elevation zones

**Interpretation**: Look for mountain ranges that create rain shadows, coastal plains where temperature gradients drive circulation, and valley systems that channel atmospheric flow. **Color transitions** clearly delineate elevation zones affecting atmospheric dynamics.

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

**Colorization Scheme** (when using colorized output):
- **Bright Blue**: Very cold temperatures (< 20th percentile) - Arctic/alpine conditions
- **Blue**: Cold temperatures (20-40th percentile) - Winter/high elevation areas
- **White**: Moderate temperatures (40-60th percentile) - Temperate zones
- **Yellow**: Warm temperatures (60-80th percentile) - Summer/subtropical conditions
- **Red**: Hot temperatures (> 80th percentile) - Desert/tropical heat

**Interpretation**: Temperature gradients drive pressure differences. Warm areas (`.` `+`) in **yellow-red colors** create low pressure, cool areas (`▒` `▓`) in **blue tones** create high pressure. Look for thermal circulation patterns enhanced by **intuitive color temperature mapping**.

### 4. Pressure Layer
**Scientific Significance**: Atmospheric pressure gradients drive wind patterns and storm formation
**ASCII Symbols**:
- `-` Low pressure systems (< 20th percentile) - Storm centers
- `.` Below average pressure (20-40th percentile)
- `0` Average pressure (40-60th percentile) - Neutral zones
- `+` Above average pressure (60-80th percentile)
- `#` High pressure systems (> 80th percentile) - Clear weather

**Colorization Scheme** (when using colorized output):
- **Bright Blue**: Very low pressure (< 20th percentile) - Active storm centers
- **Blue**: Low pressure (20-40th percentile) - Developing weather systems
- **White**: Average pressure (40-60th percentile) - Neutral atmospheric conditions
- **Yellow**: High pressure (60-80th percentile) - Stable weather patterns
- **Red**: Very high pressure (> 80th percentile) - High-pressure anticyclones

**Physics Context**: Realistic pressure range [91.2, 102.0] kPa achieved through proper thermal circulation physics (not random generation). Pressure patterns should correlate with temperature - warm areas show `-` `.` in **blue tones**, cool areas show `+` `#` in **red-yellow tones**.

### 5. Wind Layer ⭐ **NEW: Advanced Colorized Wind Visualization**
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

**Advanced Colorization Scheme** (NEW FEATURE - Combined speed + direction):
**Base Speed Intensity Colors:**
- **Dark/Black**: Calm conditions (< 1.0 m/s) - No significant wind
- **Blue**: Light breeze - Gentle air movement
- **Green**: Moderate wind - Noticeable breeze
- **Yellow**: Strong wind - Significant air flow
- **Red**: Very strong wind - Powerful atmospheric circulation

**Directional Hue Modulation** (NEW):
Base speed colors are modified by wind direction for enhanced pattern recognition:
- **East winds** (→): Preserve base intensity color
- **Northeast winds** (↗): Shift toward brighter/yellow tones
- **North winds** (↑): Shift toward cooler blue-cyan spectrum
- **Northwest winds** (↖): Enhanced blue-cyan for arctic flow
- **West winds** (←): Shift toward magenta-purple for maritime flow
- **Southwest winds** (↙): Warm red-orange shifts for subtropical flow
- **South winds** (↓): Enhanced intensity (brighter versions of base colors)
- **Southeast winds** (↘): Yellow-orange shifts for tropical influences

**Scientific Interpretation**: Wind flows from high pressure (`#` `+`) to low pressure (`-` `.`). **Color intensity indicates wind strength**, while **hue variations help distinguish flow patterns** and identify circulation systems. Look for circulation patterns around pressure systems and topographic wind channeling through valleys.

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

---

## Colorized Output Features ⭐ **NEW**

### ANSI Color Terminal Support

The ASCII framebuffer now provides **full colorized output** with ANSI color codes, offering the same rich visual information available in the graphics frontend through terminal interfaces. This enables:

- **Enhanced pattern recognition** for atmospheric phenomena
- **Intuitive data interpretation** through semantic color schemes
- **AI agent integration** - colorized ASCII provides rich visual data for text-based analysis
- **Cross-platform consistency** - identical color schemes across graphics and ASCII modes

### Color Scheme Design Principles

**Scientific Accuracy**: All color mappings follow established meteorological conventions:
- **Blue tones** → Cold, low pressure, water, calm conditions
- **Red tones** → Hot, high pressure, land, intense activity
- **Green tones** → Moderate conditions, vegetation, balanced states
- **Yellow tones** → Transitional zones, elevated terrain, moderate intensity

**Visual Clarity**: Color transitions follow natural gradients that highlight:
- **Physical boundaries** (land/water, pressure systems)
- **Intensity gradients** (temperature ranges, wind speeds)
- **Dynamic patterns** (circulation systems, weather fronts)

### Terminal Compatibility

**Supported Terminals**:
- Modern terminal emulators with ANSI color support
- xterm, iTerm2, Terminal.app, Windows Terminal
- tmux and screen sessions (with color pass-through)
- SSH connections (color preserved over remote sessions)

**Requirements**:
- UTF-8 encoding for directional wind arrows
- 256-color or true-color terminal capability (recommended)
- Minimum 80×40 character display for optimal color visibility

**Color Output Control**:
```bash
# Colorized output (default when terminal supports colors)
cargo run --bin sim-protoype -- --ascii-frames

# Force color output even if terminal detection fails
COLOR_ALWAYS=1 cargo run --bin sim-protoype -- --ascii-frames

# Disable colors for plain ASCII (compatibility mode)
NO_COLOR=1 cargo run --bin sim-protoype -- --ascii-frames
```

### Multi-Viewport TUI Integration

The colorized ASCII framebuffer integrates seamlessly with the TUI multi-viewport system:

- **Colorized viewports**: Each viewport maintains full color information
- **ANSI-to-ratatui conversion**: Terminal colors automatically mapped to TUI styling
- **Real-time color updates**: All layer changes immediately reflected in colored display
- **Consistent theming**: Identical color schemes across ASCII and TUI modes

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

**`--preset climate-analysis`** ⭐ **Enhanced with colorization**
- Layers: temperature, biomes, elevation
- Zoom: continental
- Purpose: Temperature-biome relationship analysis
- **Color benefits**: Temperature zones clearly distinguished by blue→red spectrum, biome boundaries enhanced with semantic colors

**`--preset storm-tracking`** ⭐ **Enhanced with advanced wind colorization**
- Layers: pressure, wind, temperature
- Zoom: regional
- Purpose: Atmospheric circulation monitoring
- **Color benefits**: Pressure systems visible through blue→red gradients, **NEW**: wind patterns enhanced with speed+direction color coding for superior circulation analysis

**`--preset change-detection`** ⭐ **Enhanced with temporal colorization**
- Layers: pressure, temperature, water, changes
- Zoom: continental
- Buffer: 10 frames
- Purpose: Temporal evolution analysis
- **Color benefits**: Changes highlighted through color transitions, pressure evolution visible through blue→red spectrum shifts

**`--preset regional-deep-dive`** ⭐ **Full colorization showcase**
- Layers: elevation, water, temperature, pressure, biomes, wind
- Zoom: local
- Purpose: Comprehensive local analysis
- **Color benefits**: All major atmospheric and terrain systems visualized through coordinated color schemes, **NEW**: advanced wind visualization with speed+direction encoding

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

### Understanding Atmospheric Physics Patterns ⭐ **Enhanced with Color Analysis**

**Healthy Atmospheric Circulation Indicators (Colorized Analysis):**
- **Temperature-Pressure Correlation**: Warm areas (**yellow-red** in temperature) should show low pressure (**blue** in pressure)
- **Wind-Pressure Relationship**: Wind arrows should point from high pressure (**red-yellow** `#` `+`) toward low pressure (**blue** `-` `.`), with **color intensity matching wind strength**
- **Realistic Pressure Range**: [91.2, 102.0] kPa indicates proper thermal circulation physics, visible through **smooth blue→red pressure gradients**
- **Coherent Circulation**: Pressure systems should drive organized wind patterns with **consistent color-coded circulation cells**, not random vectors
- **NEW: Wind Pattern Analysis**: **Strong circulation** visible through intense **red** high-speed winds, **direction consistency** confirmed through **directional hue patterns**

**Physics Violation Warning Signs (Color-Enhanced Detection):**
- **Random Pressure Patterns**: Scattered **red** `#` and **blue** `-` symbols with no temperature correlation, visible as **chaotic color patches**
- **Extreme Pressure Values**: Values outside [85, 115] kPa range may indicate numerical issues, shown as **excessive saturation** in red/blue extremes
- **Disconnected Wind Fields**: Wind arrows showing no relationship to pressure gradients, visible as **color-inconsistent** wind patterns (e.g., **strong red winds** in **blue low-pressure** areas flowing wrong direction)
- **Impossible Temperature Gradients**: Sudden temperature jumps without elevation changes, shown as **sharp color boundaries** between **blue** cold and **red** hot zones
- **NEW: Wind-Color Inconsistencies**: **Directional color patterns** that don't match expected circulation (e.g., **northeast colors** on **southwest arrows**)

### Continental-Scale Circulation Patterns

**Thermal Circulation (Expected Pattern) - Color-Enhanced Visualization:**
1. **Mountain Heating**: During day, mountain slopes (**red** `@` `%`) warm faster than valleys (**green-cyan** `.` `,`)
2. **Pressure Response**: Warm areas develop low pressure (**blue** `-` `.`), cool areas high pressure (**red-yellow** `+` `#`)
3. **Wind Generation**: Air flows from high pressure to low pressure, creating **color-coordinated** upslope winds (flows from **red-yellow** zones toward **blue** zones) with **green-yellow** moderate intensity (`↑` `↗` `↖`)
4. **Return Circulation**: Compensating flow at altitude creates circulation cells visible as **organized color patterns** in wind field

**Topographic Effects (Color-Enhanced Analysis):**
- **Orographic Lifting**: Wind hitting mountains (**color transition** from moderate **green-yellow** `→` hitting **red** `@`) creates upslope flow with **intensifying colors** (`↗` `↑`) as air accelerates
- **Valley Channeling**: Winds follow valley orientation, creating **strong directional color consistency** and **high-intensity red-yellow** flows through topographic corridors
- **Rain Shadow**: Downslope winds (**red high-speed** `↓` `↘`) create dry conditions on leeward sides, visible through **warm temperature colors** and **reduced moisture signatures**

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

**Climate-Biome Consistency (Color-Coordinated Validation):**
- **Temperature Zones**: Arctic (**blue** `■` `▓`) → **Gray** Tundra (`T`), Temperate (**white-green** `▒` `░`) → **Bright Green** Forest (`F`), Hot (**red** `+` `#`) → **Bright Yellow** Desert (`D`)
- **Moisture Effects**: High water + warm temperature should support **bright green** forests (`F` `R`), not **yellow** deserts (`D`) - color mismatches indicate climate simulation issues
- **Elevation Effects**: Mountains (**red** `@` `%`) should show **white** alpine biomes (`A`) or **bright white** ice (`I`) - color coordination validates elevation-climate coupling

**Problematic Patterns (Color-Flagged Issues):**
- **Bright yellow** deserts (`D`) in high-moisture, **blue** cool areas - **Color mismatch alert**
- **Bright green** tropical forests (`R`) in **blue** cold, dry regions - **Biome-climate color inconsistency**
- **Bright white** ice biomes (`I`) at low elevations with **red-yellow** warm temperatures - **Impossible color combinations**

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

**Problem**: ASCII symbols appear garbled or colors not displaying
**Cause**: Terminal encoding issues, font compatibility, or insufficient color support
**Solutions**: 
- Ensure UTF-8 terminal encoding and Unicode arrow character support
- **NEW**: Verify ANSI color support: `echo -e "\x1b[31mRed Text\x1b[0m"`
- **NEW**: For color issues, try `COLORTERM=truecolor` or `TERM=xterm-256color`
- **NEW**: Disable colors if needed: `NO_COLOR=1 cargo run ...`

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

### Integration with Core Physics ⭐ **Enhanced with Colorized Validation**

The ASCII framebuffer directly accesses validated physics modules with **semantic color encoding**:
- **Temperature Layer**: Full thermal diffusion with realistic heat transfer, **blue→red spectrum** provides immediate thermal gradient visualization
- **Pressure Layer**: Thermodynamic pressure calculation with proper equation of state, **blue→red mapping** reveals circulation physics
- **Wind Layer**: Geostrophic balance with Coriolis effects for domains > 100km, **NEW**: **speed+direction color encoding** provides unprecedented ASCII wind analysis capability
- **Water System**: Mass-conserving flow with scale-aware erosion physics, **blue depth gradients** show hydrological accuracy
- **Biome Classification**: Climate-driven ecosystem modeling with moisture-temperature coupling, **coordinated color schemes** validate climate-ecosystem relationships

This ensures the visualization represents genuine physical processes, not simplified approximations. **Color consistency across all layers** enables cross-system pattern analysis and physics validation through visual inspection.

---

## Conclusion ⭐ **Colorized ASCII: The Future of Text-Based Scientific Visualization**

The ASCII Framebuffer interface provides a powerful, scientifically accurate tool for monitoring continental-scale atmospheric physics simulations through **revolutionary colorized terminal visualization**. By combining real-time visualization with quantitative validation capabilities and **semantic color encoding**, it enables researchers to understand complex atmospheric dynamics, validate physics implementations, and conduct reproducible climate research with **unprecedented visual clarity in text-based interfaces**.

### Key Innovations

**Advanced Wind Visualization**: The new **speed+direction color encoding** system provides the most sophisticated ASCII wind visualization available, combining intensity mapping with directional hue modulation for superior circulation analysis.

**Cross-Platform Consistency**: Identical color schemes across graphics and ASCII frontends ensure **seamless workflow transitions** and **reproducible visual analysis**.

**AI Agent Integration**: Colorized ASCII framebuffers provide **rich visual data through text interfaces**, enabling AI systems to perform sophisticated atmospheric pattern recognition and analysis.

**Educational Excellence**: **Intuitive color schemes** following meteorological conventions make complex atmospheric physics **immediately accessible** to students and researchers.

### Scientific Impact

The interface bridges sophisticated computational physics with accessible terminal-based visualization, making it suitable for both advanced research applications and educational demonstrations of atmospheric science principles. **Colorized output transforms terminal interfaces** from simple monitoring tools into **powerful scientific visualization platforms** capable of revealing complex atmospheric dynamics through semantic color patterns.

**For researchers**: Enhanced pattern recognition capabilities accelerate climate analysis and physics validation.

**For educators**: Intuitive color coding makes atmospheric physics concepts immediately visual and comprehensible.

**For AI systems**: Rich colorized text provides structured visual data for automated atmospheric analysis.

### Technical Resources

For additional technical details, see the complementary documentation:
- `docs/session-handoff.md` - Current implementation status
- `docs/physics-correct-atmospheric-implementation-specification.md` - Physics implementation details
- `docs/collaborative-scientific-physics-specification.md` - Scientific validation framework
- `src/engine/rendering/ansi_colors.rs` - **NEW**: Complete ANSI colorization system implementation