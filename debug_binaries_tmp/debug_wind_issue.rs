// ABOUTME: Debug script to investigate why wind speeds are all zero
// ABOUTME: Checks atmospheric system activation and Coriolis threshold

use sim_protoype::scale::{WorldScale, DetailLevel, ScaleAware};
use sim_protoype::atmosphere::{AtmosphericSystem, AtmosphericParameters};

fn main() {
    println!("=== Wind Debug Investigation ===\n");

    // Test with the actual simulation parameters (240x120)
    let width = 240;
    let height = 120;
    
    // Calculate scale exactly like Simulation::new() does
    let base_area = 240.0 * 120.0;
    let current_area = (width * height) as f64;
    let area_ratio = current_area / base_area;
    let climate_scale = 100.0 * (area_ratio / 4.0).sqrt();
    let terrain_scale = 10.0 * area_ratio.sqrt();
    let physical_size_km = climate_scale.max(terrain_scale);
    
    println!("Simulation scale calculation:");
    println!("  Width: {}, Height: {}", width, height);
    println!("  Base area: {}, Current area: {}", base_area, current_area);
    println!("  Area ratio: {:.3}", area_ratio);
    println!("  Climate scale: {:.1}km", climate_scale);
    println!("  Terrain scale: {:.1}km", terrain_scale);
    println!("  Final physical size: {:.1}km", physical_size_km);
    
    let sim_scale = WorldScale::new(physical_size_km, (width as u32, height as u32), DetailLevel::Standard);
    println!("\nSimulation scale test:");
    test_coriolis_activation(&sim_scale);

    println!();

    // Test the exact threshold
    let threshold_scale = WorldScale::new(100.0, (width as u32, height as u32), DetailLevel::Standard);
    println!("Threshold scale test ({}km):", threshold_scale.physical_size_km);
    test_coriolis_activation(&threshold_scale);
}

fn test_coriolis_activation(world_scale: &WorldScale) {
    println!("  Physical extent: {:.1}km", world_scale.physical_size_km);
    println!("  Meters per pixel: {:.1}m", world_scale.meters_per_pixel());

    // Create atmospheric system and check parameters
    let atmospheric_params = AtmosphericParameters::default().derive_parameters(world_scale);
    let atmospheric_system = AtmosphericSystem::from_parameters(atmospheric_params, world_scale);
    
    println!("  Coriolis active: {}", atmospheric_system.is_coriolis_active());
    println!("  Geostrophic strength: {:.3}", atmospheric_system.parameters.geostrophic_strength);
    println!("  Coriolis threshold: {:.1}km", atmospheric_system.parameters.coriolis_activation_threshold_m / 1000.0);
    
    if !atmospheric_system.is_coriolis_active() {
        println!("  ⚠️  Coriolis effects disabled - no wind generation!");
    } else {
        println!("  ✅ Coriolis effects active - wind generation enabled");
    }
}