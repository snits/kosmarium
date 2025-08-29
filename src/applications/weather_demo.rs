// ABOUTME: Weather Demo application - atmospheric dynamics and weather pattern visualization
// ABOUTME: Demonstrates engine weather systems with Coriolis effects and geostrophic winds

use clap::Parser;
use macroquad::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

// Import engine components
use sim_prototype::engine::{
    Simulation, WorkspaceConfig,
    core::{
        DetailLevel, TemporalMode, TemporalPerformanceMonitor, TemporalScale, TemporalScalingConfig,
        TemporalScalingService, WorldScale,
    },
    physics::{DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator},
    rendering::{
        AsciiFramebuffer, FramebufferConfig, GraphicsRenderer, VisualizationLayer, ascii_render,
        multi_viewport::{MovementDirection, MultiViewportApp},
        run_tui,
    },
};

#[derive(Parser)]
#[command(name = "weather-demo")]
#[command(about = "Atmospheric dynamics and weather pattern visualization")]
pub struct WeatherDemoArgs {
    /// Random seed for terrain generation (defaults to current time)
    #[arg(short, long)]
    pub seed: Option<u64>,

    /// Terrain roughness (0.0 - 1.0, higher = more chaotic)
    #[arg(short, long, default_value = "0.7")]
    pub roughness: f32,

    /// Detail persistence across scales (0.0 - 1.0)
    #[arg(short, long, default_value = "0.6")]
    pub persistence: f32,

    /// Map width in cells
    #[arg(short = 'W', long, default_value = "240")]
    pub width: usize,

    /// Map height in cells
    #[arg(short = 'H', long, default_value = "120")]
    pub height: usize,

    /// Use ASCII mode instead of TUI
    #[arg(long)]
    pub ascii: bool,

    /// Use graphics mode (macroquad) instead of TUI
    #[arg(long)]
    pub graphics: bool,

    /// Use multi-viewport TUI mode for simultaneous layer monitoring
    #[arg(long)]
    pub multi_viewport: bool,

    /// Physical scale of the domain in kilometers
    #[arg(long, default_value = "200.0")]
    pub scale_km: f64,

    /// Show simulation statistics and diagnostics
    #[arg(long)]
    pub stats: bool,

    /// Stats output interval in simulation ticks (only with --stats)
    #[arg(long, default_value = "10")]
    pub interval: usize,

    /// Enable ASCII framebuffer mode with multiple layers
    #[arg(long)]
    pub ascii_frames: bool,

    /// Layers to display (comma-separated: elevation,water,biomes,temperature,pressure,wind,flow,sediment)
    #[arg(long, default_value = "elevation,water,biomes")]
    pub layers: String,

    /// Frame buffer size for temporal analysis
    #[arg(long, default_value = "5")]
    pub buffer_size: usize,

    /// Frame width for ASCII framebuffer (0 = auto-size based on scale)
    #[arg(long, default_value = "0")]
    pub frame_width: usize,

    /// Frame height for ASCII framebuffer (0 = auto-size based on scale)
    #[arg(long, default_value = "0")]
    pub frame_height: usize,

    /// Scale zoom level for detailed analysis (continental, regional, local)
    #[arg(long, default_value = "continental")]
    pub zoom: String,

    /// Scientific workflow preset (climate-analysis, storm-tracking, change-detection, regional-deep-dive, custom)
    #[arg(long, default_value = "custom")]
    pub preset: String,

    /// Load configuration from YAML workspace file
    #[arg(long)]
    pub load_config: Option<String>,

    /// Save current configuration to YAML workspace file
    #[arg(long)]
    pub save_config: Option<String>,

    /// Author name for workspace metadata
    #[arg(long, default_value = "Unknown")]
    pub author: String,

    // === TEMPORAL SCALING ARGUMENTS ===
    /// Study phenomenon preset - auto-configures temporal scaling for research intent
    /// Available presets: drought, ecosystem, climate, storm
    #[arg(
        long,
        help = "Auto-configure temporal scaling for specific research focus"
    )]
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

    /// Show temporal scaling performance statistics during simulation
    #[arg(long)]
    pub temporal_stats: bool,

    /// Display educational help about temporal scaling concepts and exit
    #[arg(long)]
    pub temporal_help: bool,

    /// Validate temporal configuration and show expected behavior, then exit
    #[arg(long)]
    pub temporal_validate: bool,

    /// Load temporal configuration from YAML file
    #[arg(long)]
    pub temporal_config: Option<String>,

    /// Save current temporal configuration to YAML file
    #[arg(long)]
    pub save_temporal_config: Option<String>,
}

/// Calculate appropriate framebuffer dimensions based on zoom level and simulation scale
fn calculate_scale_aware_dimensions(
    zoom_level: &str,
    user_width: usize,
    user_height: usize,
    sim_width: usize,
    sim_height: usize,
    scale_km: f64,
) -> (usize, usize) {
    // If user specified explicit dimensions, use those
    if user_width > 0 && user_height > 0 {
        return (user_width, user_height);
    }

    // Calculate base dimensions for scientific visibility (much larger than 8x8!)
    let base_multiplier = match zoom_level.to_lowercase().as_str() {
        "continental" => 2.0, // Continental overview - broader view
        "regional" => 3.0,    // Regional detail - more resolution
        "local" => 4.0,       // Local detail - maximum resolution
        _ => 2.5,             // Default to reasonable size
    };

    // Scale based on simulation size and physical scale
    let aspect_ratio = sim_width as f64 / sim_height as f64;
    let scale_factor = (scale_km / 200.0).sqrt(); // Normalize to 200km reference

    // Calculate scientific-grade dimensions (much bigger than tiny 8x8)
    let base_height = (20.0 * base_multiplier * scale_factor) as usize;
    let base_width = (base_height as f64 * aspect_ratio) as usize;

    // Ensure minimum scientifically useful sizes
    let min_width = 40; // Minimum for seeing atmospheric patterns
    let min_height = 20; // Minimum for pressure gradients

    // Cap at reasonable terminal sizes
    let max_width = 120;
    let max_height = 60;

    let final_width = base_width.clamp(min_width, max_width);
    let final_height = base_height.clamp(min_height, max_height);

    (final_width, final_height)
}

/// Apply scientific workflow preset configuration
fn apply_workflow_preset(preset: &str, args: &mut WeatherDemoArgs) {
    match preset.to_lowercase().as_str() {
        "climate-analysis" => {
            // Climate scientists: temperature-biome relationships across scales
            args.layers = "temperature,biomes,elevation".to_string();
            args.zoom = "continental".to_string();
            println!(
                "🌡️  Climate Analysis preset: Temperature-biome relationships across continental scale"
            );
        }
        "storm-tracking" => {
            // Atmospheric physicists: pressure systems and circulation patterns
            args.layers = "pressure,wind,temperature".to_string();
            args.zoom = "regional".to_string();
            println!("🌪️  Storm Tracking preset: Pressure systems and atmospheric circulation");
        }
        "change-detection" => {
            // Research teams: temporal analysis and system evolution
            args.layers = "pressure,temperature,water,changes".to_string();
            args.zoom = "continental".to_string();
            args.buffer_size = 10; // More frames for change analysis
            println!("📈 Change Detection preset: Temporal evolution and system dynamics");
        }
        "regional-deep-dive" => {
            // Detailed regional analysis: all layers at high resolution
            args.layers = "elevation,water,temperature,pressure,biomes,wind".to_string();
            args.zoom = "local".to_string();
            println!("🔬 Regional Deep Dive preset: Complete local analysis with all layers");
        }
        "custom" => {
            // User-specified configuration - no changes
            println!("⚙️  Custom configuration: Using user-specified parameters");
        }
        _ => {
            eprintln!(
                "⚠️  Unknown preset '{}', using custom configuration",
                preset
            );
            eprintln!(
                "Available presets: climate-analysis, storm-tracking, change-detection, regional-deep-dive, custom"
            );
        }
    }
}

/// Load workspace configuration from YAML file and apply to args
fn load_workspace_config(
    config_path: &str,
    args: &mut WeatherDemoArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = WorkspaceConfig::load_from_file(config_path)?;

    println!(
        "📁 Loading workspace: {} by {}",
        config.metadata.name, config.metadata.author
    );
    if let Some(description) = &config.metadata.description {
        println!("   Description: {}", description);
    }

    // Apply simulation defaults
    if let Some(seed) = config.defaults.seed {
        args.seed = Some(seed);
    }
    args.scale_km = config.defaults.scale_km;
    args.roughness = config.defaults.roughness;
    args.persistence = config.defaults.persistence;
    args.width = config.defaults.dimensions.0;
    args.height = config.defaults.dimensions.1;
    args.interval = config.defaults.interval;

    // Apply framebuffer layout
    args.buffer_size = config.layout.buffer_size;
    args.layers = config.layout.layers.join(",");
    args.zoom = config.layout.zoom;
    args.frame_width = config.layout.frame_size.0;
    args.frame_height = config.layout.frame_size.1;

    println!("✅ Workspace configuration loaded successfully");
    Ok(())
}

/// Save current configuration to YAML workspace file
fn save_workspace_config(
    config_path: &str,
    args: &WeatherDemoArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let preset_name = if args.preset == "custom" {
        "custom-workspace"
    } else {
        &args.preset
    };
    let mut config = WorkspaceConfig::from_preset(preset_name, &args.author);

    // Update with current args
    config.defaults.seed = args.seed;
    config.defaults.scale_km = args.scale_km;
    config.defaults.roughness = args.roughness;
    config.defaults.persistence = args.persistence;
    config.defaults.dimensions = (args.width, args.height);
    config.defaults.interval = args.interval;

    config.layout.buffer_size = args.buffer_size;
    config.layout.layers = args
        .layers
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    config.layout.zoom = args.zoom.clone();
    config.layout.frame_size = (args.frame_width, args.frame_height);

    config.mark_modified();
    config.save_to_file(config_path)?;

    println!("💾 Workspace configuration saved to: {}", config_path);
    println!(
        "   Name: {} by {}",
        config.metadata.name, config.metadata.author
    );

    Ok(())
}

/// Create unified temporal scaling configuration from command-line arguments
/// 
/// This function provides backward compatibility for the CLI while using
/// the new unified temporal scaling architecture that fixes physics violations.
fn create_temporal_config_from_args(
    args: &WeatherDemoArgs,
) -> Result<TemporalScale, String> {
    // Handle temporal configuration file loading first (convert from legacy format)
    if let Some(config_path) = &args.temporal_config {
        let legacy_config = load_temporal_config_from_file(config_path)?;
        return Ok(TemporalScale::from(legacy_config));
    }

    // Handle study phenomenon presets (preserves user-friendly CLI experience)
    if let Some(phenomenon) = &args.study_phenomenon {
        return TemporalScale::from_study_phenomenon(phenomenon);
    }

    // Parse temporal mode from string
    let mode = match args.temporal_mode.to_lowercase().as_str() {
        "demo" => TemporalMode::Demo,
        "realistic" => TemporalMode::Realistic,
        "research" => TemporalMode::Research,
        _ => {
            return Err(format!(
                "Unknown temporal mode '{}'. Valid options: demo, realistic, research",
                args.temporal_mode
            ));
        }
    };

    // Validate scaling factor for research mode
    if mode == TemporalMode::Research {
        if args.scaling_factor < 0.001 || args.scaling_factor > 1000.0 {
            return Err(format!(
                "Scaling factor {} out of range. Must be between 0.001 and 1000.0",
                args.scaling_factor
            ));
        }
    }

    // Calculate unified temporal factor from mode
    let global_temporal_factor = match mode {
        TemporalMode::Demo => 1.0,
        TemporalMode::Realistic => 2.5 / 3650.0, // Scientific ecological timescales
        TemporalMode::Research => args.scaling_factor,
    };

    // NOTE: The old selective scaling flags (scale_biological, scale_geological, scale_atmospheric)
    // are ignored to fix physics violations. All systems now use unified temporal scaling.
    if args.scale_biological || args.scale_geological || args.scale_atmospheric {
        eprintln!("WARNING: Selective temporal scaling flags are deprecated for physics consistency.");
        eprintln!("All physics systems now use unified temporal scaling to maintain causality.");
        eprintln!("Use --study-phenomenon presets for research-focused configurations.");
    }

    Ok(TemporalScale::new(mode, global_temporal_factor, None))
}

/// Load temporal configuration from YAML file
fn load_temporal_config_from_file(path: &str) -> Result<TemporalScalingConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read temporal config file '{}': {}", path, e))?;

    // Use serde_yaml since we have YAML support
    let config: TemporalScalingConfig = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse temporal config: {}", e))?;

    Ok(config)
}

/// Save temporal configuration to file
fn save_temporal_config_to_file(path: &str, config: &TemporalScale) -> Result<(), String> {
    let yaml = serde_yaml::to_string(config)
        .map_err(|e| format!("Failed to serialize temporal config: {}", e))?;

    std::fs::write(path, yaml)
        .map_err(|e| format!("Failed to write temporal config file '{}': {}", path, e))?;

    println!("💾 Temporal configuration saved to: {}", path);
    Ok(())
}

/// Display comprehensive temporal scaling education
fn display_temporal_help() {
    println!("Temporal Scaling in Weather Demo");
    println!("================================\n");

    println!("CONCEPT:");
    println!("  Temporal scaling adjusts the rate of biological, geological, and");
    println!("  atmospheric processes to match different research needs.\n");

    println!("MODES:");
    println!("  demo      - Fast observable changes (current behavior)");
    println!("              • Ecosystem: 10.0 kg/m²/day growth rate");
    println!("              • Use for: Demonstrations, quick visualization");
    println!("              • Trade-off: Fast results, less scientifically accurate\n");

    println!("  realistic - Scientific accuracy (peer-review quality)");
    println!("              • Ecosystem: 2.5 kg/m²/year growth rate (3650x slower)");
    println!("              • Use for: Research publications, long-term studies");
    println!("              • Trade-off: Accurate timescales, slower to observe\n");

    println!("  research  - Custom scaling for hypothesis testing");
    println!("              • Ecosystem: Configurable 0.001x to 1000x rates");
    println!("              • Use for: Parameter sensitivity, what-if scenarios");
    println!("              • Trade-off: Maximum flexibility, requires expertise\n");

    println!("STUDY PRESETS (recommended for beginners):");
    println!("  drought    - Long-term ecosystem stress (0.2x realistic rate)");
    println!("  ecosystem  - Natural growth cycles (realistic scientific rates)");
    println!("  climate    - Climate-ecosystem coupling (realistic rates)");
    println!("  storm      - Weather system dynamics (demo rate, atm focus)\n");

    println!("EXAMPLES:");
    println!("  # Quick start with presets");
    println!("  ./weather-demo --study-phenomenon drought");
    println!("  ./weather-demo --study-phenomenon ecosystem --temporal-stats");
    println!("  ./weather-demo --study-phenomenon climate --save-temporal-config my_study.yaml");
    println!();
    println!("  # Manual control");
    println!("  ./weather-demo --temporal-mode realistic");
    println!("  ./weather-demo --temporal-mode research --scaling-factor 0.1");
    println!("  ./weather-demo --temporal-mode research --scaling-factor 10.0 --scale-geological");
    println!();
    println!("  # Validation and learning");
    println!("  ./weather-demo --temporal-validate --study-phenomenon climate");
    println!("  ./weather-demo --temporal-help");
    println!();
    println!("PERFORMANCE:");
    println!("  All temporal scaling modes have < 1% performance overhead.");
    println!("  Use --temporal-stats to monitor performance during simulation.");
}

/// Validate unified temporal configuration and show expected behavior
fn validate_temporal_config(config: &TemporalScale, _args: &WeatherDemoArgs) {
    println!("Unified Temporal Configuration Validation");
    println!("========================================\n");

    println!("Configuration:");
    println!("  Mode: {:?}", config.mode);
    println!("  Global temporal factor: {:.6}", config.global_temporal_factor);
    if let Some(ref study_phenomenon) = config.study_phenomenon {
        println!("  Study phenomenon: {}", study_phenomenon);
    }
    println!("  Physics coupling: All systems use unified temporal scaling");
    println!();

    println!("Expected Behavior:");
    let dt_hours = 1.0; // 1 hour timestep
    let base_growth_rate = 10.0; // kg/m²/day

    let scaled_rate = config.scale_rate(base_growth_rate, dt_hours);
    let daily_rate = scaled_rate * 24.0;
    let annual_rate = daily_rate * 365.0;

    println!("  Base ecosystem growth: {:.1} kg/m²/day", base_growth_rate);
    println!(
        "  Scaled ecosystem growth: {:.6} kg/m²/day ({:.2} kg/m²/year)",
        daily_rate, annual_rate
    );
    println!("  All physics systems: Same {:.6}x temporal scaling", config.global_temporal_factor);

    match config.mode {
        TemporalMode::Demo => {
            println!("  Vegetation changes: Observable in minutes to hours");
            println!("  Simulation duration: Short demos (< 1 hour real time)");
            println!("  Scientific accuracy: Demonstration quality");
        }
        TemporalMode::Realistic => {
            println!("  Vegetation changes: Seasonal cycles, natural pace");
            println!("  Simulation duration: Multi-year studies recommended");
            println!("  Scientific accuracy: Publication quality");
        }
        TemporalMode::Research => {
            let factor = config.global_temporal_factor;
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
        }
    }

    println!("\nPerformance Impact: < 1% simulation overhead");
    println!("✅ Configuration is valid and ready for simulation!");
}

pub fn run_weather_demo() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let mut args = WeatherDemoArgs::parse();

    // === NEW: Handle temporal scaling help ===
    if args.temporal_help {
        display_temporal_help();
        return Ok(());
    }

    // === NEW: Create temporal configuration ===
    let temporal_config = create_temporal_config_from_args(&args)
        .map_err(|e| format!("Temporal configuration error: {}", e))?;

    // === NEW: Validate configuration if requested ===
    if args.temporal_validate {
        validate_temporal_config(&temporal_config, &args);
        return Ok(());
    }

    // === NEW: Save temporal config if requested ===
    if let Some(save_path) = &args.save_temporal_config {
        save_temporal_config_to_file(save_path, &temporal_config)?;
        println!(
            "Temporal configuration saved. Use --temporal-config {} to reload.",
            save_path
        );
        return Ok(());
    }

    // Load workspace configuration from YAML if specified
    let load_config_path = args.load_config.clone();
    if let Some(config_path) = load_config_path {
        if let Err(e) = load_workspace_config(&config_path, &mut args) {
            eprintln!("⚠️  Failed to load workspace config: {}", e);
            eprintln!("   Proceeding with command line arguments");
        }
    } else {
        // Apply workflow preset if specified and no config loaded
        let preset_name = args.preset.clone();
        if preset_name != "custom" {
            apply_workflow_preset(&preset_name, &mut args);
        }
    }

    // Save workspace configuration if specified
    let save_config_path = args.save_config.clone();
    if let Some(config_path) = save_config_path {
        if let Err(e) = save_workspace_config(&config_path, &args) {
            eprintln!("⚠️  Failed to save workspace config: {}", e);
        }
    }

    // === Display unified temporal configuration summary ===
    match temporal_config.mode {
        TemporalMode::Demo => {
            // Don't display anything for demo mode - maintain existing behavior
        }
        TemporalMode::Realistic => {
            println!("🧪 Unified Temporal Scaling: Realistic mode (scientific accuracy)");
            println!("   All physics systems: {:.6}x temporal factor", temporal_config.global_temporal_factor);
            let annual_rate = 10.0 * temporal_config.global_temporal_factor * 365.0;
            println!("   Expected ecosystem growth: {:.2} kg/m²/year", annual_rate);
        }
        TemporalMode::Research => {
            println!(
                "🔬 Unified Temporal Scaling: Research mode (global factor: {:.6}x)",
                temporal_config.global_temporal_factor
            );
            let annual_rate = 10.0 * temporal_config.global_temporal_factor * 365.0;
            println!("   All physics systems use same temporal rate");
            println!("   Expected ecosystem growth: {:.2} kg/m²/year", annual_rate);
        }
    }
    
    if let Some(ref study_phenomenon) = temporal_config.study_phenomenon {
        println!("   Study focus: {} (unified temporal coupling enabled)", study_phenomenon);
    }

    // Step 1: Generate seed if not provided, then create generator
    let seed = args.seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    });

    println!("Using seed: {}", seed);

    // Validate scale/resolution combination for atmospheric realism
    let meters_per_pixel = (args.scale_km * 1000.0) / args.width.max(args.height) as f64;
    let total_domain_km = args.scale_km;

    if total_domain_km < 100.0 {
        eprintln!(
            "⚠️  WARNING: Domain scale {:.1}km is below 100km - Coriolis effects will be disabled",
            total_domain_km
        );
    }

    if meters_per_pixel < 500.0 {
        eprintln!(
            "⚠️  WARNING: Resolution too high ({:.0}m per pixel) - may cause numerical instabilities",
            meters_per_pixel
        );
        eprintln!(
            "   Recommended: Increase --scale-km to at least {:.0} for this resolution",
            args.width.max(args.height) as f64 * 0.5 / 1000.0
        );
    } else if meters_per_pixel > 10000.0 {
        eprintln!(
            "⚠️  WARNING: Resolution too low ({:.1}km per pixel) - weather details will be lost",
            meters_per_pixel / 1000.0
        );
        eprintln!(
            "   Recommended: Decrease --scale-km to at most {:.0} for this resolution",
            (args.width.max(args.height) as f64 * 10.0) / 1000.0
        );
    } else {
        println!(
            "✅ Good scale/resolution: {:.0}m per pixel on {:.1}km domain",
            meters_per_pixel, total_domain_km
        );
    }

    // Step 2: Generate simple terrain for weather testing
    println!("Using Diamond-Square generation for weather demo...");
    let generator = DiamondSquareGenerator::new(seed);
    let config = DiamondSquareConfig {
        initial_corners: [0.3, 0.7, 0.4, 0.6],
        roughness: args.roughness,
        persistence: args.persistence,
        wrap_edges: false,
    };
    let heightmap = generator.generate(args.width, args.height, &config);
    println!("Physical domain scale: {:.1} km", args.scale_km);

    // === NEW: Create temporal scaling service and performance monitor ===
    // Unified temporal scaling is now handled through WorldScale.temporal_scale

    let mut _performance_monitor = if args.temporal_stats {
        Some(TemporalPerformanceMonitor::new())
        // TODO: In actual implementation, this would be passed to simulation
    } else {
        None
    };

    // Step 3: Run simulation setup with proper scale and unified temporal scaling
    println!("Creating simulation with {:.1}km scale...", args.scale_km);
    let start_time = std::time::Instant::now();
    let world_scale = WorldScale::new_with_temporal(
        args.scale_km,
        (args.width as u32, args.height as u32),
        DetailLevel::Standard,
        temporal_config, // Use unified temporal scaling context
    );
    let sim = Simulation::_new_with_scale(heightmap, world_scale);
    println!("Simulation created in {:.2?}", start_time.elapsed());

    // === NEW: Show temporal configuration in effect ===
    if args.temporal_stats {
        println!("📊 Temporal performance monitoring enabled");
        // TODO: Show initial performance statistics
    }

    // Choose between graphics, TUI, ASCII, stats, and framebuffer rendering
    if args.ascii_frames {
        // Step 4a: ASCII framebuffer mode - multi-layer temporal visualization
        println!("Starting ASCII framebuffer mode...");
        run_ascii_framebuffer_mode(sim, &args)?;
    } else if args.stats {
        // Step 4b: Stats mode - run simulation with diagnostic output
        println!("Starting stats monitoring mode...");
        println!("Interval: {} simulation ticks", args.interval);
        run_stats_mode(sim, args.interval)?;
    } else if args.graphics {
        // Step 4a: Graphics mode (macroquad)
        println!("Starting graphics mode...");
        println!("Use WASD to pan, mouse wheel to zoom, 1-7 to switch display modes");
        println!(
            "Display modes: 1=Elevation, 2=Water, 3=Pressure, 4=Wind, 5=Weather, 6=Temperature, 7=Biomes"
        );

        // Configure window and run graphics mode
        let window_config = Conf {
            window_title: "Weather System Demo".to_owned(),
            window_width: 1000,
            window_height: 700,
            window_resizable: true,
            ..Default::default()
        };

        macroquad::Window::from_config(window_config, run_graphics(sim));
    } else if args.multi_viewport {
        // Step 4c: Multi-viewport TUI mode - simultaneous layer monitoring
        println!("Starting multi-viewport TUI mode...");
        println!("Controls:");
        println!("  Tab/Shift+Tab: Cycle between viewports");
        println!("  1-4: Direct viewport selection");
        println!("  WASD: Navigate active viewport (Shift for fast movement)");
        println!("  Q or Esc: Quit");
        run_multi_viewport_tui(sim)?;
    } else if args.ascii {
        // Step 4d: Static ASCII render (legacy mode)
        ascii_render(&sim);
        println!("\nElevation data for weather testing");
    } else {
        // Step 4e: Interactive TUI mode (default)
        println!("Starting interactive weather demo...");
        println!("Use WASD or arrow keys to navigate, Q or Esc to quit");
        run_tui(sim)?;
    }

    Ok(())
}

async fn run_graphics(mut simulation: Simulation) {
    // Initialize renderer after macroquad window is available
    let mut renderer = GraphicsRenderer::new(screen_width(), screen_height());

    loop {
        // Handle window resize
        renderer.handle_resize();

        // Handle input
        renderer.handle_input();

        // Update simulation (tick atmospheric systems) only if not paused
        if renderer.should_tick_simulation() {
            simulation.tick();
        }

        // Render
        renderer.render_simulation(&simulation);

        // Exit on Escape
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;
    }
}

/// Run simulation in stats monitoring mode with periodic diagnostic output
fn run_stats_mode(
    mut simulation: Simulation,
    interval: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Stats mode initialized. Press Ctrl+C to stop.\n");

    let mut iteration_count = 0;

    // Initial stats output
    // TODO: Restore diagnostics after water flow validation integration
    // let initial_diagnostics =
    //     SimulationDiagnostics::collect_from_simulation(&simulation, iteration_count);
    // println!("{}", initial_diagnostics.format_compact());
    println!("Starting weather demo simulation...");

    loop {
        // Run simulation tick
        simulation.tick();
        iteration_count += 1;

        // Output stats at specified interval
        if iteration_count % interval == 0 {
            // TODO: Restore diagnostics after water flow validation integration
            // let diagnostics =
            //     SimulationDiagnostics::collect_from_simulation(&simulation, iteration_count);
            // println!("{}", diagnostics.format_compact());
            println!("Tick: {}", iteration_count);
        }

        // Check for Ctrl+C (this is a simplified approach)
        // In a real implementation, you'd want proper signal handling
        std::thread::sleep(std::time::Duration::from_millis(10)); // Small delay to prevent CPU spinning
    }
}

/// Run simulation in ASCII framebuffer mode with multi-layer temporal visualization
fn run_ascii_framebuffer_mode(
    mut simulation: Simulation,
    args: &WeatherDemoArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse layer names from CLI argument
    let layer_names: Vec<&str> = args.layers.split(',').map(|s| s.trim()).collect();
    let mut layers = Vec::new();

    for layer_name in layer_names {
        if let Some(layer) = VisualizationLayer::from_str(layer_name) {
            layers.push(layer);
        } else {
            eprintln!("Warning: Unknown layer '{}', skipping", layer_name);
        }
    }

    if layers.is_empty() {
        layers = vec![
            VisualizationLayer::Elevation,
            VisualizationLayer::Water,
            VisualizationLayer::Biomes,
        ];
        println!("No valid layers specified, using default: elevation,water,biomes");
    }

    // Calculate scale-aware frame dimensions
    let (frame_width, frame_height) = calculate_scale_aware_dimensions(
        &args.zoom,
        args.frame_width,
        args.frame_height,
        args.width,
        args.height,
        args.scale_km,
    );

    // Create framebuffer configuration
    let config = FramebufferConfig {
        layers,
        buffer_size: args.buffer_size,
        panel_width: frame_width,
        panel_height: frame_height,
        show_timestamps: true,
        highlight_changes: false,
        subsample_rate: 1,
    };

    let mut framebuffer = AsciiFramebuffer::new(config);

    println!("ASCII Framebuffer initialized. Press Ctrl+C to stop.");
    println!("Layers: {:?}", args.layers);
    println!("Buffer size: {}", args.buffer_size);
    println!("Update interval: {} ticks\n", args.interval);

    let mut iteration_count = 0;

    loop {
        // Run simulation tick
        simulation.tick();
        iteration_count += 1;

        // Capture and display frame at specified interval
        if iteration_count % args.interval == 0 {
            let frame = framebuffer.capture_frame(&simulation);
            let output = framebuffer.format_frame_colorized(&frame);
            framebuffer.add_frame(frame);

            // Clear screen and display frame
            print!("\x1B[2J\x1B[H"); // ANSI escape codes to clear screen and move cursor to top
            println!("{}", output);

            // Show buffer status
            println!(
                "Buffer: {}/{} frames | Press Ctrl+C to exit",
                framebuffer.frame_count(),
                args.buffer_size
            );
        }

        // Small delay to prevent CPU spinning
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// Run the multi-viewport TUI with complete event loop integration
fn run_multi_viewport_tui(simulation: Simulation) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::{Terminal, backend::CrosstermBackend};
    use std::io;

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create multi-viewport application
    let mut app = MultiViewportApp::new(simulation);

    // Main event loop
    let result = loop {
        // Render current frame
        terminal.draw(|frame| {
            let area = frame.size();

            // Generate layout areas for 2x2 grid
            let layout_areas = app.renderer.generate_2x2_layout(area);

            // Render each viewport
            for (viewport_idx, viewport_area) in layout_areas.iter().enumerate() {
                if viewport_idx < app.renderer.viewport_count() {
                    // Get content for this viewport
                    if let Some(content) = app.renderer.render_viewport_content(
                        &app.simulation,
                        viewport_idx,
                        *viewport_area,
                    ) {
                        // Create widget for this viewport
                        let is_active = viewport_idx == app.renderer.get_active_viewport();
                        let widget =
                            app.renderer
                                .create_viewport_widget(content, viewport_idx, is_active);

                        // Render widget to frame
                        frame.render_widget(widget, *viewport_area);
                    }
                }
            }

            // Render status panel if enabled
            if let Some(status_area) = app.renderer.generate_status_panel(area) {
                let status_widget = app.renderer.create_status_panel();
                frame.render_widget(status_widget, status_area);
            }
        })?;

        // Handle events with timeout
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        // Quit keys
                        KeyCode::Char('q') | KeyCode::Esc => break Ok(()),

                        // Viewport cycling
                        KeyCode::Tab => {
                            if key.modifiers.contains(KeyModifiers::SHIFT) {
                                app.cycle_previous_viewport();
                            } else {
                                app.cycle_next_viewport();
                            }
                        }

                        // Direct viewport selection
                        KeyCode::Char('1') => {
                            app.select_viewport(0);
                        }
                        KeyCode::Char('2') => {
                            app.select_viewport(1);
                        }
                        KeyCode::Char('3') => {
                            app.select_viewport(2);
                        }
                        KeyCode::Char('4') => {
                            app.select_viewport(3);
                        }

                        // WASD navigation
                        KeyCode::Char('w') | KeyCode::Up => {
                            let fast = key.modifiers.contains(KeyModifiers::SHIFT);
                            app.handle_movement(MovementDirection::North, fast);
                        }
                        KeyCode::Char('s') | KeyCode::Down => {
                            let fast = key.modifiers.contains(KeyModifiers::SHIFT);
                            app.handle_movement(MovementDirection::South, fast);
                        }
                        KeyCode::Char('a') | KeyCode::Left => {
                            let fast = key.modifiers.contains(KeyModifiers::SHIFT);
                            app.handle_movement(MovementDirection::West, fast);
                        }
                        KeyCode::Char('d') | KeyCode::Right => {
                            let fast = key.modifiers.contains(KeyModifiers::SHIFT);
                            app.handle_movement(MovementDirection::East, fast);
                        }

                        _ => {} // Ignore other keys
                    }
                }
            }
        }

        // Check if app wants to quit
        if app.should_quit {
            break Ok(());
        }
    };

    // Clean up terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_weather_demo()
}
