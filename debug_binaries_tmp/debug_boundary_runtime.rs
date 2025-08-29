// Debug program to trace boundary condition runtime behavior
// ABOUTME: Systematically tracks wind layer boundary state during simulation updates
// ABOUTME: Identifies where boundary conditions get corrupted in the update cycle

use kosmarium::engine::{
    core::scale::{DetailLevel, WorldScale},
    physics::{DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator},
    sim::Simulation,
};

fn main() {
    println!("=== BOUNDARY CONDITION RUNTIME DEBUGGING ===");

    // Create continental-scale domain (same as Jerry's failing case)
    let width = 512;
    let height = 512;
    let scale = WorldScale::new(4096.0, (width as u32, height as u32), DetailLevel::Standard); // 4096km continental domain

    println!(
        "Domain: {}x{} cells, {:.0}km physical size",
        width, height, scale.physical_size_km
    );
    println!(
        "Resolution: {:.2}km per cell",
        scale.physical_size_km / (width.min(height) as f64)
    );

    // Create Diamond-Square terrain like the weather demo
    println!("Generating Diamond-Square terrain (matching weather demo)...");
    let seed = 42;
    let generator = DiamondSquareGenerator::new(seed);
    let config = DiamondSquareConfig {
        initial_corners: [0.3, 0.7, 0.4, 0.6], // Same as weather demo
        roughness: 0.5,                        // Default weather demo roughness
        persistence: 0.5,                      // Default weather demo persistence
        wrap_edges: false,
    };
    let heightmap = generator.generate(width, height, &config);

    // Create simulation (this will generate pressure and wind layers)
    println!("\n=== STEP 1: Creating Simulation ===");
    let mut simulation = Simulation::_new_with_scale(heightmap, scale);

    println!("\n=== STEP 2: Initial Wind Analysis ===");
    let initial_momentum = analyze_simulation_boundary_momentum(&simulation);
    println!("Initial boundary momentum analysis:");
    print_momentum_analysis(&initial_momentum);

    println!("\n=== STEP 3: Runtime Update Cycle Test ===");
    // Run simulation steps and track boundary condition behavior

    let mut momentum_history = Vec::new();
    momentum_history.push(initial_momentum.boundary_total);

    for step in 1..=10 {
        // Step the simulation (this is where boundary conditions might get corrupted)
        simulation.tick();

        // **CRITICAL TEST**: Call generate_biome_map() like the graphics renderer does!
        // This is suspected to be where boundary conditions get corrupted
        simulation.generate_biome_map();

        let step_momentum = analyze_simulation_boundary_momentum(&simulation);
        momentum_history.push(step_momentum.boundary_total);

        if step <= 5 {
            println!("\n--- Step {} ---", step);
            println!("Step {} momentum (after biome generation):", step);
            print_momentum_analysis(&step_momentum);
        }

        // Check for accumulation trend
        if step > 1 {
            let prev_momentum = momentum_history[step - 1];
            let current_momentum = step_momentum.boundary_total;
            let change_percent = ((current_momentum - prev_momentum) / prev_momentum) * 100.0;

            if change_percent.abs() > 5.0 {
                println!(
                    "⚠️  SIGNIFICANT MOMENTUM CHANGE in step {}: {:.2}%",
                    step, change_percent
                );
                println!("   This suggests biome generation corrupts boundary conditions!");
            }
        }
    }

    println!("\n=== STEP 4: Boundary Stability Analysis ===");

    // Analyze momentum drift over all steps
    let initial_momentum = momentum_history[0];
    let final_momentum = momentum_history[momentum_history.len() - 1];
    let total_drift = ((final_momentum - initial_momentum) / initial_momentum) * 100.0;

    println!("Momentum history over {} steps:", momentum_history.len());
    for (i, momentum) in momentum_history.iter().enumerate().take(5) {
        println!("  Step {}: {:.6}", i, momentum);
    }
    println!("  ... (showing first 5 steps)");
    println!("  Final step: {:.6}", final_momentum);

    if total_drift.abs() > 10.0 {
        println!(
            "⚠️  BOUNDARY DRIFT DETECTED: {:.2}% change over {} steps",
            total_drift,
            momentum_history.len() - 1
        );
        println!(
            "   This indicates boundary conditions are not properly maintained during simulation!"
        );
    } else {
        println!(
            "✅ Boundary conditions stable over {} steps (drift: {:.2}%)",
            momentum_history.len() - 1,
            total_drift
        );
    }

    // Detailed edge analysis for final state
    println!("\n=== STEP 5: Final Edge Analysis ===");
    let final_analysis = analyze_simulation_boundary_momentum(&simulation);
    println!("Final boundary analysis:");
    print_momentum_analysis(&final_analysis);
}

fn analyze_simulation_boundary_momentum(simulation: &Simulation) -> BoundaryMomentumAnalysis {
    // Use the simulation's wind layer data
    let wind_layer = simulation.get_wind_layer();
    analyze_boundary_momentum_from_wind_layer(wind_layer)
}

struct BoundaryMomentumAnalysis {
    boundary_total: f32,
    interior_total: f32,
    north_edge: f32,
    south_edge: f32,
    east_edge: f32,
    west_edge: f32,
    boundary_to_interior_ratio: f32,
}

fn analyze_boundary_momentum_from_wind_layer(
    wind_layer: &kosmarium::engine::physics::atmosphere::WindLayer,
) -> BoundaryMomentumAnalysis {
    let width = wind_layer.width();
    let height = wind_layer.height();
    let mut boundary_total = 0.0;
    let mut interior_total = 0.0;
    let mut north_edge = 0.0;
    let mut south_edge = 0.0;
    let mut east_edge = 0.0;
    let mut west_edge = 0.0;

    for y in 0..height {
        for x in 0..width {
            let velocity = wind_layer.get_velocity(x, y);
            let momentum_magnitude = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();

            // Classify cells as boundary or interior
            let is_boundary = x == 0 || x == width - 1 || y == 0 || y == height - 1;

            if is_boundary {
                boundary_total += momentum_magnitude;

                // Track specific edges
                if y == 0 {
                    north_edge += momentum_magnitude;
                }
                if y == height - 1 {
                    south_edge += momentum_magnitude;
                }
                if x == 0 {
                    west_edge += momentum_magnitude;
                }
                if x == width - 1 {
                    east_edge += momentum_magnitude;
                }
            } else {
                interior_total += momentum_magnitude;
            }
        }
    }

    let ratio = if interior_total > 0.0 {
        boundary_total / interior_total
    } else {
        0.0
    };

    BoundaryMomentumAnalysis {
        boundary_total,
        interior_total,
        north_edge,
        south_edge,
        east_edge,
        west_edge,
        boundary_to_interior_ratio: ratio,
    }
}

fn print_momentum_analysis(analysis: &BoundaryMomentumAnalysis) {
    println!("  Boundary total: {:.6}", analysis.boundary_total);
    println!("  Interior total: {:.6}", analysis.interior_total);
    println!(
        "  Boundary/Interior ratio: {:.4}",
        analysis.boundary_to_interior_ratio
    );
    println!("  Edge breakdown:");
    println!("    North: {:.6}", analysis.north_edge);
    println!("    South: {:.6}", analysis.south_edge);
    println!("    East:  {:.6}", analysis.east_edge);
    println!("    West:  {:.6}", analysis.west_edge);

    // Flag suspicious patterns
    if analysis.boundary_to_interior_ratio > 0.5 {
        println!("  ⚠️  HIGH BOUNDARY/INTERIOR RATIO - possible accumulation");
    }

    let max_edge = analysis
        .north_edge
        .max(analysis.south_edge)
        .max(analysis.east_edge)
        .max(analysis.west_edge);
    let min_edge = analysis
        .north_edge
        .min(analysis.south_edge)
        .min(analysis.east_edge)
        .min(analysis.west_edge);

    if max_edge > min_edge * 3.0 {
        println!("  ⚠️  UNEVEN EDGE DISTRIBUTION - possible boundary artifact");
    }
}
