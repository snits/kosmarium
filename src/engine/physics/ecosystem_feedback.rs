// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Ecosystem feedback loops coupling - biome effects on climate and hydrology
// ABOUTME: Models how vegetation and biomes influence temperature, humidity, evapotranspiration, and water cycles

use super::super::core::scale::WorldScale;
use super::super::core::temporal_scaling::TemporalScalingService;
use super::{
    atmospheric_moisture::SurfaceMoistureLayer, flow_engine::FlowEngine,
    temperature::TemperatureField, water::WaterLayer,
};

/// Biome types affecting ecosystem feedback
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BiomeType {
    /// Desert - low vegetation, high albedo, minimal evapotranspiration
    Desert,
    /// Grassland - moderate vegetation, seasonal cycles
    Grassland,
    /// Forest - high vegetation density, strong climate regulation
    Forest,
    /// Wetland - water-adapted vegetation, high humidity generation
    Wetland,
    /// Tundra - low vegetation, cold-adapted, minimal transpiration
    Tundra,
    /// Tropical - dense vegetation, high evapotranspiration, heat regulation
    Tropical,
}

impl BiomeType {
    /// Get albedo (surface reflectivity) for this biome type
    pub fn albedo(&self) -> f32 {
        match self {
            BiomeType::Desert => 0.35,    // High reflectivity, light surfaces
            BiomeType::Grassland => 0.20, // Moderate reflectivity
            BiomeType::Forest => 0.12,    // Low reflectivity, dark canopy
            BiomeType::Wetland => 0.15,   // Low-moderate, water and vegetation
            BiomeType::Tundra => 0.25,    // Moderate-high, sparse vegetation
            BiomeType::Tropical => 0.10,  // Very low, dense dark vegetation
        }
    }

    /// Get evapotranspiration coefficient for this biome type
    pub fn evapotranspiration_coefficient(&self) -> f32 {
        match self {
            BiomeType::Desert => 0.1,    // Minimal water loss
            BiomeType::Grassland => 0.4, // Moderate transpiration
            BiomeType::Forest => 0.8,    // High transpiration, large leaf area
            BiomeType::Wetland => 0.9,   // Very high, water available
            BiomeType::Tundra => 0.2,    // Low, cold adaptation
            BiomeType::Tropical => 1.0,  // Maximum transpiration
        }
    }

    /// Get thermal regulation capacity (heat absorption/cooling)
    pub fn thermal_regulation(&self) -> f32 {
        match self {
            BiomeType::Desert => 0.1,    // Minimal thermal buffering
            BiomeType::Grassland => 0.3, // Moderate cooling effect
            BiomeType::Forest => 0.7,    // Strong cooling through shading/transpiration
            BiomeType::Wetland => 0.6,   // Good thermal moderation
            BiomeType::Tundra => 0.2,    // Limited thermal buffering
            BiomeType::Tropical => 0.9,  // Maximum cooling effect
        }
    }

    /// Get moisture retention capacity (soil/vegetation water holding)
    pub fn moisture_retention(&self) -> f32 {
        match self {
            BiomeType::Desert => 0.1,    // Poor water retention
            BiomeType::Grassland => 0.4, // Moderate retention
            BiomeType::Forest => 0.8,    // High retention, deep roots
            BiomeType::Wetland => 1.0,   // Maximum retention
            BiomeType::Tundra => 0.3,    // Limited by frozen ground
            BiomeType::Tropical => 0.7,  // Good retention despite high turnover
        }
    }
}

/// Biome distribution map
#[derive(Clone, Debug)]
pub struct BiomeMap {
    /// Grid of biome types
    biomes: Vec<Vec<BiomeType>>,
    /// Vegetation density (0.0-1.0) for each cell
    vegetation_density: Vec<Vec<f32>>,
    /// Biomass amount (kg/m²) for each cell
    biomass: Vec<Vec<f32>>,
    width: usize,
    height: usize,
}

impl BiomeMap {
    /// Create new biome map with default grassland
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            biomes: vec![vec![BiomeType::Grassland; height]; width],
            vegetation_density: vec![vec![0.5; height]; width],
            biomass: vec![vec![100.0; height]; width],
            width,
            height,
        }
    }

    /// Set biome type at position
    pub fn set_biome(&mut self, x: usize, y: usize, biome: BiomeType) {
        if x < self.width && y < self.height {
            self.biomes[x][y] = biome;
        }
    }

    /// Get biome type at position
    pub fn get_biome(&self, x: usize, y: usize) -> BiomeType {
        if x < self.width && y < self.height {
            self.biomes[x][y]
        } else {
            BiomeType::Grassland
        }
    }

    /// Set vegetation density at position
    pub fn set_vegetation_density(&mut self, x: usize, y: usize, density: f32) {
        if x < self.width && y < self.height {
            self.vegetation_density[x][y] = density.clamp(0.0, 1.0);
        }
    }

    /// Get vegetation density at position
    pub fn get_vegetation_density(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.vegetation_density[x][y]
        } else {
            0.0
        }
    }

    /// Set biomass at position
    pub fn set_biomass(&mut self, x: usize, y: usize, biomass: f32) {
        if x < self.width && y < self.height {
            self.biomass[x][y] = biomass.max(0.0);
        }
    }

    /// Get biomass at position
    pub fn get_biomass(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.biomass[x][y]
        } else {
            0.0
        }
    }

    /// Get dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

/// Ecosystem feedback effects data
#[derive(Clone, Debug)]
pub struct EcosystemFeedbackEffects {
    /// Temperature modification due to vegetation (°C/s)
    pub temperature_modification: Vec<Vec<f32>>,
    /// Humidity generation from evapotranspiration (kg/m³/s)
    pub humidity_generation: Vec<Vec<f32>>,
    /// Evapotranspiration rate (mm/day)
    pub evapotranspiration_rate: Vec<Vec<f32>>,
    /// Soil moisture change rate (mm/day)
    pub soil_moisture_change: Vec<Vec<f32>>,
    /// Surface albedo modification due to vegetation
    pub albedo_modification: Vec<Vec<f32>>,
    /// Water retention enhancement from vegetation
    pub water_retention_enhancement: Vec<Vec<f32>>,
}

impl EcosystemFeedbackEffects {
    /// Create new effects data structure
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            temperature_modification: vec![vec![0.0; height]; width],
            humidity_generation: vec![vec![0.0; height]; width],
            evapotranspiration_rate: vec![vec![0.0; height]; width],
            soil_moisture_change: vec![vec![0.0; height]; width],
            albedo_modification: vec![vec![0.0; height]; width],
            water_retention_enhancement: vec![vec![0.0; height]; width],
        }
    }

    /// Get temperature modification at position with bounds checking
    pub fn get_temperature_modification(&self, x: usize, y: usize) -> f32 {
        if x < self.temperature_modification.len() && y < self.temperature_modification[0].len() {
            self.temperature_modification[x][y]
        } else {
            0.0
        }
    }

    /// Get humidity generation at position with bounds checking
    pub fn get_humidity_generation(&self, x: usize, y: usize) -> f32 {
        if x < self.humidity_generation.len() && y < self.humidity_generation[0].len() {
            self.humidity_generation[x][y]
        } else {
            0.0
        }
    }

    /// Get evapotranspiration rate at position with bounds checking
    pub fn get_evapotranspiration_rate(&self, x: usize, y: usize) -> f32 {
        if x < self.evapotranspiration_rate.len() && y < self.evapotranspiration_rate[0].len() {
            self.evapotranspiration_rate[x][y]
        } else {
            0.0
        }
    }

    /// Get soil moisture change at position with bounds checking
    pub fn get_soil_moisture_change(&self, x: usize, y: usize) -> f32 {
        if x < self.soil_moisture_change.len() && y < self.soil_moisture_change[0].len() {
            self.soil_moisture_change[x][y]
        } else {
            0.0
        }
    }

    /// Get albedo modification at position with bounds checking
    pub fn get_albedo_modification(&self, x: usize, y: usize) -> f32 {
        if x < self.albedo_modification.len() && y < self.albedo_modification[0].len() {
            self.albedo_modification[x][y]
        } else {
            0.0
        }
    }

    /// Get water retention enhancement at position with bounds checking
    pub fn get_water_retention_enhancement(&self, x: usize, y: usize) -> f32 {
        if x < self.water_retention_enhancement.len()
            && y < self.water_retention_enhancement[0].len()
        {
            self.water_retention_enhancement[x][y]
        } else {
            0.0
        }
    }
}

/// Configuration parameters for ecosystem feedback physics
#[derive(Clone, Debug)]
pub struct EcosystemFeedbackParameters {
    /// Base evapotranspiration rate (mm/day per unit vegetation)
    pub base_evapotranspiration: f32,
    /// Temperature moderation strength (°C cooling per unit vegetation)
    pub temperature_moderation: f32,
    /// Humidity generation coefficient (kg/m³/s per mm evapotranspiration)
    pub humidity_coefficient: f32,
    /// Albedo variation range (maximum change from base)
    pub albedo_variation: f32,
    /// Soil moisture enhancement factor (water retention multiplier)
    pub moisture_enhancement: f32,
    /// Vegetation growth rate coefficient (biomass/day per favorable conditions)
    pub growth_rate: f32,
    /// Water stress threshold (below this, vegetation suffers)
    pub water_stress_threshold: f32,
    /// Temperature stress range (optimal temperature range width)
    pub temperature_stress_range: f32,
}

impl Default for EcosystemFeedbackParameters {
    fn default() -> Self {
        Self {
            base_evapotranspiration: 5.0,   // 5 mm/day base rate
            temperature_moderation: 2.0,    // 2°C cooling per full vegetation
            humidity_coefficient: 0.1,      // 0.1 kg/m³/s per mm/day evapotranspiration
            albedo_variation: 0.25,         // 25% albedo variation range
            moisture_enhancement: 2.0,      // 2x water retention with full vegetation
            growth_rate: 10.0,              // 10 kg/m²/day growth under optimal conditions
            water_stress_threshold: 0.3,    // Stress below 30% water availability
            temperature_stress_range: 20.0, // 20°C optimal temperature range
        }
    }
}

/// Ecosystem feedback loops coupling system
pub struct EcosystemFeedbackSystem {
    /// Physics parameters
    pub parameters: EcosystemFeedbackParameters,
    /// Biome distribution map
    pub biome_map: BiomeMap,
    /// Current ecosystem feedback effects
    effects: Option<EcosystemFeedbackEffects>,
    /// Temporal scaling service for realistic ecological timescales
    temporal_scaling: TemporalScalingService,
}

impl EcosystemFeedbackSystem {
    /// Create new ecosystem feedback system
    pub fn new(parameters: EcosystemFeedbackParameters, width: usize, height: usize) -> Self {
        use super::super::core::temporal_scaling::{TemporalMode, TemporalScalingConfig};

        Self {
            parameters,
            biome_map: BiomeMap::new(width, height),
            effects: None,
            temporal_scaling: TemporalScalingService::new(TemporalScalingConfig {
                mode: TemporalMode::Demo, // Default to Demo mode for backward compatibility
                ..Default::default()
            }),
        }
    }

    /// Create new ecosystem feedback system with explicit temporal scaling configuration
    pub fn new_with_temporal_scaling(
        parameters: EcosystemFeedbackParameters,
        width: usize,
        height: usize,
        temporal_scaling: TemporalScalingService,
    ) -> Self {
        Self {
            parameters,
            biome_map: BiomeMap::new(width, height),
            effects: None,
            temporal_scaling,
        }
    }

    /// Check if ecosystem effects are currently active
    pub fn has_active_effects(&self) -> bool {
        self.effects.is_some()
    }

    /// Get current ecosystem effects (if any)
    pub fn get_effects(&self) -> Option<&EcosystemFeedbackEffects> {
        self.effects.as_ref()
    }

    /// Get mutable reference to biome map
    pub fn biome_map_mut(&mut self) -> &mut BiomeMap {
        &mut self.biome_map
    }

    /// Get immutable reference to biome map
    pub fn biome_map(&self) -> &BiomeMap {
        &self.biome_map
    }

    /// Get current temporal scaling mode
    pub fn get_temporal_mode(&self) -> super::super::core::temporal_scaling::TemporalMode {
        self.temporal_scaling.mode()
    }

    /// Update temporal scaling configuration
    pub fn update_temporal_scaling(&mut self, temporal_scaling: TemporalScalingService) {
        self.temporal_scaling = temporal_scaling;
    }

    /// Check if currently in demo mode (preserves exact behavior)
    pub fn is_demo_mode(&self) -> bool {
        matches!(
            self.temporal_scaling.mode(),
            super::super::core::temporal_scaling::TemporalMode::Demo
        )
    }

    /// Get human-readable description of current temporal scaling
    pub fn temporal_scaling_description(&self) -> String {
        self.temporal_scaling.scaling_description()
    }

    /// Update ecosystem feedback effects
    pub fn update(
        &mut self,
        temperature_field: &mut TemperatureField,
        water_layer: &mut WaterLayer,
        moisture_layer: &mut SurfaceMoistureLayer,
        _flow_engine: &FlowEngine,
        scale: &WorldScale,
        dt: f32,
    ) {
        let (width, height) = self.biome_map.dimensions();
        let mut effects = EcosystemFeedbackEffects::new(width, height);

        // Physical constants
        let cell_size_m = scale.meters_per_pixel() as f32;
        let seconds_per_day = 86400.0;

        // Calculate ecosystem feedback for each cell
        for x in 0..width {
            for y in 0..height {
                let biome = self.biome_map.get_biome(x, y);
                let vegetation_density = self.biome_map.get_vegetation_density(x, y);
                let biomass = self.biome_map.get_biomass(x, y);

                // Get current environmental conditions
                let temperature = temperature_field.get_temperature(x, y);
                let water_depth = water_layer.get_water_depth(x, y);
                let moisture_content = moisture_layer.get_moisture(x, y);

                // Calculate vegetation stress factors
                let water_stress = self.calculate_water_stress(water_depth, x, y);
                let temperature_stress = self.calculate_temperature_stress(temperature, biome);
                let overall_health = (1.0 - water_stress) * (1.0 - temperature_stress);

                // Calculate evapotranspiration based on biome and conditions
                let evapotranspiration_coefficient = biome.evapotranspiration_coefficient();
                let potential_evapotranspiration = self.parameters.base_evapotranspiration
                    * evapotranspiration_coefficient
                    * vegetation_density;

                // Modify by environmental stress
                let actual_evapotranspiration = potential_evapotranspiration * overall_health;
                effects.evapotranspiration_rate[x][y] = actual_evapotranspiration;

                // Calculate temperature modification from vegetation cooling
                let thermal_regulation = biome.thermal_regulation();
                let cooling_effect_per_second = self.parameters.temperature_moderation
                    * thermal_regulation
                    * vegetation_density
                    * (temperature - 15.0).max(0.0) / 30.0 // More cooling when hot
                    / seconds_per_day; // Convert to °C/second

                effects.temperature_modification[x][y] = -cooling_effect_per_second; // Negative = cooling per second

                // Apply temperature modification with proper timestep scaling
                let temperature_change = cooling_effect_per_second * dt;
                let new_temperature = temperature - temperature_change;
                temperature_field.set_temperature(x, y, new_temperature);

                // Calculate humidity generation from evapotranspiration
                let humidity_generation = actual_evapotranspiration
                    * self.parameters.humidity_coefficient
                    / seconds_per_day; // Convert mm/day to kg/m³/s

                effects.humidity_generation[x][y] = humidity_generation;

                // Add humidity to atmospheric moisture
                let new_moisture = moisture_content + humidity_generation * dt;
                moisture_layer.set_moisture(x, y, new_moisture);

                // Calculate soil moisture change from evapotranspiration
                let water_loss_rate = actual_evapotranspiration / 1000.0; // mm to m
                let soil_moisture_change = -water_loss_rate / seconds_per_day; // m/s
                effects.soil_moisture_change[x][y] = soil_moisture_change * seconds_per_day; // Convert back to mm/day for display

                // Apply water consumption (remove from water layer)
                if water_depth > 0.001 {
                    let water_consumed = water_loss_rate * dt;
                    let new_water_depth = (water_depth - water_consumed).max(0.0);
                    // Note: WaterLayer doesn't have direct set_depth, so we calculate consumption differently
                    // This represents the transpiration loss from available water
                    let consumption_effect = water_consumed / water_depth; // Fraction consumed
                    effects.soil_moisture_change[x][y] =
                        soil_moisture_change * seconds_per_day * (1.0 - consumption_effect);
                }

                // Calculate albedo modification
                let base_albedo = biome.albedo();
                let vegetation_albedo_effect =
                    self.parameters.albedo_variation * (vegetation_density - 0.5) * 2.0; // Scale -1 to 1
                effects.albedo_modification[x][y] = base_albedo + vegetation_albedo_effect;

                // Calculate water retention enhancement
                let moisture_retention = biome.moisture_retention();
                let retention_enhancement = 1.0
                    + (self.parameters.moisture_enhancement - 1.0)
                        * moisture_retention
                        * vegetation_density;
                effects.water_retention_enhancement[x][y] = retention_enhancement;

                // Update vegetation growth/decline based on conditions
                // CRITICAL: Apply unified temporal scaling factor for consistency with all physics systems
                let temporal_factor = scale.temporal_scale.temporal_factor() as f32;
                let scaled_growth_rate = self.parameters.growth_rate * temporal_factor;

                let growth_factor = overall_health * scaled_growth_rate;
                let optimal_biomass = self.get_optimal_biomass(biome);
                let biomass_change = if biomass < optimal_biomass {
                    growth_factor * (optimal_biomass - biomass) / optimal_biomass
                } else {
                    -growth_factor * 0.1 // Slow decline if over-capacity
                };

                let new_biomass = (biomass + biomass_change).max(0.0);
                self.biome_map.set_biomass(x, y, new_biomass);

                // Update vegetation density based on biomass
                let new_vegetation_density = (new_biomass / optimal_biomass).clamp(0.0, 1.0);
                self.biome_map
                    .set_vegetation_density(x, y, new_vegetation_density);
            }
        }

        self.effects = Some(effects);
    }

    /// Calculate water stress factor (0.0 = no stress, 1.0 = maximum stress)
    fn calculate_water_stress(&self, water_depth: f32, _x: usize, _y: usize) -> f32 {
        let water_availability = water_depth / self.parameters.water_stress_threshold;
        if water_availability >= 1.0 {
            0.0 // No stress
        } else {
            1.0 - water_availability // Linear stress increase
        }
    }

    /// Calculate temperature stress factor (0.0 = no stress, 1.0 = maximum stress)
    fn calculate_temperature_stress(&self, temperature: f32, biome: BiomeType) -> f32 {
        let optimal_temperature = match biome {
            BiomeType::Desert => 30.0,
            BiomeType::Grassland => 20.0,
            BiomeType::Forest => 15.0,
            BiomeType::Wetland => 18.0,
            BiomeType::Tundra => 5.0,
            BiomeType::Tropical => 25.0,
        };

        let temperature_deviation = (temperature - optimal_temperature).abs();
        let stress = temperature_deviation / self.parameters.temperature_stress_range;
        stress.clamp(0.0, 1.0)
    }

    /// Get optimal biomass for biome type
    fn get_optimal_biomass(&self, biome: BiomeType) -> f32 {
        match biome {
            BiomeType::Desert => 20.0,     // Very low biomass
            BiomeType::Grassland => 150.0, // Moderate biomass
            BiomeType::Forest => 400.0,    // High biomass
            BiomeType::Wetland => 300.0,   // High but different composition
            BiomeType::Tundra => 50.0,     // Low biomass, cold adaptation
            BiomeType::Tropical => 500.0,  // Maximum biomass
        }
    }
}

/// Helper function to determine biome type from environmental conditions
pub fn classify_biome_from_environment(
    temperature: f32,
    water_availability: f32,
    elevation: f32,
) -> BiomeType {
    if temperature < 5.0 {
        BiomeType::Tundra
    } else if water_availability < 0.2 {
        BiomeType::Desert
    } else if temperature > 22.0 && water_availability > 0.6 {
        BiomeType::Tropical
    } else if water_availability > 0.8 && temperature > 10.0 {
        BiomeType::Wetland
    } else if temperature > 15.0 && water_availability > 0.4 {
        BiomeType::Forest
    } else {
        BiomeType::Grassland
    }
}

/// Helper function to calculate leaf area index from vegetation density
pub fn calculate_leaf_area_index(vegetation_density: f32, biome: BiomeType) -> f32 {
    let max_lai = match biome {
        BiomeType::Desert => 0.5,
        BiomeType::Grassland => 3.0,
        BiomeType::Forest => 6.0,
        BiomeType::Wetland => 4.0,
        BiomeType::Tundra => 1.0,
        BiomeType::Tropical => 8.0,
    };

    vegetation_density * max_lai
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::core::scale::{DetailLevel, WorldScale};

    #[test]
    fn test_biome_type_properties() {
        // Test albedo values are reasonable
        assert!(BiomeType::Desert.albedo() > BiomeType::Forest.albedo());
        assert!(BiomeType::Tropical.albedo() < BiomeType::Tundra.albedo());

        // Test evapotranspiration makes sense
        assert!(
            BiomeType::Tropical.evapotranspiration_coefficient()
                > BiomeType::Desert.evapotranspiration_coefficient()
        );
        assert!(
            BiomeType::Wetland.evapotranspiration_coefficient()
                > BiomeType::Tundra.evapotranspiration_coefficient()
        );

        // Test thermal regulation is logical
        assert!(BiomeType::Forest.thermal_regulation() > BiomeType::Desert.thermal_regulation());
        assert!(
            BiomeType::Tropical.thermal_regulation() > BiomeType::Grassland.thermal_regulation()
        );
    }

    #[test]
    fn test_biome_map_creation() {
        let biome_map = BiomeMap::new(10, 10);

        assert_eq!(biome_map.dimensions(), (10, 10));
        assert_eq!(biome_map.get_biome(5, 5), BiomeType::Grassland);
        assert_eq!(biome_map.get_vegetation_density(5, 5), 0.5);
        assert_eq!(biome_map.get_biomass(5, 5), 100.0);
    }

    #[test]
    fn test_biome_map_modification() {
        let mut biome_map = BiomeMap::new(5, 5);

        biome_map.set_biome(2, 2, BiomeType::Forest);
        biome_map.set_vegetation_density(2, 2, 0.8);
        biome_map.set_biomass(2, 2, 250.0);

        assert_eq!(biome_map.get_biome(2, 2), BiomeType::Forest);
        assert_eq!(biome_map.get_vegetation_density(2, 2), 0.8);
        assert_eq!(biome_map.get_biomass(2, 2), 250.0);
    }

    #[test]
    fn test_ecosystem_feedback_effects_creation() {
        let effects = EcosystemFeedbackEffects::new(8, 8);

        assert_eq!(effects.temperature_modification.len(), 8);
        assert_eq!(effects.humidity_generation.len(), 8);
        assert_eq!(effects.evapotranspiration_rate.len(), 8);
        assert_eq!(effects.soil_moisture_change.len(), 8);
        assert_eq!(effects.albedo_modification.len(), 8);
        assert_eq!(effects.water_retention_enhancement.len(), 8);
    }

    #[test]
    fn test_ecosystem_feedback_system_initialization() {
        let params = EcosystemFeedbackParameters::default();
        let system = EcosystemFeedbackSystem::new(params, 6, 6);

        assert!(!system.has_active_effects());
        assert!(system.get_effects().is_none());
        assert_eq!(system.biome_map.dimensions(), (6, 6));
    }

    #[test]
    fn test_biome_classification() {
        // Test various environmental conditions
        assert_eq!(
            classify_biome_from_environment(30.0, 0.1, 100.0),
            BiomeType::Desert
        );
        assert_eq!(
            classify_biome_from_environment(2.0, 0.5, 500.0),
            BiomeType::Tundra
        );
        assert_eq!(
            classify_biome_from_environment(25.0, 0.9, 50.0),
            BiomeType::Tropical
        );
        assert_eq!(
            classify_biome_from_environment(18.0, 0.9, 20.0),
            BiomeType::Wetland
        );
        assert_eq!(
            classify_biome_from_environment(18.0, 0.5, 200.0),
            BiomeType::Forest
        );
        assert_eq!(
            classify_biome_from_environment(12.0, 0.3, 300.0),
            BiomeType::Grassland
        );
    }

    #[test]
    fn test_leaf_area_index_calculation() {
        assert!(
            calculate_leaf_area_index(1.0, BiomeType::Tropical)
                > calculate_leaf_area_index(1.0, BiomeType::Desert)
        );
        assert!(
            calculate_leaf_area_index(1.0, BiomeType::Forest)
                > calculate_leaf_area_index(1.0, BiomeType::Grassland)
        );
        assert!(
            calculate_leaf_area_index(0.5, BiomeType::Forest)
                < calculate_leaf_area_index(1.0, BiomeType::Forest)
        );
    }

    #[test]
    fn test_ecosystem_feedback_parameters() {
        let params = EcosystemFeedbackParameters::default();

        // Verify parameters are in reasonable ranges
        assert!(params.base_evapotranspiration > 0.0 && params.base_evapotranspiration < 20.0);
        assert!(params.temperature_moderation > 0.0 && params.temperature_moderation < 10.0);
        assert!(params.humidity_coefficient > 0.0 && params.humidity_coefficient < 1.0);
        assert!(params.albedo_variation > 0.0 && params.albedo_variation < 1.0);
        assert!(params.moisture_enhancement > 1.0 && params.moisture_enhancement < 5.0);
        assert!(params.growth_rate > 0.0 && params.growth_rate < 100.0);
        assert!(params.water_stress_threshold > 0.0 && params.water_stress_threshold < 1.0);
        assert!(params.temperature_stress_range > 0.0 && params.temperature_stress_range < 50.0);
    }
}
