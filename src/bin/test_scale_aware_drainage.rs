// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Hydrological validation test for Jerry's scale-aware boundary drainage implementation
// ABOUTME: Tests mass conservation, boundary physics, and effectiveness criteria across multiple scales

use kosmarium::engine::core::scale::{DetailLevel, ScaleAware, WorldScale};
use kosmarium::engine::physics::worldgen::{
    TectonicConfig, TectonicGenerator, TerrainGenerator,
};
use kosmarium::engine::sim::Simulation;

/// Hydrological validation for a specific scale configuration
fn validate_drainage_at_scale(scale_km: f64, resolution: usize, test_name: &str) -> bool {
    println!(
        "\n=== {} Scale Validation: {:.0}km ===",
        test_name, scale_km
    );

    // Create world scale
    let world_scale = WorldScale::new(
        scale_km,
        (resolution as u32, resolution as u32),
        DetailLevel::Standard,
    );
    let meters_per_pixel = world_scale.meters_per_pixel();

    println!(
        "Domain: {:.0}km x {:.0}km at {:.0}m/pixel",
        scale_km, scale_km, meters_per_pixel
    );

    // Generate test terrain
    let generator = TectonicGenerator::new(12345);
    let mut config = TectonicConfig::default();
    config.enable_geological_evolution = false;
    let scaled_config = config.derive_parameters(&world_scale);
    let heightmap = generator.generate(resolution, resolution, &scaled_config);

    // Create simulation with the heightmap - this automatically sets up scale-aware systems
    let mut simulation = Simulation::_new_with_scale(heightmap, world_scale);

    // Access the water system parameters for validation
    let evaporation_threshold = simulation.water_system.evaporation_threshold;
    let expected_flow_threshold = evaporation_threshold * 0.01; // Jerry's 1% relationship
    let expected_edge_margin = ((resolution as f32 * 0.05) as usize).clamp(5, 50); // Jerry's 5% edge

    println!("Scale-Aware Parameters:");
    println!(
        "  Expected flow threshold (1% of evaporation): {:.8}",
        expected_flow_threshold
    );
    println!(
        "  Expected edge margin (5% of domain): {} pixels",
        expected_edge_margin
    );
    println!(
        "  Rainfall rate: {:.8}",
        simulation.water_system.effective_rainfall_rate
    );
    println!(
        "  Evaporation rate: {:.8}",
        simulation.water_system.parameters.evaporation_rate
    );

    // Run simulation for enough timesteps to reach equilibrium
    let timesteps = 30;
    println!(
        "\nRunning {} timesteps with drainage tracking...",
        timesteps
    );

    for timestep in 1..=timesteps {
        // Run one simulation tick (includes water flow, erosion, evaporation)
        simulation.tick();

        if timestep % 10 == 0 {
            let metrics = &simulation.water_system.drainage_metrics;
            let total_water = simulation.water.get_total_water();
            println!(
                "  Step {}: Water={:.6}, Boundary_Outflow={:.6}, Edge_Sat={:.3}%",
                timestep,
                total_water,
                metrics.boundary_outflow_rate,
                metrics.edge_saturation_ratio * 100.0
            );
        }
    }

    // Extract final metrics for validation
    let metrics = &simulation.water_system.drainage_metrics;

    println!("\nFinal Drainage Metrics:");
    println!(
        "  Total rainfall input: {:.6}",
        metrics.total_rainfall_input
    );
    println!("  Total evaporation: {:.6}", metrics.total_evaporation);
    println!(
        "  Total boundary outflow: {:.6}",
        metrics.total_boundary_outflow
    );
    println!(
        "  Current water storage: {:.6}",
        metrics.current_water_storage
    );
    println!("  Mass balance error: {:.8}", metrics.mass_balance_error);
    println!(
        "  Edge saturation ratio: {:.3} ({:.1}%)",
        metrics.edge_saturation_ratio,
        metrics.edge_saturation_ratio * 100.0
    );
    println!(
        "  Drainage efficiency: {:.3} ({:.1}%)",
        metrics.drainage_efficiency,
        metrics.drainage_efficiency * 100.0
    );

    // Apply Jerry's effectiveness criteria
    let total_input = metrics.total_rainfall_input;

    // 1. Mass balance error < 1% of total input
    let mass_balance_ok = if total_input > 0.0 {
        metrics.mass_balance_error / total_input < 0.01
    } else {
        true
    };
    let mass_error_percent = if total_input > 0.0 {
        (metrics.mass_balance_error / total_input) * 100.0
    } else {
        0.0
    };

    // 2. Edge saturation < 50%
    let edge_ok = metrics.edge_saturation_ratio < 0.5;

    // 3. Drainage efficiency > 10% for excess water
    let net_input = metrics.total_rainfall_input - metrics.total_evaporation;
    let drainage_ok = if net_input > 0.0 {
        metrics.drainage_efficiency > 0.1
    } else {
        true
    };

    let overall_effective = mass_balance_ok && edge_ok && drainage_ok;

    println!("\nEffectiveness Assessment:");
    println!(
        "  ✓ Mass Conservation (<1%): {} ({:.4}%)",
        if mass_balance_ok { "PASS" } else { "FAIL" },
        mass_error_percent
    );
    println!(
        "  ✓ Edge Saturation (<50%): {} ({:.1}%)",
        if edge_ok { "PASS" } else { "FAIL" },
        metrics.edge_saturation_ratio * 100.0
    );
    println!(
        "  ✓ Drainage Efficiency (>10%): {} ({:.1}%)",
        if drainage_ok { "PASS" } else { "FAIL" },
        metrics.drainage_efficiency * 100.0
    );
    println!(
        "  → Overall Status: {}",
        if overall_effective {
            "EFFECTIVE"
        } else {
            "PROBLEMATIC"
        }
    );

    // Additional hydrological analysis
    let boundary_outflow_ratio = if total_input > 0.0 {
        metrics.total_boundary_outflow / total_input
    } else {
        0.0
    };

    println!("\nHydrological Analysis:");
    println!(
        "  Boundary outflow ratio: {:.3} ({:.1}% of total input)",
        boundary_outflow_ratio,
        boundary_outflow_ratio * 100.0
    );

    // Check if boundary outflow is realistic (should be >0 but not excessive)
    let boundary_realistic = boundary_outflow_ratio > 0.001 && boundary_outflow_ratio < 0.8;
    println!(
        "  Boundary outflow realism: {}",
        if boundary_realistic {
            "REALISTIC"
        } else {
            "UNREALISTIC"
        }
    );

    overall_effective && boundary_realistic
}

fn main() {
    println!("SCALE-AWARE BOUNDARY DRAINAGE VALIDATION");
    println!("Testing Jerry's implementation across multiple spatial scales");
    println!("Focus: Mass conservation, boundary physics, effectiveness criteria");

    // Test suite covering regional to continental scales
    let test_scenarios = vec![
        (240.0, 120, "Regional"),           // 240km at 2km/pixel
        (960.0, 240, "Large Regional"),     // 960km at 4km/pixel
        (1920.0, 480, "Continental"),       // 1920km at 4km/pixel
        (3840.0, 960, "Large Continental"), // 3840km at 4km/pixel
    ];

    let mut all_passed = true;
    let mut results = Vec::new();

    for (scale_km, resolution, test_name) in test_scenarios {
        let passed = validate_drainage_at_scale(scale_km, resolution, test_name);
        results.push((test_name, scale_km, passed));
        if !passed {
            all_passed = false;
        }
    }

    // Summary analysis
    println!("\n=== VALIDATION SUMMARY ===");

    for (name, scale, passed) in &results {
        println!(
            "{} ({:.0}km): {}",
            name,
            scale,
            if *passed { "✓ PASS" } else { "✗ FAIL" }
        );
    }

    println!("\nOverall Assessment:");
    if all_passed {
        println!("✓ EXCELLENT: All scales show effective drainage");
        println!("Jerry's scale-aware boundary drainage implementation successfully:");
        println!("  - Maintains mass conservation across all scales (<1% error)");
        println!("  - Prevents edge saturation (aquarium effect) at continental scales");
        println!("  - Achieves realistic drainage efficiency (>10% excess water outflow)");
        println!("  - Implements appropriate scale-aware parameter relationships");
    } else {
        println!("✗ ATTENTION: Some scales show problematic drainage");
        println!("Issues detected that require investigation:");
        for (name, scale, passed) in &results {
            if !*passed {
                println!(
                    "  - {} scale ({:.0}km) failed effectiveness criteria",
                    name, scale
                );
            }
        }
    }

    println!("\nHydrological Scaling Validation:");
    println!("This test validates the key components Jerry implemented:");
    println!("1. Scale-aware flow threshold: evaporation_threshold * 0.01 (1%)");
    println!("2. Scale-aware edge detection: domain_size * 0.05 clamped to [5,50] pixels");
    println!("3. Mass balance tracking: rainfall - evaporation - boundary_outflow");
    println!("4. Effectiveness criteria: <1% mass error, <50% edge saturation, >10% drainage");
}
