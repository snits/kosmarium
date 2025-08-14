// ABOUTME: Demo binary showcasing ecosystem feedback loops coupling system
// ABOUTME: Demonstrates biome effects on climate and hydrology with realistic scenarios

use sim_prototype::engine::{
    core::{
        heightmap::HeightMap,
        scale::{DetailLevel, WorldScale},
    },
    physics::{
        atmospheric_moisture::SurfaceMoistureLayer,
        ecosystem_feedback::{
            BiomeType, EcosystemFeedbackEffects, EcosystemFeedbackParameters,
            EcosystemFeedbackSystem, classify_biome_from_environment,
        },
        flow_engine::FlowEngine,
        temperature::TemperatureField,
        water::WaterLayer,
    },
};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌿 Ecosystem Feedback Loops Demo");
    println!("================================");
    println!();

    // Create world scale and dimensions
    let width = 40;
    let height = 30;
    let scale = WorldScale::new(2000.0, (width, height), DetailLevel::Standard);

    println!("🗺️  World Setup:");
    println!("   Dimensions: {}x{} cells", width, height);
    println!("   Scale: {:.1} meters/pixel", scale.meters_per_pixel());
    println!(
        "   Total area: {:.1} km²",
        (scale.meters_per_pixel() * width as f64 * scale.meters_per_pixel() * height as f64)
            / 1_000_000.0
    );
    println!();

    // Create demo scenarios
    demo_forest_climate_regulation(width as usize, height as usize, &scale)?;
    demo_desert_ecosystem_stress(width as usize, height as usize, &scale)?;
    demo_tropical_rainforest_cycle(width as usize, height as usize, &scale)?;
    demo_biome_classification_system(width as usize, height as usize, &scale)?;

    println!("✅ Ecosystem feedback demonstrations completed!");
    println!();
    println!("🔬 Key Ecosystem Processes Demonstrated:");
    println!("   • Vegetation cooling through evapotranspiration");
    println!("   • Biome-specific water use efficiency");
    println!("   • Humidity generation from plant transpiration");
    println!("   • Surface albedo modification by vegetation");
    println!("   • Water stress effects on ecosystem health");
    println!("   • Temperature regulation by forest canopies");

    Ok(())
}

fn demo_forest_climate_regulation(
    width: usize,
    height: usize,
    scale: &WorldScale,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌲 Demo 1: Forest Climate Regulation");
    println!("   Scenario: Dense forest providing cooling and humidity regulation");
    println!();

    // Create temperature field with warm initial conditions
    let mut temperature_field = TemperatureField::new(width, height, 28.0);

    // Create water layer with adequate moisture for forest
    let mut water_layer = WaterLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            water_layer.add_water(x, y, 2.0); // Good water availability
        }
    }

    // Create atmospheric moisture layer
    let mut moisture_layer = SurfaceMoistureLayer::new(width, height);

    // Create flow engine
    let flow_engine = FlowEngine::for_climate(width, height, scale);

    // Initialize ecosystem feedback system with enhanced regulation
    let mut params = EcosystemFeedbackParameters::default();
    params.temperature_moderation = 4.0; // Strong cooling effect
    params.base_evapotranspiration = 8.0; // High transpiration
    let mut ecosystem_system = EcosystemFeedbackSystem::new(params, width, height);

    // Create forest patch in center
    for x in 10..30 {
        for y in 8..22 {
            ecosystem_system
                .biome_map_mut()
                .set_biome(x, y, BiomeType::Forest);
            ecosystem_system
                .biome_map_mut()
                .set_vegetation_density(x, y, 0.95);
            ecosystem_system.biome_map_mut().set_biomass(x, y, 380.0);
        }
    }

    // Set surrounding areas as grassland
    for x in 0..width {
        for y in 0..height {
            if x < 10 || x >= 30 || y < 8 || y >= 22 {
                ecosystem_system
                    .biome_map_mut()
                    .set_biome(x, y, BiomeType::Grassland);
                ecosystem_system
                    .biome_map_mut()
                    .set_vegetation_density(x, y, 0.4);
                ecosystem_system.biome_map_mut().set_biomass(x, y, 120.0);
            }
        }
    }

    println!("   🌲 Simulating forest climate regulation effects...");

    // Record initial conditions
    let initial_forest_temp = temperature_field.get_temperature(20, 15);
    let initial_grassland_temp = temperature_field.get_temperature(5, 15);
    let initial_moisture = moisture_layer.get_moisture(20, 15);

    // Run ecosystem feedback simulation
    for iteration in 0..12 {
        ecosystem_system.update(
            &mut temperature_field,
            &mut water_layer,
            &mut moisture_layer,
            &flow_engine,
            scale,
            1800.0, // 30-minute timesteps
        );

        if iteration % 3 == 0 {
            println!(
                "      Time step {}/12: Forest transpiration cooling...",
                iteration + 1
            );
        }
    }

    // Display results
    display_ecosystem_analysis(&ecosystem_system, width, height, "Forest Climate")?;
    display_temperature_comparison(
        &temperature_field,
        width,
        height,
        initial_forest_temp,
        initial_grassland_temp,
    )?;
    display_biome_distribution(&ecosystem_system, width, height)?;

    println!("   📊 Analysis: Forest creates microclimate cooling zone");
    println!("               Strong evapotranspiration increases local humidity");
    println!();

    Ok(())
}

fn demo_desert_ecosystem_stress(
    width: usize,
    height: usize,
    scale: &WorldScale,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏜️ Demo 2: Desert Ecosystem Water Stress");
    println!("   Scenario: Arid conditions with limited vegetation survival");
    println!();

    // Create temperature field with hot desert conditions
    let mut temperature_field = TemperatureField::new(width, height, 38.0);

    // Create water layer with very limited moisture
    let mut water_layer = WaterLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let water_amount = if x > width / 2 {
                0.05 // Extreme drought conditions
            } else {
                0.2 // Slightly better conditions
            };
            water_layer.add_water(x, y, water_amount);
        }
    }

    // Create atmospheric moisture layer (dry)
    let mut moisture_layer = SurfaceMoistureLayer::new(width, height);

    // Create flow engine
    let flow_engine = FlowEngine::for_climate(width, height, scale);

    // Initialize ecosystem feedback system with stress sensitivity
    let mut params = EcosystemFeedbackParameters::default();
    params.water_stress_threshold = 0.15; // High sensitivity to water stress
    params.growth_rate = 2.0; // Slow growth in harsh conditions
    let mut ecosystem_system = EcosystemFeedbackSystem::new(params, width, height);

    // Set initial vegetation distribution
    for x in 0..width {
        for y in 0..height {
            if x < width / 2 {
                // Struggling grassland
                ecosystem_system
                    .biome_map_mut()
                    .set_biome(x, y, BiomeType::Grassland);
                ecosystem_system
                    .biome_map_mut()
                    .set_vegetation_density(x, y, 0.3);
                ecosystem_system.biome_map_mut().set_biomass(x, y, 80.0);
            } else {
                // Desert conditions
                ecosystem_system
                    .biome_map_mut()
                    .set_biome(x, y, BiomeType::Desert);
                ecosystem_system
                    .biome_map_mut()
                    .set_vegetation_density(x, y, 0.1);
                ecosystem_system.biome_map_mut().set_biomass(x, y, 15.0);
            }
        }
    }

    println!("   🏜️ Modeling water stress and vegetation decline...");

    // Run ecosystem stress simulation
    for iteration in 0..8 {
        ecosystem_system.update(
            &mut temperature_field,
            &mut water_layer,
            &mut moisture_layer,
            &flow_engine,
            scale,
            43200.0, // 12-hour timesteps for daily cycles
        );

        if iteration % 2 == 0 {
            println!(
                "      Day {}/4: Water stress affecting vegetation...",
                (iteration / 2) + 1
            );
        }
    }

    // Display results
    display_ecosystem_analysis(&ecosystem_system, width, height, "Desert Stress")?;
    display_vegetation_health(&ecosystem_system, width, height)?;

    println!("   📊 Analysis: Water stress severely limits vegetation growth");
    println!("               Desert biome shows adaptation to arid conditions");
    println!();

    Ok(())
}

fn demo_tropical_rainforest_cycle(
    width: usize,
    height: usize,
    scale: &WorldScale,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌴 Demo 3: Tropical Rainforest Water Cycle");
    println!("   Scenario: High-transpiration ecosystem creating local rainfall");
    println!();

    // Create temperature field with warm tropical conditions
    let mut temperature_field = TemperatureField::new(width, height, 26.0);

    // Create water layer with abundant moisture
    let mut water_layer = WaterLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            water_layer.add_water(x, y, 4.0); // Abundant water
        }
    }

    // Create atmospheric moisture layer
    let mut moisture_layer = SurfaceMoistureLayer::new(width, height);

    // Create flow engine
    let flow_engine = FlowEngine::for_climate(width, height, scale);

    // Initialize ecosystem feedback system for tropical conditions
    let mut params = EcosystemFeedbackParameters::default();
    params.base_evapotranspiration = 12.0; // Very high transpiration
    params.humidity_coefficient = 0.15; // Strong humidity generation
    params.temperature_moderation = 3.5; // Strong cooling
    let mut ecosystem_system = EcosystemFeedbackSystem::new(params, width, height);

    // Set up tropical rainforest
    for x in 0..width {
        for y in 0..height {
            ecosystem_system
                .biome_map_mut()
                .set_biome(x, y, BiomeType::Tropical);
            ecosystem_system
                .biome_map_mut()
                .set_vegetation_density(x, y, 1.0);
            ecosystem_system.biome_map_mut().set_biomass(x, y, 480.0);
        }
    }

    println!("   🌴 Computing tropical water cycle dynamics...");

    // Record initial moisture
    let initial_atmospheric_moisture = moisture_layer.get_moisture(width / 2, height / 2);

    // Run tropical ecosystem simulation
    for iteration in 0..6 {
        ecosystem_system.update(
            &mut temperature_field,
            &mut water_layer,
            &mut moisture_layer,
            &flow_engine,
            scale,
            7200.0, // 2-hour timesteps for rapid cycling
        );

        if iteration % 2 == 0 {
            println!(
                "      Cycle {}/3: Massive transpiration creating humidity...",
                (iteration / 2) + 1
            );
        }
    }

    // Display results
    display_ecosystem_analysis(&ecosystem_system, width, height, "Tropical Cycle")?;
    display_humidity_generation(&moisture_layer, width, height, initial_atmospheric_moisture)?;

    println!("   📊 Analysis: Tropical forest drives intense water recycling");
    println!("               High evapotranspiration creates humid microclimate");
    println!();

    Ok(())
}

fn demo_biome_classification_system(
    width: usize,
    height: usize,
    scale: &WorldScale,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🗺️ Demo 4: Automated Biome Classification");
    println!("   Scenario: Environmental conditions determining biome distribution");
    println!();

    // Create temperature gradient (cold to warm)
    let mut temperature_field = TemperatureField::new(width, height, 15.0);
    for x in 0..width {
        for y in 0..height {
            let temp = 2.0 + (x as f32 / width as f32) * 35.0; // 2°C to 37°C gradient
            temperature_field.set_temperature(x, y, temp);
        }
    }

    // Create water availability gradient (dry to wet)
    let mut water_layer = WaterLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let water = 0.05 + (y as f32 / height as f32) * 3.0; // Dry to very wet
            water_layer.add_water(x, y, water);
        }
    }

    // Create heightmap for elevation effects
    let mut heightmap = HeightMap::new(width, height, 100.0);
    for x in 0..width {
        for y in 0..height {
            let elevation = 50.0 + ((x + y) as f32 * 5.0); // Elevation gradient
            heightmap.set(x, y, elevation);
        }
    }

    // Create ecosystem system
    let mut ecosystem_system =
        EcosystemFeedbackSystem::new(EcosystemFeedbackParameters::default(), width, height);

    println!("   🗺️ Classifying biomes based on climate conditions...");

    // Classify biomes based on environmental conditions
    for x in 0..width {
        for y in 0..height {
            let temperature = temperature_field.get_temperature(x, y);
            let water_depth = water_layer.get_water_depth(x, y);
            let elevation = heightmap.get(x, y);

            let water_availability = (water_depth / 2.0).clamp(0.0, 1.0);
            let biome = classify_biome_from_environment(temperature, water_availability, elevation);

            ecosystem_system.biome_map_mut().set_biome(x, y, biome);

            // Set appropriate vegetation for biome
            let (density, biomass) = match biome {
                BiomeType::Desert => (0.1, 18.0),
                BiomeType::Grassland => (0.4, 130.0),
                BiomeType::Forest => (0.8, 350.0),
                BiomeType::Wetland => (0.7, 280.0),
                BiomeType::Tundra => (0.2, 45.0),
                BiomeType::Tropical => (1.0, 450.0),
            };

            ecosystem_system
                .biome_map_mut()
                .set_vegetation_density(x, y, density);
            ecosystem_system.biome_map_mut().set_biomass(x, y, biomass);
        }
    }

    // Display classification results
    display_biome_classification_map(&ecosystem_system, width, height)?;
    display_environmental_gradients(&temperature_field, &water_layer, width, height)?;

    println!("   📊 Analysis: Environmental conditions create distinct biome zones");
    println!("               Temperature and water availability drive biome distribution");
    println!();

    Ok(())
}

fn display_ecosystem_analysis(
    ecosystem_system: &EcosystemFeedbackSystem,
    width: usize,
    height: usize,
    scenario: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !ecosystem_system.has_active_effects() {
        println!("   ⚠️  No ecosystem feedback effects generated");
        return Ok(());
    }

    let effects = ecosystem_system.get_effects().unwrap();

    // Calculate ecosystem statistics
    let mut max_evapotranspiration = 0.0f32;
    let mut max_cooling = 0.0f32;
    let mut max_humidity_gen = 0.0f32;
    let mut total_transpiration = 0.0f64;
    let mut active_vegetation_cells = 0;
    let mut cooling_cells = 0;

    for x in 0..width {
        for y in 0..height {
            let evapotranspiration = effects.get_evapotranspiration_rate(x, y);
            let cooling = -effects.get_temperature_modification(x, y); // Make positive for display
            let humidity_gen = effects.get_humidity_generation(x, y);

            max_evapotranspiration = max_evapotranspiration.max(evapotranspiration);
            max_cooling = max_cooling.max(cooling);
            max_humidity_gen = max_humidity_gen.max(humidity_gen);
            total_transpiration += evapotranspiration as f64;

            if evapotranspiration > 1.0 {
                active_vegetation_cells += 1;
            }
            if cooling > 0.5 {
                cooling_cells += 1;
            }
        }
    }

    println!("   📈 {} Ecosystem Analysis:", scenario);
    println!(
        "      Max evapotranspiration: {:.2} mm/day",
        max_evapotranspiration
    );
    println!("      Max cooling effect: {:.2} °C", max_cooling);
    println!(
        "      Max humidity generation: {:.4} kg/m³/s",
        max_humidity_gen
    );
    println!(
        "      Total transpiration: {:.1} mm/day",
        total_transpiration
    );
    println!(
        "      Active vegetation: {} / {} cells ({:.1}%)",
        active_vegetation_cells,
        width * height,
        (active_vegetation_cells as f32 / (width * height) as f32) * 100.0
    );
    println!(
        "      Cooling zones: {} cells ({:.1}%)",
        cooling_cells,
        (cooling_cells as f32 / (width * height) as f32) * 100.0
    );

    Ok(())
}

fn display_temperature_comparison(
    temperature_field: &TemperatureField,
    width: usize,
    height: usize,
    initial_forest: f32,
    initial_grassland: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let final_forest = temperature_field.get_temperature(20, 15);
    let final_grassland = temperature_field.get_temperature(5, 15);

    println!("   🌡️ Temperature Comparison:");
    println!(
        "      Forest zone: {:.1}°C → {:.1}°C (Δ{:.1}°C)",
        initial_forest,
        final_forest,
        final_forest - initial_forest
    );
    println!(
        "      Grassland zone: {:.1}°C → {:.1}°C (Δ{:.1}°C)",
        initial_grassland,
        final_grassland,
        final_grassland - initial_grassland
    );

    let cooling_difference =
        (initial_forest - final_forest) - (initial_grassland - final_grassland);
    println!(
        "      Forest cooling advantage: {:.1}°C",
        cooling_difference
    );

    Ok(())
}

fn display_biome_distribution(
    ecosystem_system: &EcosystemFeedbackSystem,
    width: usize,
    height: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🗺️ Biome Distribution:");
    println!("      🌲=Forest, 🌾=Grassland, 🏜️=Desert, 🌿=Wetland, ❄️=Tundra, 🌴=Tropical");

    for y in 0..height.min(15) {
        // Limit display size
        print!("      ");
        for x in 0..width.min(35) {
            let biome = ecosystem_system.biome_map().get_biome(x, y);
            let symbol = match biome {
                BiomeType::Forest => "🌲",
                BiomeType::Grassland => "🌾",
                BiomeType::Desert => "🏜️",
                BiomeType::Wetland => "🌿",
                BiomeType::Tundra => "❄️",
                BiomeType::Tropical => "🌴",
            };
            print!("{}", symbol);
        }
        println!();
    }
    println!();

    Ok(())
}

fn display_vegetation_health(
    ecosystem_system: &EcosystemFeedbackSystem,
    width: usize,
    height: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🌱 Vegetation Health (Density):");

    for y in 0..height.min(12) {
        print!("      ");
        for x in 0..width.min(30) {
            let density = ecosystem_system.biome_map().get_vegetation_density(x, y);
            let symbol = if density < 0.2 {
                "🟫" // Very sparse
            } else if density < 0.4 {
                "🟤" // Sparse
            } else if density < 0.6 {
                "🟡" // Moderate
            } else if density < 0.8 {
                "🟢" // Good
            } else {
                "🟩" // Excellent
            };
            print!("{}", symbol);
        }
        println!();
    }
    println!();

    Ok(())
}

fn display_humidity_generation(
    moisture_layer: &SurfaceMoistureLayer,
    width: usize,
    height: usize,
    initial_moisture: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let final_moisture = moisture_layer.get_moisture(width / 2, height / 2);
    let moisture_increase = final_moisture - initial_moisture;

    println!("   💨 Atmospheric Moisture Changes:");
    println!("      Initial moisture: {:.3} kg/m³", initial_moisture);
    println!("      Final moisture: {:.3} kg/m³", final_moisture);
    println!("      Moisture increase: {:.3} kg/m³", moisture_increase);

    // Display moisture distribution
    println!("      🔵=Low, 🟦=Medium, 🟨=High, 🟧=Very High humidity");

    let mut min_moisture = f32::INFINITY;
    let mut max_moisture = f32::NEG_INFINITY;

    for x in 0..width {
        for y in 0..height {
            let moisture = moisture_layer.get_moisture(x, y);
            min_moisture = min_moisture.min(moisture);
            max_moisture = max_moisture.max(moisture);
        }
    }

    for y in 0..height.min(10) {
        print!("      ");
        for x in 0..width.min(25) {
            let moisture = moisture_layer.get_moisture(x, y);
            let normalized = if max_moisture > min_moisture {
                (moisture - min_moisture) / (max_moisture - min_moisture)
            } else {
                0.0
            };

            let symbol = if normalized < 0.25 {
                "🔵"
            } else if normalized < 0.5 {
                "🟦"
            } else if normalized < 0.75 {
                "🟨"
            } else {
                "🟧"
            };
            print!("{}", symbol);
        }
        println!();
    }
    println!();

    Ok(())
}

fn display_biome_classification_map(
    ecosystem_system: &EcosystemFeedbackSystem,
    width: usize,
    height: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🗺️ Automatic Biome Classification Results:");

    // Count biome types
    let mut biome_counts = [0; 6]; // Desert, Grassland, Forest, Wetland, Tundra, Tropical

    for x in 0..width {
        for y in 0..height {
            let biome = ecosystem_system.biome_map().get_biome(x, y);
            let index = match biome {
                BiomeType::Desert => 0,
                BiomeType::Grassland => 1,
                BiomeType::Forest => 2,
                BiomeType::Wetland => 3,
                BiomeType::Tundra => 4,
                BiomeType::Tropical => 5,
            };
            biome_counts[index] += 1;
        }
    }

    let total_cells = width * height;
    println!("      Biome Distribution:");
    println!(
        "        🏜️ Desert: {} cells ({:.1}%)",
        biome_counts[0],
        (biome_counts[0] as f32 / total_cells as f32) * 100.0
    );
    println!(
        "        🌾 Grassland: {} cells ({:.1}%)",
        biome_counts[1],
        (biome_counts[1] as f32 / total_cells as f32) * 100.0
    );
    println!(
        "        🌲 Forest: {} cells ({:.1}%)",
        biome_counts[2],
        (biome_counts[2] as f32 / total_cells as f32) * 100.0
    );
    println!(
        "        🌿 Wetland: {} cells ({:.1}%)",
        biome_counts[3],
        (biome_counts[3] as f32 / total_cells as f32) * 100.0
    );
    println!(
        "        ❄️ Tundra: {} cells ({:.1}%)",
        biome_counts[4],
        (biome_counts[4] as f32 / total_cells as f32) * 100.0
    );
    println!(
        "        🌴 Tropical: {} cells ({:.1}%)",
        biome_counts[5],
        (biome_counts[5] as f32 / total_cells as f32) * 100.0
    );

    Ok(())
}

fn display_environmental_gradients(
    temperature_field: &TemperatureField,
    water_layer: &WaterLayer,
    width: usize,
    height: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let (min_temp, max_temp) = temperature_field.temperature_range();

    let mut min_water = f32::INFINITY;
    let mut max_water = f32::NEG_INFINITY;
    for x in 0..width {
        for y in 0..height {
            let water = water_layer.get_water_depth(x, y);
            min_water = min_water.min(water);
            max_water = max_water.max(water);
        }
    }

    println!("   🌡️ Environmental Gradients:");
    println!(
        "      Temperature range: {:.1}°C to {:.1}°C",
        min_temp, max_temp
    );
    println!(
        "      Water availability: {:.2} to {:.2} m",
        min_water, max_water
    );

    Ok(())
}
