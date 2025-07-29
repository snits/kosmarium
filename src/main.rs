// sim-prototype/src/main.rs

mod render;
mod sim;
mod tui;
mod worldgen;

use clap::Parser;
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Choose between TUI and ASCII rendering
    if args.ascii {
        // Step 4a: Static ASCII render (legacy mode)
        ascii_render(&sim);

        // Debug info
        println!("\nGenerated using: {}", generator.name());
        println!(
            "Supports arbitrary dimensions: {}",
            generator.supports_arbitrary_dimensions()
        );
    } else {
        // Step 4b: Interactive TUI mode (default)
        println!("Starting interactive terrain explorer...");
        println!("Use WASD or arrow keys to navigate, Q or Esc to quit");
        run_tui(sim)?;
    }

    Ok(())
}
