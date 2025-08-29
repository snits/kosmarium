// ABOUTME: Debug program to investigate biome degradation from realistic → lake world pattern
// ABOUTME: Systematically tracks water, temperature, and biome changes over time to find root cause

use kosmarium::engine::{
    core::{
        heightmap::HeightMap,
        scale::{DetailLevel, WorldScale},
    },
    sim::Simulation,
    agents::biome::{BiomeClassifier, BiomeType},
};
use std::collections::HashMap;

fn main() {
    println!("Investigating biome degradation issue...\n");

    // Create a realistic continental-scale simulation
    let world_scale = WorldScale::new(200.0, (240, 120), DetailLevel::Standard);
    
    // Create varied terrain to test biome diversity
    let mut heightmap = HeightMap::new(240, 120, 0.0);
    for y in 0..120 {
        for x in 0..240 {
            // Create diverse terrain: coasts, plains, hills, mountains
            let center_x = 120.0;
            let center_y = 60.0;
            let dist_from_center = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
            let max_dist = (center_x.powi(2) + center_y.powi(2)).sqrt();
            let distance_factor = (dist_from_center / max_dist).min(1.0);
            
            // Elevation increases toward interior, with noise for variety
            let base_elevation = 0.1 + distance_factor * 0.7; // 0.1 to 0.8 range
            let noise = ((x as f32 * 0.1).sin() + (y as f32 * 0.1).cos()) * 0.1;
            let elevation = (base_elevation + noise).max(0.0).min(1.0);
            
            heightmap.set(x, y, elevation);
        }
    }
    
    // Create simulation
    let mut simulation = Simulation::_new_with_scale(heightmap, world_scale);
    
    println!("=== INITIAL STATE ANALYSIS ===");
    let initial_analysis = analyze_simulation_state(&mut simulation, 0);
    print_analysis(&initial_analysis);
    
    // Track key metrics over time
    let mut biome_history = Vec::new();
    biome_history.push(initial_analysis);
    
    println!("\n=== TEMPORAL EVOLUTION TRACKING ===");
    
    // Run for many ticks to observe long-term trends
    let total_ticks = 50;
    let analysis_interval = 5;
    
    for tick in 1..=total_ticks {
        simulation.tick();
        
        if tick % analysis_interval == 0 {
            println!("\n--- Tick {} Analysis ---", tick);
            let current_analysis = analyze_simulation_state(&mut simulation, tick);
            
            // Compare to previous state
            let previous = &biome_history.last().unwrap();
            detect_biome_changes(previous, &current_analysis);
            
            print_analysis(&current_analysis);
            biome_history.push(current_analysis);
        }
    }
    
    println!("\n=== FINAL ANALYSIS ===");
    analyze_degradation_pattern(&biome_history);
}

#[derive(Debug, Clone)]
struct SystemAnalysis {
    tick: u64,
    // Water system metrics
    total_water: f32,
    avg_water_depth: f32,
    max_water_depth: f32,
    water_cells_count: usize,
    
    // Temperature system metrics
    avg_temperature: f32,
    min_temperature: f32,
    max_temperature: f32,
    cold_cells_count: usize, // < 0°C
    very_cold_cells_count: usize, // < -10°C
    
    // Pressure system metrics (to verify fix)
    avg_pressure: f32,
    pressure_spread: f32,
    
    // Biome distribution
    biome_counts: HashMap<BiomeType, u32>,
    total_cells: u32,
    
    // Key biome percentages
    water_biome_percentage: f32,
    ice_biome_percentage: f32,
    terrestrial_biome_percentage: f32,
}

fn analyze_simulation_state(simulation: &mut Simulation, tick: u64) -> SystemAnalysis {
    let width = simulation.get_width();
    let height = simulation.get_height();
    let total_cells = (width * height) as u32;
    
    // Water system analysis
    let water_layer = simulation.get_water_layer();
    let total_water = water_layer.get_total_water();
    let avg_water_depth = total_water / total_cells as f32;
    
    let mut max_water_depth = 0.0;
    let mut water_cells_count = 0;
    
    for y in 0..height {
        for x in 0..width {
            let depth = water_layer.get_water_depth(x, y);
            if depth > 0.001 { // Count cells with significant water
                water_cells_count += 1;
            }
            max_water_depth = max_water_depth.max(depth);
        }
    }
    
    // Temperature system analysis
    let temperature_layer = simulation.get_temperature_layer();
    let avg_temperature = temperature_layer.get_average_temperature();
    
    let mut min_temperature = f32::INFINITY;
    let mut max_temperature = f32::NEG_INFINITY;
    let mut cold_cells_count = 0;
    let mut very_cold_cells_count = 0;
    
    for y in 0..height {
        for x in 0..width {
            let temp = temperature_layer.get_current_temperature(x, y, simulation.climate_system.current_season);
            min_temperature = min_temperature.min(temp);
            max_temperature = max_temperature.max(temp);
            if temp < 0.0 {
                cold_cells_count += 1;
            }
            if temp < -10.0 {
                very_cold_cells_count += 1;
            }
        }
    }
    
    // Pressure system analysis (verify fix is working)
    let pressure_layer = simulation.get_atmospheric_pressure_layer();
    let avg_pressure = pressure_layer.get_average_pressure();
    
    let mut min_pressure = f32::INFINITY;
    let mut max_pressure = f32::NEG_INFINITY;
    for y in 0..height {
        for x in 0..width {
            let pressure = pressure_layer.get_pressure(x, y);
            min_pressure = min_pressure.min(pressure);
            max_pressure = max_pressure.max(pressure);
        }
    }
    let pressure_spread = max_pressure - min_pressure;
    
    // Biome analysis
    let biome_map = simulation.generate_biome_map();
    let distribution = biome_map.biome_distribution();
    
    let mut biome_counts = HashMap::new();
    let mut water_biome_count = 0;
    let mut ice_biome_count = 0;
    let mut terrestrial_biome_count = 0;
    
    for (biome_idx, &count) in distribution.iter().enumerate() {
        if let Some(biome_type) = BiomeType::from_u8(biome_idx as u8) {
            biome_counts.insert(biome_type, count);
            
            if biome_type.is_aquatic() {
                water_biome_count += count;
            } else if biome_type == BiomeType::Ice {
                ice_biome_count += count;
            } else {
                terrestrial_biome_count += count;
            }
        }
    }
    
    SystemAnalysis {
        tick,
        total_water,
        avg_water_depth,
        max_water_depth,
        water_cells_count,
        avg_temperature,
        min_temperature,
        max_temperature,
        cold_cells_count,
        very_cold_cells_count,
        avg_pressure,
        pressure_spread,
        biome_counts,
        total_cells,
        water_biome_percentage: (water_biome_count as f32 / total_cells as f32) * 100.0,
        ice_biome_percentage: (ice_biome_count as f32 / total_cells as f32) * 100.0,
        terrestrial_biome_percentage: (terrestrial_biome_count as f32 / total_cells as f32) * 100.0,
    }
}

fn print_analysis(analysis: &SystemAnalysis) {
    println!("Tick {}: Water System:", analysis.tick);
    println!("  Total water: {:.6}", analysis.total_water);
    println!("  Avg depth: {:.6}, Max depth: {:.6}", analysis.avg_water_depth, analysis.max_water_depth);
    println!("  Cells with water: {} ({:.1}%)", analysis.water_cells_count, 
             (analysis.water_cells_count as f32 / analysis.total_cells as f32) * 100.0);
    
    println!("  Temperature System:");
    println!("    Avg: {:.2}°C, Range: {:.2}°C to {:.2}°C", 
             analysis.avg_temperature, analysis.min_temperature, analysis.max_temperature);
    println!("    Cold cells (<0°C): {} ({:.1}%)", analysis.cold_cells_count,
             (analysis.cold_cells_count as f32 / analysis.total_cells as f32) * 100.0);
    println!("    Very cold cells (<-10°C): {} ({:.1}%)", analysis.very_cold_cells_count,
             (analysis.very_cold_cells_count as f32 / analysis.total_cells as f32) * 100.0);
    
    println!("  Pressure System:");
    println!("    Avg: {:.2} kPa, Spread: {:.2} kPa", 
             analysis.avg_pressure / 1000.0, analysis.pressure_spread / 1000.0);
    
    println!("  Biome Distribution:");
    println!("    Water biomes: {:.1}%", analysis.water_biome_percentage);
    println!("    Ice biomes: {:.1}%", analysis.ice_biome_percentage);  
    println!("    Terrestrial biomes: {:.1}%", analysis.terrestrial_biome_percentage);
    
    // Show top biome types
    let mut sorted_biomes: Vec<_> = analysis.biome_counts.iter().collect();
    sorted_biomes.sort_by(|a, b| b.1.cmp(a.1));
    println!("    Top biomes:");
    for (biome_type, count) in sorted_biomes.iter().take(5) {
        let percentage = (**count as f32 / analysis.total_cells as f32) * 100.0;
        println!("      {:?}: {:.1}%", biome_type, percentage);
    }
}

fn detect_biome_changes(previous: &SystemAnalysis, current: &SystemAnalysis) {
    let water_change = current.water_biome_percentage - previous.water_biome_percentage;
    let ice_change = current.ice_biome_percentage - previous.ice_biome_percentage;
    let terrestrial_change = current.terrestrial_biome_percentage - previous.terrestrial_biome_percentage;
    
    println!("  Changes since tick {}:", previous.tick);
    if water_change.abs() > 0.1 {
        println!("    Water biomes: {:+.1}%", water_change);
    }
    if ice_change.abs() > 0.1 {
        println!("    Ice biomes: {:+.1}%", ice_change);
    }
    if terrestrial_change.abs() > 0.1 {
        println!("    Terrestrial biomes: {:+.1}%", terrestrial_change);
    }
    
    // Check for concerning trends
    if water_change > 2.0 {
        println!("    🚨 ALERT: Significant increase in water biomes!");
    }
    if ice_change > 2.0 {
        println!("    🚨 ALERT: Significant increase in ice biomes!");
    }
    if terrestrial_change < -2.0 {
        println!("    🚨 ALERT: Significant decrease in terrestrial biomes!");
    }
}

fn analyze_degradation_pattern(history: &[SystemAnalysis]) {
    println!("=== DEGRADATION PATTERN ANALYSIS ===");
    
    let initial = &history[0];
    let final_state = history.last().unwrap();
    
    println!("Initial → Final changes:");
    println!("  Water biomes: {:.1}% → {:.1}% ({:+.1}%)", 
             initial.water_biome_percentage, final_state.water_biome_percentage,
             final_state.water_biome_percentage - initial.water_biome_percentage);
    println!("  Ice biomes: {:.1}% → {:.1}% ({:+.1}%)", 
             initial.ice_biome_percentage, final_state.ice_biome_percentage,
             final_state.ice_biome_percentage - initial.ice_biome_percentage);
    println!("  Terrestrial biomes: {:.1}% → {:.1}% ({:+.1}%)", 
             initial.terrestrial_biome_percentage, final_state.terrestrial_biome_percentage,
             final_state.terrestrial_biome_percentage - initial.terrestrial_biome_percentage);
    
    // Water system trend analysis
    println!("\nWater system trends:");
    println!("  Total water: {:.6} → {:.6} ({:+.6})", 
             initial.total_water, final_state.total_water,
             final_state.total_water - initial.total_water);
    println!("  Max water depth: {:.6} → {:.6} ({:+.6})", 
             initial.max_water_depth, final_state.max_water_depth,
             final_state.max_water_depth - initial.max_water_depth);
    
    // Temperature system trend analysis  
    println!("\nTemperature system trends:");
    println!("  Average temperature: {:.2}°C → {:.2}°C ({:+.2}°C)", 
             initial.avg_temperature, final_state.avg_temperature,
             final_state.avg_temperature - initial.avg_temperature);
    println!("  Very cold cells: {} → {} ({:+})", 
             initial.very_cold_cells_count, final_state.very_cold_cells_count,
             final_state.very_cold_cells_count as i32 - initial.very_cold_cells_count as i32);
    
    // Pressure system verification
    println!("\nPressure system verification:");
    println!("  Pressure spread: {:.2} kPa → {:.2} kPa ({:+.2} kPa)", 
             initial.pressure_spread / 1000.0, final_state.pressure_spread / 1000.0,
             (final_state.pressure_spread - initial.pressure_spread) / 1000.0);
    
    // Identify root cause
    println!("\n=== ROOT CAUSE ANALYSIS ===");
    
    let water_accumulation = final_state.total_water > initial.total_water * 1.1;
    let cooling_trend = final_state.avg_temperature < initial.avg_temperature - 1.0;
    let pressure_stable = (final_state.pressure_spread - initial.pressure_spread).abs() < 1000.0;
    
    println!("System stability check:");
    println!("  Pressure system stable: {}", if pressure_stable { "✓" } else { "✗" });
    println!("  Water accumulating: {}", if water_accumulation { "✗ (ISSUE)" } else { "✓" });
    println!("  Temperature cooling: {}", if cooling_trend { "✗ (ISSUE)" } else { "✓" });
    
    if water_accumulation {
        println!("\n🔍 FINDING: Water system may still have temporal integration bugs");
        println!("   Water continues to accumulate despite supposed fixes");
    }
    
    if cooling_trend {
        println!("\n🔍 FINDING: Temperature system may have cooling bias");
        println!("   Average temperature decreasing over time could cause ice bias");
    }
    
    if !water_accumulation && !cooling_trend {
        println!("\n🔍 FINDING: Issue may be in biome classification logic itself");
        println!("   Small changes in water/temperature triggering aquatic/ice bias");
    }
}