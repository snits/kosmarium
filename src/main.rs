// ABOUTME: CLI dispatcher - routes to different application modes
// ABOUTME: Simple entry point that delegates to application-specific implementations

mod applications;
mod engine;

use applications::run_weather_demo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // For weather system testing, run the weather demo
    // This demonstrates atmospheric dynamics and weather pattern visualization
    run_weather_demo()
}
