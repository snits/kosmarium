// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Physics simulation systems - terrain generation, water flow, climate, atmosphere
// ABOUTME: Provides scale-aware physics implementations for environmental simulation

pub mod atmosphere;
pub mod atmospheric_moisture;
pub mod atmospheric_pressure_coupling;
pub mod climate;
pub mod convergence;
pub mod convergence_detection;
pub mod corrected_water_flow;
pub mod drainage;
pub mod ecosystem_feedback;
pub mod flow_engine;
pub mod geological_evolution;
pub mod hydro_biome_coupling;
pub mod maritime_climate_coupling;
pub mod optimized_geological_evolution;
pub mod orographic_precipitation;
pub mod spatial_partitioning;
pub mod tectonics;
pub mod temperature;
pub mod thermal_circulation;
pub mod water;
pub mod wind_erosion_coupling;
pub mod worldgen;

// Re-export key terrain generation types
pub use worldgen::{
    DiamondSquareConfig, DiamondSquareGenerator, TectonicConfig, TectonicGenerator,
    TerrainGenerator,
};

// Re-export geological evolution
pub use geological_evolution::GeologicalEvolutionConfig;

// Re-export unified flow engine
pub use flow_engine::{FlowAlgorithm, FlowEngine, FlowParameters, VelocityField};

// Re-export hydrology-biome coupling
pub use hydro_biome_coupling::{HydrologyAwareBiomeClassifier, WaterAvailability};

// Re-export maritime-climate coupling
pub use maritime_climate_coupling::{CoastalThermalEffects, MaritimAwareAtmosphereSystem};

// Re-export atmospheric-pressure coupling
pub use atmospheric_pressure_coupling::{AtmosphericPressureEffects, PressureAwareWaterFlowSystem};

// Re-export wind-erosion coupling
pub use wind_erosion_coupling::{WindAwareGeologicalSystem, WindErosionEffects};

// Re-export orographic-precipitation coupling
pub use orographic_precipitation::{
    OrographicEffects, OrographicParameters, OrographicPrecipitationSystem,
};

// Re-export thermal-circulation coupling
pub use thermal_circulation::{
    ThermalCirculationEffects, ThermalCirculationParameters, ThermalCirculationSystem,
};

// Re-export ecosystem-feedback coupling
pub use ecosystem_feedback::{
    BiomeMap, BiomeType, EcosystemFeedbackEffects, EcosystemFeedbackParameters,
    EcosystemFeedbackSystem,
};

// Re-export temperature field
pub use temperature::TemperatureField;
