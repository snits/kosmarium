// ABOUTME: Debug script to analyze climate parameter scaling at different resolutions  
// ABOUTME: Identifies resolution-dependent thermal circulation parameter values

use sim_protoype::engine::core::{WorldScale, DetailLevel};
use sim_protoype::engine::core::scale::ScaleAware;
use sim_protoype::engine::physics::climate::ClimateParameters;

fn main() {
    println!("=== RESOLUTION-DEPENDENT PARAMETER ANALYSIS ===\n");
    
    // Test the same physical scale (4096km) at different resolutions
    let physical_size_km = 4096.0;
    
    // Working case: 64x32 resolution
    let low_res_scale = WorldScale::new(physical_size_km, (64, 32), DetailLevel::Standard);
    let low_res_params = ClimateParameters::default().derive_parameters(&low_res_scale);
    
    // Failing case: 512x256 resolution
    let high_res_scale = WorldScale::new(physical_size_km, (512, 256), DetailLevel::Standard);  
    let high_res_params = ClimateParameters::default().derive_parameters(&high_res_scale);
    
    println!("PHYSICAL DOMAIN: {}km x {}km", physical_size_km, physical_size_km);
    println!();
    
    println!("LOW RESOLUTION (WORKING): {}x{}", 64, 32);
    println!("  Meters per pixel: {:.1} m", low_res_scale.meters_per_pixel());
    println!("  Pressure-temperature coupling: {:.1} Pa/°C", low_res_params.pressure_temperature_coupling);
    println!("  Seasonal pressure amplitude: {:.1} Pa", low_res_params.seasonal_pressure_amplitude);
    println!("  Pressure noise amplitude: {:.1} Pa", low_res_params.pressure_noise_amplitude);
    println!("  Latitude gradient: {:.2} °C", low_res_params.latitude_gradient);
    println!();
    
    println!("HIGH RESOLUTION (FAILING): {}x{}", 512, 256);
    println!("  Meters per pixel: {:.1} m", high_res_scale.meters_per_pixel());
    println!("  Pressure-temperature coupling: {:.1} Pa/°C", high_res_params.pressure_temperature_coupling);
    println!("  Seasonal pressure amplitude: {:.1} Pa", high_res_params.seasonal_pressure_amplitude);
    println!("  Pressure noise amplitude: {:.1} Pa", high_res_params.pressure_noise_amplitude);
    println!("  Latitude gradient: {:.2} °C", high_res_params.latitude_gradient);
    println!();
    
    // Calculate resolution-dependent ratios
    let coupling_ratio = high_res_params.pressure_temperature_coupling / low_res_params.pressure_temperature_coupling;
    let noise_ratio = high_res_params.pressure_noise_amplitude / low_res_params.pressure_noise_amplitude;
    let resolution_ratio = (512.0 * 256.0) / (64.0 * 32.0);
    
    println!("=== SCALING ANALYSIS ===");
    println!("Resolution increase factor: {:.1}x", resolution_ratio);
    println!("Pressure coupling scaling: {:.2}x", coupling_ratio);
    println!("Pressure noise scaling: {:.2}x", noise_ratio);
    println!();
    
    // Identify potential issues
    println!("=== POTENTIAL PHYSICS ISSUES ===");
    
    if coupling_ratio > 2.0 {
        println!("⚠️  THERMAL CIRCULATION: Pressure-temperature coupling scales too strongly with resolution");
        println!("   High resolution coupling ({:.1}) may create excessive pressure gradients", high_res_params.pressure_temperature_coupling);
        println!("   This could drive unrealistic water circulation patterns");
    }
    
    if noise_ratio > 3.0 {
        println!("⚠️  PRESSURE NOISE: Weather noise scales too aggressively with resolution");
        println!("   High resolution noise ({:.1}) may dominate thermal circulation", high_res_params.pressure_noise_amplitude);
        println!("   This could mask continental-scale pressure patterns");
    }
    
    // Check for CFL-like stability concerns
    let pixel_size_ratio = high_res_scale.meters_per_pixel() / low_res_scale.meters_per_pixel();
    println!("⚠️  GRID RESOLUTION: Pixel size decreased by {:.1}x", 1.0 / pixel_size_ratio);
    println!("   Fine-scale pressure variations may not have time to equilibrate");
    println!("   Consider resolution-dependent relaxation time constants");
    
    // Check latitude gradient effects
    if high_res_params.latitude_gradient > low_res_params.latitude_gradient * 1.5 {
        println!("⚠️  LATITUDE GRADIENT: Temperature gradient increased {:.1}x", 
                high_res_params.latitude_gradient / low_res_params.latitude_gradient);
        println!("   Strong north-south temperature differences may drive excessive circulation");
    }
}