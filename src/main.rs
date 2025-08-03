// ABOUTME: CLI dispatcher - routes to different application modes
// ABOUTME: Simple entry point that delegates to application-specific implementations

mod applications;
mod engine;

use applications::run_terrain_explorer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // For now, just run the terrain explorer
    // In the future, this could dispatch to different applications based on CLI args
    run_terrain_explorer()
}
