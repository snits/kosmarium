// ABOUTME: Test binary for water flow diagnostic framework validation
// ABOUTME: Validates diagnostic system works and identifies physics issues before implementation

use sim_protoype::engine::core::heightmap::HeightMap;
use sim_protoype::engine::core::scale::{DetailLevel, WorldScale};
use sim_protoype::engine::physics::water::WaterLayer;
use sim_protoype::engine::{WaterFlowDiagnostics, WaterFlowSystem};

fn main() {
    println!("=== WATER FLOW DIAGNOSTIC FRAMEWORK TEST ===");
    println!("Testing diagnostic system before physics implementation fixes");
    println!();

    // Test different scales like atmospheric system
    let test_scales = vec![
        (
            "Small Domain",
            WorldScale::new(10.0, (50, 25), DetailLevel::Standard),
        ),
        (
            "Medium Domain",
            WorldScale::new(100.0, (120, 60), DetailLevel::Standard),
        ),
        (
            "Large Domain",
            WorldScale::new(1000.0, (240, 120), DetailLevel::Standard),
        ),
        (
            "Continental",
            WorldScale::new(10000.0, (480, 240), DetailLevel::Standard),
        ),
    ];

    for (scale_name, scale) in test_scales {
        println!(
            "--- Testing Scale: {} ({:.0} km domain) ---",
            scale_name, scale.physical_size_km
        );

        test_water_flow_diagnostics(&scale);
        println!();
    }

    println!("DIAGNOSTIC FRAMEWORK TEST COMPLETE");
    println!("Ready for Phase 3: Safety parameter derivation");
}

fn test_water_flow_diagnostics(scale: &WorldScale) {
    // Create test water flow system
    let water_system = WaterFlowSystem::new_for_scale(scale);

    // Create simple test terrain and water
    let width = scale.resolution.0 as usize;
    let height = scale.resolution.1 as usize;

    let heightmap = HeightMap::new(width, height, 1.0);
    let mut water = WaterLayer::new(width, height);

    // Add some test water with varying depths
    for y in 0..height {
        for x in 0..width {
            let depth = if x < width / 3 {
                0.5
            } else if x < 2 * width / 3 {
                1.0
            } else {
                2.0
            };
            water.add_water(x, y, depth);

            // Add some test velocities
            let vel_x = (x as f32 / width as f32 - 0.5) * 0.1;
            let vel_y = (y as f32 / height as f32 - 0.5) * 0.1;
            water.velocity.set(x, y, (vel_x, vel_y));
        }
    }

    // Create and run diagnostics
    let mut diagnostics = WaterFlowDiagnostics::new(scale.clone());
    let validation = diagnostics.validate_water_flow_physics(&water_system, &heightmap, &water);

    // Display results
    println!(
        "Physics Quality Score: {:.3}/1.0",
        validation.physics_quality_score
    );
    println!(
        "Mass Conservation: {} (error: {:.2e})",
        if validation.is_mass_conserved {
            "✓"
        } else {
            "❌"
        },
        validation.mass_conservation_error
    );
    println!(
        "CFL Stability: {} (violation: {:.2}x)",
        if validation.is_cfl_stable {
            "✓"
        } else {
            "❌"
        },
        validation.max_cfl_violation
    );
    println!(
        "Realistic Velocities: {:.1}%",
        validation.velocity_statistics.realistic_velocity_fraction * 100.0
    );
    println!(
        "Max Velocity: {:.3} m/s",
        validation.velocity_statistics.max_velocity_ms
    );

    // Test diagnostic report generation
    if validation.physics_quality_score < 0.8 {
        println!("\n--- DETAILED DIAGNOSTIC REPORT ---");
        let report = diagnostics.generate_diagnostic_report(&validation);
        println!("{}", report);
    }

    // Predictions for physics issues
    if validation.max_cfl_violation > 1.5 {
        println!("⚠️  PREDICTED: CFL instability will cause numerical artifacts");
    }
    if validation.velocity_statistics.realistic_velocity_fraction < 0.5 {
        println!("⚠️  PREDICTED: Unphysical flow velocities detected");
    }
    if !validation.is_mass_conserved {
        println!(
            "⚠️  PREDICTED: Mass conservation violations will cause water accumulation issues"
        );
    }
}
