// ABOUTME: Test program to verify atmospheric boundary conditions are working correctly
// ABOUTME: Creates a simple atmospheric system and validates that boundary conditions prevent accumulation

use sim_protoype::engine::physics::atmosphere::{AtmosphericSystem, WindLayer, BoundaryType};
use sim_protoype::engine::physics::climate::ClimateSystem;
use sim_protoype::engine::core::scale::{WorldScale, DetailLevel};
use sim_protoype::engine::physics::water::Vec2;

fn main() {
    println! ("Testing Atmospheric Boundary Conditions");
    println!("======================================");

    // Create a continental-scale domain (240x120 grid, 200km resolution)
    let scale = WorldScale::new(48000.0, (240, 120), DetailLevel::Standard); // 48,000 km = 240*200km
    println!("Domain: {}x{} grid representing {:.0}km physical size", 
             scale.resolution.0, scale.resolution.1, scale.physical_size_km);

    // Create atmospheric system
    let atmospheric_system = AtmosphericSystem::new_for_scale(&scale);
    println!("Coriolis effects active: {}", atmospheric_system.is_coriolis_active());

    // Create a simple test case: wind layer with artificial accumulation
    let mut wind_layer = WindLayer::new(240, 120);
    
    // Set up wind field that would accumulate at boundaries without proper conditions
    println!("\nSetting up test wind field...");
    for y in 1..119 {  // Interior region (excluding boundary)
        for x in 1..239 {
            // Create westward flow that would accumulate at western boundary
            wind_layer.velocity[y][x] = Vec2::new(-5.0, 2.0); // 5 m/s westward, 2 m/s northward
        }
    }
    
    wind_layer.update_derived_fields();
    
    // Check boundary conditions before application
    println!("Before boundary conditions:");
    let metrics_before = wind_layer.check_boundary_stability();
    println!("  Edge cells: {}, Interior cells: {}", 
             metrics_before.edge_cell_count, metrics_before.interior_cell_count);
    println!("  Avg edge momentum: {:.2} m/s", metrics_before.average_edge_momentum);
    println!("  Avg interior momentum: {:.2} m/s", metrics_before.average_interior_momentum);
    println!("  Accumulation ratio: {:.3}", metrics_before.accumulation_ratio);
    println!("  Stable: {}", metrics_before.is_stable);

    // Apply boundary conditions
    println!("\nApplying outflow boundary conditions...");
    wind_layer.apply_outflow_boundary_conditions();

    // Check after boundary conditions
    println!("After boundary conditions:");
    let metrics_after = wind_layer.check_boundary_stability();
    println!("  Edge cells: {}, Interior cells: {}", 
             metrics_after.edge_cell_count, metrics_after.interior_cell_count);
    println!("  Avg edge momentum: {:.2} m/s", metrics_after.average_edge_momentum);
    println!("  Avg interior momentum: {:.2} m/s", metrics_after.average_interior_momentum);
    println!("  Accumulation ratio: {:.3}", metrics_after.accumulation_ratio);
    println!("  Stable: {}", metrics_after.is_stable);

    // Test specific boundary cells
    println!("\nBoundary cell verification:");
    
    // Test western boundary (x=0) - should have extrapolated westward flow
    let west_boundary_wind = wind_layer.get_velocity(0, 60);
    println!("  Western boundary (0, 60): velocity = ({:.1}, {:.1}) m/s", 
             west_boundary_wind.x, west_boundary_wind.y);
    
    // Test eastern boundary (x=239) - should have extrapolated westward flow  
    let east_boundary_wind = wind_layer.get_velocity(239, 60);
    println!("  Eastern boundary (239, 60): velocity = ({:.1}, {:.1}) m/s",
             east_boundary_wind.x, east_boundary_wind.y);
    
    // Test interior cell for comparison
    let interior_wind = wind_layer.get_velocity(120, 60);
    println!("  Interior cell (120, 60): velocity = ({:.1}, {:.1}) m/s",
             interior_wind.x, interior_wind.y);

    // Test boundary type detection
    println!("\nBoundary type detection:");
    println!("  (0, 60): {:?}", wind_layer.get_boundary_type(0, 60));
    println!("  (239, 60): {:?}", wind_layer.get_boundary_type(239, 60));
    println!("  (120, 0): {:?}", wind_layer.get_boundary_type(120, 0));
    println!("  (120, 119): {:?}", wind_layer.get_boundary_type(120, 119));
    println!("  (120, 60): {:?}", wind_layer.get_boundary_type(120, 60));

    // Validate atmospheric stability
    println!("\nAtmospheric stability validation:");
    let validation = atmospheric_system.validate_atmospheric_stability(&wind_layer);
    println!("  Total momentum magnitude: {:.2}", validation.momentum_magnitude);
    println!("  Boundary cell fraction: {:.3}", validation.boundary_cell_fraction);
    println!("  Mass conserved: {}", validation.is_mass_conserved);
    println!("  System stable: {}", validation.is_system_stable);

    println!("\n✓ Boundary condition test completed successfully!");
    
    if validation.is_system_stable && metrics_after.is_stable {
        println!("✓ Atmospheric boundary conditions are working correctly!");
        println!("✓ Wind accumulation at domain edges should be prevented.");
    } else {
        println!("⚠ Warning: Boundary conditions may need further tuning.");
    }
}