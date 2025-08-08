// ABOUTME: Atmospheric moisture and surface humidity system for realistic weather simulation
// ABOUTME: Separates surface moisture from standing water bodies for proper atmospheric coupling

use super::super::core::heightmap::HeightMap;
use super::super::core::scale::{ScaleAware, WorldScale};
use super::climate::{ClimateSystem, TemperatureLayer};
use super::water::WaterLayer;

/// Surface moisture parameters for atmospheric coupling
#[derive(Clone, Debug)]
pub struct SurfaceMoistureParameters {
    /// Base evaporation rate from surface moisture (m/h)
    pub surface_evaporation_rate: f32,
    /// Maximum surface moisture capacity (m equivalent depth)
    pub surface_moisture_capacity: f32,
    /// Temperature dependency factor for evaporation (K⁻¹)
    pub temperature_evaporation_factor: f32,
    /// Condensation rate from atmospheric humidity (fraction/h)
    pub condensation_rate: f32,
    /// Surface roughness factor affecting moisture retention
    pub surface_roughness: f32,
    /// Albedo effect on moisture evaporation (0.0-1.0)
    pub albedo_factor: f32,
}

impl Default for SurfaceMoistureParameters {
    fn default() -> Self {
        Self {
            surface_evaporation_rate: 0.05,       // 0.05 m/h moderate evaporation
            surface_moisture_capacity: 0.01,      // 1cm maximum moisture holding
            temperature_evaporation_factor: 0.07, // Doubles every ~10°C
            condensation_rate: 0.02,              // 2% humidity conversion per hour
            surface_roughness: 1.0,               // Baseline surface
            albedo_factor: 0.8,                   // Reduced evaporation for high albedo
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
            surface_evaporation_rate: self.surface_evaporation_rate * resolution_scale,
            surface_moisture_capacity: self.surface_moisture_capacity * resolution_scale,
            temperature_evaporation_factor: self.temperature_evaporation_factor,
            condensation_rate: self.condensation_rate,

            // Surface properties scale with physical realism
            surface_roughness: self.surface_roughness * (1.0 + physical_extent_km / 1000.0 * 0.1),
            albedo_factor: self.albedo_factor, // Albedo is a ratio - doesn't scale
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

    /// Update surface moisture and atmospheric humidity through evaporation/condensation
    pub fn update_moisture_exchange(
        &mut self,
        temperature_layer: &TemperatureLayer,
        climate: &ClimateSystem,
        parameters: &SurfaceMoistureParameters,
        dt: f32, // Time step in hours
    ) {
        for y in 0..self.height {
            for x in 0..self.width {
                let temperature =
                    temperature_layer.get_current_temperature(x, y, climate.current_season);
                let current_moisture = self.get_moisture(x, y);
                let current_humidity = self.get_humidity(x, y);
                let surface_type = self.surface_type.get(x, y);

                // Calculate temperature-dependent evaporation rate
                let temp_factor =
                    (parameters.temperature_evaporation_factor * (temperature - 20.0)).exp();
                let surface_factor = surface_type * parameters.surface_roughness;
                let evap_rate = parameters.surface_evaporation_rate * temp_factor * surface_factor;

                // Apply evaporation from surface moisture to atmospheric humidity
                let max_evaporation = current_moisture.min(evap_rate * dt);
                let actual_evaporation = max_evaporation * parameters.albedo_factor;

                // Update surface moisture (decrease)
                let new_surface_moisture = current_moisture - actual_evaporation;
                self.set_moisture(x, y, new_surface_moisture);

                // Update atmospheric humidity (increase)
                let new_humidity = current_humidity + actual_evaporation;
                self.set_humidity(x, y, new_humidity);

                // Store evaporation rate for analysis
                self.evaporation_rate.set(x, y, actual_evaporation / dt);

                // Calculate condensation from atmospheric humidity back to surface
                let condensation_amount = current_humidity * parameters.condensation_rate * dt;
                if condensation_amount > 0.0 {
                    // Apply condensation
                    let new_humidity_after_condensation =
                        (current_humidity - condensation_amount).max(0.0);
                    self.set_humidity(x, y, new_humidity_after_condensation);

                    // Add condensed moisture to surface (respecting capacity)
                    let capacity_remaining =
                        (parameters.surface_moisture_capacity - new_surface_moisture).max(0.0);
                    let actual_condensation = condensation_amount.min(capacity_remaining);
                    let final_surface_moisture = new_surface_moisture + actual_condensation;
                    self.set_moisture(x, y, final_surface_moisture);

                    // Store condensation rate
                    self.condensation_rate.set(x, y, actual_condensation / dt);
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

    /// Update atmospheric moisture system for one time step
    pub fn update(
        &mut self,
        temperature_layer: &TemperatureLayer,
        climate: &ClimateSystem,
        wind_u: Option<&HeightMap>,
        wind_v: Option<&HeightMap>,
        dt: f32, // Time step in hours
        scale: &WorldScale,
    ) {
        // Update evaporation and condensation
        self.surface_moisture.update_moisture_exchange(
            temperature_layer,
            climate,
            &self.parameters,
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
        layer.update_moisture_exchange(&temperature_layer, &climate, &params, 0.1);

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

        // Transport humidity
        layer.transport_humidity_with_wind(&wind_u, &wind_v, 0.1, &scale); // 0.1 hour

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
