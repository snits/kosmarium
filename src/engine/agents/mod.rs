// ABOUTME: Agent and social systems - NPCs, creatures, biomes, and behavioral simulation
// ABOUTME: Provides high-performance agent systems with spatial indexing and social dynamics

pub mod agents;
pub mod biome;

// Re-export key agent types
pub use agents::{
    AgentError, AgentId, AgentResult, AgentSystem, AgentType, SimulationContext, WorldBounds,
};

// Re-export biome system
pub use biome::{BiomeClassifier, BiomeMap, BiomeType};
