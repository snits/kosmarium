// ABOUTME: Diagnostic tool to analyze drainage regression issue causing biome classification changes
// ABOUTME: Compares flow accumulation values and biome outputs to identify root cause

use sim_protoype::engine::agents::biome::{BiomeClassifier, BiomeType};
use sim_protoype::engine::core::heightmap::HeightMap;
use sim_protoype::engine::core::{DetailLevel, WorldScale};
use sim_protoype::engine::physics::climate::ClimateSystem;
use sim_protoype::engine::physics::drainage::DrainageNetwork;
use sim_protoype::engine::physics::water::WaterLayer;
use sim_protoype::engine::physics::worldgen::{
    DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator,
};

fn main() {
    println!("=== DRAINAGE REGRESSION ANALYSIS ===\n");

    // Create the same small test case that showed the problem
    test_small_map();
    println!("\n{}\n", "=".repeat(60));
    test_larger_map();
}

fn test_small_map() {
    println!("SMALL MAP (20x10) - Detailed Analysis:");

    // Use DiamondSquareGenerator to create realistic terrain

    let generator = DiamondSquareGenerator::new(42); // Fixed seed for consistency
    let config = DiamondSquareConfig::default();
    let heightmap = generator.generate(20, 10, &config);

    let scale = WorldScale::new(10.0, (20, 10), DetailLevel::Standard);

    // Print heightmap for reference
    println!("Heightmap elevations (sample corners):");
    println!("  Top-left: {:.3}", heightmap.get(0, 0));
    println!("  Top-right: {:.3}", heightmap.get(19, 0));
    println!("  Bottom-left: {:.3}", heightmap.get(0, 9));
    println!("  Bottom-right: {:.3}", heightmap.get(19, 9));

    // Create drainage network
    println!("\nCreating drainage network...");
    let drainage_network = DrainageNetwork::from_heightmap(&heightmap, &scale);
    let stats = drainage_network.get_statistics();

    println!("Drainage Statistics:");
    println!("  Max accumulation: {:.1}", stats.max_accumulation);
    println!("  Mean accumulation: {:.1}", stats.mean_accumulation);
    println!("  Min accumulation: {:.1}", stats.min_accumulation);
    println!(
        "  River cells: {} ({:.1}%)",
        stats.river_cells,
        stats.river_coverage() * 100.0
    );
    println!(
        "  Major river cells: {} ({:.1}%)",
        stats.major_river_cells,
        stats.major_river_coverage() * 100.0
    );
    println!(
        "  Depression cells: {} ({:.1}%)",
        stats.depression_cells,
        stats.lake_coverage() * 100.0
    );
    println!(
        "  Sink cells: {} ({:.1}%)",
        stats.sink_cells,
        stats.sink_coverage() * 100.0
    );

    // Show flow accumulation distribution
    println!("\nFlow accumulation distribution (sampling):");
    let mut accumulation_samples = Vec::new();
    for y in 0..heightmap.height() {
        for x in 0..heightmap.width() {
            accumulation_samples.push(drainage_network.get_flow_accumulation(x, y));
        }
    }
    accumulation_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let len = accumulation_samples.len();
    println!("  10th percentile: {:.1}", accumulation_samples[len / 10]);
    println!(
        "  50th percentile (median): {:.1}",
        accumulation_samples[len / 2]
    );
    println!(
        "  90th percentile: {:.1}",
        accumulation_samples[len * 9 / 10]
    );
    println!("  Maximum: {:.1}", accumulation_samples[len - 1]);

    // Create water layer and concentrate water
    println!("\nConcentrating water...");
    let mut water_layer = WaterLayer::new(heightmap.width(), heightmap.height());

    // Add uniform initial water
    for y in 0..heightmap.height() {
        for x in 0..heightmap.width() {
            water_layer.depth.set(x, y, 0.01); // 1% initial water
        }
    }

    let initial_water = water_layer.get_total_water();
    drainage_network.concentrate_water(&mut water_layer);
    let final_water = water_layer.get_total_water();

    println!("Water concentration results:");
    println!("  Initial total water: {:.6}", initial_water);
    println!("  Final total water: {:.6}", final_water);
    println!("  Conservation ratio: {:.6}", final_water / initial_water);

    // Show water depth distribution
    let mut water_depths = Vec::new();
    for y in 0..heightmap.height() {
        for x in 0..heightmap.width() {
            water_depths.push(water_layer.get_water_depth(x, y));
        }
    }
    water_depths.sort_by(|a, b| a.partial_cmp(b).unwrap());

    println!("Water depth distribution:");
    println!("  Minimum: {:.6}", water_depths[0]);
    println!("  10th percentile: {:.6}", water_depths[len / 10]);
    println!("  50th percentile: {:.6}", water_depths[len / 2]);
    println!("  90th percentile: {:.6}", water_depths[len * 9 / 10]);
    println!("  Maximum: {:.6}", water_depths[len - 1]);

    // Generate biomes with drainage
    println!("\nGenerating biomes...");
    let climate_system = ClimateSystem::new_for_scale(&scale);
    let heightmap_nested = heightmap.to_nested();
    let temperature_layer = climate_system.generate_temperature_layer(&heightmap_nested);
    let classifier = BiomeClassifier::new_for_scale(&scale);

    let biome_map = classifier.generate_biome_map_with_drainage(
        &heightmap,
        &temperature_layer,
        &water_layer,
        &climate_system,
        &drainage_network,
    );

    // Analyze biome distribution
    let distribution = biome_map.biome_distribution();
    println!("Biome distribution:");

    let biome_types = [
        (BiomeType::Ocean, "Ocean"),
        (BiomeType::Lake, "Lake"),
        (BiomeType::River, "River"),
        (BiomeType::Wetland, "Wetland"),
        (BiomeType::Grassland, "Grassland"),
        (BiomeType::Savanna, "Savanna"),
        (BiomeType::Shrubland, "Shrubland"),
        (BiomeType::TemperateForest, "TemperateForest"),
        (BiomeType::Tundra, "Tundra"),
        (BiomeType::Desert, "Desert"),
        (BiomeType::RainForest, "RainForest"),
        (BiomeType::BorealForest, "BorealForest"),
        (BiomeType::Alpine, "Alpine"),
        (BiomeType::Ice, "Ice"),
    ];

    for (biome_type, name) in biome_types {
        let count = distribution[biome_type.to_u8() as usize];
        let percentage = count as f32 / (heightmap.width() * heightmap.height()) as f32 * 100.0;
        if count > 0 {
            println!("  {}: {} cells ({:.1}%)", name, count, percentage);
        }
    }

    // Show the actual map for visual inspection
    println!("\nBiome map visualization:");
    for y in 0..heightmap.height() {
        for x in 0..heightmap.width() {
            let biome = biome_map.get(x, y);
            print!("{}", biome.display_char());
        }
        println!();
    }
}

fn test_larger_map() {
    println!("LARGER MAP (100x50) - Performance and Results Check:");

    use std::time::Instant;

    let generator = DiamondSquareGenerator::new(42); // Same seed
    let config = DiamondSquareConfig::default();
    let heightmap = generator.generate(100, 50, &config);

    let scale = WorldScale::new(100.0, (100, 50), DetailLevel::Standard);

    // Time the drainage calculation
    let start = Instant::now();
    let drainage_network = DrainageNetwork::from_heightmap(&heightmap, &scale);
    let drainage_time = start.elapsed();

    println!(
        "Drainage calculation time: {:.2}ms",
        drainage_time.as_millis()
    );

    let stats = drainage_network.get_statistics();
    println!("Large map drainage statistics:");
    println!("  Max accumulation: {:.1}", stats.max_accumulation);
    println!("  Mean accumulation: {:.1}", stats.mean_accumulation);
    println!(
        "  River cells: {} ({:.1}%)",
        stats.river_cells,
        stats.river_coverage() * 100.0
    );
    println!(
        "  Major river cells: {} ({:.1}%)",
        stats.major_river_cells,
        stats.major_river_coverage() * 100.0
    );

    // Water concentration timing
    let mut water_layer = WaterLayer::new(heightmap.width(), heightmap.height());
    for y in 0..heightmap.height() {
        for x in 0..heightmap.width() {
            water_layer.depth.set(x, y, 0.01);
        }
    }

    let start = Instant::now();
    drainage_network.concentrate_water(&mut water_layer);
    let water_time = start.elapsed();

    println!("Water concentration time: {:.2}ms", water_time.as_millis());

    // Biome generation timing
    let climate_system = ClimateSystem::new_for_scale(&scale);
    let heightmap_nested = heightmap.to_nested();
    let temperature_layer = climate_system.generate_temperature_layer(&heightmap_nested);
    let classifier = BiomeClassifier::new_for_scale(&scale);

    let start = Instant::now();
    let biome_map = classifier.generate_biome_map_with_drainage(
        &heightmap,
        &temperature_layer,
        &water_layer,
        &climate_system,
        &drainage_network,
    );
    let biome_time = start.elapsed();

    println!("Biome generation time: {:.2}ms", biome_time.as_millis());

    // Check biome diversity
    let distribution = biome_map.biome_distribution();
    let total_cells = (heightmap.width() * heightmap.height()) as f32;

    println!("Large map biome diversity:");
    let mut non_tundra_biomes = 0;
    let mut total_biome_types = 0;

    let biome_types = [
        (BiomeType::Ocean, "Ocean"),
        (BiomeType::Lake, "Lake"),
        (BiomeType::River, "River"),
        (BiomeType::Wetland, "Wetland"),
        (BiomeType::Grassland, "Grassland"),
        (BiomeType::Savanna, "Savanna"),
        (BiomeType::Shrubland, "Shrubland"),
        (BiomeType::TemperateForest, "TemperateForest"),
        (BiomeType::Tundra, "Tundra"),
        (BiomeType::Desert, "Desert"),
        (BiomeType::RainForest, "RainForest"),
        (BiomeType::BorealForest, "BorealForest"),
        (BiomeType::Alpine, "Alpine"),
        (BiomeType::Ice, "Ice"),
    ];

    for (biome_type, name) in biome_types {
        let count = distribution[biome_type.to_u8() as usize];
        let percentage = count as f32 / total_cells * 100.0;
        if count > 0 {
            println!("  {}: {} cells ({:.1}%)", name, count, percentage);
            total_biome_types += 1;
            if biome_type != BiomeType::Tundra {
                non_tundra_biomes += count;
            }
        }
    }

    let tundra_count = distribution[BiomeType::Tundra.to_u8() as usize];
    let tundra_percentage = tundra_count as f32 / total_cells * 100.0;

    println!("\nDiversity analysis:");
    println!("  Total biome types present: {}", total_biome_types);
    println!(
        "  Tundra dominance: {:.1}% ({} cells)",
        tundra_percentage, tundra_count
    );
    println!(
        "  Non-tundra cells: {} ({:.1}%)",
        non_tundra_biomes,
        non_tundra_biomes as f32 / total_cells * 100.0
    );

    if tundra_percentage > 70.0 {
        println!("  ⚠️  WARNING: Excessive tundra dominance detected!");
        println!("     This suggests the biome classification parameters may be incorrect.");
    }

    // Sample some biome decisions for debugging
    println!("\nSample biome classification details (first 10 non-tundra cells):");
    let mut samples_found = 0;
    for y in 0..heightmap.height().min(10) {
        for x in 0..heightmap.width().min(10) {
            let biome = biome_map.get(x, y);
            if biome != BiomeType::Tundra && samples_found < 5 {
                let elevation = heightmap.get(x, y);
                let temperature =
                    temperature_layer.get_current_temperature(x, y, climate_system.current_season);
                let water_depth = water_layer.get_water_depth(x, y);
                let accumulation = drainage_network.get_flow_accumulation(x, y);

                println!(
                    "  [{}, {}]: {:?} (elev: {:.3}, temp: {:.1}°C, water: {:.6}, flow_acc: {:.1})",
                    x, y, biome, elevation, temperature, water_depth, accumulation
                );
                samples_found += 1;
            }
        }
    }
}
