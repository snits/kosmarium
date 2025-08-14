// ABOUTME: Test binary for corrected water flow physics implementation
// ABOUTME: Validates improvements and compares before/after diagnostic results

use sim_prototype::engine::core::heightmap::HeightMap;
use sim_prototype::engine::core::scale::{DetailLevel, WorldScale};
use sim_prototype::engine::physics::corrected_water_flow::CorrectedWaterFlowSystem;
use sim_prototype::engine::physics::water::WaterLayer;
use sim_prototype::engine::{WaterFlowDiagnostics, WaterFlowSystem};

fn main() {
    println!("=== CORRECTED WATER FLOW VALIDATION TEST ===");
    println!("Testing physics improvements following atmospheric success pattern");
    println!();

    // Test the same scales as before to compare improvements
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

        test_corrected_vs_original(&scale);
        println!();
    }

    println!("CORRECTED WATER FLOW TEST COMPLETE");
    println!("Expected: Similar improvements to atmospheric physics (99.6% reduction)");
}

fn test_corrected_vs_original(scale: &WorldScale) {
    // Create systems for comparison
    let base_system = WaterFlowSystem::new_for_scale(scale);
    let base_system_for_corrected = WaterFlowSystem::new_for_scale(scale);
    let corrected_system = CorrectedWaterFlowSystem::new(base_system_for_corrected, scale.clone());

    // Create identical test setup
    let width = scale.resolution.0 as usize;
    let height = scale.resolution.1 as usize;
    let mut heightmap = HeightMap::new(width, height, 1.0);

    // Create terrain slope for flow testing
    for y in 0..height {
        for x in 0..width {
            let slope_factor = x as f32 / width as f32;
            heightmap.set(x, y, 2.0 - slope_factor); // Decreasing from left to right
        }
    }

    // Test original system
    let mut water_original = create_test_water_setup(width, height);
    let mut diagnostics = WaterFlowDiagnostics::new(scale.clone());
    let validation_original =
        diagnostics.validate_water_flow_physics(&base_system, &heightmap, &water_original);

    // Test corrected system
    let mut water_corrected = create_test_water_setup(width, height);
    let mut heightmap_corrected = HeightMap::new(width, height, 1.0);

    // Recreate terrain slope for corrected system
    for y in 0..height {
        for x in 0..width {
            let slope_factor = x as f32 / width as f32;
            heightmap_corrected.set(x, y, 2.0 - slope_factor);
        }
    }

    // Run corrected physics for several timesteps
    let mut corrected_system = corrected_system;
    for _ in 0..5 {
        corrected_system.update_corrected_water_flow(
            &mut heightmap_corrected,
            &mut water_corrected,
            None,
        );
    }

    let validation_corrected = diagnostics.validate_water_flow_physics(
        &base_system,
        &heightmap_corrected,
        &water_corrected,
    );

    // Compare results
    println!("ORIGINAL vs CORRECTED COMPARISON:");
    println!(
        "Physics Quality Score: {:.3} → {:.3} ({:+.3})",
        validation_original.physics_quality_score,
        validation_corrected.physics_quality_score,
        validation_corrected.physics_quality_score - validation_original.physics_quality_score
    );

    println!(
        "Realistic Velocities: {:.1}% → {:.1}% ({:+.1}%)",
        validation_original
            .velocity_statistics
            .realistic_velocity_fraction
            * 100.0,
        validation_corrected
            .velocity_statistics
            .realistic_velocity_fraction
            * 100.0,
        (validation_corrected
            .velocity_statistics
            .realistic_velocity_fraction
            - validation_original
                .velocity_statistics
                .realistic_velocity_fraction)
            * 100.0
    );

    println!(
        "Max Velocity: {:.1} → {:.1} m/s ({:+.1})",
        validation_original.velocity_statistics.max_velocity_ms,
        validation_corrected.velocity_statistics.max_velocity_ms,
        validation_corrected.velocity_statistics.max_velocity_ms
            - validation_original.velocity_statistics.max_velocity_ms
    );

    println!(
        "CFL Stability: {} → {}",
        if validation_original.is_cfl_stable {
            "✓"
        } else {
            "❌"
        },
        if validation_corrected.is_cfl_stable {
            "✓"
        } else {
            "❌"
        }
    );

    // Calculate improvement metrics
    let velocity_improvement = if validation_original.velocity_statistics.max_velocity_ms > 0.0 {
        let original_excess =
            (validation_original.velocity_statistics.max_velocity_ms - 10.0).max(0.0);
        let corrected_excess =
            (validation_corrected.velocity_statistics.max_velocity_ms - 10.0).max(0.0);

        if original_excess > 0.0 {
            (1.0 - corrected_excess / original_excess) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    if velocity_improvement > 0.0 {
        println!(
            "⚡ VELOCITY IMPROVEMENT: {:.1}% reduction in excess velocity",
            velocity_improvement
        );
    }

    let realistic_improvement = (validation_corrected
        .velocity_statistics
        .realistic_velocity_fraction
        - validation_original
            .velocity_statistics
            .realistic_velocity_fraction)
        * 100.0;
    if realistic_improvement > 0.0 {
        println!(
            "🎯 REALISM IMPROVEMENT: {:.1}% more cells with realistic velocities",
            realistic_improvement
        );
    }

    // Show corrected system diagnostics
    let corrected_diagnostics = corrected_system.get_diagnostic_info();
    println!("Corrected System Parameters:");
    println!(
        "  H_MIN_THRESHOLD: {:.1e} m",
        corrected_diagnostics.h_min_threshold
    );
    println!(
        "  CFL_SAFETY_FACTOR: {:.3}",
        corrected_diagnostics.cfl_safety_factor
    );
    println!(
        "  Velocity Bounds: {:.2}-{:.1} m/s",
        corrected_diagnostics.velocity_bounds.0, corrected_diagnostics.velocity_bounds.1
    );
    println!("  Gravity: {:.2} m/s²", corrected_diagnostics.gravity);

    // Predict issues if still present
    if !validation_corrected.is_cfl_stable {
        println!("⚠️  REMAINING ISSUE: CFL instability detected");
    }
    if validation_corrected
        .velocity_statistics
        .realistic_velocity_fraction
        < 0.8
    {
        println!(
            "⚠️  REMAINING ISSUE: {:.1}% unrealistic velocities",
            (1.0 - validation_corrected
                .velocity_statistics
                .realistic_velocity_fraction)
                * 100.0
        );
    }
    if validation_corrected.physics_quality_score < 0.9 {
        println!("⚠️  REMAINING ISSUE: Physics quality score below target (0.9)");
    }
}

fn create_test_water_setup(width: usize, height: usize) -> WaterLayer {
    let mut water = WaterLayer::new(width, height);

    // Add varying water depths and velocities
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

            // Add test velocities (these should be corrected by the new physics)
            let vel_x = (x as f32 / width as f32 - 0.5) * 0.1;
            let vel_y = (y as f32 / height as f32 - 0.5) * 0.1;
            water.velocity.set(x, y, (vel_x, vel_y));
        }
    }

    water
}
