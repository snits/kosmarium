// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Agent and social systems - NPCs, creatures, biomes, and behavioral simulation
// ABOUTME: Provides high-performance agent systems with spatial indexing and social dynamics

pub mod agents;
pub mod biome;

// Re-export key agent types

// Re-export biome and vegetation classification systems for rendering integration
pub use biome::{
    BiomeClassificationParameters, BiomeClassifier, BiomeMap, BiomeType, VegetationState,
    VegetationStateClassifier, VegetationStateParameters,
};
