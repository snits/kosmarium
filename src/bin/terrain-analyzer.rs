// ABOUTME: Headless terrain analysis tool for batch testing terrain generation and atmospheric systems
// ABOUTME: Generates multiple worlds with different seeds and provides statistical analysis of results

use clap::Parser;
use sim_protoype::engine::agents::biome::{BiomeClassifier, BiomeType};
use sim_protoype::engine::physics::worldgen::{
    DiamondSquareConfig, DiamondSquareGenerator, TectonicConfig, TectonicGenerator,
    TerrainGenerator,
};
use sim_protoype::engine::sim::Simulation;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "terrain-analyzer")]
#[command(about = "Analyze terrain generation patterns and atmospheric systems")]
struct Args {
    /// Number of seeds to test
    #[arg(short, long, default_value = "20")]
    seeds: usize,

    /// Use tectonic generation instead of Diamond-Square
    #[arg(long)]
    tectonic: bool,

    /// Map width
    #[arg(short = 'W', long, default_value = "240")]
    width: usize,

    /// Map height  
    #[arg(short = 'H', long, default_value = "120")]
    height: usize,

    /// Show detailed analysis for each seed
    #[arg(long)]
    verbose: bool,

    /// Quiet mode - minimal output, just recommendations
    #[arg(short, long)]
    quiet: bool,

    /// Only show seeds that meet continental criteria
    #[arg(long)]
    continental_only: bool,

    /// Analyze atmospheric systems (pressure variation)
    #[arg(long)]
    atmosphere: bool,
}

#[derive(Debug, Clone)]
struct TerrainAnalysis {
    seed: u64,
    land_percentage: f32,
    water_percentage: f32,
    mountain_percentage: f32,
    elevation_variance: f32,
    biome_distribution: [u32; 14],
    terrain_type: TerrainType,
    pressure_variance: Option<f32>,
    has_pressure_systems: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TerrainType {
    Oceanic,     // < 25% land
    Island,      // 25-45% land
    Continental, // 45-75% land
    Mountainous, // > 15% mountains
}

impl TerrainType {
    fn classify(land_pct: f32, mountain_pct: f32) -> Self {
        if mountain_pct > 15.0 {
            TerrainType::Mountainous
        } else if land_pct < 25.0 {
            TerrainType::Oceanic
        } else if land_pct < 45.0 {
            TerrainType::Island
        } else {
            TerrainType::Continental
        }
    }

    fn symbol(&self) -> &'static str {
        match self {
            TerrainType::Oceanic => "🌊",
            TerrainType::Island => "🏝️",
            TerrainType::Continental => "🏔️",
            TerrainType::Mountainous => "⛰️",
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🔍 Terrain Analysis Tool");
    println!(
        "Analyzing {} seeds using {} generation",
        args.seeds,
        if args.tectonic {
            "tectonic"
        } else {
            "Diamond-Square"
        }
    );
    println!("Map size: {}x{}", args.width, args.height);
    if args.atmosphere {
        println!("Including atmospheric analysis");
    }
    println!();

    let mut analyses = Vec::new();
    let base_seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for i in 0..args.seeds {
        let seed = base_seed + i as u64;

        if !args.quiet {
            if args.verbose {
                print!("Analyzing seed {}: ", seed);
            } else {
                print!(".");
                if (i + 1) % 50 == 0 {
                    println!(" {}/{}", i + 1, args.seeds);
                }
            }
        }

        let analysis = analyze_seed(seed, &args)?;

        if !args.quiet && args.verbose {
            println!(
                "{} {}% land, {}% mountains",
                analysis.terrain_type.symbol(),
                analysis.land_percentage as u32,
                analysis.mountain_percentage as u32
            );
        }

        // Filter results if requested
        if args.continental_only {
            if matches!(
                analysis.terrain_type,
                TerrainType::Continental | TerrainType::Mountainous
            ) {
                analyses.push(analysis);
            }
        } else {
            analyses.push(analysis);
        }
    }

    if !args.quiet && !args.verbose {
        println!(); // New line after progress dots
    }

    if !args.quiet {
        print_summary(&analyses, &args);
    }
    print_recommendations(&analyses, &args);

    Ok(())
}

fn analyze_seed(seed: u64, args: &Args) -> Result<TerrainAnalysis, Box<dyn std::error::Error>> {
    // Generate terrain
    let heightmap = if args.tectonic {
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
            enable_geological_evolution: false, // Skip for speed
            geological_evolution_config: None,
        };
        generator.generate(args.width, args.height, &config)
    } else {
        let generator = DiamondSquareGenerator::new(seed);
        let config = DiamondSquareConfig {
            initial_corners: [0.3, 0.7, 0.4, 0.6],
            roughness: 0.7,
            persistence: 0.6,
            wrap_edges: false,
        };
        generator.generate(args.width, args.height, &config)
    };

    // Analyze elevation distribution
    let total_cells = (args.width * args.height) as f32;
    let mut land_cells = 0;
    let mut water_cells = 0;
    let mut mountain_cells = 0;
    let mut elevation_sum = 0.0;
    let mut elevation_sq_sum = 0.0;

    for y in 0..args.height {
        for x in 0..args.width {
            let elevation = heightmap.get(x, y);
            elevation_sum += elevation;
            elevation_sq_sum += elevation * elevation;

            if elevation < 0.2 {
                water_cells += 1;
            } else {
                land_cells += 1;
                if elevation > 0.8 {
                    mountain_cells += 1;
                }
            }
        }
    }

    let land_percentage = (land_cells as f32 / total_cells) * 100.0;
    let water_percentage = (water_cells as f32 / total_cells) * 100.0;
    let mountain_percentage = (mountain_cells as f32 / total_cells) * 100.0;

    // Calculate elevation variance
    let mean_elevation = elevation_sum / total_cells;
    let elevation_variance = (elevation_sq_sum / total_cells) - (mean_elevation * mean_elevation);

    // Classify terrain type
    let terrain_type = TerrainType::classify(land_percentage, mountain_percentage);

    // Generate biome map and analyze
    let mut sim = Simulation::new(heightmap);
    let biome_map = sim.generate_biome_map();
    let biome_distribution = biome_map.biome_distribution();

    // Atmospheric analysis if requested
    let (pressure_variance, has_pressure_systems) = if args.atmosphere {
        analyze_atmospheric_systems(&sim)
    } else {
        (None, None)
    };

    Ok(TerrainAnalysis {
        seed,
        land_percentage,
        water_percentage,
        mountain_percentage,
        elevation_variance,
        biome_distribution,
        terrain_type,
        pressure_variance,
        has_pressure_systems,
    })
}

fn analyze_atmospheric_systems(sim: &Simulation) -> (Option<f32>, Option<bool>) {
    // Get pressure layer and analyze variation
    let pressure_layer = sim.get_atmospheric_pressure_layer();
    let width = pressure_layer.width();
    let height = pressure_layer.height();

    let mut pressure_sum = 0.0;
    let mut pressure_sq_sum = 0.0;
    let mut min_pressure = f32::INFINITY;
    let mut max_pressure = f32::NEG_INFINITY;
    let total_cells = (width * height) as f32;

    for y in 0..height {
        for x in 0..width {
            let pressure = pressure_layer.get_pressure(x, y);
            pressure_sum += pressure;
            pressure_sq_sum += pressure * pressure;
            min_pressure = min_pressure.min(pressure);
            max_pressure = max_pressure.max(pressure);
        }
    }

    let mean_pressure = pressure_sum / total_cells;
    let pressure_variance = (pressure_sq_sum / total_cells) - (mean_pressure * mean_pressure);
    let pressure_range = max_pressure - min_pressure;

    // Check if we have meaningful pressure systems
    // If variance is very low or range is tiny, pressure field is essentially uniform
    let has_systems = pressure_variance > 0.01 && pressure_range > 0.1;

    (Some(pressure_variance), Some(has_systems))
}

fn print_summary(analyses: &[TerrainAnalysis], args: &Args) {
    if args.quiet {
        return;
    }

    println!("\n📊 TERRAIN ANALYSIS SUMMARY");
    println!("{}", "=".repeat(50));

    // Count terrain types
    let mut type_counts = HashMap::new();
    for analysis in analyses {
        *type_counts.entry(&analysis.terrain_type).or_insert(0) += 1;
    }

    println!("Terrain Type Distribution:");
    for (terrain_type, count) in &type_counts {
        let percentage = (*count as f32 / analyses.len() as f32) * 100.0;
        println!(
            "  {} {:<12} {:3} ({:4.1}%)",
            terrain_type.symbol(),
            format!("{:?}", terrain_type),
            count,
            percentage
        );
    }

    // Average statistics
    let avg_land = analyses.iter().map(|a| a.land_percentage).sum::<f32>() / analyses.len() as f32;
    let avg_mountains =
        analyses.iter().map(|a| a.mountain_percentage).sum::<f32>() / analyses.len() as f32;
    let avg_variance =
        analyses.iter().map(|a| a.elevation_variance).sum::<f32>() / analyses.len() as f32;

    println!("\nAverage Statistics:");
    println!("  Land Coverage:      {:.1}%", avg_land);
    println!("  Mountain Coverage:  {:.1}%", avg_mountains);
    println!("  Elevation Variance: {:.4}", avg_variance);

    // Atmospheric analysis if available
    if args.atmosphere {
        let pressure_analyses: Vec<_> = analyses
            .iter()
            .filter_map(|a| a.pressure_variance.zip(a.has_pressure_systems))
            .collect();

        if !pressure_analyses.is_empty() {
            let avg_pressure_variance = pressure_analyses
                .iter()
                .map(|(variance, _)| variance)
                .sum::<f32>()
                / pressure_analyses.len() as f32;

            let systems_count = pressure_analyses
                .iter()
                .filter(|(_, has_systems)| *has_systems)
                .count();

            println!("\nAtmospheric Analysis:");
            println!("  Average Pressure Variance: {:.6}", avg_pressure_variance);
            println!(
                "  Seeds with Pressure Systems: {}/{} ({:.1}%)",
                systems_count,
                pressure_analyses.len(),
                (systems_count as f32 / pressure_analyses.len() as f32) * 100.0
            );
        }
    }
}

fn print_recommendations(analyses: &[TerrainAnalysis], args: &Args) {
    println!("\n🎯 RECOMMENDATIONS");
    println!("{}", "=".repeat(50));

    // Find best continental formations
    let mut continental: Vec<_> = analyses
        .iter()
        .filter(|a| {
            matches!(
                a.terrain_type,
                TerrainType::Continental | TerrainType::Mountainous
            )
        })
        .collect();

    // Sort by land percentage (more land = more interesting)
    continental.sort_by(|a, b| b.land_percentage.partial_cmp(&a.land_percentage).unwrap());

    println!("Best Continental Formations:");
    for (i, analysis) in continental.iter().take(5).enumerate() {
        println!(
            "  {}. Seed {} - {} {:.1}% land, {:.1}% mountains{}",
            i + 1,
            analysis.seed,
            analysis.terrain_type.symbol(),
            analysis.land_percentage,
            analysis.mountain_percentage,
            if let Some(has_systems) = analysis.has_pressure_systems {
                if has_systems {
                    " ✅ Pressure Systems"
                } else {
                    " ⚠️ No Pressure"
                }
            } else {
                ""
            }
        );
    }

    // Find most varied terrain (highest variance)
    let mut by_variance: Vec<_> = analyses.iter().collect();
    by_variance.sort_by(|a, b| {
        b.elevation_variance
            .partial_cmp(&a.elevation_variance)
            .unwrap()
    });

    println!("\nMost Varied Terrain (High Elevation Variance):");
    for (i, analysis) in by_variance.iter().take(3).enumerate() {
        println!(
            "  {}. Seed {} - variance {:.4}, {} {:.1}% land",
            i + 1,
            analysis.seed,
            analysis.elevation_variance,
            analysis.terrain_type.symbol(),
            analysis.land_percentage
        );
    }

    // Atmospheric testing recommendations
    if args.atmosphere {
        let with_systems: Vec<_> = analyses
            .iter()
            .filter(|a| a.has_pressure_systems == Some(true))
            .collect();

        if with_systems.is_empty() {
            println!("\n⚠️  ATMOSPHERIC ISSUE DETECTED");
            println!("   No seeds generated meaningful pressure systems!");
            println!("   This suggests a bug in atmospheric pressure generation.");
        } else {
            println!("\nBest for Atmospheric Testing:");
            for (i, analysis) in with_systems.iter().take(3).enumerate() {
                println!(
                    "  {}. Seed {} - {} with active pressure systems",
                    i + 1,
                    analysis.seed,
                    analysis.terrain_type.symbol()
                );
            }
        }
    }

    println!("\nTo test a specific seed:");
    println!(
        "  cargo run -- --seed <SEED> --graphics{}",
        if args.tectonic { " --tectonic" } else { "" }
    );
}
