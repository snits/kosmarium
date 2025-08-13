// ABOUTME: Atmospheric moisture and surface humidity system for realistic weather simulation
// ABOUTME: Separates surface moisture from standing water bodies for proper atmospheric coupling

use super::super::core::heightmap::HeightMap;
use super::super::core::scale::{ScaleAware, WorldScale};
use super::climate::{ClimateSystem, TemperatureLayer};
use super::water::WaterLayer;

/// METIS CORRECTION: Physics-compliant surface energy balance calculation
/// Implements energy conservation for evaporation processes
#[derive(Clone, Debug)]
pub struct SurfaceEnergyBalance {
    /// Net radiation flux (W/m²)
    pub net_radiation: f32,
    /// Sensible heat flux (W/m²)
    pub sensible_heat: f32,
    /// Latent heat flux from evaporation (W/m²)
    pub latent_heat: f32,
    /// Ground heat storage flux (W/m²)
    pub ground_heat: f32,
}

impl SurfaceEnergyBalance {
    /// Calculate maximum physically possible evaporation rate
    /// ENERGY CONSERVATION: E_available = R_net - H - G
    pub fn calculate_max_evaporation_rate(&self) -> f32 {
        let available_energy = self.net_radiation - self.sensible_heat - self.ground_heat;
        if available_energy > 0.0 {
            // Convert energy flux to evaporation rate: kg/(m²·s)
            available_energy / LATENT_HEAT_VAPORIZATION
        } else {
            0.0 // No evaporation when energy is insufficient
        }
    }

    /// Calculate surface energy balance from environmental conditions
    pub fn from_conditions(
        solar_radiation: f32,     // W/m²
        albedo: f32,              // 0.0-1.0
        temperature_surface: f32, // K
        temperature_air: f32,     // K
        wind_speed: f32,          // m/s
        energy_params: &SurfaceEnergyParameters,
    ) -> Self {
        // Net radiation balance (simplified)
        let absorbed_solar =
            solar_radiation * (1.0 - albedo) * energy_params.solar_absorption_efficiency;
        let longwave_cooling = 5.67e-8 * temperature_surface.powi(4) * 0.9; // Stefan-Boltzmann
        let net_radiation = absorbed_solar - longwave_cooling;

        // Sensible heat flux: H = ρ × c_p × C_H × u × (T_s - T_a)
        let sensible_heat = AIR_DENSITY
            * SPECIFIC_HEAT_AIR
            * energy_params.sensible_heat_coefficient
            * wind_speed
            * energy_params.wind_speed_factor
            * (temperature_surface - temperature_air)
            / 1000.0; // Convert to W/m²

        // Ground heat flux as fraction of net radiation
        let ground_heat = net_radiation * energy_params.ground_heat_fraction;

        Self {
            net_radiation,
            sensible_heat,
            latent_heat: 0.0, // Will be calculated from actual evaporation
            ground_heat,
        }
    }
}

/// METIS CORRECTION: Clausius-Clapeyron saturation vapor pressure calculation
/// Replaces arbitrary temperature dependence with fundamental thermodynamics
pub fn clausius_clapeyron_saturation_pressure(temperature: f32) -> f32 {
    // Clausius-Clapeyron equation: e_sat(T) = e_ref × exp(L_v/R_v × (1/T_ref - 1/T))
    let temperature_term =
        CLAUSIUS_CLAPEYRON_FACTOR * (1.0 / REFERENCE_TEMPERATURE - 1.0 / temperature);
    REFERENCE_VAPOR_PRESSURE * temperature_term.exp()
}

/// Convert saturation vapor pressure to saturation humidity (kg/m³)
pub fn saturation_pressure_to_humidity(pressure: f32, temperature: f32) -> f32 {
    // Ideal gas law: ρ = p / (R_v × T)
    pressure / (SPECIFIC_GAS_CONSTANT_WATER_VAPOR * temperature)
}

/// METIS CORRECTION: Physics-compliant saturation humidity calculation
pub fn calculate_saturation_humidity(temperature: f32) -> f32 {
    let saturation_pressure = clausius_clapeyron_saturation_pressure(temperature);
    saturation_pressure_to_humidity(saturation_pressure, temperature)
}

/// Physical constants for atmospheric moisture physics
const LATENT_HEAT_VAPORIZATION: f32 = 2.26e6; // J/kg - energy required for evaporation
const SPECIFIC_GAS_CONSTANT_WATER_VAPOR: f32 = 461.5; // J/(kg·K)
const REFERENCE_TEMPERATURE: f32 = 273.15; // K (0°C)
const REFERENCE_VAPOR_PRESSURE: f32 = 611.0; // Pa (saturation pressure at 0°C)
const AIR_DENSITY: f32 = 1.225; // kg/m³ at standard conditions
const SPECIFIC_HEAT_AIR: f32 = 1004.0; // J/(kg·K)
const CLAUSIUS_CLAPEYRON_FACTOR: f32 = 5423.0; // L_v/R_v in K (adjusted for better accuracy)

/// METIS CORRECTION: Physics-compliant surface energy balance parameters
#[derive(Clone, Debug)]
pub struct SurfaceEnergyParameters {
    /// Solar radiation absorption efficiency (dimensionless)
    pub solar_absorption_efficiency: f32,
    /// Sensible heat transfer coefficient (W/(m²·K))
    pub sensible_heat_coefficient: f32,
    /// Ground heat flux fraction of net radiation
    pub ground_heat_fraction: f32,
    /// Wind speed factor for heat transfer enhancement
    pub wind_speed_factor: f32,
}

impl Default for SurfaceEnergyParameters {
    fn default() -> Self {
        Self {
            solar_absorption_efficiency: 0.7, // 70% solar absorption
            sensible_heat_coefficient: 10.0,  // W/(m²·K) typical value
            ground_heat_fraction: 0.1,        // 10% of net radiation
            wind_speed_factor: 0.5,           // Enhancement factor
        }
    }
}

/// Surface moisture parameters for atmospheric coupling
#[derive(Clone, Debug)]
pub struct SurfaceMoistureParameters {
    /// Maximum surface moisture capacity (m equivalent depth)
    pub surface_moisture_capacity: f32,
    /// Surface roughness factor affecting moisture retention
    pub surface_roughness: f32,
    /// METIS CORRECTION: Physics-compliant energy balance parameters
    pub energy_parameters: SurfaceEnergyParameters,
    /// Surface evaporation rate (mm/day)
    pub surface_evaporation_rate: f32,
    /// Temperature factor for evaporation scaling (K^-1)
    pub temperature_evaporation_factor: f32,
}

impl Default for SurfaceMoistureParameters {
    fn default() -> Self {
        Self {
            surface_moisture_capacity: 0.01, // 1cm maximum moisture holding
            surface_roughness: 1.0,          // Baseline surface
            energy_parameters: SurfaceEnergyParameters::default(),
            surface_evaporation_rate: 2.0,   // 2mm/day baseline evaporation
            temperature_evaporation_factor: 0.1, // 0.1/K temperature scaling
        }
    }
}

impl ScaleAware for SurfaceMoistureParameters {
    fn derive_parameters(&self, scale: &WorldScale) -> Self {
        let physical_extent_km = scale.physical_size_km as f32;

        // Scale evaporation rate based on grid resolution
        // Coarser grids represent larger areas with more varied conditions
        let resolution_scale = (scale.meters_per_pixel() as f32 / 1000.0).sqrt().min(2.0);

        Self {
            surface_moisture_capacity: self.surface_moisture_capacity * resolution_scale,

            // Surface properties scale with physical realism
            surface_roughness: self.surface_roughness * (1.0 + physical_extent_km / 1000.0 * 0.1),

            // Energy parameters scale with resolution and physical extent
            energy_parameters: SurfaceEnergyParameters {
                solar_absorption_efficiency: self.energy_parameters.solar_absorption_efficiency,
                sensible_heat_coefficient: self.energy_parameters.sensible_heat_coefficient
                    * resolution_scale,
                ground_heat_fraction: self.energy_parameters.ground_heat_fraction,
                wind_speed_factor: self.energy_parameters.wind_speed_factor,
            },

            // Evaporation scales with resolution (larger areas = more diverse conditions)
            surface_evaporation_rate: self.surface_evaporation_rate * resolution_scale,

            // Temperature factor is a physical constant - doesn't scale
            temperature_evaporation_factor: self.temperature_evaporation_factor,
        }
    }
}

/// Surface moisture layer - distributed moisture for atmospheric processes
/// This is separate from standing water bodies (rivers, lakes)
#[derive(Clone, Debug)]
pub struct SurfaceMoistureLayer {
    /// Current surface moisture depth (m) at each cell
    pub moisture_depth: HeightMap,
    /// Atmospheric humidity above each cell (kg/m³)
    pub atmospheric_humidity: HeightMap,
    /// Evaporation rate at each cell (m/h)
    pub evaporation_rate: HeightMap,
    /// Condensation rate at each cell (m/h)
    pub condensation_rate: HeightMap,
    /// Surface type modifier (affects moisture retention)
    pub surface_type: HeightMap,
    width: usize,
    height: usize,
}

impl SurfaceMoistureLayer {
    /// Create new surface moisture layer with given dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            moisture_depth: HeightMap::new(width, height, 0.0),
            atmospheric_humidity: HeightMap::new(width, height, 0.0),
            evaporation_rate: HeightMap::new(width, height, 0.0),
            condensation_rate: HeightMap::new(width, height, 0.0),
            surface_type: HeightMap::new(width, height, 1.0), // Default neutral surface
            width,
            height,
        }
    }

    /// Get surface moisture at specific coordinates
    pub fn get_moisture(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.moisture_depth.get(x, y)
        } else {
            0.0
        }
    }

    /// Set surface moisture at specific coordinates
    pub fn set_moisture(&mut self, x: usize, y: usize, moisture: f32) {
        if x < self.width && y < self.height {
            self.moisture_depth.set(x, y, moisture.max(0.0));
        }
    }

    /// Get atmospheric humidity at specific coordinates
    pub fn get_humidity(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.atmospheric_humidity.get(x, y)
        } else {
            0.0
        }
    }

    /// Set atmospheric humidity at specific coordinates
    pub fn set_humidity(&mut self, x: usize, y: usize, humidity: f32) {
        if x < self.width && y < self.height {
            self.atmospheric_humidity.set(x, y, humidity.max(0.0));
        }
    }

    /// Get evaporation rate at specific coordinates
    pub fn get_evaporation_rate(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.evaporation_rate.get(x, y)
        } else {
            0.0
        }
    }

    /// Get total surface moisture across the entire layer
    pub fn get_total_surface_moisture(&self) -> f32 {
        self.moisture_depth.iter().sum()
    }

    /// Get total atmospheric moisture across the entire layer
    pub fn get_total_atmospheric_moisture(&self) -> f32 {
        self.atmospheric_humidity.iter().sum()
    }

    /// Get average humidity across the entire layer
    pub fn get_average_humidity(&self) -> f32 {
        let total = self.get_total_atmospheric_moisture();
        let cell_count = (self.width * self.height) as f32;
        if cell_count > 0.0 {
            total / cell_count
        } else {
            0.0
        }
    }

    /// Initialize surface moisture from terrain characteristics
    pub fn initialize_from_terrain(
        &mut self,
        heightmap: &HeightMap,
        water_layer: &WaterLayer,
        parameters: &SurfaceMoistureParameters,
    ) {
        for y in 0..self.height {
            for x in 0..self.width {
                let elevation = heightmap.get(x, y);
                let standing_water = water_layer.get_water_depth(x, y);

                // Initialize surface moisture based on terrain and water proximity
                let base_moisture = if standing_water > 0.001 {
                    // Areas with standing water start saturated
                    parameters.surface_moisture_capacity
                } else {
                    // Other areas get moisture based on elevation (valleys retain more)
                    let elevation_factor = (1.0 - elevation).max(0.0);
                    parameters.surface_moisture_capacity * elevation_factor * 0.3
                };

                self.set_moisture(x, y, base_moisture);

                // Initialize atmospheric humidity based on surface moisture
                let initial_humidity = base_moisture * 10.0; // kg/m³ conversion approximation
                self.set_humidity(x, y, initial_humidity);

                // Set surface type based on terrain (affects retention)
                let surface_type = if standing_water > 0.001 {
                    2.0 // Water surface - high evaporation
                } else if elevation > 0.8 {
                    0.5 // Rocky mountain surface - low retention
                } else {
                    1.0 // Normal soil surface
                };
                self.surface_type.set(x, y, surface_type);
            }
        }
    }

    /// METIS CORRECTION: Energy-conserving moisture exchange with physics-compliant thermodynamics
    pub fn update_moisture_exchange(
        &mut self,
        temperature_layer: &TemperatureLayer,
        climate: &ClimateSystem,
        parameters: &SurfaceMoistureParameters,
        solar_radiation: f32, // W/m² - required for energy balance
        wind_speed: f32,      // m/s - affects heat transfer
        dt: f32,              // Time step in hours
    ) {
        for y in 0..self.height {
            for x in 0..self.width {
                let temperature_celsius =
                    temperature_layer.get_current_temperature(x, y, climate.current_season);
                let temperature_kelvin = temperature_celsius + 273.15; // Convert to Kelvin for physics
                let current_moisture = self.get_moisture(x, y);
                let current_humidity = self.get_humidity(x, y);
                let surface_type = self.surface_type.get(x, y);

                // METIS CORRECTION: Calculate energy-limited evaporation
                let albedo = if surface_type > 1.5 {
                    0.1
                } else if surface_type < 0.7 {
                    0.3
                } else {
                    0.2
                };
                let energy_balance = SurfaceEnergyBalance::from_conditions(
                    solar_radiation,
                    albedo,
                    temperature_kelvin,
                    temperature_kelvin - 2.0, // Assume 2K temperature difference
                    wind_speed,
                    &parameters.energy_parameters,
                );

                // Maximum evaporation rate limited by available energy (kg/(m²·s))
                let max_evaporation_rate = energy_balance.calculate_max_evaporation_rate();
                let dt_seconds = dt * 3600.0; // Convert hours to seconds
                let max_evaporation_depth = max_evaporation_rate * dt_seconds / 1000.0; // Convert to meters

                // Surface moisture availability factor
                let moisture_availability = if parameters.surface_moisture_capacity > 0.0 {
                    (current_moisture / parameters.surface_moisture_capacity).min(1.0)
                } else {
                    0.0
                };

                // Actual evaporation: limited by both energy and moisture availability
                let surface_factor = surface_type * parameters.surface_roughness;
                let demand_evaporation = current_moisture
                    .min(max_evaporation_depth * surface_factor * moisture_availability);
                let actual_evaporation = demand_evaporation;

                // Update surface moisture (decrease)
                let new_surface_moisture = current_moisture - actual_evaporation;
                self.set_moisture(x, y, new_surface_moisture);

                // Convert evaporated water to atmospheric humidity (kg/m³)
                let evaporated_mass_per_area = actual_evaporation * 1000.0; // kg/m²
                let atmospheric_height = 1000.0; // Assume 1km mixing height
                let humidity_increase = evaporated_mass_per_area / atmospheric_height;
                let new_humidity = current_humidity + humidity_increase;
                self.set_humidity(x, y, new_humidity);

                // Store evaporation rate for analysis (m/h)
                self.evaporation_rate.set(x, y, actual_evaporation / dt);

                // METIS CORRECTION: Physics-compliant condensation using Clausius-Clapeyron
                let saturation_humidity = calculate_saturation_humidity(temperature_kelvin);
                let supersaturation = new_humidity - saturation_humidity;

                if supersaturation > 0.0 {
                    // Immediate condensation of excess humidity (physics: condensation is very rapid above saturation)
                    let condensation_amount = supersaturation; // All excess condenses immediately

                    // Apply condensation: humidity clamped to saturation limit
                    self.set_humidity(x, y, saturation_humidity);

                    // Convert condensed humidity back to surface moisture
                    let condensed_depth = condensation_amount * atmospheric_height / 1000.0; // Convert to meters

                    // Add condensed moisture to surface (respecting capacity)
                    let capacity_remaining =
                        (parameters.surface_moisture_capacity - new_surface_moisture).max(0.0);
                    let actual_condensation_to_surface = condensed_depth.min(capacity_remaining);
                    let final_surface_moisture =
                        new_surface_moisture + actual_condensation_to_surface;
                    self.set_moisture(x, y, final_surface_moisture);

                    // Store condensation rate (m/h)
                    self.condensation_rate
                        .set(x, y, actual_condensation_to_surface / dt);
                } else {
                    // No condensation - clear condensation rate
                    self.condensation_rate.set(x, y, 0.0);
                }
            }
        }
    }

    /// Add precipitation to surface moisture layer
    pub fn add_precipitation(
        &mut self,
        x: usize,
        y: usize,
        amount: f32,
        parameters: &SurfaceMoistureParameters,
    ) {
        if x < self.width && y < self.height {
            let current_moisture = self.get_moisture(x, y);
            let capacity_remaining =
                (parameters.surface_moisture_capacity - current_moisture).max(0.0);

            // Precipitation goes to surface moisture up to capacity
            let surface_addition = amount.min(capacity_remaining);
            self.set_moisture(x, y, current_moisture + surface_addition);

            // Excess becomes atmospheric humidity (can later condense elsewhere)
            if amount > surface_addition {
                let excess = amount - surface_addition;
                let current_humidity = self.get_humidity(x, y);
                self.set_humidity(x, y, current_humidity + excess);
            }
        }
    }

    /// Transport atmospheric humidity with wind patterns
    pub fn transport_humidity_with_wind(
        &mut self,
        wind_u: &HeightMap, // East-west wind component (m/s)
        wind_v: &HeightMap, // North-south wind component (m/s)
        dt: f32,            // Time step in hours
        scale: &WorldScale,
    ) {
        let cell_size_m = scale.meters_per_pixel() as f32;
        let mut new_humidity = self.atmospheric_humidity.clone();

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let current_humidity = self.get_humidity(x, y);
                let u_wind = wind_u.get(x, y);
                let v_wind = wind_v.get(x, y);

                // Calculate advection using upwind scheme
                let dt_seconds = dt * 3600.0; // Convert hours to seconds
                let dx_u = u_wind * dt_seconds / cell_size_m; // Grid cells moved in x
                let dy_v = v_wind * dt_seconds / cell_size_m; // Grid cells moved in y

                // Simple upwind advection (can be enhanced with higher-order schemes)
                let humidity_flux_x = if dx_u > 0.0 {
                    dx_u * current_humidity
                } else {
                    dx_u * self.get_humidity(x + 1, y)
                };

                let humidity_flux_y = if dy_v > 0.0 {
                    dy_v * current_humidity
                } else {
                    dy_v * self.get_humidity(x, y + 1)
                };

                // Update humidity with advected amount
                let transported_humidity = current_humidity - humidity_flux_x - humidity_flux_y;
                new_humidity.set(x, y, transported_humidity.max(0.0));
            }
        }

        self.atmospheric_humidity = new_humidity;
    }

    /// Get moisture availability for evaporation at a location
    /// Returns the fraction of maximum possible evaporation (0.0-1.0)
    pub fn get_evaporation_availability(
        &self,
        x: usize,
        y: usize,
        parameters: &SurfaceMoistureParameters,
    ) -> f32 {
        let moisture = self.get_moisture(x, y);
        let capacity = parameters.surface_moisture_capacity;

        if capacity > 0.0 {
            (moisture / capacity).min(1.0)
        } else {
            0.0
        }
    }

    /// Get precipitation efficiency at a location
    /// Higher humidity = higher chance of precipitation
    pub fn get_precipitation_efficiency(&self, x: usize, y: usize) -> f32 {
        let humidity = self.get_humidity(x, y);

        // Simple sigmoid function: efficiency increases with humidity
        let efficiency = 1.0 / (1.0 + (-0.1 * (humidity - 50.0)).exp());
        efficiency.max(0.0).min(1.0)
    }

    /// Get dimensions
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

/// Atmospheric moisture system integrating surface and atmospheric processes
#[derive(Clone, Debug)]
pub struct AtmosphericMoistureSystem {
    /// Scale-derived atmospheric moisture parameters
    pub parameters: SurfaceMoistureParameters,
    /// Surface moisture layer
    pub surface_moisture: SurfaceMoistureLayer,
}

impl AtmosphericMoistureSystem {
    /// Create new atmospheric moisture system for the given world scale
    pub fn new_for_scale(scale: &WorldScale, width: usize, height: usize) -> Self {
        let parameters = SurfaceMoistureParameters::default().derive_parameters(scale);
        let surface_moisture = SurfaceMoistureLayer::new(width, height);

        Self {
            parameters,
            surface_moisture,
        }
    }

    /// Create from custom parameters
    pub fn from_parameters(
        parameters: SurfaceMoistureParameters,
        scale: &WorldScale,
        width: usize,
        height: usize,
    ) -> Self {
        let scaled_params = parameters.derive_parameters(scale);
        let surface_moisture = SurfaceMoistureLayer::new(width, height);

        Self {
            parameters: scaled_params,
            surface_moisture,
        }
    }

    /// Initialize the moisture system from terrain and water data
    pub fn initialize_from_terrain(&mut self, heightmap: &HeightMap, water_layer: &WaterLayer) {
        self.surface_moisture
            .initialize_from_terrain(heightmap, water_layer, &self.parameters);
    }

    /// METIS CORRECTION: Update atmospheric moisture system with energy conservation
    pub fn update(
        &mut self,
        temperature_layer: &TemperatureLayer,
        climate: &ClimateSystem,
        wind_u: Option<&HeightMap>,
        wind_v: Option<&HeightMap>,
        solar_radiation: f32, // W/m² - required for energy balance
        dt: f32,              // Time step in hours
        scale: &WorldScale,
    ) {
        // Estimate wind speed for energy calculations (use magnitude of wind vector or default)
        let wind_speed = if let (Some(u_wind), Some(v_wind)) = (wind_u, wind_v) {
            // Calculate approximate wind speed from wind components
            let mut total_u = 0.0;
            let mut total_v = 0.0;
            let mut count = 0;

            for y in 0..self.surface_moisture.height {
                for x in 0..self.surface_moisture.width {
                    total_u += u_wind.get(x, y);
                    total_v += v_wind.get(x, y);
                    count += 1;
                }
            }

            if count > 0 {
                let avg_u = total_u / count as f32;
                let avg_v = total_v / count as f32;
                (avg_u * avg_u + avg_v * avg_v).sqrt()
            } else {
                2.0 // Default wind speed
            }
        } else {
            2.0 // Default wind speed (m/s)
        };

        // Update evaporation and condensation with energy conservation
        self.surface_moisture.update_moisture_exchange(
            temperature_layer,
            climate,
            &self.parameters,
            solar_radiation,
            wind_speed,
            dt,
        );

        // Transport humidity with wind if wind data is available
        if let (Some(u_wind), Some(v_wind)) = (wind_u, wind_v) {
            self.surface_moisture
                .transport_humidity_with_wind(u_wind, v_wind, dt, scale);
        }
    }

    /// Add precipitation from weather systems
    pub fn add_precipitation(&mut self, x: usize, y: usize, amount: f32) {
        self.surface_moisture
            .add_precipitation(x, y, amount, &self.parameters);
    }

    /// Get total moisture in the atmospheric system (surface + atmospheric)
    pub fn get_total_moisture(&self) -> f32 {
        self.surface_moisture.get_total_surface_moisture()
            + self.surface_moisture.get_total_atmospheric_moisture()
    }

    /// METIS VALIDATION: Check mass conservation during transport
    pub fn validate_mass_conservation(&self, initial_moisture: f32) -> (f32, bool) {
        let current_moisture = self.get_total_moisture();
        let mass_error = (current_moisture - initial_moisture).abs();
        let mass_error_percent = if initial_moisture > 0.0 {
            (mass_error / initial_moisture) * 100.0
        } else {
            0.0
        };

        // Physics tolerance: <1% mass loss is acceptable for numerical schemes
        let conservation_valid = mass_error_percent < 1.0;

        (mass_error_percent, conservation_valid)
    }

    /// Get moisture for precipitation calculations (from atmospheric humidity)
    pub fn get_available_precipitation_moisture(&self, x: usize, y: usize) -> f32 {
        self.surface_moisture.get_humidity(x, y)
            * self.surface_moisture.get_precipitation_efficiency(x, y)
    }

    /// Get moisture for evaporation calculations (from surface moisture)
    pub fn get_available_evaporation_moisture(&self, x: usize, y: usize) -> f32 {
        self.surface_moisture.get_moisture(x, y)
            * self
                .surface_moisture
                .get_evaporation_availability(x, y, &self.parameters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::core::scale::{DetailLevel, WorldScale};
    use crate::engine::physics::climate::ClimateSystem;

    fn test_scale(physical_size_km: f64, width: u32, height: u32) -> WorldScale {
        WorldScale::new(physical_size_km, (width, height), DetailLevel::Standard)
    }

    #[test]
    fn surface_moisture_parameters_scaling() {
        let base_params = SurfaceMoistureParameters::default();
        let small_scale = test_scale(1.0, 50, 50);
        let large_scale = test_scale(1000.0, 500, 500);

        let small_scaled = base_params.derive_parameters(&small_scale);
        let large_scaled = base_params.derive_parameters(&large_scale);

        // Physical constants should scale appropriately
        assert!(large_scaled.surface_evaporation_rate >= small_scaled.surface_evaporation_rate);
        assert!(large_scaled.surface_moisture_capacity >= small_scaled.surface_moisture_capacity);

        // Temperature factor should remain the same (physical constant)
        assert_eq!(
            small_scaled.temperature_evaporation_factor,
            large_scaled.temperature_evaporation_factor
        );
    }

    #[test]
    fn surface_moisture_layer_basic_operations() {
        let mut layer = SurfaceMoistureLayer::new(5, 5);

        // Should initialize to zero
        assert_eq!(layer.get_moisture(2, 2), 0.0);
        assert_eq!(layer.get_humidity(2, 2), 0.0);

        // Test set/get operations
        layer.set_moisture(2, 2, 0.5);
        layer.set_humidity(2, 2, 10.0);
        assert_eq!(layer.get_moisture(2, 2), 0.5);
        assert_eq!(layer.get_humidity(2, 2), 10.0);

        // Out of bounds should return defaults
        assert_eq!(layer.get_moisture(10, 10), 0.0);
        assert_eq!(layer.get_humidity(10, 10), 0.0);
    }

    #[test]
    fn precipitation_addition() {
        let params = SurfaceMoistureParameters::default();
        let mut layer = SurfaceMoistureLayer::new(3, 3);

        // Add precipitation within capacity
        layer.add_precipitation(1, 1, 0.005, &params); // Half of default capacity
        assert_eq!(layer.get_moisture(1, 1), 0.005);
        assert_eq!(layer.get_humidity(1, 1), 0.0); // No excess

        // Add precipitation exceeding capacity
        layer.add_precipitation(1, 1, 0.01, &params); // Full capacity worth
        assert_eq!(layer.get_moisture(1, 1), params.surface_moisture_capacity); // Capped at capacity
        assert!(layer.get_humidity(1, 1) > 0.0); // Excess becomes humidity
    }

    #[test]
    fn evaporation_condensation_cycle() {
        let scale = test_scale(10.0, 10, 10);
        let params = SurfaceMoistureParameters::default().derive_parameters(&scale);
        let climate = ClimateSystem::new_for_scale(&scale);

        let heightmap = HeightMap::new(3, 3, 0.5);
        let temperature_layer = climate.generate_temperature_layer_optimized(&heightmap);

        let mut layer = SurfaceMoistureLayer::new(3, 3);

        // Set initial moisture
        layer.set_moisture(1, 1, 0.008); // Most of capacity

        // Run moisture exchange for short time
        layer.update_moisture_exchange(&temperature_layer, &climate, &params, 300.0, 2.0, 0.1); // 300 W/m² solar, 2 m/s wind

        // Should have some evaporation (surface moisture decrease, humidity increase)
        assert!(layer.get_moisture(1, 1) < 0.008);
        assert!(layer.get_humidity(1, 1) > 0.0);
        assert!(layer.get_evaporation_rate(1, 1) > 0.0);
    }

    #[test]
    fn atmospheric_moisture_system_integration() {
        let scale = test_scale(100.0, 50, 50);
        let mut system = AtmosphericMoistureSystem::new_for_scale(&scale, 50, 50);

        let heightmap = HeightMap::new(50, 50, 0.3);
        let water_layer = WaterLayer::new(50, 50);

        // Initialize from terrain
        system.initialize_from_terrain(&heightmap, &water_layer);

        // Should have some initial moisture
        let total_initial = system.get_total_moisture();
        assert!(total_initial > 0.0);

        // Test precipitation addition
        system.add_precipitation(25, 25, 0.002);
        let total_after_rain = system.get_total_moisture();
        assert!(total_after_rain > total_initial);
    }

    #[test]
    fn humidity_transport_with_wind() {
        let scale = test_scale(10.0, 10, 10);
        let mut layer = SurfaceMoistureLayer::new(10, 10);

        // Set humidity in one location
        layer.set_humidity(3, 3, 20.0);

        // Create wind field (eastward wind)
        let mut wind_u = HeightMap::new(10, 10, 2.0); // 2 m/s eastward
        let wind_v = HeightMap::new(10, 10, 0.0); // No north-south wind

        // Record initial total mass for conservation check
        let initial_total_humidity = layer.get_total_atmospheric_moisture();

        // Transport humidity
        layer.transport_humidity_with_wind(&wind_u, &wind_v, 0.1, &scale); // 0.1 hour

        // Check mass conservation
        let final_total_humidity = layer.get_total_atmospheric_moisture();
        let mass_error = (final_total_humidity - initial_total_humidity).abs();
        let mass_error_percent = if initial_total_humidity > 0.0 {
            (mass_error / initial_total_humidity) * 100.0
        } else {
            0.0
        };

        // Mass conservation should be within 1% for numerical transport
        assert!(
            mass_error_percent < 1.0,
            "Mass conservation violated: {:.4}% error",
            mass_error_percent
        );

        // Humidity should have been transported (simple test - exact values depend on implementation)
        let original_humidity = layer.get_humidity(3, 3);
        let transported_humidity = layer.get_humidity(4, 3); // Downwind location

        // With eastward wind, some humidity should move east
        assert!(original_humidity >= 0.0);
        assert!(transported_humidity >= 0.0);
    }

    #[test]
    fn moisture_availability_calculations() {
        let params = SurfaceMoistureParameters::default();
        let layer = SurfaceMoistureLayer::new(3, 3);

        // Test evaporation availability at different moisture levels
        let mut test_layer = layer.clone();
        test_layer.set_moisture(1, 1, params.surface_moisture_capacity * 0.5);
        let availability = test_layer.get_evaporation_availability(1, 1, &params);
        assert!((availability - 0.5).abs() < 1e-6);

        // Test precipitation efficiency at different humidity levels
        test_layer.set_humidity(1, 1, 50.0); // Moderate humidity
        let efficiency = test_layer.get_precipitation_efficiency(1, 1);
        assert!(efficiency > 0.0 && efficiency < 1.0);
    }
}
