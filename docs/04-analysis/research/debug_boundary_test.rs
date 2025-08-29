// Debug boundary outflow test issue
use kosmarium::engine::core::heightmap::HeightMap;
use kosmarium::engine::sim::Simulation;

fn main() {
    println!("Debugging boundary outflow test...\n");
    
    // Replicate exact test conditions
    let heightmap = HeightMap::from_nested(vec![
        vec![1.0, 0.8, 0.6, 0.4, 0.2], // Steep slope toward right boundary
        vec![1.0, 0.8, 0.6, 0.4, 0.2],
        vec![1.0, 0.8, 0.6, 0.4, 0.2],
    ]);
    
    let mut test_sim = Simulation::new(heightmap);
    
    println!("Water System Parameters for 5x3 grid:");
    println!("  Effective rainfall rate: {:.8}", test_sim.water_system.effective_rainfall_rate);
    println!("  Evaporation rate: {:.6}", test_sim.water_system.parameters.evaporation_rate);
    println!("  Evaporation threshold: {:.8}", test_sim.water_system.evaporation_threshold);
    println!("  Flow threshold: {:.8}", test_sim.water_system.evaporation_threshold * 10.0);
    
    // Add water that will flow toward boundary - only left side
    for y in 0..3 {
        for x in 0..2 { // Only left side - water will flow toward right boundary
            test_sim.water.depth.set(x, y, 0.2); // 20cm water depth
        }
    }
    
    let initial_water = test_sim.water.get_total_water();
    println!("\nInitial Setup:");
    println!("  Water on left 2 columns: 0.2 m depth");
    println!("  Right 3 columns: 0.0 m depth");
    println!("  Total initial water: {:.6} m³", initial_water);
    
    println!("\nElevation profile (should cause flow to right):");
    for y in 0..3 {
        print!("  Row {}: ", y);
        for x in 0..5 {
            print!("{:.1} ", test_sim.heightmap.get(x, y));
        }
        println!();
    }
    
    println!("\nInitial water distribution:");
    for y in 0..3 {
        print!("  Row {}: ", y);
        for x in 0..5 {
            print!("{:.3} ", test_sim.water.depth.get(x, y));
        }
        println!();
    }
    
    // Step through first few ticks to see what happens
    println!("\n=== Tick Analysis ===");
    
    for tick in 0..5 {
        let pre_tick_water = test_sim.water.get_total_water();
        
        test_sim.tick();
        
        let post_tick_water = test_sim.water.get_total_water();
        let water_change = post_tick_water - pre_tick_water;
        
        println!("\nTick {}: {:.6} → {:.6} m³ (Δ {:.6})", 
                 tick + 1, pre_tick_water, post_tick_water, water_change);
        
        // Show water distribution
        println!("  Water distribution:");
        for y in 0..3 {
            print!("    Row {}: ", y);
            for x in 0..5 {
                print!("{:.3} ", test_sim.water.depth.get(x, y));
            }
            println!();
        }
        
        // Show velocities to diagnose flow
        println!("  Velocities:");
        for y in 0..3 {
            print!("    Row {}: ", y);
            for x in 0..5 {
                let (vx, vy) = test_sim.water.velocity.get(x, y);
                let vel_mag = (vx * vx + vy * vy).sqrt();
                print!("{:.3} ", vel_mag);
            }
            println!();
        }
        
        // Check if any water is flowing
        let mut max_velocity = 0.0f32;
        let mut cells_above_flow_threshold = 0;
        let flow_threshold = test_sim.water_system.evaporation_threshold * 10.0;
        
        for y in 0..3 {
            for x in 0..5 {
                let (vx, vy) = test_sim.water.velocity.get(x, y);
                let velocity_mag = (vx * vx + vy * vy).sqrt();
                let depth = test_sim.water.depth.get(x, y);
                let flow_amount = depth * velocity_mag.min(0.5);
                
                max_velocity = max_velocity.max(velocity_mag);
                if flow_amount > flow_threshold {
                    cells_above_flow_threshold += 1;
                }
            }
        }
        
        println!("  Max velocity: {:.6}", max_velocity);
        println!("  Cells above flow threshold: {}/{}", cells_above_flow_threshold, 15);
        println!("  Flow threshold: {:.8}", flow_threshold);
        
        if water_change.abs() < 1e-8 {
            println!("  → No water movement detected");
        }
    }
    
    println!("\n=== Analysis Complete ===");
}