// ABOUTME: Weather Demo application - atmospheric dynamics and weather pattern visualization
// ABOUTME: Demonstrates engine weather systems with Coriolis effects and geostrophic winds

use clap::Parser;
use macroquad::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

// Import engine components
use crate::engine::{
    Simulation,
    core::{DetailLevel, WorldScale},
    physics::{DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator},
    rendering::{GraphicsRenderer, ascii_render, run_tui},
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

    /// Physical scale of the domain in kilometers
    #[arg(long, default_value = "200.0")]
    pub scale_km: f64,
}

pub fn run_weather_demo() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = WeatherDemoArgs::parse();

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

    // Choose between graphics, TUI, and ASCII rendering
    if args.graphics {
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
    } else if args.ascii {
        // Step 4b: Static ASCII render (legacy mode)
        ascii_render(&sim);
        println!("\nElevation data for weather testing");
    } else {
        // Step 4c: Interactive TUI mode (default)
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
