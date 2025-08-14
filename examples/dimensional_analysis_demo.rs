// ABOUTME: Demonstration of dimensional analysis functionality for water flow physics
// ABOUTME: Shows how to validate physical parameters and convert between units

use sim_prototype::heightmap::HeightMap;
use sim_prototype::scale::{DetailLevel, WorldScale};
use sim_prototype::sim::Simulation;

fn main() {
    // Create a simple heightmap
    let heightmap = HeightMap::from_nested(vec![vec![0.5; 100]; 100]);

    // Create simulation with explicit world scale (10km physical size)
    let world_scale = WorldScale::new(10.0, (100, 100), DetailLevel::Standard);
    let sim = Simulation::_new_with_scale(heightmap, world_scale);

    println!("=== Dimensional Analysis Demo ===\n");

    // Get dimensional analysis
    let dimensional_params = sim.get_dimensional_analysis();

    println!("Physical Parameters:");
    println!("- Cell size: {:.1} m", dimensional_params.cell_size.value);
    println!(
        "- Max velocity: {:.2} m/s",
        dimensional_params.max_velocity.value
    );
    println!(
        "- Rainfall rate: {:.2} mm/h",
        dimensional_params.rainfall_rate.value
    );
    println!("- Timestep: {:.4} s", dimensional_params.timestep.value);
    println!(
        "- Depth threshold: {:.4} m",
        dimensional_params.depth_threshold.value
    );

    // Validate CFL condition
    let cfl_result = dimensional_params.validate_cfl_condition(0.5);
    println!("\nCFL Analysis:");
    println!("- CFL number: {:.4}", cfl_result.cfl_number);
    println!("- Is stable: {}", cfl_result.is_stable);
    if let Some(recommended_dt) = cfl_result.recommended_timestep_s {
        println!("- Recommended timestep: {:.4} s", recommended_dt);
    }

    // Check for any physical warnings
    println!("\nPhysical Validation:");
    let warnings = sim.validate_physics();
    if warnings.is_empty() {
        println!("✅ All parameters are physically reasonable");
    } else {
        for warning in warnings {
            println!("⚠️  {}", warning);
        }
    }

    // Show rainfall and evaporation rates in proper units
    let rainfall_depth = sim.get_physical_rainfall_rate();
    let evaporation_depth = sim.get_physical_evaporation_rate();

    println!("\nWater Budget (per timestep):");
    println!("- Rainfall: {:.6} m depth", rainfall_depth.value);
    println!("- Evaporation: {:.6} m depth", evaporation_depth.value);

    // Convert to different units for intuition
    let rainfall_mm =
        rainfall_depth.convert_to(sim_prototype::dimensional::PhysicalUnit::Millimeters);
    println!("- Rainfall: {:.6} mm depth", rainfall_mm.value);

    println!("\n=== Demo Complete ===");
}
