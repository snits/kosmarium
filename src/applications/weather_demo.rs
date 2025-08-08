// ABOUTME: Weather Demo application - atmospheric dynamics and weather pattern visualization
// ABOUTME: Demonstrates engine weather systems with Coriolis effects and geostrophic winds

use clap::Parser;
use macroquad::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

// Import engine components
use crate::engine::{
    Simulation, SimulationDiagnostics, WorkspaceConfig,
    core::{DetailLevel, WorldScale},
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

pub fn run_weather_demo() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let mut args = WeatherDemoArgs::parse();

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

    // Step 3: Run simulation setup with proper scale
    println!("Creating simulation with {:.1}km scale...", args.scale_km);
    let start_time = std::time::Instant::now();
    let world_scale = WorldScale::new(
        args.scale_km,
        (args.width as u32, args.height as u32),
        DetailLevel::Standard,
    );
    let sim = Simulation::_new_with_scale(heightmap, world_scale);
    println!("Simulation created in {:.2?}", start_time.elapsed());

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
    let initial_diagnostics =
        SimulationDiagnostics::collect_from_simulation(&simulation, iteration_count);
    println!("{}", initial_diagnostics.format_compact());

    loop {
        // Run simulation tick
        simulation.tick();
        iteration_count += 1;

        // Output stats at specified interval
        if iteration_count % interval == 0 {
            let diagnostics =
                SimulationDiagnostics::collect_from_simulation(&simulation, iteration_count);
            println!("{}", diagnostics.format_compact());
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
            let output = framebuffer.format_frame(&frame);
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
                    if let Some(content) = app
                        .renderer
                        .render_viewport_content(&app.simulation, viewport_idx)
                    {
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
