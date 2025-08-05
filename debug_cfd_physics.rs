// ABOUTME: CFD physics validation program for diagnosing water flow issues at high resolution
// ABOUTME: Tests mass conservation, velocity scaling, and pressure coupling to identify lake buildup causes

use desert_island_sim::engine::core::heightmap::HeightMap;
use desert_island_sim::engine::core::scale::{DetailLevel, WorldScale};
use desert_island_sim::engine::physics::climate::ClimateSystem;
use desert_island_sim::engine::sim::{Simulation, WaterFlowSystem, WaterFlowParameters, RainfallScaling};

fn main() {
    println!("=== CFD PHYSICS VALIDATION FOR LAKE BUILDUP DIAGNOSIS ===\n");
    
    // Test the problematic 512x256 configuration
    let width = 512;
    let height = 256;
    let domain_size_km = 4096.0;
    
    println!("Testing Configuration:");
    println!("  Resolution: {}x{}", width, height);
    println!("  Domain Size: {:.0} km", domain_size_km);
    println!("  Pixel Resolution: {:.1} km/pixel", domain_size_km / width as f64);
    println!();
    
    // Create world scale
    let world_scale = WorldScale::new(domain_size_km, (width as u32, height as u32), DetailLevel::Standard);
    
    // 1. VELOCITY SCALING ANALYSIS
    println!("=== 1. VELOCITY SCALING ANALYSIS ===");
    analyze_velocity_scaling(&world_scale);
    println!();
    
    // 2. PRESSURE COUPLING ANALYSIS  
    println!("=== 2. THERMAL CIRCULATION PRESSURE COUPLING ===");
    analyze_pressure_coupling(&world_scale);
    println!();
    
    // 3. MASS CONSERVATION TEST
    println!("=== 3. MASS CONSERVATION TEST ===");
    test_mass_conservation(&world_scale);
    println!();
    
    // 4. CFL STABILITY CHECK
    println!("=== 4. CFL STABILITY ANALYSIS ===");
    analyze_cfl_stability(&world_scale);
    println!();
    
    // 5. RAINFALL/EVAPORATION BALANCE
    println!("=== 5. RAINFALL/EVAPORATION BALANCE ===");
    analyze_water_balance(&world_scale);
    println!();
    
    println!("=== CFD DIAGNOSIS COMPLETE ===");
    println!("Review the analysis above to identify physics violations causing lake buildup.");
}

fn analyze_velocity_scaling(scale: &WorldScale) {
    let meters_per_pixel = scale.meters_per_pixel() as f32;
    
    println!("Velocity Scaling Analysis:");
    println!("  Grid Resolution: {:.0} m/pixel", meters_per_pixel);
    
    // Current velocity limiting
    let current_velocity_limit = 1.0; // Hard-coded .min(1.0) in current code
    let current_physical_velocity = current_velocity_limit * meters_per_pixel / 360.0; // 6-minute timestep
    
    println!("  Current Velocity Limit: {:.1} simulation units", current_velocity_limit);
    println!("  Physical Velocity (6min timestep): {:.1} m/s", current_physical_velocity);
    
    // Recommended velocity limiting
    let max_physical_velocity = 2.0; // Realistic water flow velocity (m/s)
    let recommended_limit = max_physical_velocity * 360.0 / meters_per_pixel; // Convert to simulation units
    
    println!("  Recommended Physical Limit: {:.1} m/s", max_physical_velocity);
    println!("  Recommended Simulation Limit: {:.3} simulation units", recommended_limit);
    
    if current_velocity_limit > recommended_limit * 2.0 {
        println!("  ⚠️  CRITICAL: Current velocity limit is {:.1}x too high!", 
                current_velocity_limit / recommended_limit);
        println!("      This prevents proper drainage at high resolution!");
    } else {
        println!("  ✅ Velocity scaling appears reasonable");
    }
}

fn analyze_pressure_coupling(scale: &WorldScale) {
    let climate_system = ClimateSystem::new_for_scale(scale);
    let coupling = climate_system.parameters.pressure_temperature_coupling;
    
    println!("Pressure-Temperature Coupling Analysis:");
    println!("  Domain Size: {:.0} km", scale.physical_size_km);
    println!("  Resolution: {:.0} m/pixel", scale.meters_per_pixel());
    println!("  Pressure Coupling: {:.1} Pa/°C", coupling);
    
    // Realistic pressure coupling should be ~50-100 Pa/°C for continental scales
    let realistic_coupling_max = 100.0;
    
    if coupling > realistic_coupling_max * 2.0 {
        println!("  ⚠️  CRITICAL: Pressure coupling is {:.1}x too strong!", 
                coupling / realistic_coupling_max);
        println!("      This creates artificial pressure gradients that disrupt water flow!");
    } else if coupling > realistic_coupling_max {
        println!("  ⚠️  WARNING: Pressure coupling is {:.1}x higher than realistic", 
                coupling / realistic_coupling_max);
    } else {
        println!("  ✅ Pressure coupling appears reasonable");
    }
    
    // Calculate expected pressure gradient magnitude
    let temp_variation = 20.0; // °C across domain
    let pressure_variation = coupling * temp_variation;
    let domain_extent = scale.physical_size_km * 1000.0; // Convert to meters
    let pressure_gradient = pressure_variation / domain_extent;
    
    println!("  Expected pressure gradient: {:.3} Pa/m", pressure_gradient);
    
    if pressure_gradient > 0.01 {
        println!("  ⚠️  WARNING: Pressure gradients may be unrealistically strong");
    }
}

fn test_mass_conservation(scale: &WorldScale) {
    // Create a simple test heightmap (flat terrain)
    let width = 10;
    let height = 10;
    let heightmap = HeightMap::new(width, height, 0.5);
    
    // Create a small-scale simulation for testing
    let test_scale = WorldScale::new(10.0, (width as u32, height as u32), DetailLevel::Standard);
    let mut sim = Simulation::new_with_scale(heightmap, test_scale);
    
    // Add initial water
    let initial_water = 100.0;
    sim.add_water_at(5, 5, initial_water);
    let start_total = sim.get_water_layer().get_total_water();
    
    println!("Mass Conservation Test:");
    println!("  Initial Water: {:.3}", start_total);
    
    // Run several timesteps and track water
    let mut water_history = vec![start_total];
    for tick in 1..=10 {
        sim.tick();
        let current_water = sim.get_water_layer().get_total_water();
        water_history.push(current_water);
        
        if tick <= 3 {
            println!("  Tick {}: {:.3} water", tick, current_water);
        }
    }
    
    let final_water = water_history.last().unwrap();
    let water_loss = start_total - final_water;
    let loss_rate = water_loss / 10.0; // Per tick
    
    println!("  Final Water: {:.3}", final_water);
    println!("  Total Loss: {:.3} ({:.1}%)", water_loss, (water_loss/start_total)*100.0);
    println!("  Loss Rate: {:.3} per tick", loss_rate);
    
    // Check if loss is reasonable (should be mostly evaporation)
    let expected_evaporation_rate = sim.water_system.parameters.evaporation_rate;
    let expected_loss_per_tick = start_total * expected_evaporation_rate * 0.3; // Rough estimate
    
    if loss_rate > expected_loss_per_tick * 3.0 {
        println!("  ⚠️  WARNING: Water loss rate seems too high - possible mass conservation issue");
    } else if water_loss < 0.0 {
        println!("  ⚠️  CRITICAL: Water is increasing without external input - mass conservation violation!");
    } else {
        println!("  ✅ Mass conservation appears reasonable");
    }
}

fn analyze_cfl_stability(scale: &WorldScale) {
    let water_system = WaterFlowSystem::new_for_scale(scale);
    let stable_timestep = water_system._get_stable_timestep_seconds();
    let current_timestep = 360.0; // 6 minutes in seconds
    
    println!("CFL Stability Analysis:");
    println!("  Grid Resolution: {:.0} m", scale.meters_per_pixel());
    println!("  Max Expected Velocity: {:.1} m/s", water_system.parameters.max_expected_velocity_ms);
    println!("  CFL-Stable Timestep: {:.1} seconds ({:.1} minutes)", 
             stable_timestep, stable_timestep / 60.0);
    println!("  Current Timestep: {:.1} seconds ({:.1} minutes)", 
             current_timestep, current_timestep / 60.0);
    
    let cfl_ratio = current_timestep / stable_timestep;
    
    if cfl_ratio > 1.0 {
        println!("  ⚠️  WARNING: Current timestep is {:.2}x larger than CFL limit!", cfl_ratio);
        println!("      This may cause numerical instabilities");
    } else {
        println!("  ✅ CFL stability condition satisfied (ratio: {:.2})", cfl_ratio);
    }
}

fn analyze_water_balance(scale: &WorldScale) {
    let water_system = WaterFlowSystem::new_for_scale(scale);
    
    println!("Water Balance Analysis:");
    println!("  Effective Rainfall Rate: {:.6} per tick", water_system.effective_rainfall_rate);
    println!("  Evaporation Rate: {:.3}% per tick", water_system.parameters.evaporation_rate * 100.0);
    println!("  Evaporation Threshold: {:.2e}", water_system.evaporation_threshold);
    
    // Calculate steady-state water depth
    let rainfall_per_tick = water_system.effective_rainfall_rate;
    let evaporation_rate = water_system.parameters.evaporation_rate;
    let steady_state_depth = rainfall_per_tick / evaporation_rate;
    
    println!("  Theoretical Steady State: {:.6} depth units", steady_state_depth);
    
    // Check if evaporation threshold allows accumulation
    let post_evap_rainfall = rainfall_per_tick * (1.0 - evaporation_rate);
    
    if post_evap_rainfall <= water_system.evaporation_threshold {
        println!("  ⚠️  CRITICAL: Post-evaporation rainfall ({:.2e}) ≤ threshold ({:.2e})", 
                post_evap_rainfall, water_system.evaporation_threshold);
        println!("      Water cannot accumulate - will be cleared immediately!");
    } else {
        println!("  ✅ Post-evaporation rainfall exceeds threshold - accumulation possible");
        
        let accumulation_ratio = post_evap_rainfall / water_system.evaporation_threshold;
        println!("  Accumulation Safety Margin: {:.1}x threshold", accumulation_ratio);
    }
    
    // Mass conserving scaling check
    if matches!(water_system.parameters.rainfall_scaling, RainfallScaling::MassConserving) {
        println!("  Rainfall Scaling: Mass Conserving ✅");
        let domain_area = (scale.resolution.0 * scale.resolution.1) as f32;
        let total_water_input = rainfall_per_tick * domain_area;
        println!("  Total Water Input: {:.3} per tick", total_water_input);
    }
}