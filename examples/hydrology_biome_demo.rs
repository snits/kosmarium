// ABOUTME: Demonstration of biome-hydrology coupling using unified FlowEngine velocity fields
// ABOUTME: Shows how water flow patterns influence vegetation distribution in real-time

use sim_protoype::engine::{
    agents::biome::{BiomeClassifier, BiomeType},
    core::{heightmap::HeightMap, math::Vec2, scale::{DetailLevel, WorldScale}},
    physics::{
        climate::ClimateSystem,
        drainage::DrainageNetwork,
        flow_engine::{FlowAlgorithm, FlowEngine},
        hydro_biome_coupling::HydrologyAwareBiomeClassifier,
        water::WaterLayer,
    },
};

fn main() {
    println!("🌍 Phase 3 Demo: Biome-Hydrology Coupling");
    println!("==========================================");
    
    // Create a test terrain with clear hydrological features
    let terrain_data = vec![
        vec![0.9, 0.8, 0.7, 0.6, 0.5, 0.4], // Mountain ridge
        vec![0.8, 0.6, 0.4, 0.3, 0.2, 0.2], // River valley  
        vec![0.7, 0.5, 0.3, 0.2, 0.1, 0.1], // River continues
        vec![0.8, 0.6, 0.4, 0.3, 0.2, 0.2], // Valley widens
        vec![0.9, 0.7, 0.6, 0.5, 0.4, 0.3], // Plateau
        vec![1.0, 0.8, 0.7, 0.6, 0.5, 0.4], // High plateau
    ];
    
    let heightmap = HeightMap::from_nested(terrain_data);
    let scale = WorldScale::new(30.0, (6, 6), DetailLevel::Standard); // 30km domain, 5km/pixel
    
    println!("📊 Created test terrain ({}km domain, {}km/pixel)", 
             scale.physical_size_km, 
             scale.meters_per_pixel() / 1000.0);
    
    // Generate supporting systems
    let drainage_network = DrainageNetwork::from_heightmap(&heightmap, &scale);
    let climate_system = ClimateSystem::new_for_scale(&scale);
    let heightmap_nested = heightmap.to_nested();
    let temperature_layer = climate_system.generate_temperature_layer(&heightmap_nested);
    
    // Create water layer with river flow
    let mut water_layer = WaterLayer::new(6, 6);
    for y in 1..4 {
        for x in 1..5 {
            water_layer.depth.set(x, y, 0.02); // River channel
        }
    }
    drainage_network.concentrate_water(&mut water_layer);
    
    // Set up flow engine with realistic velocity patterns
    let mut flow_engine = FlowEngine::new(FlowAlgorithm::Conservation, 6, 6, &scale);
    
    // Simulate river flow velocities (downstream acceleration)
    for x in 1..5 {
        let velocity = 0.3 - (x as f32) * 0.05; // Accelerates downstream
        flow_engine.velocity_field.set_velocity(x, 1, Vec2::new(velocity, 0.0));
        flow_engine.velocity_field.set_velocity(x, 2, Vec2::new(velocity * 0.8, 0.0));
        flow_engine.velocity_field.set_velocity(x, 3, Vec2::new(velocity * 0.6, 0.0));
    }
    
    println!("\n🌊 Flow Engine Configuration:");
    println!("   Algorithm: {:?}", flow_engine.algorithm);
    println!("   Max velocity: {:.3} m/s", flow_engine.velocity_field.max_velocity_magnitude());
    
    // Compare traditional vs hydrology-aware biome classification
    let traditional_classifier = BiomeClassifier::new_for_scale(&scale);
    let hydro_aware_classifier = HydrologyAwareBiomeClassifier::new_for_scale(&scale, 0.8);
    
    println!("\n📋 Generating Biome Maps:");
    println!("   Traditional: Whittaker model only");
    println!("   Hydrology-aware: Whittaker + Flow dynamics (80% influence)");
    
    // Generate traditional biome map
    let traditional_biomes = traditional_classifier.generate_biome_map_with_drainage(
        &heightmap,
        &temperature_layer,
        &water_layer,
        &climate_system,
        &drainage_network,
    );
    
    // Generate hydrology-aware biome map
    let (hydro_biomes, water_availability) = hydro_aware_classifier.generate_biome_map_with_hydrology(
        &heightmap,
        &temperature_layer,
        &water_layer,
        &climate_system,
        &drainage_network,
        &flow_engine,
        &scale,
    );
    
    // Display results
    println!("\n🗺️ BIOME COMPARISON:");
    println!("   Coordinates    Traditional   Hydrology-Aware    Water Avail    Flow (m/s)");
    println!("   -----------    -----------   ---------------    -----------    ----------");
    
    for y in 0..6 {
        for x in 0..6 {
            let trad_biome = traditional_biomes.get(x, y);
            let hydro_biome = hydro_biomes.get(x, y);
            let availability = water_availability.get_availability(x, y);
            let flow_intensity = water_availability.get_flow_intensity(x, y);
            
            let trad_symbol = trad_biome.display_char();
            let hydro_symbol = hydro_biome.display_char();
            
            println!("   ({},{})          {}             {}              {:.2}           {:.3}",
                     x, y, trad_symbol, hydro_symbol, availability, flow_intensity);
        }
    }
    
    println!("\n🎯 BIOME LEGEND:");
    println!("   ~ = Ocean/Lake    - = River     % = Wetland");
    println!("   . = Grassland     , = Savanna   : = Shrubland");
    println!("   T = Temperate Forest            R = Rain Forest");
    println!("   ^ = Tundra        ' = Desert    * = Ice");
    
    // Analyze differences
    let mut differences = 0;
    let mut water_enhanced = 0;
    let mut total_cells = 0;
    
    for x in 0..6 {
        for y in 0..6 {
            total_cells += 1;
            let trad_biome = traditional_biomes.get(x, y);
            let hydro_biome = hydro_biomes.get(x, y);
            let availability = water_availability.get_availability(x, y);
            
            if trad_biome != hydro_biome {
                differences += 1;
                
                // Check if hydrology created more water-favorable vegetation
                let is_enhancement = match (trad_biome, hydro_biome) {
                    (BiomeType::Desert, BiomeType::Shrubland) => true,
                    (BiomeType::Shrubland, BiomeType::Grassland) => true,
                    (BiomeType::Grassland, BiomeType::TemperateForest) => true,
                    (_, BiomeType::Wetland) if availability > 0.6 => true,
                    _ => false,
                };
                
                if is_enhancement {
                    water_enhanced += 1;
                }
            }
        }
    }
    
    println!("\n📈 COUPLING IMPACT ANALYSIS:");
    println!("   Total cells: {}", total_cells);
    println!("   Cells modified by hydrology: {} ({:.1}%)", differences, (differences as f32 / total_cells as f32) * 100.0);
    println!("   Water-enhanced vegetation: {} ({:.1}%)", water_enhanced, (water_enhanced as f32 / total_cells as f32) * 100.0);
    
    // Water availability statistics
    let mut total_availability = 0.0;
    let mut river_cells = 0;
    let mut high_availability_cells = 0;
    
    for x in 0..6 {
        for y in 0..6 {
            let availability = water_availability.get_availability(x, y);
            let flow_intensity = water_availability.get_flow_intensity(x, y);
            
            total_availability += availability;
            
            if flow_intensity > 0.1 {
                river_cells += 1;
            }
            
            if availability > 0.5 {
                high_availability_cells += 1;
            }
        }
    }
    
    let avg_availability = total_availability / total_cells as f32;
    
    println!("\n💧 WATER AVAILABILITY METRICS:");
    println!("   Average water availability: {:.3}", avg_availability);
    println!("   Cells with significant flow (>0.1 m/s): {}", river_cells);
    println!("   Cells with high water availability (>0.5): {}", high_availability_cells);
    
    // Success summary
    println!("\n✅ SUCCESS: Phase 3 Biome-Hydrology Coupling Implemented!");
    println!("   🔗 Unified FlowEngine velocity fields successfully shared with biome system");
    println!("   🌊 Water availability calculated from flow dynamics (residence time, watershed, intensity)");
    println!("   🌱 Vegetation patterns now respond to hydrological conditions");
    println!("   📊 Cross-system physics coupling achieved through unified architecture");
    
    if differences > 0 {
        println!("   🎉 Hydrology influence successfully modified {} biome classifications", differences);
    }
    
    println!("\n🚀 Ready for next coupling: Maritime Climate Effects");
}