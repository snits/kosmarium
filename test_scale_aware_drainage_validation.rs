// ABOUTME: Hydrological validation of Jerry's scale-aware boundary drainage implementation
// ABOUTME: Tests mass conservation, boundary physics, and scaling relationships across multiple scales

use kosmarium::engine::core::heightmap::HeightMap;
use kosmarium::engine::core::scale::{DetailLevel, WorldScale};
use kosmarium::engine::physics::worldgen::{TectonicConfig, TectonicGenerator, TerrainGenerator};
use kosmarium::engine::physics::water::WaterLayer;
use kosmarium::engine::sim::{Simulation, WaterFlowSystem};

/// Hydrological validation metrics for scale-aware drainage assessment
#[derive(Debug, Clone)]
struct HydrologicalValidation {
    scale_km: f64,
    resolution: usize,
    meters_per_pixel: f64,
    
    // Scale-aware parameter validation
    flow_threshold: f32,
    edge_margin: usize,
    edge_margin_percentage: f32,
    
    // Mass balance validation
    mass_balance_error: f32,
    mass_balance_error_percentage: f32,
    drainage_efficiency: f32,
    edge_saturation_ratio: f32,
    
    // Hydrological physics validation
    rainfall_evaporation_ratio: f32,
    boundary_outflow_ratio: f32,
    water_residence_time: f32,
    
    // Effectiveness assessment
    is_drainage_effective: bool,
    passes_mass_conservation: bool,
    passes_edge_saturation: bool,
    passes_drainage_efficiency: bool,
}

/// Test scale-aware boundary drainage across multiple scales with hydrological validation
fn validate_scale_aware_drainage(scale_km: f64, resolution: usize, num_timesteps: usize) -> HydrologicalValidation {
    println!("\n=== Hydrological Validation: {:.0}km Scale ===", scale_km);
    
    // Create world scale and report key parameters
    let world_scale = WorldScale::new(scale_km, (resolution, resolution), DetailLevel::Standard);
    let meters_per_pixel = world_scale.meters_per_pixel();
    
    println!("Domain: {:.0}km x {:.0}km at {:.0}m/pixel", scale_km, scale_km, meters_per_pixel);
    
    // Generate terrain
    let generator = TectonicGenerator::new(12345);
    let mut config = TectonicConfig::default();
    config.enable_geological_evolution = false;
    let scaled_config = config.derive_parameters(&world_scale);
    let mut heightmap = generator.generate(resolution, resolution, &scaled_config);
    
    // Create water system with scale-aware parameters
    let mut water_system = WaterFlowSystem::new_for_scale(&world_scale);
    let mut water = WaterLayer::new(resolution, resolution);
    
    // Extract scale-aware parameters for validation
    let evaporation_threshold = water_system.evaporation_threshold;
    let flow_threshold = evaporation_threshold * 0.01; // Jerry's 1% threshold
    let edge_margin = ((resolution as f32 * 0.05) as usize).clamp(5, 50); // Jerry's 5% edge margin
    let edge_margin_percentage = edge_margin as f32 / resolution as f32 * 100.0;
    
    println!("Scale-Aware Parameters:");
    println!("  Flow threshold: {:.8} (1% of evaporation threshold {:.8})", flow_threshold, evaporation_threshold);
    println!("  Edge margin: {} pixels ({:.1}% of domain)", edge_margin, edge_margin_percentage);
    println!("  Rainfall rate: {:.8}", water_system.effective_rainfall_rate);
    println!("  Evaporation rate: {:.8}", water_system.parameters.evaporation_rate);
    
    // Track initial state for mass balance validation
    let initial_water = water.get_total_water();
    water_system.drainage_metrics.start_tick();
    
    // Run simulation with drainage metrics tracking
    println!("\nRunning {} timesteps with drainage tracking...", num_timesteps);
    
    for timestep in 1..=num_timesteps {
        water_system.drainage_metrics.start_tick();
        
        // Add rainfall (tracked by drainage metrics)
        water_system.add_rainfall(&mut water);
        water_system.drainage_metrics.total_rainfall_input += 
            water_system.effective_rainfall_rate * (resolution * resolution) as f32;
        
        // Move water with boundary outflow tracking
        water_system.move_water(&mut water);
        
        // Apply erosion 
        water_system.apply_erosion(&mut heightmap, &mut water);
        
        // Apply evaporation (tracked by drainage metrics)
        let pre_evap_water = water.get_total_water();
        water_system.apply_evaporation(&mut water);
        let post_evap_water = water.get_total_water();
        let evaporation_amount = pre_evap_water - post_evap_water;
        water_system.drainage_metrics.total_evaporation += evaporation_amount;
        
        // Update drainage metrics
        water_system.drainage_metrics.end_tick(&water);
        
        if timestep % 10 == 0 || timestep <= 5 {
            println!("  Step {}: Water={:.6}, Boundary Outflow={:.6}, Mass Balance Error={:.8}", 
                   timestep, 
                   water.get_total_water(),
                   water_system.drainage_metrics.boundary_outflow_rate,
                   water_system.drainage_metrics.mass_balance_error);
        }
    }
    
    // Extract final metrics for hydrological analysis
    let metrics = &water_system.drainage_metrics;
    
    // Calculate hydrological validity metrics
    let total_input = metrics.total_rainfall_input;
    let mass_balance_error_percentage = if total_input > 0.0 {
        (metrics.mass_balance_error / total_input) * 100.0
    } else {
        0.0
    };
    
    let rainfall_evaporation_ratio = if metrics.total_evaporation > 0.0 {
        metrics.total_rainfall_input / metrics.total_evaporation
    } else {
        f32::INFINITY
    };
    
    let boundary_outflow_ratio = if total_input > 0.0 {
        metrics.total_boundary_outflow / total_input
    } else {
        0.0
    };
    
    // Estimate water residence time (simplified)
    let average_water_storage = metrics.current_water_storage;
    let total_outflow_rate = metrics.total_evaporation + metrics.total_boundary_outflow;
    let water_residence_time = if total_outflow_rate > 0.0 {
        average_water_storage / (total_outflow_rate / num_timesteps as f32)
    } else {
        f32::INFINITY
    };
    
    // Apply Jerry's effectiveness criteria
    let passes_mass_conservation = if total_input > 0.0 {
        metrics.mass_balance_error / total_input < 0.01 // 1% threshold
    } else {
        true
    };
    
    let passes_edge_saturation = metrics.edge_saturation_ratio < 0.5; // 50% threshold
    
    let net_input = metrics.total_rainfall_input - metrics.total_evaporation;
    let passes_drainage_efficiency = if net_input > 0.0 {
        metrics.drainage_efficiency > 0.1 // 10% threshold
    } else {
        true
    };
    
    let is_drainage_effective = passes_mass_conservation && passes_edge_saturation && passes_drainage_efficiency;
    
    println!("\nHydrological Validation Results:");
    println!("  Mass Balance Error: {:.8} ({:.4}% of input)", metrics.mass_balance_error, mass_balance_error_percentage);
    println!("  Edge Saturation: {:.3} ({:.1}%)", metrics.edge_saturation_ratio, metrics.edge_saturation_ratio * 100.0);
    println!("  Drainage Efficiency: {:.3} ({:.1}%)", metrics.drainage_efficiency, metrics.drainage_efficiency * 100.0);
    println!("  Boundary Outflow Ratio: {:.3} ({:.1}% of input)", boundary_outflow_ratio, boundary_outflow_ratio * 100.0);
    println!("  Water Residence Time: {:.2} timesteps", water_residence_time);
    
    println!("\nEffectiveness Criteria:");
    println!("  ✓ Mass Conservation (< 1%): {}", if passes_mass_conservation { "PASS" } else { "FAIL" });
    println!("  ✓ Edge Saturation (< 50%): {}", if passes_edge_saturation { "PASS" } else { "FAIL" });
    println!("  ✓ Drainage Efficiency (> 10%): {}", if passes_drainage_efficiency { "PASS" } else { "FAIL" });
    println!("  → Overall Effectiveness: {}", if is_drainage_effective { "EFFECTIVE" } else { "PROBLEMATIC" });
    
    HydrologicalValidation {
        scale_km,
        resolution,
        meters_per_pixel,
        flow_threshold,
        edge_margin,
        edge_margin_percentage,
        mass_balance_error: metrics.mass_balance_error,
        mass_balance_error_percentage,
        drainage_efficiency: metrics.drainage_efficiency,
        edge_saturation_ratio: metrics.edge_saturation_ratio,
        rainfall_evaporation_ratio,
        boundary_outflow_ratio,
        water_residence_time,
        is_drainage_effective,
        passes_mass_conservation,
        passes_edge_saturation,
        passes_drainage_efficiency,
    }
}

/// Analyze hydrological scaling relationships across different scales
fn analyze_scaling_relationships(validations: &[HydrologicalValidation]) {
    println!("\n=== HYDROLOGICAL SCALING ANALYSIS ===");
    
    // 1. Scale-aware parameter validation
    println!("\n1. Scale-Aware Parameter Scaling:");
    for validation in validations {
        println!("  {:.0}km: Edge margin = {:.1}% ({} pixels), Flow threshold = {:.2e}",
               validation.scale_km,
               validation.edge_margin_percentage,
               validation.edge_margin,
               validation.flow_threshold);
    }
    
    // Check if edge margin scaling is appropriate (should be ~5% for all scales)
    let edge_margin_consistent = validations.iter().all(|v| 
        (v.edge_margin_percentage - 5.0).abs() < 1.0 || 
        v.edge_margin == 5 || v.edge_margin == 50 // Clamped values
    );
    println!("  → Edge margin scaling consistency: {}", 
           if edge_margin_consistent { "GOOD" } else { "NEEDS ATTENTION" });
    
    // 2. Mass conservation scaling
    println!("\n2. Mass Conservation Across Scales:");
    for validation in validations {
        println!("  {:.0}km: Error = {:.4}% of input ({})",
               validation.scale_km,
               validation.mass_balance_error_percentage,
               if validation.passes_mass_conservation { "PASS" } else { "FAIL" });
    }
    
    let mass_conservation_consistent = validations.iter().all(|v| v.passes_mass_conservation);
    println!("  → Mass conservation scaling: {}", 
           if mass_conservation_consistent { "EXCELLENT" } else { "PROBLEMATIC" });
    
    // 3. Boundary condition physics
    println!("\n3. Boundary Condition Physics:");
    for validation in validations {
        println!("  {:.0}km: Edge saturation = {:.1}%, Boundary outflow = {:.1}% of input",
               validation.scale_km,
               validation.edge_saturation_ratio * 100.0,
               validation.boundary_outflow_ratio * 100.0);
    }
    
    // Check for realistic drainage behavior
    let drainage_realistic = validations.iter().all(|v| 
        v.boundary_outflow_ratio > 0.01 && // At least 1% outflow
        v.boundary_outflow_ratio < 0.8     // Not more than 80% outflow
    );
    println!("  → Boundary outflow realism: {}", 
           if drainage_realistic { "REALISTIC" } else { "UNREALISTIC" });
    
    // 4. Effectiveness criteria validation
    println!("\n4. Effectiveness Criteria Validation:");
    let all_effective = validations.iter().all(|v| v.is_drainage_effective);
    let num_effective = validations.iter().filter(|v| v.is_drainage_effective).count();
    
    println!("  Effective scales: {}/{}", num_effective, validations.len());
    
    if all_effective {
        println!("  → EXCELLENT: All scales show effective drainage");
    } else {
        println!("  → ATTENTION: Some scales show problematic drainage");
        for validation in validations.iter().filter(|v| !v.is_drainage_effective) {
            println!("    {:.0}km: Mass={}, Edge={}, Drainage={}",
                   validation.scale_km,
                   if validation.passes_mass_conservation { "✓" } else { "✗" },
                   if validation.passes_edge_saturation { "✓" } else { "✗" },
                   if validation.passes_drainage_efficiency { "✓" } else { "✗" });
        }
    }
    
    // 5. Hydrological timescale analysis  
    println!("\n5. Hydrological Timescale Analysis:");
    for validation in validations {
        println!("  {:.0}km: Residence time = {:.1} timesteps",
               validation.scale_km,
               validation.water_residence_time);
    }
    
    // Check for realistic timescale scaling
    let timescales_realistic = validations.iter().all(|v| 
        v.water_residence_time > 1.0 && // Water should persist
        v.water_residence_time < 1000.0  // But not indefinitely
    );
    println!("  → Timescale realism: {}", 
           if timescales_realistic { "REASONABLE" } else { "UNREALISTIC" });
}

fn main() {
    println!("HYDROLOGICAL VALIDATION OF SCALE-AWARE BOUNDARY DRAINAGE");
    println!("Testing Jerry's implementation of drainage metrics and boundary tracking");
    
    // Test multiple scales with appropriate resolutions
    let test_scenarios = vec![
        (240.0, 120),   // Regional scale - 240km at 2km/pixel  
        (960.0, 240),   // Large regional - 960km at 4km/pixel
        (1920.0, 480),  // Continental - 1920km at 4km/pixel
        (3840.0, 960),  // Large continental - 3840km at 4km/pixel
    ];
    
    let mut validations = Vec::new();
    
    for (scale_km, resolution) in test_scenarios {
        let validation = validate_scale_aware_drainage(scale_km, resolution, 50);
        validations.push(validation);
    }
    
    // Analyze scaling relationships and hydrological validity
    analyze_scaling_relationships(&validations);
    
    println!("\n=== HYDROLOGICAL ASSESSMENT SUMMARY ===");
    println!("This analysis validates Jerry's scale-aware boundary drainage implementation");
    println!("focusing on mass conservation, boundary physics, and scaling relationships");
    println!("appropriate for watershed dynamics across regional to continental scales.");
}