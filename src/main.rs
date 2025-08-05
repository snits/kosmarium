// ABOUTME: CLI dispatcher - routes to different application modes
// ABOUTME: Simple entry point that delegates to application-specific implementations

mod applications;
mod debug_flow_analysis;
mod debug_interval_issue;
mod debug_water_conservation;
mod engine;

use applications::run_weather_demo;
use debug_flow_analysis::{
    analyze_evaporation_loss, analyze_flow_calculation, analyze_flow_update_intervals,
    analyze_temperature_evaporation,
};
use debug_interval_issue::{
    analyze_tick_details, test_continuous_flow_updates, test_flow_interval_conservation,
};
use debug_water_conservation::{test_512x256_conservation, test_resolution_scaling_conservation};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for diagnostic mode
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "debug-water" {
        println!("Running water conservation diagnostics...\n");
        test_512x256_conservation();
        test_resolution_scaling_conservation();
        return Ok(());
    }

    if args.len() > 1 && args[1] == "debug-flow" {
        println!("Running detailed flow analysis...\n");
        analyze_flow_calculation();
        analyze_evaporation_loss();
        analyze_temperature_evaporation();
        analyze_flow_update_intervals();
        return Ok(());
    }

    if args.len() > 1 && args[1] == "debug-interval" {
        println!("Running flow interval analysis...\n");
        test_flow_interval_conservation();
        test_continuous_flow_updates();
        analyze_tick_details();
        return Ok(());
    }

    // For weather system testing, run the weather demo
    // This demonstrates atmospheric dynamics and weather pattern visualization
    run_weather_demo()
}
