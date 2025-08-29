# Temporal Scaling CLI Architecture Design

## Overview

This document specifies the complete CLI interface architecture for integrating temporal scaling control into weather_demo.rs. The design enables scientists to control the temporal scaling backend through command-line arguments while maintaining backward compatibility and educational value.

## Design Principles

1. **Scientist-Friendly**: CLI arguments use domain terminology (drought, ecosystem, storm) rather than implementation details
2. **Educational**: Help systems explain temporal scaling concepts and trade-offs
3. **Performance Transparent**: Users can monitor actual scaling ratios and performance impact
4. **Research Reproducible**: Configuration can be saved/loaded for repeatable studies
5. **Backward Compatible**: Default behavior remains unchanged (demo mode)

## CLI Argument Structure

### New Arguments to Add to WeatherDemoArgs

```rust
// === TEMPORAL SCALING ARGUMENTS ===

/// Study phenomenon preset - auto-configures temporal scaling for research intent
/// Available presets: drought, ecosystem, climate, storm
#[arg(long, help = "Auto-configure temporal scaling for specific research focus")]
pub study_phenomenon: Option<String>,

/// Temporal scaling mode (demo, realistic, research)
/// demo: Current behavior (fast changes for observation)
/// realistic: Scientific rates (2.5 kg/m²/year ecological accuracy)
/// research: Custom scaling factors for hypothesis testing
#[arg(long, default_value = "demo")]
pub temporal_mode: String,

/// Custom scaling factor for research mode (0.001 to 1000.0)
/// Values < 1.0 slow down processes, > 1.0 accelerate them
#[arg(long, default_value = "1.0")]
pub scaling_factor: f64,

/// Scale biological processes (ecosystem growth, vegetation dynamics)
#[arg(long, default_value = "true")]
pub scale_biological: bool,

/// Scale geological processes (erosion, sediment transport)
#[arg(long, default_value = "false")]
pub scale_geological: bool,

/// Scale atmospheric processes (precipitation, evaporation)
#[arg(long, default_value = "false")]
pub scale_atmospheric: bool,

/// Show temporal scaling performance statistics
#[arg(long)]
pub temporal_stats: bool,

/// Display educational help about temporal scaling concepts
#[arg(long)]
pub temporal_help: bool,

/// Validate temporal configuration and show expected behavior
#[arg(long)]
pub temporal_validate: bool,

/// Load temporal configuration from TOML file
#[arg(long)]
pub temporal_config: Option<String>,

/// Save current temporal configuration to TOML file
#[arg(long)]
pub save_temporal_config: Option<String>,
```

## Study Phenomenon Presets

### Preset Definitions

| Preset | Mode | Custom Factor | Bio | Geo | Atm | Use Case |
|--------|------|---------------|-----|-----|-----|----------|
| `drought` | Research | 0.2 | ✓ | ✗ | ✓ | Long-term ecosystem stress analysis |
| `ecosystem` | Realistic | - | ✓ | ✗ | ✓ | Natural biological growth cycles |
| `climate` | Realistic | - | ✓ | ✗ | ✓ | Climate-ecosystem coupling studies |
| `storm` | Demo | 1.0 | ✗ | ✗ | ✗ | Short-term weather dynamics |

### Example CLI Usage

```bash
# Quick presets for common research scenarios
./weather-demo --study-phenomenon drought
./weather-demo --study-phenomenon ecosystem --temporal-stats
./weather-demo --study-phenomenon climate --save-temporal-config climate_study.toml

# Manual temporal control
./weather-demo --temporal-mode realistic
./weather-demo --temporal-mode research --scaling-factor 0.5
./weather-demo --temporal-mode research --scaling-factor 10.0 --scale-geological

# Educational and validation
./weather-demo --temporal-help
./weather-demo --temporal-validate --temporal-mode realistic
```

## Architecture Components

### 1. Temporal Preset Helper System

**Location**: `src/engine/core/temporal_scaling.rs`

```rust
impl TemporalScalingService {
    /// Create temporal configuration from study phenomenon preset
    pub fn from_study_phenomenon(phenomenon: &str) -> Result<TemporalScalingConfig, String> {
        match phenomenon.to_lowercase().as_str() {
            "drought" => Ok(TemporalScalingConfig {
                mode: TemporalMode::Research,
                custom_scaling_factor: 0.2,  // Slower for long-term effects
                scale_biological: true,
                scale_geological: false,
                scale_atmospheric: true,
            }),
            "ecosystem" => Ok(TemporalScalingConfig {
                mode: TemporalMode::Realistic,
                custom_scaling_factor: 1.0,
                scale_biological: true,
                scale_geological: false,
                scale_atmospheric: true,
            }),
            "climate" => Ok(TemporalScalingConfig {
                mode: TemporalMode::Realistic,
                custom_scaling_factor: 1.0,
                scale_biological: true,
                scale_geological: false,
                scale_atmospheric: true,
            }),
            "storm" => Ok(TemporalScalingConfig {
                mode: TemporalMode::Demo,
                custom_scaling_factor: 1.0,
                scale_biological: false,
                scale_geological: false,
                scale_atmospheric: false,
            }),
            _ => Err(format!(
                "Unknown study phenomenon '{}'. Valid options: drought, ecosystem, climate, storm",
                phenomenon
            )),
        }
    }
}
```

### 2. Temporal Performance Monitor

**Location**: `src/engine/core/temporal_performance.rs` (new file)

```rust
/// Performance monitoring for temporal scaling operations
pub struct TemporalPerformanceMonitor {
    scaling_call_count: u64,
    total_scaling_time: Duration,
    last_performance_summary: Option<PerformanceSummary>,
}

impl TemporalPerformanceMonitor {
    pub fn new() -> Self { /* ... */ }
    
    /// Record a temporal scaling operation
    pub fn record_scaling_operation(&mut self, duration: Duration) { /* ... */ }
    
    /// Generate performance summary for display
    pub fn generate_summary(&self, simulation_ticks: u64) -> PerformanceSummary { /* ... */ }
    
    /// Get current performance statistics
    pub fn current_stats(&self) -> PerformanceStats { /* ... */ }
}

pub struct PerformanceSummary {
    pub total_scaling_operations: u64,
    pub average_operation_time: Duration,
    pub operations_per_second: f64,
    pub percentage_of_simulation_time: f64,
    pub scaling_overhead_assessment: String,
}
```

### 3. Educational Help System

**Location**: `src/applications/weather_demo.rs`

```rust
/// Display comprehensive temporal scaling education
fn display_temporal_help() {
    println!("Temporal Scaling in Weather Demo");
    println!("================================\n");
    
    println!("CONCEPT:");
    println!("  Temporal scaling adjusts the rate of biological, geological, and");
    println!("  atmospheric processes to match different research needs:\n");
    
    println!("MODES:");
    println!("  demo      - Fast observable changes (current behavior)");
    println!("              • Ecosystem: 10.0 kg/m²/day growth rate");
    println!("              • Use for: Demonstrations, quick visualization");
    println!("              • Trade-off: Fast results, less scientifically accurate\n");
    
    println!("  realistic - Scientific accuracy (peer-review quality)");
    println!("              • Ecosystem: 2.5 kg/m²/year growth rate");
    println!("              • Use for: Research publications, long-term studies");
    println!("              • Trade-off: Accurate timescales, slower to observe\n");
    
    println!("  research  - Custom scaling for hypothesis testing");
    println!("              • Ecosystem: Configurable 0.001x to 1000x rates");
    println!("              • Use for: Parameter sensitivity, what-if scenarios");
    println!("              • Trade-off: Maximum flexibility, requires expertise\n");
    
    println!("STUDY PRESETS:");
    println!("  drought    - Long-term ecosystem stress (0.2x realistic rate)");
    println!("  ecosystem  - Natural growth cycles (realistic scientific rates)");
    println!("  climate    - Climate-ecosystem coupling (realistic rates)");
    println!("  storm      - Weather system dynamics (demo rate, atm focus)\n");
    
    println!("EXAMPLES:");
    println!("  # Quick start with presets");
    println!("  ./weather-demo --study-phenomenon drought");
    println!("  ./weather-demo --study-phenomenon ecosystem --temporal-stats");
    println!();
    println!("  # Manual control");
    println!("  ./weather-demo --temporal-mode realistic");
    println!("  ./weather-demo --temporal-mode research --scaling-factor 0.1");
    println!();
    println!("  # Validation and learning");
    println!("  ./weather-demo --temporal-validate --study-phenomenon climate");
    println!("  ./weather-demo --temporal-help");
}
```

### 4. Configuration Validation System

```rust
/// Validate temporal configuration and show expected behavior
fn validate_temporal_config(config: &TemporalScalingConfig, args: &WeatherDemoArgs) {
    println!("Temporal Configuration Validation");
    println!("=================================\n");
    
    let service = TemporalScalingService::new(config.clone());
    
    // Show configuration
    println!("Configuration:");
    println!("  Mode: {:?}", config.mode);
    if config.mode == TemporalMode::Research {
        println!("  Custom scaling factor: {}", config.custom_scaling_factor);
    }
    println!("  Scale biological: {}", config.scale_biological);
    println!("  Scale geological: {}", config.scale_geological);
    println!("  Scale atmospheric: {}", config.scale_atmospheric);
    println!();
    
    // Calculate expected behavior
    println!("Expected Behavior:");
    let dt_hours = 1.0; // 1 hour timestep
    let base_growth_rate = 10.0; // kg/m²/day
    
    let scaled_rate = service.scale_ecosystem_growth_rate(base_growth_rate, dt_hours);
    let daily_rate = scaled_rate * 24.0;
    let annual_rate = daily_rate * 365.0;
    
    println!("  Base ecosystem growth: {:.1} kg/m²/day", base_growth_rate);
    println!("  Scaled ecosystem growth: {:.6} kg/m²/day ({:.2} kg/m²/year)", daily_rate, annual_rate);
    
    // Show simulation expectations
    match config.mode {
        TemporalMode::Demo => {
            println!("  Vegetation changes: Observable in minutes to hours");
            println!("  Simulation duration: Short demos (< 1 hour real time)");
            println!("  Scientific accuracy: Demonstration quality");
        },
        TemporalMode::Realistic => {
            println!("  Vegetation changes: Seasonal cycles, natural pace");
            println!("  Simulation duration: Multi-year studies recommended");
            println!("  Scientific accuracy: Publication quality");
        },
        TemporalMode::Research => {
            let factor = config.custom_scaling_factor;
            if factor < 1.0 {
                println!("  Effect: {:.1}x slower than realistic mode", 1.0 / factor);
                println!("  Use case: Extended timescale studies");
            } else if factor > 1.0 {
                println!("  Effect: {:.1}x faster than realistic mode", factor);
                println!("  Use case: Accelerated hypothesis testing");
            } else {
                println!("  Effect: Same pace as realistic mode");
                println!("  Use case: Realistic research with custom process selection");
            }
        },
    }
    
    println!("\nPerformance Impact: < 1% simulation overhead");
    println!("Configuration is valid and ready for simulation!");
}
```

## Integration Points

### 1. Main Function Integration

**Location**: `src/applications/weather_demo.rs::run_weather_demo()`

```rust
pub fn run_weather_demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = WeatherDemoArgs::parse();

    // === NEW: Handle temporal scaling help ===
    if args.temporal_help {
        display_temporal_help();
        return Ok(());
    }

    // === NEW: Create temporal configuration ===
    let temporal_config = create_temporal_config_from_args(&args)?;
    
    // === NEW: Validate configuration if requested ===
    if args.temporal_validate {
        validate_temporal_config(&temporal_config, &args);
        return Ok(());
    }

    // === NEW: Save temporal config if requested ===
    if let Some(save_path) = &args.save_temporal_config {
        save_temporal_config_to_file(save_path, &temporal_config)?;
        println!("Temporal configuration saved to: {}", save_path);
    }

    // === EXISTING: Continue with current logic ===
    // ... existing seed, terrain generation, etc. ...

    // === NEW: Create temporal scaling service ===
    let temporal_service = TemporalScalingService::new(temporal_config);
    
    // === NEW: Initialize performance monitor if requested ===
    let mut performance_monitor = if args.temporal_stats {
        Some(TemporalPerformanceMonitor::new())
    } else {
        None
    };

    // === MODIFIED: Pass temporal service to simulation ===
    let world_scale = WorldScale::new(
        args.scale_km,
        (args.width as u32, args.height as u32),
        DetailLevel::Standard,
    );
    let sim = Simulation::new_with_temporal_scaling(heightmap, world_scale, temporal_service);

    // === EXISTING: Continue with rendering modes ===
    // ... existing graphics/TUI/ASCII logic ...
}
```

### 2. Configuration System Integration

**Location**: `src/engine/config/mod.rs`

The TemporalScalingConfig is already integrated in SimulationDefaults. The CLI needs to populate this configuration:

```rust
// Update WorkspaceConfig loading to handle temporal settings
impl WorkspaceConfig {
    pub fn apply_temporal_args(&mut self, args: &WeatherDemoArgs, temporal_config: TemporalScalingConfig) {
        self.defaults.temporal_scaling = temporal_config;
    }
}
```

### 3. Simulation Integration

**Location**: `src/engine/sim.rs`

```rust
impl Simulation {
    pub fn new_with_temporal_scaling(
        heightmap: Vec<Vec<f32>>,
        world_scale: WorldScale,
        temporal_service: TemporalScalingService,
    ) -> Self {
        // Store temporal service in simulation
        // Pass to ecosystem feedback and other temporal processes
    }
}
```

## Performance Monitoring Implementation

### Real-Time Statistics Display

When `--temporal-stats` is specified, show periodic performance updates:

```
Temporal Scaling Performance (10 seconds):
==========================================
Scaling operations: 1,247,892
Average operation time: 0.023 μs
Operations per second: 43,421,738
Simulation overhead: 0.3%
Scaling factor efficiency: Optimal

Current temporal state:
  Mode: Realistic
  Biological scaling: 0.000685 (1/3650)
  Geological scaling: 1.0 (disabled)
  Atmospheric scaling: 1.0 (disabled)
```

### Integration with Existing Stats Mode

The existing `--stats` mode already has diagnostic output. The temporal stats should integrate seamlessly:

```rust
fn run_stats_mode(
    mut simulation: Simulation,
    interval: usize,
    temporal_monitor: Option<&mut TemporalPerformanceMonitor>,
) -> Result<(), Box<dyn std::error::Error>> {
    // ... existing stats logic ...
    
    // Add temporal stats if monitoring enabled
    if let Some(monitor) = temporal_monitor {
        let perf_summary = monitor.generate_summary(iteration_count as u64);
        println!("Temporal: {} ops/sec, {:.1}% overhead", 
                 perf_summary.operations_per_second as u32,
                 perf_summary.percentage_of_simulation_time * 100.0);
    }
}
```

## Configuration File Support

### TOML Configuration Format

Support loading/saving temporal configurations:

```toml
# Example: drought_study_2024.toml
[metadata]
study_name = "Mountain Ecosystem Drought Response"
researcher = "Dr. Smith"
created = "2024-08-28"
description = "Long-term drought impact on alpine vegetation"

[temporal]
mode = "research"
custom_scaling_factor = 0.2
scale_biological = true
scale_geological = false
scale_atmospheric = true

[simulation]
width = 240
height = 120
scale_km = 500.0
seed = 42
roughness = 0.7
```

### CLI Integration

```bash
# Load complete configuration
./weather-demo --temporal-config drought_study_2024.toml

# Override specific temporal settings
./weather-demo --temporal-config base_config.toml --scaling-factor 0.1

# Save current configuration
./weather-demo --study-phenomenon ecosystem --save-temporal-config my_study.toml
```

## Error Handling and Validation

### Input Validation

```rust
fn validate_temporal_arguments(args: &WeatherDemoArgs) -> Result<(), String> {
    // Validate temporal mode
    let valid_modes = ["demo", "realistic", "research"];
    if !valid_modes.contains(&args.temporal_mode.to_lowercase().as_str()) {
        return Err(format!("Invalid temporal mode '{}'. Options: {}", 
                          args.temporal_mode, valid_modes.join(", ")));
    }

    // Validate scaling factor bounds
    if args.temporal_mode.to_lowercase() == "research" {
        if args.scaling_factor < 0.001 || args.scaling_factor > 1000.0 {
            return Err("Scaling factor must be between 0.001 and 1000.0".to_string());
        }
    }

    // Validate study phenomenon
    if let Some(phenomenon) = &args.study_phenomenon {
        let valid_phenomena = ["drought", "ecosystem", "climate", "storm"];
        if !valid_phenomena.contains(&phenomenon.to_lowercase().as_str()) {
            return Err(format!("Invalid study phenomenon '{}'. Options: {}", 
                              phenomenon, valid_phenomena.join(", ")));
        }
    }

    // Validate argument conflicts
    if args.study_phenomenon.is_some() && args.temporal_mode != "demo" {
        return Err("Cannot specify both --study-phenomenon and --temporal-mode".to_string());
    }

    Ok(())
}
```

### Helpful Error Messages

```rust
fn handle_temporal_error(error: String) {
    eprintln!("❌ Temporal configuration error: {}", error);
    eprintln!();
    eprintln!("💡 Quick fixes:");
    eprintln!("   • Use --temporal-help for complete documentation");
    eprintln!("   • Try a preset: --study-phenomenon ecosystem");
    eprintln!("   • Validate config: --temporal-validate --temporal-mode realistic");
    eprintln!();
    std::process::exit(1);
}
```

## Implementation Summary

### Files to Create

1. `src/engine/core/temporal_performance.rs` - Performance monitoring
2. `docs/examples/temporal_config_examples/` - Example TOML files

### Files to Modify

1. `src/applications/weather_demo.rs` - Add CLI arguments and integration logic
2. `src/engine/core/temporal_scaling.rs` - Add preset helper methods
3. `src/engine/sim.rs` - Accept temporal scaling service in constructor
4. `src/engine/config/mod.rs` - Enhance TOML loading/saving

### Testing Strategy

1. **CLI Argument Parsing**: Validate all argument combinations
2. **Preset Configurations**: Verify each preset produces expected config
3. **Performance Monitoring**: Ensure < 1% overhead in all modes  
4. **Educational Help**: Validate help text accuracy and completeness
5. **Configuration Persistence**: Test TOML loading/saving roundtrip

### Backward Compatibility

- Default behavior remains identical (demo mode, no temporal scaling)
- All new arguments are optional
- Existing command lines continue to work unchanged
- New features are opt-in through explicit arguments

This architecture provides a scientist-friendly, educational, and performance-transparent interface to the temporal scaling system while maintaining the robust foundation already established in weather_demo.rs.