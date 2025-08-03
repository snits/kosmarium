// ABOUTME: Application implementations - different ways to use the simulation engine
// ABOUTME: Demonstrates engine flexibility through specialized application instances

pub mod terrain_explorer;

// Re-export application entry points
pub use terrain_explorer::run_terrain_explorer;
