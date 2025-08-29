// Debug water mass balance test issue
use kosmarium::engine::core::heightmap::HeightMap;
use kosmarium::engine::sim::Simulation;

fn main() {
    println!("Debugging water mass balance test issue...\n");
    
    // Replicate exact test conditions from test_water_flow_mass_conservation_basic
    let heightmap = HeightMap::from_nested(vec![
        vec![1.0, 0.8, 0.6, 0.4], // Simple slope from left to right
        vec![1.0, 0.8, 0.6, 0.4],
        vec![1.0, 0.8, 0.6, 0.4],
        vec![1.0, 0.8, 0.6, 0.4],
    ]);
    let mut test_sim = Simulation::new(heightmap);
    
    // Examine water system parameters for this scale
    println!("Water System Parameters for 4x4 grid:");
    println!("  Rainfall rate: {:.8}", test_sim.water_system.effective_rainfall_rate);
    println!("  Evaporation rate: {:.6}", test_sim.water_system.parameters.evaporation_rate);
    println!("  Evaporation threshold: {:.6}", test_sim.water_system.evaporation_threshold);
    
    // Set initial water distribution exactly like test
    let initial_water_per_cell = 0.1; // 10cm
    for y in 0..4 {
        for x in 0..4 {
            test_sim.water.depth.set(x, y, initial_water_per_cell);
        }
    }
    
    let initial_total_water = test_sim.water.get_total_water();
    println!("\nInitial Setup:");
    println!("  Water per cell: {:.6} m", initial_water_per_cell);
    println!("  Total initial water: {:.6} m³", initial_total_water);
    
    // Step through multiple ticks showing detailed water dynamics
    println!("\nDetailed Tick Analysis:");
    
    for tick in 0..5 {
        let pre_tick_water = test_sim.water.get_total_water();
        
        // Show water distribution before tick
        println!("\nBefore Tick {}: Total water {:.6} m³", tick + 1, pre_tick_water);
        print!("  Water depths: ");
        for y in 0..4 {
            for x in 0..4 {
                print!("{:.4} ", test_sim.water.depth.get(x, y));
            }
        }
        println!();
        
        test_sim.tick();
        
        let post_tick_water = test_sim.water.get_total_water();
        let water_change = post_tick_water - pre_tick_water;
        let relative_change = if pre_tick_water > 0.0 { water_change / pre_tick_water } else { 0.0 };
        
        // Show water distribution after tick
        println!("After Tick {}: Total water {:.6} m³ (Δ {:.6}, {:.2}%)", 
                 tick + 1, post_tick_water, water_change, relative_change * 100.0);
        print!("  Water depths: ");
        for y in 0..4 {
            for x in 0..4 {
                print!("{:.4} ", test_sim.water.depth.get(x, y));
            }
        }
        println!();
        
        // Calculate expected changes
        let expected_rainfall = test_sim.water_system.effective_rainfall_rate * 16.0; // 4x4 cells
        let avg_water_depth = post_tick_water / 16.0;
        let expected_evaporation_loss = pre_tick_water * test_sim.water_system.parameters.evaporation_rate;
        
        println!("  Expected rainfall: +{:.6} m³", expected_rainfall);
        println!("  Expected evaporation: -{:.6} m³", expected_evaporation_loss);
        println!("  Average depth after: {:.6} m", avg_water_depth);
        
        // Check if evaporation threshold is being triggered
        let cells_below_threshold = (0..4).map(|y| {
            (0..4).filter(|&x| test_sim.water.depth.get(x, y) < test_sim.water_system.evaporation_threshold).count()
        }).sum::<usize>();
        
        println!("  Cells below evaporation threshold: {}/16", cells_below_threshold);
        
        if water_change > pre_tick_water * 0.5 {
            println!("  ⚠️  WARNING: Water increased by more than 50% in one tick!");
        }
        if water_change > pre_tick_water * 5.0 {
            println!("  💀 CRITICAL: Water increased by more than 500% - mass balance failure!");
        }
    }
    
    println!("\n=== Analysis Complete ===");
}