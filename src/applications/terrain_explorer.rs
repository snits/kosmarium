// ABOUTME: Terrain Explorer application - interactive terrain generation and visualization
// ABOUTME: Demonstrates engine usage for terrain exploration with multiple rendering modes

use clap::Parser;
use macroquad::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

// Import engine components
use crate::engine::{
    Simulation,
    physics::{
        DiamondSquareConfig, DiamondSquareGenerator, TectonicConfig, TectonicGenerator,
        TerrainGenerator,
    },
    rendering::{GraphicsRenderer, ascii_render, ascii_render_biomes, run_tui},
};

#[derive(Parser)]
#[command(name = "terrain-explorer")]
#[command(about = "Interactive terrain generation and visualization")]
pub struct TerrainExplorerArgs {
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

    /// Use tectonic plate generation instead of Diamond-Square
    #[arg(long)]
    pub tectonic: bool,

    /// Show biome classification instead of elevation (ASCII mode only)
    #[arg(long)]
    pub biomes: bool,
}

pub fn run_terrain_explorer() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = TerrainExplorerArgs::parse();

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
            surface_detail: 0.6,
            mountain_scale: 1.0,
            ocean_depth_scale: 1.0,
            continental_roughness: 0.7,
            oceanic_roughness: 0.3,
            detail_persistence: 0.5,
            tectonic_influence: 0.7,
            coastal_blending: 15.0,
            enable_geological_evolution: true,
            geological_evolution_config: Some(crate::engine::physics::GeologicalEvolutionConfig {
                evolution_iterations: 1000,
                progress_interval: 100,
                verbose_logging: true,
                ..Default::default()
            }),
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
            initial_corners: [0.3, 0.7, 0.4, 0.6],
            roughness: args.roughness,
            persistence: args.persistence,
            wrap_edges: false,
        };
        let heightmap = generator.generate(args.width, args.height, &config);
        (
            heightmap,
            generator.name(),
            generator.supports_arbitrary_dimensions(),
        )
    };

    // Step 3: Run simulation setup
    println!("Creating simulation...");
    let start_time = std::time::Instant::now();
    let mut sim = Simulation::new(heightmap);
    println!("Simulation created in {:.2?}", start_time.elapsed());

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
        if args.biomes {
            // Show biome classification
            let biome_map = sim.generate_biome_map();
            ascii_render_biomes(&biome_map);
            println!("\nBiome classification using Whittaker model");
        } else {
            // Show elevation data (default)
            ascii_render(&sim);
        }

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
        renderer.render_simulation(&mut simulation);

        // Exit on Escape
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;
    }
}
