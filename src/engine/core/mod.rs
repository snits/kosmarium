// ABOUTME: Core engine foundation - fundamental data structures and scaling systems
// ABOUTME: Provides basic building blocks for all other engine components

pub mod cache_system;
pub mod dimensional;
pub mod heightmap;
pub mod optimized_heightmap;
pub mod physics_grid;
pub mod scale;

// Re-export key types for convenience
pub use physics_grid::PhysicsGrid;
pub use scale::{DetailLevel, WorldScale};
