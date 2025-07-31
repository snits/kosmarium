// sim-prototype/src/main.rs

pub mod atmosphere;
pub mod climate;
pub mod convergence;
pub mod dimensional;
pub mod graphics_render;
pub mod render;
pub mod scale;
pub mod sim;
pub mod tui;
pub mod worldgen;

use clap::Parser;
use graphics_render::GraphicsRenderer;
use macroquad::prelude::*;
use render::ascii_render;
use sim::Simulation;
use tui::run_tui;
use worldgen::{DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator};

#[derive(Parser)]
#[command(name = "sim-prototype")]
#[command(about = "A terrain generation and simulation prototype")]
struct Args {
    /// Random seed for terrain generation
    #[arg(short, long, default_value = "12345")]
    seed: u64,

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
}

#[macroquad::main("Atmospheric Simulation")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Step 1: Create generator and configuration from arguments
    let generator = DiamondSquareGenerator::new(args.seed);
    let config = DiamondSquareConfig {
        initial_corners: [0.3, 0.7, 0.4, 0.6], // Varied starting elevations
        roughness: args.roughness,
        persistence: args.persistence,
        wrap_edges: false, // No wrapping
    };

    // Step 2: Generate map with specified dimensions
    let heightmap = generator.generate(args.width, args.height, &config);

    // Step 3: Run simulation setup (placeholder for now)
    let sim = Simulation::new(heightmap);

    // Choose between graphics, TUI, and ASCII rendering
    if args.graphics {
        // Step 4a: Graphics mode (macroquad)
        println!("Starting graphics mode...");
        println!("Use WASD to pan, mouse wheel to zoom, 1-6 to switch display modes");
        run_graphics(sim).await;
    } else if args.ascii {
        // Step 4b: Static ASCII render (legacy mode)
        ascii_render(&sim);

        // Debug info
        println!("\nGenerated using: {}", generator.name());
        println!(
            "Supports arbitrary dimensions: {}",
            generator.supports_arbitrary_dimensions()
        );
    } else {
        // Step 4c: Interactive TUI mode (default)
        println!("Starting interactive terrain explorer...");
        println!("Use WASD or arrow keys to navigate, Q or Esc to quit");
        run_tui(sim)?;
    }

    Ok(())
}

async fn run_graphics(mut simulation: Simulation) {
    let mut renderer = GraphicsRenderer::new(screen_width(), screen_height());

    loop {
        // Handle input
        renderer.handle_input();

        // Update simulation (tick atmospheric systems)
        simulation.tick();

        // Render
        renderer.render_simulation(&simulation);

        // Exit on Escape
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;
    }
}
