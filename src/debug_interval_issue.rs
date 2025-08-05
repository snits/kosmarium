// ABOUTME: Test the water flow interval issue to confirm the 66% conservation error cause
// ABOUTME: Validates that the issue is flow update timing rather than actual water loss

use crate::engine::core::heightmap::HeightMap;
use crate::engine::core::scale::{DetailLevel, WorldScale};
use crate::engine::sim::Simulation;

/// Test the water flow interval hypothesis
pub fn test_flow_interval_conservation() {
    println!("=== FLOW INTERVAL CONSERVATION TEST ===\n");

    let heightmap = HeightMap::new(10, 10, 0.5); // Flat terrain to isolate the issue
    let world_scale = WorldScale::new(10.0, (10, 10), DetailLevel::Standard);
    let mut sim = Simulation::_new_with_scale(heightmap, world_scale);

    println!("Flow update interval: every 3 ticks");
    println!(
        "Rainfall rate per tick: {:.6}",
        sim.water_system.effective_rainfall_rate
    );
    println!(
        "Expected rainfall per cell per tick: {:.6}",
        sim.water_system.effective_rainfall_rate
    );
    println!("Total cells: {}", 10 * 10);
    println!(
        "Expected total rainfall per tick: {:.6}\n",
        sim.water_system.effective_rainfall_rate * 100.0
    );

    // Track water changes tick by tick
    let mut water_history = Vec::new();

    for tick in 0..10 {
        let water_before = sim.water.get_total_water();
        let will_flow_update = sim.tick_count % 3 == 0;

        println!(
            "Tick {} (before): Water = {:.6}, Flow update = {}",
            sim.tick_count, water_before, will_flow_update
        );

        sim.tick();

        let water_after = sim.water.get_total_water();
        let change = water_after - water_before;

        println!(
            "Tick {} (after):  Water = {:.6}, Change = {:.6}\n",
            sim.tick_count, water_after, change
        );

        water_history.push((
            sim.tick_count,
            water_before,
            water_after,
            change,
            will_flow_update,
        ));
    }

    // Analyze the pattern
    println!("=== ANALYSIS ===");

    let flow_update_changes: Vec<f32> = water_history
        .iter()
        .filter(|(_, _, _, _, will_update)| *will_update)
        .map(|(_, _, _, change, _)| *change)
        .collect();

    let no_flow_update_changes: Vec<f32> = water_history
        .iter()
        .filter(|(_, _, _, _, will_update)| !*will_update)
        .map(|(_, _, _, change, _)| *change)
        .collect();

    println!(
        "Water changes on flow-update ticks: {:?}",
        flow_update_changes
    );
    println!(
        "Water changes on non-flow-update ticks: {:?}",
        no_flow_update_changes
    );

    let avg_flow_update_change = if !flow_update_changes.is_empty() {
        flow_update_changes.iter().sum::<f32>() / flow_update_changes.len() as f32
    } else {
        0.0
    };

    let avg_no_flow_change = if !no_flow_update_changes.is_empty() {
        no_flow_update_changes.iter().sum::<f32>() / no_flow_update_changes.len() as f32
    } else {
        0.0
    };

    println!(
        "Average change on flow-update ticks: {:.6}",
        avg_flow_update_change
    );
    println!(
        "Average change on non-flow-update ticks: {:.6}",
        avg_no_flow_change
    );

    // Check the hypothesis
    if no_flow_update_changes.iter().all(|&x| x == 0.0) {
        println!("\n✅ HYPOTHESIS CONFIRMED:");
        println!("   - Water only changes on flow-update ticks");
        println!("   - Non-flow-update ticks have zero water change");
        println!("   - This explains the conservation error in diagnostics!");
    } else {
        println!("\n❌ HYPOTHESIS REJECTED:");
        println!("   - Water changes on non-flow-update ticks too");
        println!("   - The interval timing is not the issue");
    }
}

/// Test what happens if we force a flow update every tick
pub fn test_continuous_flow_updates() {
    println!("\n=== CONTINUOUS FLOW UPDATES TEST ===\n");

    let heightmap = HeightMap::new(5, 5, 0.5);
    let world_scale = WorldScale::new(10.0, (5, 5), DetailLevel::Standard);
    let mut sim = Simulation::_new_with_scale(heightmap, world_scale);

    println!("Forcing water flow update every tick (bypassing interval)...\n");

    let mut total_rainfall_added = 0.0f32;
    let mut total_evaporation_estimate = 0.0f32;

    for tick in 0..5 {
        let water_before = sim.water.get_total_water();

        // Force a water flow update by calling the system directly
        sim.water_system
            .update_water_flow_with_climate_and_drainage(
                &mut sim.heightmap,
                &mut sim.water,
                &sim.temperature_layer,
                &sim.climate_system,
                &sim.drainage_network,
            );

        // Also advance climate (this is what tick() does)
        sim.climate_system.tick();
        sim.tick_count += 1;

        let water_after = sim.water.get_total_water();
        let change = water_after - water_before;

        let expected_rainfall = sim.water_system.effective_rainfall_rate * 25.0; // 5x5 cells
        total_rainfall_added += expected_rainfall;

        // Rough evaporation estimate
        let rough_evaporation = water_before * sim.water_system.parameters.evaporation_rate;
        total_evaporation_estimate += rough_evaporation;

        println!(
            "Tick {}: Water = {:.6} -> {:.6}, Change = {:.6}",
            tick, water_before, water_after, change
        );
        println!(
            "   Expected rainfall: {:.6}, Rough evaporation: {:.6}",
            expected_rainfall, rough_evaporation
        );
    }

    let final_water = sim.water.get_total_water();
    let expected_net = total_rainfall_added - total_evaporation_estimate;

    println!("\n=== CONTINUOUS FLOW RESULTS ===");
    println!("Final water: {:.6}", final_water);
    println!("Total rainfall added: {:.6}", total_rainfall_added);
    println!(
        "Total evaporation estimate: {:.6}",
        total_evaporation_estimate
    );
    println!("Expected net: {:.6}", expected_net);
    println!("Actual net: {:.6}", final_water);
    println!(
        "Conservation error: {:.6} ({:.1}%)",
        final_water - expected_net,
        if expected_net != 0.0 {
            100.0 * (final_water - expected_net) / expected_net
        } else {
            0.0
        }
    );
}

/// Detailed tick-by-tick analysis of what happens during simulation
pub fn analyze_tick_details() {
    println!("\n=== DETAILED TICK ANALYSIS ===\n");

    let heightmap = HeightMap::new(3, 3, 0.5);
    let world_scale = WorldScale::new(10.0, (3, 3), DetailLevel::Standard);
    let mut sim = Simulation::_new_with_scale(heightmap, world_scale);

    println!("Analyzing exactly what happens in each phase of tick()...\n");

    for tick in 0..6 {
        println!("--- TICK {} ---", tick);
        let initial_water = sim.water.get_total_water();
        println!("Initial water: {:.6}", initial_water);

        // Check if this tick will update water flow
        let will_update_water = sim.tick_count % 3 == 0;
        println!("Will update water flow: {}", will_update_water);

        // Manual step through what tick() does:

        // 1. Climate system advances
        sim.climate_system.tick();
        let after_climate = sim.water.get_total_water();
        println!(
            "After climate advance: {:.6} (change: {:.6})",
            after_climate,
            after_climate - initial_water
        );

        // 2. Atmospheric updates (don't affect water directly)
        // ... atmospheric update logic happens here but doesn't change water

        // 3. Water flow update (conditional)
        if will_update_water {
            println!("Performing water flow update...");
            sim.water_system
                .update_water_flow_with_climate_and_drainage(
                    &mut sim.heightmap,
                    &mut sim.water,
                    &sim.temperature_layer,
                    &sim.climate_system,
                    &sim.drainage_network,
                );
            let after_water_flow = sim.water.get_total_water();
            println!(
                "After water flow: {:.6} (change: {:.6})",
                after_water_flow,
                after_water_flow - after_climate
            );
        } else {
            println!("Skipping water flow update (interval)");
        }

        // 4. Increment tick counter
        sim.tick_count += 1;

        let final_water = sim.water.get_total_water();
        println!(
            "Final water: {:.6} (total change: {:.6})\n",
            final_water,
            final_water - initial_water
        );
    }
}
