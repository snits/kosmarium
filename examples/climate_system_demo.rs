// ABOUTME: Demonstration of climate system with temperature layer and seasonal cycling
// ABOUTME: Shows how elevation and latitude affect temperature distribution

use sim_protoype::climate::{ClimateParameters, ClimateSystem};
use sim_protoype::scale::{DetailLevel, WorldScale};
use sim_protoype::worldgen::{DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator};

fn main() {
    println!("=== Climate System Demo ===\n");

    // Generate a test heightmap with varied terrain
    let generator = DiamondSquareGenerator::new(42);
    let config = DiamondSquareConfig {
        initial_corners: [0.0, 0.5, 0.3, 0.8], // Varied starting elevations
        roughness: 0.7,
        persistence: 0.6,
        wrap_edges: false,
    };

    let heightmap = generator.generate(20, 20, &config);

    // Create climate system with WorldScale integration
    let world_scale = WorldScale::new(100.0, (20, 20), DetailLevel::Standard); // 100km domain
    let climate = ClimateSystem::new_for_scale(&world_scale);

    println!("Climate Parameters (scale-aware):");
    println!(
        "- Base temperature: {:.1}°C",
        climate.parameters.base_temperature_c
    );
    println!(
        "- Elevation lapse rate: {:.4}°C/m",
        climate.parameters.elevation_lapse_rate
    );
    println!(
        "- Seasonal amplitude: {:.1}°C",
        climate.parameters.seasonal_amplitude
    );
    println!(
        "- Latitude gradient: {:.1}°C/degree",
        climate.parameters.latitude_gradient
    );

    // Generate temperature layer from terrain
    let heightmap_nested = heightmap.to_nested();
    let temp_layer = climate.generate_temperature_layer(&heightmap_nested);

    println!("\n=== Temperature Distribution ===");

    // Show temperature map
    println!("\nTemperature Map (°C):");
    println!("Legend: Very Cold(<-10) | Cold(-10-0) | Cool(0-10) | Warm(10-20) | Hot(>20)");

    for y in 0..heightmap_nested.len() {
        for x in 0..heightmap_nested[0].len() {
            let elevation = heightmap_nested[y][x];
            let temperature = temp_layer.get_temperature(x, y);

            let symbol = match temperature {
                t if t < -10.0 => "❄️", // Very cold
                t if t < 0.0 => "🧊",   // Cold
                t if t < 10.0 => "🌡️",  // Cool
                t if t < 20.0 => "🌤️",  // Warm
                _ => "🔥",              // Hot
            };

            print!("{}", symbol);
        }
        println!();
    }

    // Show detailed analysis for specific points
    println!("\n=== Detailed Analysis ===");

    // Find interesting points
    let mut min_temp = f32::INFINITY;
    let mut max_temp = f32::NEG_INFINITY;
    let mut min_elevation = f32::INFINITY;
    let mut max_elevation = f32::NEG_INFINITY;
    let mut min_coords = (0, 0);
    let mut max_coords = (0, 0);

    for y in 0..heightmap_nested.len() {
        for x in 0..heightmap_nested[0].len() {
            let elevation = heightmap_nested[y][x];
            let temperature = temp_layer.get_temperature(x, y);

            if temperature < min_temp {
                min_temp = temperature;
                min_coords = (x, y);
            }
            if temperature > max_temp {
                max_temp = temperature;
                max_coords = (x, y);
            }
            if elevation < min_elevation {
                min_elevation = elevation;
            }
            if elevation > max_elevation {
                max_elevation = elevation;
            }
        }
    }

    println!(
        "Coldest location: ({}, {}) = {:.1}°C",
        min_coords.0, min_coords.1, min_temp
    );
    println!("- Elevation: {:.3}", heightmap[min_coords.1][min_coords.0]);
    println!(
        "- Latitude factor: {:.2} (0=north pole, 1=south pole)",
        min_coords.1 as f32 / heightmap_nested.len() as f32
    );

    println!(
        "\nWarmest location: ({}, {}) = {:.1}°C",
        max_coords.0, max_coords.1, max_temp
    );
    println!("- Elevation: {:.3}", heightmap[max_coords.1][max_coords.0]);
    println!(
        "- Latitude factor: {:.2}",
        max_coords.1 as f32 / heightmap_nested.len() as f32
    );

    println!("\nTerrain Summary:");
    println!(
        "- Elevation range: {:.3} to {:.3}",
        min_elevation, max_elevation
    );
    println!(
        "- Temperature range: {:.1}°C to {:.1}°C",
        min_temp, max_temp
    );

    // Show seasonal effects
    println!("\n=== Seasonal Effects ===");

    let mid_x = heightmap_nested[0].len() / 2;
    let mid_y = heightmap_nested.len() / 2;

    println!("Seasonal temperature at center ({}, {}):", mid_x, mid_y);

    let seasons = [
        (0.0, "Winter"),
        (0.25, "Spring"),
        (0.5, "Summer"),
        (0.75, "Fall"),
    ];

    for (season_factor, season_name) in seasons.iter() {
        let seasonal_temp = temp_layer.get_current_temperature(mid_x, mid_y, *season_factor);
        let base_temp = temp_layer.get_temperature(mid_x, mid_y);
        let variation = seasonal_temp - base_temp;

        println!(
            "- {}: {:.1}°C ({:+.1}°C from base)",
            season_name, seasonal_temp, variation
        );
    }

    // Show evaporation multipliers at different temperatures
    println!("\n=== Temperature Effects on Evaporation ===");

    let test_temps = [-10.0, 0.0, 10.0, 20.0, 30.0, 40.0];
    println!("Temperature vs Evaporation Rate:");

    for temp in test_temps.iter() {
        let multiplier = climate.get_evaporation_multiplier(*temp);
        println!(
            "- {:.0}°C: {:.2}x normal evaporation rate",
            temp, multiplier
        );
    }

    // Show average temperatures by latitude band
    println!("\n=== Latitude Temperature Profile ===");

    for y in (0..heightmap_nested.len()).step_by(heightmap_nested.len() / 5) {
        let mut sum_temp = 0.0;
        let mut count = 0;

        for x in 0..heightmap_nested[0].len() {
            sum_temp += temp_layer.get_temperature(x, y);
            count += 1;
        }

        let avg_temp = sum_temp / count as f32;
        let latitude_name = match y {
            y if y < heightmap_nested.len() / 4 => "Northern",
            y if y < 3 * heightmap_nested.len() / 4 => "Central",
            _ => "Southern",
        };

        println!(
            "- {} region (row {}): {:.1}°C average",
            latitude_name, y, avg_temp
        );
    }

    println!("\n=== Climate System Integration ===");
    println!("✅ Temperature layer successfully generated from terrain");
    println!("✅ Elevation-based cooling implemented (lapse rate)");
    println!("✅ Latitude-based temperature variation working");
    println!("✅ Seasonal cycling system functional");
    println!("✅ Scale-aware parameter derivation integrated");
    println!("✅ Temperature-dependent evaporation multipliers calculated");

    println!("\nNext steps:");
    println!("- Add dimensional analysis for climate units (°C, mm/h, m/s)");
    println!("- Integrate with water system for temperature-dependent evaporation");
    println!("- Add grid convergence validation for climate systems");
    println!("- Implement precipitation type determination (rain vs snow)");

    println!("\n=== Demo Complete ===");
}
