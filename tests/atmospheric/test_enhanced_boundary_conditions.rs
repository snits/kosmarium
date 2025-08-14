// ABOUTME: Enhanced test program for atmospheric boundary conditions with sponge layer damping
// ABOUTME: Compares standard zero-gradient vs enhanced sponge layer boundary conditions for momentum conservation

use sim_prototype::engine::physics::atmosphere::{AtmosphericSystem, WindLayer};
// Removed unused ClimateSystem import
use sim_prototype::engine::core::scale::{WorldScale, DetailLevel};
use sim_prototype::engine::physics::water::Vec2;

fn main() {
    println!("Enhanced Atmospheric Boundary Conditions Test");
    println!("============================================");

    // Create a continental-scale domain (240x120 grid, 200km resolution -> 48,000km total)
    let scale = WorldScale::new(48000.0, (240, 120), DetailLevel::Standard);
    println!("Domain: {}x{} grid representing {:.0}km physical size", 
             scale.resolution.0, scale.resolution.1, scale.physical_size_km);
    println!("Resolution: {:.0}m per pixel", scale.meters_per_pixel());

    // Create atmospheric system
    let atmospheric_system = AtmosphericSystem::new_for_scale(&scale);
    println!("Coriolis effects active: {}", atmospheric_system.is_coriolis_active());

    // Test Case 1: Standard zero-gradient boundary conditions
    println!("\n=== TEST 1: Standard Zero-Gradient Boundary Conditions ===");
    test_boundary_conditions(&atmospheric_system, false, "Standard");

    // Test Case 2: Enhanced boundary conditions with sponge layer
    println!("\n=== TEST 2: Enhanced Sponge Layer Boundary Conditions ===");
    test_boundary_conditions(&atmospheric_system, true, "Enhanced");

    // Test Case 3: Real-world scenario with pressure gradients
    println!("\n=== TEST 3: Real-World Pressure Gradient Scenario ===");
    test_pressure_gradient_scenario(&atmospheric_system, &scale);

    println!("\n✓ All boundary condition tests completed!");
}

fn test_boundary_conditions(atmospheric_system: &AtmosphericSystem, use_sponge: bool, test_name: &str) {
    // Create wind field with systematic flow that would accumulate without proper boundaries
    let mut wind_layer = WindLayer::new(240, 120);
    
    println!("Setting up {} wind field...", test_name.to_lowercase());
    
    // Create a more realistic wind pattern: westward flow with some convergence
    for y in 1..119 {  // Interior region
        for x in 1..239 {
            // Westward flow with slight northward component and some convergence toward center
            let center_x = 120.0;
            let center_y = 60.0;
            let dx = (x as f32 - center_x) / 120.0; // -1 to 1
            let dy = (y as f32 - center_y) / 60.0;  // -1 to 1
            
            // Base westward flow with convergence toward center
            let u = -8.0 + 2.0 * dx; // Stronger westward flow away from center
            let v = 1.0 - 1.5 * dy;   // Slight northward with variation
            
            wind_layer.velocity[y][x] = Vec2::new(u, v);
        }
    }
    
    wind_layer.update_derived_fields();
    
    // Check before boundary conditions
    println!("Before {} boundary conditions:", test_name.to_lowercase());
    let metrics_before = wind_layer.check_boundary_stability();
    let momentum_before = wind_layer.calculate_total_momentum();
    println!("  Total momentum magnitude: {:.1}", momentum_before.magnitude());
    println!("  Avg edge momentum: {:.2} m/s", metrics_before.average_edge_momentum);
    println!("  Avg interior momentum: {:.2} m/s", metrics_before.average_interior_momentum);
    println!("  Accumulation ratio: {:.3}", metrics_before.accumulation_ratio);
    
    // Apply appropriate boundary conditions
    if use_sponge {
        wind_layer.apply_enhanced_outflow_boundary_conditions(true);
    } else {
        wind_layer.apply_outflow_boundary_conditions();
    }
    
    // Check after boundary conditions
    println!("After {} boundary conditions:", test_name.to_lowercase());
    let metrics_after = wind_layer.check_boundary_stability();
    let momentum_after = wind_layer.calculate_total_momentum();
    println!("  Total momentum magnitude: {:.1}", momentum_after.magnitude());
    println!("  Avg edge momentum: {:.2} m/s", metrics_after.average_edge_momentum);
    println!("  Avg interior momentum: {:.2} m/s", metrics_after.average_interior_momentum);
    println!("  Accumulation ratio: {:.3}", metrics_after.accumulation_ratio);
    
    // Calculate momentum reduction
    let momentum_reduction = ((momentum_before.magnitude() - momentum_after.magnitude()) 
                              / momentum_before.magnitude()) * 100.0;
    println!("  Momentum reduction: {:.1}%", momentum_reduction);
    
    // Validate stability
    let validation = atmospheric_system.validate_atmospheric_stability(&wind_layer);
    println!("  System stability:");
    println!("    Mass conserved: {}", validation.is_mass_conserved);
    println!("    Boundaries stable: {}", validation.boundary_stability.is_stable);
    println!("    Overall stable: {}", validation.is_system_stable);
    
    // Performance assessment
    if validation.is_system_stable {
        println!("  ✓ {} boundary conditions working correctly!", test_name);
    } else if validation.boundary_stability.is_stable {
        println!("  ⚠ {} boundary conditions prevent edge accumulation but momentum conservation needs improvement", test_name);
    } else {
        println!("  ❌ {} boundary conditions need further tuning", test_name);
    }
}

fn test_pressure_gradient_scenario(atmospheric_system: &AtmosphericSystem, scale: &WorldScale) {
    println!("Creating realistic pressure field with geostrophic winds...");
    
    let width = 240;
    let height = 120;
    
    // Create a simple pressure field manually for testing
    // Low pressure in the west, high pressure in the east (creates westward flow)
    let mut pressure_layer = sim_prototype::engine::physics::climate::AtmosphericPressureLayer::new(width, height);
    
    let base_pressure = 101325.0; // Standard sea level pressure (Pa)
    
    for y in 0..height {
        for x in 0..width {
            // Create east-west pressure gradient: lower pressure in west, higher in east
            let x_normalized = x as f32 / (width - 1) as f32; // 0 to 1 from west to east
            
            // Add a low pressure system in the center-west
            let center_x = (width as f32 * 0.3) as i32; // 30% from west
            let center_y = (height as f32 * 0.5) as i32; // Center north-south
            
            let dx = (x as i32 - center_x) as f32;
            let dy = (y as i32 - center_y) as f32;
            let distance_from_center = (dx * dx + dy * dy).sqrt();
            
            // Low pressure system: -1000 Pa at center, decreasing with distance
            let low_pressure_effect = -1000.0 * (-distance_from_center / 30.0).exp();
            
            // Background east-west gradient: +500 Pa from west to east
            let gradient_effect = 500.0 * (x_normalized - 0.5);
            
            let total_pressure = base_pressure + low_pressure_effect + gradient_effect;
            pressure_layer.pressure[y][x] = total_pressure;
        }
    }
    
    // Calculate pressure gradients
    pressure_layer.calculate_pressure_gradients(scale.meters_per_pixel() as f32);
    
    // Generate geostrophic winds from pressure gradients
    let wind_layer = atmospheric_system.generate_geostrophic_winds(&pressure_layer, scale);
    
    println!("Geostrophic wind field generated");
    println!("  Average wind speed: {:.2} m/s", wind_layer.get_average_wind_speed());
    
    // Check system stability  
    let validation = atmospheric_system.validate_atmospheric_stability(&wind_layer);
    println!("Real-world scenario validation:");
    println!("  Total momentum magnitude: {:.1}", validation.momentum_magnitude);
    println!("  Mass conserved: {}", validation.is_mass_conserved);
    println!("  Boundaries stable: {}", validation.boundary_stability.is_stable);
    println!("  Overall stable: {}", validation.is_system_stable);
    
    if validation.is_system_stable {
        println!("  ✓ Real-world atmospheric simulation is stable!");
    } else {
        println!("  ⚠ Real-world simulation shows some instability - may need parameter tuning");
    }
    
    // Analyze weather patterns
    let weather_analysis = atmospheric_system.analyze_weather_patterns(&pressure_layer, &wind_layer, scale);
    println!("Weather pattern analysis:");
    println!("  Detected {} weather patterns", weather_analysis.patterns.len());
    
    for (i, pattern) in weather_analysis.patterns.iter().enumerate().take(3) {
        println!("    Pattern {}: {:?} at ({}, {}) with {:.1} m/s winds", 
                 i + 1, pattern.pattern_type, pattern.center.0, pattern.center.1, pattern.max_wind_speed);
    }
}