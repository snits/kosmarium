// ABOUTME: Physics simulation systems - terrain generation, water flow, climate, atmosphere
// ABOUTME: Provides scale-aware physics implementations for environmental simulation

pub mod atmosphere;
pub mod atmospheric_moisture;
pub mod climate;
pub mod convergence;
pub mod convergence_detection;
pub mod drainage;
pub mod geological_evolution;
pub mod optimized_geological_evolution;
pub mod spatial_partitioning;
pub mod tectonics;
pub mod water;
pub mod worldgen;

// Re-export key terrain generation types
pub use worldgen::{
    DiamondSquareConfig, DiamondSquareGenerator, TectonicConfig, TectonicGenerator,
    TerrainGenerator,
};

// Re-export geological evolution
pub use geological_evolution::GeologicalEvolutionConfig;
