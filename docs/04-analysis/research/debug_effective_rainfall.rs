use kosmarium::engine::core::heightmap::HeightMap;
use kosmarium::engine::sim::Simulation;

fn main() {
    // Create the same terrain as the failing test
    let mut heightmap_data = vec![vec![0.0; 25]; 25];
    for y in 0..25 {
        for x in 0..25 {
            let center1_dist = ((x as f32 - 8.0).powi(2) + (y as f32 - 8.0).powi(2)).sqrt();
            let center2_dist = ((x as f32 - 16.0).powi(2) + (y as f32 - 16.0).powi(2)).sqrt();
            let elevation =
                0.5 + 0.3 * (-center1_dist / 3.0).exp() + 0.4 * (-center2_dist / 4.0).exp();
            heightmap_data[y][x] = elevation;
        }
    }
    let heightmap = HeightMap::from_nested(heightmap_data);

    let simulation = Simulation::new(heightmap);

    println!(
        "Water system effective rainfall rate: {:.9}",
        simulation.water_system.effective_rainfall_rate
    );
    println!(
        "Base water amount (rainfall/10): {:.9}",
        simulation.water_system.effective_rainfall_rate / 10.0
    );
    println!("Grid size: 25x25 = {} cells", 25 * 25);
    println!(
        "Expected total initial water: {:.9}",
        (25 * 25) as f32 * simulation.water_system.effective_rainfall_rate / 10.0
    );

    // Check actual initial water after initialization
    let actual_initial_water = simulation.water.get_total_water();
    println!(
        "Actual initial water after init: {:.9}",
        actual_initial_water
    );

    // Calculate what one rainfall tick should add
    let rainfall_per_tick = (25 * 25) as f32 * simulation.water_system.effective_rainfall_rate;
    println!("Expected rainfall per tick: {:.9}", rainfall_per_tick);
}
