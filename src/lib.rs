// ABOUTME: Library interface for sim-prototype terrain generation and simulation engine
// ABOUTME: Exposes clean public API for external use while keeping internal organization

pub mod engine;

// Re-export key engine components for library users
pub use engine::{
    Simulation,
    agents::{AgentId, AgentSystem, AgentType, BiomeClassifier, BiomeMap, BiomeType},
    core::{HeightMap, PhysicalQuantity, PhysicalUnit, ScaleAware, WorldScale},
    physics::{
        DiamondSquareConfig, DiamondSquareGenerator, GeologicalEvolution,
        GeologicalEvolutionConfig, TectonicConfig, TectonicGenerator, TerrainGenerator,
    },
    rendering::{GraphicsRenderer, ascii_render, ascii_render_biomes, run_tui},
};

// Re-export applications for convenience
pub mod applications {
    pub use crate::engine::*;
}
