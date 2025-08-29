// ABOUTME: Demonstrates how water visualization thresholds scale with map size
// ABOUTME: Shows the relationship between evaporation thresholds and visualization levels

use kosmarium::scale::{DetailLevel, WorldScale};
use kosmarium::sim::{Simulation, WaterFlowSystem};
use kosmarium::worldgen::{DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator};

fn main() {
    println!("=== Water Visualization Threshold Scaling Demo ===\n");

    // Test different map sizes
    let map_sizes = [
        (60, 30, "Tiny"),
        (240, 120, "Default/Reference"),
        (480, 240, "Medium"),
        (1024, 512, "Large"),
        (2048, 1024, "Very Large"),
    ];

    for (width, height, name) in map_sizes {
        println!("{}x{} Map ({}):", width, height, name);

        // Create simulation for this size
        let generator = DiamondSquareGenerator::new(42);
        let config = DiamondSquareConfig::default();
        let heightmap = generator.generate(width, height, &config);
        let sim = Simulation::new(heightmap);

        let evaporation_threshold = sim.water_system.evaporation_threshold;
        let effective_rainfall = sim.water_system.effective_rainfall_rate;

        println!("  Evaporation threshold: {:.8}", evaporation_threshold);
        println!("  Effective rainfall: {:.8}", effective_rainfall);
        println!(
            "  Rainfall/threshold ratio: {:.1}x",
            effective_rainfall / evaporation_threshold
        );

        // Show what the visualization thresholds will be
        println!("  Water visualization levels:");
        println!("    Trace moisture (▒): > {:.8}", evaporation_threshold);
        println!(
            "    Light moisture (░): > {:.8}",
            evaporation_threshold * 1.5
        );
        println!(
            "    Shallow water (·): > {:.8}",
            evaporation_threshold * 2.5
        );
        println!("    Medium water (~): > {:.8}", evaporation_threshold * 4.0);
        println!("    Deep water (≋): > {:.8}", evaporation_threshold * 6.0);
        println!("    Very deep (■): > {:.8}", evaporation_threshold * 8.0);

        // Calculate steady-state water depth (rainfall / evaporation_rate)
        let steady_state_depth = effective_rainfall / sim.water_system.parameters.evaporation_rate;
        println!("  Expected steady-state depth: {:.8}", steady_state_depth);

        // Determine which visualization level steady-state would reach
        let vis_level = match steady_state_depth {
            x if x > evaporation_threshold * 8.0 => "Very deep pools (■)",
            x if x > evaporation_threshold * 6.0 => "Deep flowing water (≋)",
            x if x > evaporation_threshold * 4.0 => "Medium water (~)",
            x if x > evaporation_threshold * 2.5 => "Shallow water (·)",
            x if x > evaporation_threshold * 1.5 => "Light moisture (░)",
            x if x > evaporation_threshold => "Trace moisture (▒)",
            _ => "No visible water",
        };
        println!("  → Steady-state visualization: {}", vis_level);
        println!();
    }

    println!("=== Key Insights ===");
    println!("✅ Thresholds automatically scale with map size via evaporation_threshold");
    println!("✅ Large maps have smaller absolute thresholds but same relative scaling");
    println!("✅ All map sizes should reach similar steady-state visualization levels");
    println!("✅ Rainfall rate decreases on large maps but threshold decreases proportionally");
    println!("\n=== Demo Complete ===");
}
