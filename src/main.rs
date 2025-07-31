// sim-prototype/src/main.rs

pub mod atmosphere;
pub mod climate;
pub mod convergence;
pub mod dimensional;
pub mod geological_evolution;
pub mod graphics_render;
pub mod render;
pub mod scale;
pub mod sim;
pub mod tectonics;
pub mod tui;
pub mod worldgen;

use clap::Parser;
use graphics_render::GraphicsRenderer;
use macroquad::prelude::*;
use render::ascii_render;
use sim::Simulation;
use std::time::{SystemTime, UNIX_EPOCH};
use tui::run_tui;
use worldgen::{
    DiamondSquareConfig, DiamondSquareGenerator, TectonicConfig, TectonicGenerator,
    TerrainGenerator,
};

#[derive(Parser)]
#[command(name = "sim-prototype")]
#[command(about = "A terrain generation and simulation prototype")]
struct Args {
    /// Random seed for terrain generation (defaults to current time)
    #[arg(short, long)]
    seed: Option<u64>,

    /// Terrain roughness (0.0 - 1.0, higher = more chaotic)
    #[arg(short, long, default_value = "0.7")]
    roughness: f32,

    /// Detail persistence across scales (0.0 - 1.0)
    #[arg(short, long, default_value = "0.6")]
    persistence: f32,

    /// Map width in cells
    #[arg(short = 'W', long, default_value = "240")]
    width: usize,

    /// Map height in cells
    #[arg(short = 'H', long, default_value = "120")]
    height: usize,

    /// Use ASCII mode instead of TUI
    #[arg(long)]
    ascii: bool,

    /// Use graphics mode (macroquad) instead of TUI
    #[arg(long)]
    graphics: bool,

    /// Use tectonic plate generation instead of Diamond-Square
    #[arg(long)]
    tectonic: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Step 1: Generate seed if not provided, then create generator
    let seed = args.seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    println!("Using seed: {}", seed);

    // Step 2: Generate map with specified dimensions
    let (heightmap, generator_name, supports_arbitrary) = if args.tectonic {
        println!("Using tectonic plate generation...");
        let generator = TectonicGenerator::new(seed);
        let config = TectonicConfig {
            num_plates: 8,
            surface_detail: 0.6, // Increased for more realistic detail
            mountain_scale: 1.0,
            ocean_depth_scale: 1.0,
            continental_roughness: 0.7, // Higher roughness for varied continental terrain
            oceanic_roughness: 0.3,     // Lower roughness for smoother ocean floors
            detail_persistence: 0.5,    // Standard fractal persistence
            tectonic_influence: 0.7,    // Strong tectonic foundation with fractal detail
            coastal_blending: 15.0,     // Blend detail types over 15 pixels

            // Enable geological evolution for testing
            enable_geological_evolution: true,
            geological_evolution_config: Some(
                crate::geological_evolution::GeologicalEvolutionConfig {
                    evolution_iterations: 1000, // Shorter for testing
                    progress_interval: 100,     // Report every 100 iterations
                    verbose_logging: true,      // Show detailed output
                    ..Default::default()
                },
            ),
        };
        let heightmap = generator.generate(args.width, args.height, &config);
        (
            heightmap,
            generator.name(),
            generator.supports_arbitrary_dimensions(),
        )
    } else {
        println!("Using Diamond-Square generation...");
        let generator = DiamondSquareGenerator::new(seed);
        let config = DiamondSquareConfig {
            initial_corners: [0.3, 0.7, 0.4, 0.6], // Varied starting elevations
            roughness: args.roughness,
            persistence: args.persistence,
            wrap_edges: false, // No wrapping
        };
        let heightmap = generator.generate(args.width, args.height, &config);
        (
            heightmap,
            generator.name(),
            generator.supports_arbitrary_dimensions(),
        )
    };

    // Step 3: Run simulation setup (placeholder for now)
    let sim = Simulation::new(heightmap);

    // Choose between graphics, TUI, and ASCII rendering
    if args.graphics {
        // Step 4a: Graphics mode (macroquad)
        println!("Starting graphics mode...");
        println!("Use WASD to pan, mouse wheel to zoom, 1-6 to switch display modes");

        // Configure window and run graphics mode
        let window_config = Conf {
            window_title: "Terrain Simulation".to_owned(),
            window_width: 800,
            window_height: 600,
            window_resizable: true,
            ..Default::default()
        };

        macroquad::Window::from_config(window_config, run_graphics(sim));
    } else if args.ascii {
        // Step 4b: Static ASCII render (legacy mode)
        ascii_render(&sim);

        // Debug info
        println!("\nGenerated using: {}", generator_name);
        println!("Supports arbitrary dimensions: {}", supports_arbitrary);
    } else {
        // Step 4c: Interactive TUI mode (default)
        println!("Starting interactive terrain explorer...");
        println!("Use WASD or arrow keys to navigate, Q or Esc to quit");
        run_tui(sim)?;
    }

    Ok(())
}

async fn run_graphics(mut simulation: Simulation) {
    // Initialize renderer after macroquad window is available
    // This ensures screen_width() and screen_height() work properly
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
