use sim_prototype::engine::core::heightmap::HeightMap;
use sim_prototype::engine::sim::Simulation;

fn main() {
    println!("=== WORKING TEST SETUP (4x4) ===");
    let heightmap_4x4 = HeightMap::from_nested(vec![
        vec![1.0, 0.8, 0.6, 0.4], // Simple slope from left to right
        vec![1.0, 0.8, 0.6, 0.4],
        vec![1.0, 0.8, 0.6, 0.4],
        vec![1.0, 0.8, 0.6, 0.4],
    ]);
    let sim_4x4 = Simulation::new(heightmap_4x4);

    println!(
        "Effective rainfall rate: {:.9}",
        sim_4x4.water_system.effective_rainfall_rate
    );
    println!("Grid size: 4x4 = {} cells", 16);
    println!(
        "Expected rainfall per update: {:.9}",
        16.0 * sim_4x4.water_system.effective_rainfall_rate
    );

    println!("\n=== FAILING TEST SETUP (25x25) ===");
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
    let heightmap_25x25 = HeightMap::from_nested(heightmap_data);
    let sim_25x25 = Simulation::new(heightmap_25x25);

    println!(
        "Effective rainfall rate: {:.9}",
        sim_25x25.water_system.effective_rainfall_rate
    );
    println!("Grid size: 25x25 = {} cells", 625);
    println!(
        "Expected rainfall per update: {:.9}",
        625.0 * sim_25x25.water_system.effective_rainfall_rate
    );

    println!("\n=== COMPARISON ===");
    let ratio_4x4_per_cell = sim_4x4.water_system.effective_rainfall_rate;
    let ratio_25x25_per_cell = sim_25x25.water_system.effective_rainfall_rate;
    println!("4x4 rainfall per cell: {:.9}", ratio_4x4_per_cell);
    println!("25x25 rainfall per cell: {:.9}", ratio_25x25_per_cell);
    println!(
        "Scale factor: {:.3}x",
        ratio_25x25_per_cell / ratio_4x4_per_cell
    );

    // Expected with mass-conserving scaling: smaller grid should have higher per-cell rainfall
    println!("\nExpected mass-conserving behavior:");
    let area_ratio = 625.0 / 16.0; // 25x25 vs 4x4 
    println!("Area ratio (25x25 vs 4x4): {:.2}", area_ratio);
    println!(
        "Expected 4x4 should have {}x higher rainfall per cell",
        area_ratio
    );
    println!(
        "Actual ratio: {:.3}x",
        ratio_4x4_per_cell / ratio_25x25_per_cell
    );
}
