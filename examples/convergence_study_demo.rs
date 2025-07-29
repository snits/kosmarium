// ABOUTME: Demonstration of grid convergence testing framework for simulation validation
// ABOUTME: Shows how the simulation behaves as grid resolution increases

use sim_protoype::convergence::{ConvergenceStudy, ConvergenceStudyConfig};

fn main() {
    println!("=== Grid Convergence Study Demo ===\n");

    // Configure convergence study
    let config = ConvergenceStudyConfig {
        domain_size_km: 10.0,
        base_resolution: 50,
        refinement_factor: 2,
        num_levels: 4, // Test 50x50, 100x100, 200x200, 400x400
        simulation_steps: 15,
        test_water_amount: 1.0,
        terrain_seed: 42,
    };

    println!("Study Configuration:");
    println!("- Domain size: {} km", config.domain_size_km);
    println!(
        "- Base resolution: {}x{}",
        config.base_resolution, config.base_resolution
    );
    println!("- Refinement factor: {}", config.refinement_factor);
    println!("- Number of levels: {}", config.num_levels);
    println!("- Simulation steps: {}", config.simulation_steps);
    println!("- Test water amount: {}", config.test_water_amount);

    // Run convergence study
    println!("\nRunning convergence study...");
    let study = ConvergenceStudy::new(config);
    let result = study.run_study();

    // Display results
    println!("\n=== Convergence Results ===");

    println!("\nGrid Resolution Analysis:");
    println!(
        "{:>10} {:>15} {:>15} {:>15} {:>15}",
        "Resolution", "Grid Spacing", "Total Water", "Max Depth", "Entropy"
    );
    println!(
        "{:->10} {:->15} {:->15} {:->15} {:->15}",
        "", "", "", "", ""
    );

    for metric in &result.metrics {
        println!(
            "{:>10} {:>15.2} {:>15.6} {:>15.6} {:>15.6}",
            format!("{}x{}", metric.resolution, metric.resolution),
            metric.grid_spacing,
            metric.total_water,
            metric.max_water_depth,
            metric.water_entropy
        );
    }

    // Convergence analysis
    println!("\n=== Convergence Analysis ===");

    if let Some(order) = result.convergence_order {
        println!("Estimated convergence order: {:.2}", order);

        if order > 0.5 && order < 3.0 {
            println!("✅ Convergence order is reasonable for numerical simulation");
        } else if order < 0.5 {
            println!("⚠️  Low convergence order - solution may not be converging properly");
        } else {
            println!("⚠️  High convergence order - might indicate numerical artifacts");
        }
    } else {
        println!("❌ Could not estimate convergence order");
    }

    println!(
        "Overall convergence: {}",
        if result.is_converged {
            "✅ CONVERGED"
        } else {
            "❌ NOT CONVERGED"
        }
    );

    // Display warnings
    if !result.warnings.is_empty() {
        println!("\n=== Warnings ===");
        for warning in &result.warnings {
            println!("⚠️  {}", warning);
        }
    }

    // Analysis of solution behavior
    println!("\n=== Solution Analysis ===");

    if result.metrics.len() >= 2 {
        // Check water conservation
        let first_water = result.metrics[0].total_water;
        let last_water = result.metrics.last().unwrap().total_water;
        let conservation_error = ((last_water - first_water) / first_water).abs();

        println!("Water conservation:");
        println!("- Coarsest grid: {:.6}", first_water);
        println!("- Finest grid: {:.6}", last_water);
        println!("- Relative change: {:.4}%", conservation_error * 100.0);

        if conservation_error < 0.05 {
            println!("✅ Good water conservation (< 5% change)");
        } else if conservation_error < 0.20 {
            println!("⚠️  Moderate water conservation (5-20% change)");
        } else {
            println!("❌ Poor water conservation (> 20% change)");
        }

        // Check solution smoothness
        println!("\nSolution smoothness:");
        for i in 1..result.metrics.len() {
            let prev = &result.metrics[i - 1];
            let curr = &result.metrics[i];
            let change = ((curr.total_water - prev.total_water) / prev.total_water).abs();

            println!(
                "- {}x{} → {}x{}: {:.2}% change",
                prev.resolution,
                prev.resolution,
                curr.resolution,
                curr.resolution,
                change * 100.0
            );
        }
    }

    println!("\n=== Recommendations ===");

    if result.is_converged {
        let finest_resolution = result.metrics.last().unwrap().resolution;
        let grid_spacing = result.metrics.last().unwrap().grid_spacing;

        println!("✅ Simulation shows good convergence behavior");
        println!(
            "✅ Finest tested resolution ({}x{}) appears adequate",
            finest_resolution, finest_resolution
        );
        println!(
            "✅ Grid spacing of {:.1}m provides good balance of accuracy and performance",
            grid_spacing
        );

        if let Some(order) = result.convergence_order {
            if order > 1.0 {
                println!("✅ Convergence order > 1 indicates high-quality numerical method");
            }
        }
    } else {
        println!("⚠️  Consider investigating numerical stability");
        println!("⚠️  May need finer grids or adjusted simulation parameters");

        if result.warnings.iter().any(|w| w.contains("conservation")) {
            println!("⚠️  Water conservation issues detected - check boundary conditions");
        }
    }

    println!("\n=== Demo Complete ===");
}
