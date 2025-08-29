# Temporal Scaling Interface Design

<!-- ABOUTME: UX design specification for scientist-friendly temporal scaling controls -->
<!-- ABOUTME: Covers command-line, TUI, and graphics interfaces with educational elements -->

## Overview

This document specifies user interface patterns for temporal scaling control in our simulation system. The design prioritizes scientific workflow integration while maintaining accessibility for educational users.

## Core UX Principles

### 1. **Scientific Mental Model Alignment**
- Match how scientists think about temporal scales in their research
- Connect interface choices to real-world phenomena they study
- Provide clear mapping from research questions to temporal modes

### 2. **Progressive Disclosure**
- Simple mode selection for quick starts
- Advanced configuration available when needed
- Expert controls accessible but not overwhelming

### 3. **Immediate Feedback**
- Show effects of temporal scaling choices in real-time
- Visualize how scaling affects different physical processes
- Provide quantitative feedback on computational trade-offs

## Command-Line Interface Design

### Basic Mode Selection
```bash
# Simple mode selection - matches research intent
./weather-demo --temporal-mode demo         # 10.0 kg/m²/day - visible changes
./weather-demo --temporal-mode realistic    # 2.5 kg/m²/year - scientific accuracy  
./weather-demo --temporal-mode research     # Custom scaling - specialized studies

# Educational shortcuts with explanations
./weather-demo --temporal-mode classroom    # Demo mode with educational annotations
./weather-demo --temporal-mode publication  # Realistic mode optimized for figures
```

### Research Mode Configuration
```bash
# Research mode with custom scaling
./weather-demo --temporal-mode research --scaling-factor 0.1   # 10x slower than realistic
./weather-demo --temporal-mode research --scaling-factor 100   # 100x faster than realistic

# Process-specific scaling (advanced)
./weather-demo --temporal-mode research \
  --scale-biological true \
  --scale-geological false \
  --scale-atmospheric false \
  --custom-scaling-factor 2.0

# Reproducible research configurations
./weather-demo --config-file temporal_configs/drought_study_2024.toml
```

### Scientific Context Arguments
```bash
# Study duration-based selection
./weather-demo --study-duration "1 year" --auto-temporal     # Chooses realistic mode
./weather-demo --study-duration "1 day" --auto-temporal      # Chooses demo mode
./weather-demo --study-duration "10 years" --auto-temporal   # Research mode, optimized scaling

# Phenomenon-focused selection
./weather-demo --study-phenomenon drought        # Optimizes for multi-year processes
./weather-demo --study-phenomenon storm          # Optimizes for hour/day processes
./weather-demo --study-phenomenon erosion        # Optimizes for geological timescales
```

### Integration with Existing Arguments
```bash
# Current weather-demo command structure (enhanced with temporal scaling)
./weather-demo \
  --seed 12345 \
  --roughness 0.7 \
  --width 240 \
  --height 120 \
  --scale-km 200.0 \
  --temporal-mode realistic \        # NEW: Temporal scaling mode
  --scaling-factor 1.0 \            # NEW: Custom scaling for research mode
  --temporal-config-file config.toml \  # NEW: Load temporal configuration
  --graphics                        # Existing interface mode
```

## TUI Interface Design

### Mode Selection Panel
```
┌─ Temporal Scaling Configuration ─────────────────────────────────────┐
│                                                                       │
│ Choose temporal mode based on your research goals:                   │
│                                                                       │
│ ○ Demo Mode          Fast ecosystem changes for demonstration         │
│   └─ 10.0 kg/m²/day  Visible results in minutes                     │
│                                                                       │
│ ● Realistic Mode     Scientific accuracy for research                 │
│   └─ 2.5 kg/m²/year  Matches real-world ecological timescales       │
│                                                                       │
│ ○ Research Mode      Custom scaling for specialized studies           │
│   └─ [Configure...]  Advanced temporal parameter control             │
│                                                                       │
│ [Start Simulation]   [Save Config]   [Load Config]   [Help]         │
└───────────────────────────────────────────────────────────────────────┘
```

### Runtime Mode Switching
```
┌─ Simulation: Mountain Erosion Study ─────── Realistic Mode ──────────┐
│                                                                       │
│ Precipitation: 1,247 mm/year  │  Erosion Rate: 0.3 mm/year          │
│ Time Scale: 2.5 kg/m²/year   │  Simulation Year: 23.4               │
│                                                                       │
│ Press 'T' to change temporal scaling                                 │
│ Press 'R' to reset to year 0                                         │
│ Press 'S' to save current configuration                              │
│                                                                       │
│ [Terrain visualization area...]                                      │
└───────────────────────────────────────────────────────────────────────┘

# When 'T' is pressed:
┌─ Change Temporal Mode ────────────────────────────────────────────────┐
│                                                                       │
│ Current: Realistic Mode (2.5 kg/m²/year)                            │
│                                                                       │
│ Switch to:                                                            │
│ [D] Demo Mode      - See changes faster (10.0 kg/m²/day)            │
│ [R] Research Mode  - Custom scaling factors                          │
│                                                                       │
│ Warning: Changing temporal mode will affect all physical processes   │
│ Continue? [Y/N]                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### Research Mode Configuration Panel
```
┌─ Research Mode Configuration ────────────────────────────────────────┐
│                                                                       │
│ Base Scaling Factor: [1.0    ] (1.0 = realistic, >1 = faster)      │
│                                                                       │
│ Process-Specific Scaling:                                            │
│ ├─ Precipitation:  [1.0    ] │ Current: 2.5 kg/m²/year             │
│ ├─ Erosion:        [1.0    ] │ Current: 0.3 mm/year                │
│ ├─ Ecosystem:      [1.0    ] │ Current: Standard growth rates       │
│ └─ Tectonics:      [0.001  ] │ Current: 1 mm/10kyear (geological)  │
│                                                                       │
│ Study Duration: [10 years  ] │ Computational Cost: ████░░ Medium    │
│                                                                       │
│ Presets: [Drought Study] [Storm Analysis] [Long-term Evolution]     │
│                                                                       │
│ [Apply] [Save As...] [Reset] [Cancel]                               │
└───────────────────────────────────────────────────────────────────────┘
```

## Graphics Mode Interface Design

### Temporal Control Overlay
```
Scientific Simulation - Temporal Scaling Controls

┌─────────────────────────────────┐  ┌─────────────────────────────────┐
│ Mode: Realistic                 │  │ Time: Year 15.7                 │
│ Scale: 2.5 kg/m²/year          │  │ Rate: 3.2x real-time           │
│                                 │  │                                 │
│ [Demo] [Realistic] [Research]   │  │ Effects This Year:              │
│                                 │  │ • Precipitation: 1,247mm        │
│ Quick Study Types:              │  │ • Erosion: 4.5mm               │
│ [Climate Change]                │  │ • Vegetation: +12% coverage     │
│ [Storm Systems]                 │  │                                 │
│ [Ecosystem Evolution]           │  │ [Pause] [Reset] [Export]        │
└─────────────────────────────────┘  └─────────────────────────────────┘

             [Main terrain visualization area]
```

### Mode Transition Visual Feedback
When switching modes, show animated transition with quantitative effects:

```
Switching from Demo Mode to Realistic Mode...

┌─ Temporal Scaling Adjustment ─────────────────────────────────────────┐
│                                                                       │
│ Precipitation Rate:    10.0 kg/m²/day  ──→  2.5 kg/m²/year         │
│                        [████████████████████] 1,460x slower          │
│                                                                       │
│ Erosion Rate:          Fast visual      ──→  Geological timescale    │
│                        [████████████████████] Realistic physics      │
│                                                                       │
│ Ecosystem Changes:     Minutes          ──→  Seasonal cycles         │
│                        [████████████████████] Natural growth rates   │
│                                                                       │
│ Expected Simulation:   Quick demo       ──→  Long-term study         │
│                        [████████████████████] Research accuracy      │
│                                                                       │
│ This will affect all current processes. Continue? [Yes] [No]         │
└───────────────────────────────────────────────────────────────────────┘
```

## Educational Elements

### Temporal Scaling Concept Introduction
```
┌─ Understanding Temporal Scaling ──────────────────────────────────────┐
│                                                                       │
│ Real ecosystems change slowly - a forest takes decades to mature.    │
│ Climate patterns emerge over years. Geological processes take        │
│ millennia.                                                            │
│                                                                       │
│ To make these processes observable in simulation, we can:             │
│                                                                       │
│ Demo Mode:     Speed up time for visible changes                      │
│ ├─ Use Case:   Classroom demonstrations, quick visualization          │
│ └─ Trade-off:  Less scientifically accurate, more observable          │
│                                                                       │
│ Realistic Mode: Use actual scientific timescales                      │
│ ├─ Use Case:   Research, publications, long-term studies             │
│ └─ Trade-off:  Scientifically accurate, slower to observe            │
│                                                                       │
│ Research Mode:  Custom scaling for specialized studies               │
│ ├─ Use Case:   Hypothesis testing, parameter sensitivity             │
│ └─ Trade-off:  Maximum flexibility, requires expertise               │
│                                                                       │
│ [Continue] [Learn More] [Start Tutorial]                             │
└───────────────────────────────────────────────────────────────────────┘
```

### Interactive Tutorial Mode
```
Tutorial: Temporal Scaling Effects

Step 1: Observe Demo Mode (10.0 kg/m²/day)
Watch how quickly vegetation responds to rainfall...
[Simulation running with annotations...]

Step 2: Switch to Realistic Mode (2.5 kg/m²/year)  
Notice how the same processes now follow natural timescales...
[Side-by-side comparison...]

Step 3: Research Configuration
Try adjusting just precipitation scaling while keeping other processes realistic...
[Interactive parameter adjustment...]
```

## Configuration Management

### Research Reproducibility Features

#### Configuration File Format
```toml
# temporal_configs/drought_study_2024.toml
[metadata]
study_name = "Drought Impact on Mountain Ecosystems"
researcher = "Dr. Smith"
created = "2024-01-15"
description = "Long-term drought effects with accelerated erosion"

[temporal_scaling]
mode = "research"
base_scaling = 1.0
precipitation_scaling = 0.5  # Reduced rainfall
erosion_scaling = 2.0        # Accelerated weathering
ecosystem_scaling = 1.0      # Normal growth rates
tectonics_scaling = 0.001    # Geological timescale

[simulation]
duration_years = 50
output_interval = 1.0
initial_conditions = "temperate_mountain"
```

#### Configuration Management UI
```
┌─ Configuration Manager ───────────────────────────────────────────────┐
│                                                                       │
│ Saved Configurations:                                                 │
│                                                                       │
│ ├─ 📁 My Studies                                                     │
│ │  ├─ drought_study_2024.toml     [Load] [Edit] [Duplicate]         │
│ │  ├─ storm_analysis_spring.toml   [Load] [Edit] [Duplicate]         │
│ │  └─ ecosystem_recovery.toml      [Load] [Edit] [Duplicate]         │
│ │                                                                     │
│ ├─ 📁 Templates                                                      │
│ │  ├─ climate_change_template.toml [Load] [Customize]                │
│ │  ├─ erosion_study_template.toml  [Load] [Customize]                │
│ │  └─ quick_demo_template.toml     [Load] [Customize]                │
│ │                                                                     │
│ └─ 📁 Shared                                                         │
│    ├─ published_study_replication.toml [Load] [Info]                 │
│    └─ classroom_examples.toml           [Load] [Info]                 │
│                                                                       │
│ [New Configuration] [Import] [Export] [Help]                         │
└───────────────────────────────────────────────────────────────────────┘
```

## Real-Time Feedback Mechanisms

### Temporal Effects Visualization
```
Current Effects of Temporal Scaling:

Precipitation System:
████████████████████ 2.5 kg/m²/year (Realistic)
Effects: Natural seasonal patterns, realistic storm intensity

Erosion Processes:  
██████░░░░░░░░░░░░░░ 0.3 mm/year (Geological timescale)
Effects: Long-term landscape evolution, river carving

Ecosystem Dynamics:
████████████████████ Standard growth rates
Effects: Seasonal vegetation cycles, natural succession

Computational Load:
██████░░░░░░░░░░░░░░ Medium (23% CPU utilization)
Effects: Real-time simulation possible for ~50 simulation years
```

### Process Impact Indicators
```
┌─ Temporal Scaling Impact Analysis ────────────────────────────────────┐
│                                                                       │
│ Your current settings will affect:                                    │
│                                                                       │
│ ✓ Precipitation patterns  │ Realistic seasonal cycles                │
│ ✓ River flow dynamics     │ Natural flood/drought patterns           │
│ ✓ Vegetation growth        │ Seasonal growth spurts                   │
│ ✓ Soil erosion            │ Long-term geological processes           │
│ ⚠ Storm formation         │ May need faster timescales               │
│                                                                       │
│ Recommendations:                                                      │
│ • For storm studies: Consider Demo mode for observable events        │
│ • For erosion research: Current Realistic mode is appropriate        │
│ • For ecosystem studies: Current settings show seasonal patterns     │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

## Accessibility and Workflow Integration

### Keyboard Shortcuts (All Interfaces)
```
T        - Toggle temporal mode selection
R        - Reset simulation to time zero
S        - Save current configuration
L        - Load configuration
Ctrl+T   - Quick mode switch (Demo ↔ Realistic)
F1       - Show temporal scaling help
F2       - Show process impact analysis
```

### Command Completion and Help
```bash
$ ./weather-demo --temporal-mode <TAB>
demo      realistic     research      classroom    publication

$ ./weather-demo --help temporal
Temporal Scaling Options:

Modes:
  demo         Fast ecosystem changes (10.0 kg/m²/day)
  realistic    Scientific accuracy (2.5 kg/m²/year)  
  research     Custom scaling factors (see --scaling-factor)
  classroom    Demo mode with educational annotations
  publication  Realistic mode optimized for figures

Research Mode Options:
  --scaling-factor FLOAT     Overall temporal multiplier
  --config-file FILE         Load research configuration
  
Study Type Shortcuts:
  --study-phenomenon TYPE    Auto-configure for phenomenon
                            (drought, storm, erosion, ecosystem)
```

### Integration with Scientific Workflow
```python
# Python API for programmatic control
from kosmarium import TemporalSimulation

# Configuration-driven research
sim = TemporalSimulation.from_config("drought_study_2024.toml")
results = sim.run_batch([
    {"precipitation_scaling": 0.1},  # Severe drought
    {"precipitation_scaling": 0.5},  # Moderate drought  
    {"precipitation_scaling": 1.0},  # Normal conditions
])

# Publication-ready output
sim.export_data("drought_results.csv")
sim.export_figures("figures/", format="publication")
```

## Implementation Roadmap

### Phase 1: Command-Line Integration (Immediate)
**Target**: Add temporal scaling arguments to `WeatherDemoArgs` structure

```rust
// Add to WeatherDemoArgs in weather_demo.rs
#[derive(Parser)]
pub struct WeatherDemoArgs {
    // ... existing args ...
    
    /// Temporal scaling mode (demo, realistic, research)
    #[arg(long, default_value = "demo")]
    pub temporal_mode: String,
    
    /// Custom scaling factor for research mode
    #[arg(long, default_value = "1.0")]
    pub scaling_factor: f64,
    
    /// Scale biological processes (research mode)
    #[arg(long, default_value = "true")]
    pub scale_biological: bool,
    
    /// Scale geological processes (research mode)  
    #[arg(long, default_value = "false")]
    pub scale_geological: bool,
    
    /// Scale atmospheric processes (research mode)
    #[arg(long, default_value = "false")]  
    pub scale_atmospheric: bool,
    
    /// Load temporal configuration from TOML file
    #[arg(long)]
    pub temporal_config: Option<String>,
    
    /// Study phenomenon for auto-configuration
    #[arg(long)]
    pub study_phenomenon: Option<String>,
}
```

**Integration Point**: Create `TemporalScalingConfig` from args before simulation initialization:

```rust
fn create_temporal_config_from_args(args: &WeatherDemoArgs) -> TemporalScalingConfig {
    use crate::engine::core::{TemporalMode, TemporalScalingConfig};
    
    let mode = match args.temporal_mode.to_lowercase().as_str() {
        "demo" => TemporalMode::Demo,
        "realistic" => TemporalMode::Realistic, 
        "research" => TemporalMode::Research,
        _ => {
            eprintln!("Warning: Unknown temporal mode '{}', using demo", args.temporal_mode);
            TemporalMode::Demo
        }
    };
    
    TemporalScalingConfig {
        mode,
        custom_scaling_factor: args.scaling_factor,
        scale_biological: args.scale_biological,
        scale_geological: args.scale_geological,
        scale_atmospheric: args.scale_atmospheric,
    }
}
```

### Phase 2: Runtime Mode Switching (TUI Enhancement)

**Target**: Add temporal scaling controls to TUI interface

```rust
// Enhanced TUI event handling
match event {
    // Existing controls...
    Event::Key(KeyEvent { code: KeyCode::Char('t'), .. }) => {
        show_temporal_mode_selector(&mut sim, &mut terminal)?;
    },
    Event::Key(KeyEvent { code: KeyCode::Char('T'), .. }) => {
        quick_temporal_mode_toggle(&mut sim)?;
    },
    // ...
}
```

### Phase 3: Configuration Management (File I/O)

**Target**: Enable saving/loading temporal configurations

```toml
# Example: drought_study_config.toml
[metadata]
study_name = "Drought Impact Analysis"
researcher = "Dr. Smith"
created = "2024-08-28"

[temporal]
mode = "research"
custom_scaling_factor = 0.5
scale_biological = true
scale_geological = false
scale_atmospheric = false

[simulation]  
width = 240
height = 120
scale_km = 500.0
seed = 12345
```

### Phase 4: Graphics Mode Integration

**Target**: Add temporal scaling overlay to macroquad graphics interface

```rust
// Graphics mode temporal controls
fn draw_temporal_controls(sim: &Simulation, font: Font) {
    let mode_text = format!("Mode: {}", sim.get_temporal_mode_description());
    let rate_text = format!("Rate: {}", sim.get_current_scaling_factor());
    
    // Draw control overlay
    draw_text_ex(&mode_text, 10.0, 30.0, TextParams {
        font,
        font_size: 16,
        color: WHITE,
        ..Default::default()
    });
    
    // Mode switching hints
    draw_text("Press T to change temporal mode", 10.0, screen_height() - 40.0, 14.0, WHITE);
}
```

## Validation Testing Plan

### Usability Testing Scenarios

#### Scenario 1: Climate Scientist - Long-term Study
**User Goal**: Study 50-year climate evolution with realistic timescales
**Test Workflow**:
```bash
./weather-demo --temporal-mode realistic --scale-km 1000 --width 120 --height 60
# Expect: Slow, scientifically accurate ecosystem changes
```

#### Scenario 2: Educator - Classroom Demonstration  
**User Goal**: Show students ecosystem dynamics in 10-minute class period
**Test Workflow**:
```bash
./weather-demo --temporal-mode demo --graphics
# Press 'T' to show mode explanation
# Expect: Visible vegetation changes within minutes
```

#### Scenario 3: Researcher - Parameter Sensitivity
**User Goal**: Test drought sensitivity with 50% precipitation scaling
**Test Workflow**:
```bash
./weather-demo --temporal-mode research --scaling-factor 0.5 --scale-biological true
# Save configuration for reproducibility
```

#### Scenario 4: Student - Learning Temporal Concepts
**User Goal**: Understand difference between demo and realistic modes
**Test Workflow**:
```bash
./weather-demo --temporal-mode demo --graphics
# Run for 5 minutes, observe changes
# Switch to realistic mode with 'T' key
# Observe dramatically different change rates
```

### Performance Validation
- **Startup time**: Should remain < 2 seconds with temporal configuration
- **Mode switching**: Should complete < 1 second in TUI/graphics
- **Memory overhead**: Should add < 5MB for temporal scaling service
- **Simulation performance**: Should maintain >300 ticks/10s baseline

### User Experience Metrics

#### Quantitative Validation
- **Command completion success rate**: >95% for basic temporal arguments
- **Mode switching success rate**: >98% in TUI interface
- **Configuration save/load success rate**: 100% accuracy
- **Performance impact**: <5% simulation speed reduction

#### Qualitative Assessment
- **Intuitive discovery**: New users find temporal controls within 30 seconds
- **Scientific accuracy communication**: Researchers understand mode implications
- **Educational effectiveness**: Students grasp temporal scaling concepts
- **Workflow integration**: Fits naturally into existing scientific processes

## Success Metrics

### Usability Goals
- **Time to first successful simulation**: < 2 minutes for new users
- **Mode switching efficiency**: < 10 seconds during runtime
- **Configuration reproducibility**: 100% exact result replication from saved configs
- **Educational clarity**: Non-experts can understand temporal scaling concepts within 5 minutes

### User Experience Validation
- **Scientist feedback**: "Matches my research workflow naturally"
- **Educator feedback**: "Students grasp temporal scaling concepts quickly"  
- **Research reproducibility**: Configurations can be shared and replicated exactly
- **Performance predictability**: Users can estimate computational requirements accurately

This design prioritizes the intersection of scientific rigor, educational accessibility, and practical workflow integration while maintaining the underlying temporal scaling architecture's flexibility and precision.

## Next Steps for Implementation

### Immediate Actions (Phase 1)
1. **Add temporal arguments to WeatherDemoArgs** - Extend command-line interface
2. **Create argument parsing functions** - Convert CLI args to TemporalScalingConfig  
3. **Integrate with simulation initialization** - Pass temporal config to simulation creation
4. **Add help text and examples** - Provide clear usage documentation

### Short-term Goals (Phase 2)
1. **Implement TUI mode switching** - Runtime temporal mode changes
2. **Add visual feedback indicators** - Show current temporal mode and effects
3. **Create configuration templates** - Common study type presets
4. **Test with target users** - Validate UX with scientists and educators

This implementation plan builds on the existing temporal scaling architecture while creating intuitive interfaces that match how scientists actually work with simulation tools.