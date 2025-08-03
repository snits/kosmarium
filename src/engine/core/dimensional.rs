// ABOUTME: Dimensional analysis system for proper physical units in water flow simulation
// ABOUTME: Provides unit conversion, physical validation, and dimensional correctness checking

use super::scale::WorldScale;

/// Physical units for dimensional analysis
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PhysicalUnit {
    // Length units
    Meters,
    Millimeters,
    Kilometers,

    // Time units
    Seconds,
    Hours,

    // Velocity units
    MetersPerSecond,
    KilometersPerHour,

    // Volume units
    CubicMeters,
    CubicMetersPerSecond,

    // Area units
    SquareMeters,

    // Precipitation units
    MillimetersPerHour,

    // Temperature units
    Celsius,
    Kelvin,

    // Temperature gradient units
    CelsiusPerMeter,
    CelsiusPerKilometer,

    // Dimensionless
    Dimensionless,
}

/// A physical quantity with both value and units
#[derive(Clone, Copy, Debug)]
pub struct PhysicalQuantity {
    pub value: f64,
    pub unit: PhysicalUnit,
}

impl PhysicalQuantity {
    pub fn new(value: f64, unit: PhysicalUnit) -> Self {
        Self { value, unit }
    }

    /// Convert to different unit (panics if incompatible)
    pub fn convert_to(&self, target_unit: PhysicalUnit) -> PhysicalQuantity {
        let new_value = match (self.unit, target_unit) {
            // Same unit - no conversion needed
            (unit1, unit2) if unit1 == unit2 => self.value,

            // Length conversions
            (PhysicalUnit::Meters, PhysicalUnit::Millimeters) => self.value * 1000.0,
            (PhysicalUnit::Millimeters, PhysicalUnit::Meters) => self.value / 1000.0,
            (PhysicalUnit::Meters, PhysicalUnit::Kilometers) => self.value / 1000.0,
            (PhysicalUnit::Kilometers, PhysicalUnit::Meters) => self.value * 1000.0,
            (PhysicalUnit::Millimeters, PhysicalUnit::Kilometers) => self.value / 1_000_000.0,
            (PhysicalUnit::Kilometers, PhysicalUnit::Millimeters) => self.value * 1_000_000.0,

            // Time conversions
            (PhysicalUnit::Hours, PhysicalUnit::Seconds) => self.value * 3600.0,
            (PhysicalUnit::Seconds, PhysicalUnit::Hours) => self.value / 3600.0,

            // Velocity conversions
            (PhysicalUnit::MetersPerSecond, PhysicalUnit::KilometersPerHour) => self.value * 3.6,
            (PhysicalUnit::KilometersPerHour, PhysicalUnit::MetersPerSecond) => self.value / 3.6,

            // Temperature conversions
            (PhysicalUnit::Celsius, PhysicalUnit::Kelvin) => self.value + 273.15,
            (PhysicalUnit::Kelvin, PhysicalUnit::Celsius) => self.value - 273.15,

            // Temperature gradient conversions
            (PhysicalUnit::CelsiusPerMeter, PhysicalUnit::CelsiusPerKilometer) => {
                self.value * 1000.0
            }
            (PhysicalUnit::CelsiusPerKilometer, PhysicalUnit::CelsiusPerMeter) => {
                self.value / 1000.0
            }

            // Area conversions (derived from length)
            (PhysicalUnit::SquareMeters, PhysicalUnit::SquareMeters) => self.value,

            // Volume rate conversions
            (PhysicalUnit::CubicMetersPerSecond, PhysicalUnit::CubicMetersPerSecond) => self.value,

            // Precipitation conversions
            (PhysicalUnit::MillimetersPerHour, PhysicalUnit::MillimetersPerHour) => self.value,

            // Dimensionless
            (PhysicalUnit::Dimensionless, PhysicalUnit::Dimensionless) => self.value,

            _ => panic!(
                "Incompatible unit conversion: {:?} to {:?}",
                self.unit, target_unit
            ),
        };

        PhysicalQuantity::new(new_value, target_unit)
    }

    /// Check if units are compatible for conversion
    pub fn is_compatible_with(&self, other_unit: PhysicalUnit) -> bool {
        use PhysicalUnit::*;

        // Same units are always compatible
        if self.unit == other_unit {
            return true;
        }

        // Check specific compatible conversions
        matches!(
            (self.unit, other_unit),
            // Length units
            (Meters, Millimeters) | (Millimeters, Meters) |
            (Meters, Kilometers) | (Kilometers, Meters) |
            (Millimeters, Kilometers) | (Kilometers, Millimeters) |
            // Time units
            (Hours, Seconds) | (Seconds, Hours) |
            // Velocity units
            (MetersPerSecond, KilometersPerHour) | (KilometersPerHour, MetersPerSecond) |
            // Temperature units
            (Celsius, Kelvin) | (Kelvin, Celsius) |
            // Temperature gradient units
            (CelsiusPerMeter, CelsiusPerKilometer) | (CelsiusPerKilometer, CelsiusPerMeter)
        )
    }
}

/// Physical parameters for water flow with proper dimensional analysis
#[derive(Clone, Debug)]
pub struct DimensionalWaterFlowParameters {
    /// Maximum flow velocity (m/s)
    pub max_velocity: PhysicalQuantity,

    /// Rainfall rate (mm/h)
    pub rainfall_rate: PhysicalQuantity,

    /// Evaporation rate (mm/h)
    pub evaporation_rate: PhysicalQuantity,

    /// Grid cell size (m)
    pub cell_size: PhysicalQuantity,

    /// Time step for numerical integration (s)
    pub timestep: PhysicalQuantity,

    /// Water depth threshold for numerical stability (m)
    pub depth_threshold: PhysicalQuantity,
}

impl DimensionalWaterFlowParameters {
    /// Create parameters with physical units
    pub fn new(
        max_velocity_ms: f64,
        rainfall_rate_mmh: f64,
        evaporation_rate_mmh: f64,
        cell_size_m: f64,
        timestep_s: f64,
        depth_threshold_m: f64,
    ) -> Self {
        Self {
            max_velocity: PhysicalQuantity::new(max_velocity_ms, PhysicalUnit::MetersPerSecond),
            rainfall_rate: PhysicalQuantity::new(
                rainfall_rate_mmh,
                PhysicalUnit::MillimetersPerHour,
            ),
            evaporation_rate: PhysicalQuantity::new(
                evaporation_rate_mmh,
                PhysicalUnit::MillimetersPerHour,
            ),
            cell_size: PhysicalQuantity::new(cell_size_m, PhysicalUnit::Meters),
            timestep: PhysicalQuantity::new(timestep_s, PhysicalUnit::Seconds),
            depth_threshold: PhysicalQuantity::new(depth_threshold_m, PhysicalUnit::Meters),
        }
    }

    /// Validate CFL condition for numerical stability
    /// CFL condition: v * dt / dx < C (where C ≤ 1)
    pub fn validate_cfl_condition(&self, safety_factor: f64) -> CflValidationResult {
        let velocity_ms = self
            .max_velocity
            .convert_to(PhysicalUnit::MetersPerSecond)
            .value;
        let timestep_s = self.timestep.convert_to(PhysicalUnit::Seconds).value;
        let cell_size_m = self.cell_size.convert_to(PhysicalUnit::Meters).value;

        let cfl_number = velocity_ms * timestep_s / cell_size_m;
        let is_stable = cfl_number <= safety_factor;

        CflValidationResult {
            cfl_number,
            is_stable,
            recommended_timestep_s: if !is_stable {
                Some(safety_factor * cell_size_m / velocity_ms)
            } else {
                None
            },
        }
    }

    /// Calculate volume flow rate (m³/s) for a given cross-sectional area
    pub fn calculate_volume_flow_rate(&self, cross_section_area_m2: f64) -> PhysicalQuantity {
        let velocity_ms = self
            .max_velocity
            .convert_to(PhysicalUnit::MetersPerSecond)
            .value;
        let flow_rate_m3s = velocity_ms * cross_section_area_m2;

        PhysicalQuantity::new(flow_rate_m3s, PhysicalUnit::CubicMetersPerSecond)
    }

    /// Convert rainfall rate to water depth per timestep (m)
    pub fn rainfall_depth_per_timestep(&self) -> PhysicalQuantity {
        let rainfall_mmh = self
            .rainfall_rate
            .convert_to(PhysicalUnit::MillimetersPerHour)
            .value;
        let timestep_s = self.timestep.convert_to(PhysicalUnit::Seconds).value;

        // Convert mm/h to m/s, then multiply by timestep
        let rainfall_ms = rainfall_mmh / (1000.0 * 3600.0); // mm/h to m/s
        let depth_m = rainfall_ms * timestep_s;

        PhysicalQuantity::new(depth_m, PhysicalUnit::Meters)
    }

    /// Convert evaporation rate to water depth lost per timestep (m)
    pub fn evaporation_depth_per_timestep(&self) -> PhysicalQuantity {
        let evaporation_mmh = self
            .evaporation_rate
            .convert_to(PhysicalUnit::MillimetersPerHour)
            .value;
        let timestep_s = self.timestep.convert_to(PhysicalUnit::Seconds).value;

        // Convert mm/h to m/s, then multiply by timestep
        let evaporation_ms = evaporation_mmh / (1000.0 * 3600.0); // mm/h to m/s
        let depth_m = evaporation_ms * timestep_s;

        PhysicalQuantity::new(depth_m, PhysicalUnit::Meters)
    }
}

/// Result of CFL stability validation
#[derive(Clone, Debug)]
pub struct CflValidationResult {
    pub cfl_number: f64,
    pub is_stable: bool,
    pub recommended_timestep_s: Option<f64>,
}

/// Dimensional analysis utilities for WorldScale integration
pub struct DimensionalAnalysis;

impl DimensionalAnalysis {
    /// Create dimensional parameters from WorldScale
    pub fn from_world_scale(
        scale: &WorldScale,
        max_velocity_ms: f64,
        rainfall_rate_mmh: f64,
        evaporation_rate_mmh: f64,
    ) -> DimensionalWaterFlowParameters {
        let cell_size_m = scale.meters_per_pixel();

        // Calculate stable timestep using CFL condition
        let cfl_safety_factor = 0.5; // Conservative safety factor
        let stable_timestep_s = cfl_safety_factor * cell_size_m / max_velocity_ms;

        // Set depth threshold based on cell size (typically 1% of cell size)
        let depth_threshold_m = cell_size_m * 0.01;

        DimensionalWaterFlowParameters::new(
            max_velocity_ms,
            rainfall_rate_mmh,
            evaporation_rate_mmh,
            cell_size_m,
            stable_timestep_s,
            depth_threshold_m,
        )
    }

    /// Validate dimensional consistency across parameters
    pub fn validate_dimensional_consistency(
        params: &DimensionalWaterFlowParameters,
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check if CFL condition is satisfied
        let cfl_result = params.validate_cfl_condition(0.5);
        if !cfl_result.is_stable {
            warnings.push(format!(
                "CFL condition violated: CFL number = {:.3}, consider reducing timestep to {:.6}s",
                cfl_result.cfl_number,
                cfl_result.recommended_timestep_s.unwrap_or(0.0)
            ));
        }

        // Check for reasonable physical values
        let velocity_ms = params
            .max_velocity
            .convert_to(PhysicalUnit::MetersPerSecond)
            .value;
        if velocity_ms > 10.0 {
            warnings.push(format!(
                "Unusually high water velocity: {:.2} m/s (typical river velocities: 0.1-3 m/s)",
                velocity_ms
            ));
        }

        let rainfall_mmh = params
            .rainfall_rate
            .convert_to(PhysicalUnit::MillimetersPerHour)
            .value;
        if rainfall_mmh > 100.0 {
            warnings.push(format!(
                "Extremely high rainfall rate: {:.2} mm/h (heavy rain typically < 50 mm/h)",
                rainfall_mmh
            ));
        }

        let cell_size_m = params.cell_size.convert_to(PhysicalUnit::Meters).value;
        if cell_size_m < 1.0 || cell_size_m > 10000.0 {
            warnings.push(format!(
                "Unusual cell size: {:.2} m (typical range: 1-10000 m)",
                cell_size_m
            ));
        }

        warnings
    }
}

/// Physical parameters for climate system with proper dimensional analysis
#[derive(Clone, Debug)]
pub struct DimensionalClimateParameters {
    /// Base temperature at sea level (°C)
    pub base_temperature: PhysicalQuantity,

    /// Temperature decrease per meter elevation (°C/m)  
    pub elevation_lapse_rate: PhysicalQuantity,

    /// Seasonal temperature variation amplitude (°C)
    pub seasonal_amplitude: PhysicalQuantity,

    /// Temperature change per degree latitude (°C/degree)
    pub latitude_gradient: PhysicalQuantity,

    /// Grid cell size for spatial calculations (m)
    pub cell_size: PhysicalQuantity,

    /// Physical domain extent for scaling (km)
    pub domain_size: PhysicalQuantity,
}

impl DimensionalClimateParameters {
    /// Create dimensional climate parameters
    pub fn new(
        base_temperature_c: f64,
        elevation_lapse_rate_c_per_m: f64,
        seasonal_amplitude_c: f64,
        latitude_gradient_c_per_deg: f64,
        cell_size_m: f64,
        domain_size_km: f64,
    ) -> Self {
        Self {
            base_temperature: PhysicalQuantity::new(base_temperature_c, PhysicalUnit::Celsius),
            elevation_lapse_rate: PhysicalQuantity::new(
                elevation_lapse_rate_c_per_m,
                PhysicalUnit::CelsiusPerMeter,
            ),
            seasonal_amplitude: PhysicalQuantity::new(seasonal_amplitude_c, PhysicalUnit::Celsius),
            latitude_gradient: PhysicalQuantity::new(
                latitude_gradient_c_per_deg,
                PhysicalUnit::Celsius,
            ), // Per degree
            cell_size: PhysicalQuantity::new(cell_size_m, PhysicalUnit::Meters),
            domain_size: PhysicalQuantity::new(domain_size_km, PhysicalUnit::Kilometers),
        }
    }

    /// Calculate temperature at given elevation (above sea level)
    pub fn temperature_at_elevation(&self, elevation_m: f64) -> PhysicalQuantity {
        let base_temp_c = self
            .base_temperature
            .convert_to(PhysicalUnit::Celsius)
            .value;
        let lapse_rate_c_per_m = self
            .elevation_lapse_rate
            .convert_to(PhysicalUnit::CelsiusPerMeter)
            .value;

        let temperature_c = base_temp_c - (elevation_m * lapse_rate_c_per_m);
        PhysicalQuantity::new(temperature_c, PhysicalUnit::Celsius)
    }

    /// Calculate seasonal temperature variation
    pub fn seasonal_temperature(
        &self,
        base_temp: PhysicalQuantity,
        season_factor: f64,
    ) -> PhysicalQuantity {
        let base_c = base_temp.convert_to(PhysicalUnit::Celsius).value;
        let amplitude_c = self
            .seasonal_amplitude
            .convert_to(PhysicalUnit::Celsius)
            .value;

        // Season factor: 0.0 = winter, 0.5 = spring/fall, 1.0 = summer
        let seasonal_offset = amplitude_c * (season_factor - 0.5) * 2.0;
        let seasonal_temp_c = base_c + seasonal_offset;

        PhysicalQuantity::new(seasonal_temp_c, PhysicalUnit::Celsius)
    }

    /// Calculate latitude-based temperature variation
    pub fn latitude_temperature_adjustment(&self, latitude_factor: f64) -> PhysicalQuantity {
        let gradient_c = self
            .latitude_gradient
            .convert_to(PhysicalUnit::Celsius)
            .value;

        // Latitude factor: 0.0 = equator, 1.0 = pole
        let temp_adjustment_c = -gradient_c * latitude_factor * 90.0; // 90 degrees from equator to pole

        PhysicalQuantity::new(temp_adjustment_c, PhysicalUnit::Celsius)
    }

    /// Validate climate parameters for physical realism
    pub fn validate_parameters(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check base temperature
        let base_temp_c = self
            .base_temperature
            .convert_to(PhysicalUnit::Celsius)
            .value;
        if base_temp_c < -50.0 || base_temp_c > 50.0 {
            warnings.push(format!(
                "Extreme base temperature: {:.1}°C (typical range: -50°C to 50°C)",
                base_temp_c
            ));
        }

        // Check lapse rate
        let lapse_rate = self
            .elevation_lapse_rate
            .convert_to(PhysicalUnit::CelsiusPerKilometer)
            .value;
        if lapse_rate < 2.0 || lapse_rate > 12.0 {
            warnings.push(format!(
                "Unusual lapse rate: {:.1}°C/km (typical atmospheric: 6.5°C/km, range: 2-12°C/km)",
                lapse_rate
            ));
        }

        // Check seasonal amplitude
        let amplitude_c = self
            .seasonal_amplitude
            .convert_to(PhysicalUnit::Celsius)
            .value;
        if amplitude_c > 50.0 {
            warnings.push(format!(
                "Extreme seasonal variation: {:.1}°C (typical continental: 10-40°C)",
                amplitude_c
            ));
        }

        // Check domain size reasonableness
        let domain_km = self.domain_size.convert_to(PhysicalUnit::Kilometers).value;
        if domain_km < 0.1 || domain_km > 40000.0 {
            warnings.push(format!(
                "Unusual domain size: {:.1} km (typical simulation range: 0.1-40000 km)",
                domain_km
            ));
        }

        warnings
    }
}

/// Dimensional analysis utilities for climate systems
pub struct ClimateAnalysis;

impl ClimateAnalysis {
    /// Create dimensional climate parameters from WorldScale
    pub fn from_world_scale(
        scale: &WorldScale,
        base_temperature_c: f64,
        elevation_lapse_rate_c_per_m: f64,
        seasonal_amplitude_c: f64,
        latitude_gradient_c_per_deg: f64,
    ) -> DimensionalClimateParameters {
        let cell_size_m = scale.meters_per_pixel();
        let domain_size_km = scale.physical_size_km;

        DimensionalClimateParameters::new(
            base_temperature_c,
            elevation_lapse_rate_c_per_m,
            seasonal_amplitude_c,
            latitude_gradient_c_per_deg,
            cell_size_m,
            domain_size_km,
        )
    }

    /// Validate dimensional consistency for climate systems
    pub fn validate_climate_consistency(params: &DimensionalClimateParameters) -> Vec<String> {
        let mut warnings = params.validate_parameters();

        // Check cell size vs domain size ratio
        let cell_size_m = params.cell_size.convert_to(PhysicalUnit::Meters).value;
        let domain_size_m = params.domain_size.convert_to(PhysicalUnit::Meters).value;
        let resolution = domain_size_m / cell_size_m;

        if resolution < 10.0 {
            warnings.push(format!(
                "Very low resolution: {:.0} cells across domain (may miss climate features)",
                resolution
            ));
        }

        if resolution > 10000.0 {
            warnings.push(format!(
                "Very high resolution: {:.0} cells across domain (may be computationally expensive)",
                resolution
            ));
        }

        // Check if lapse rate and cell size create reasonable temperature gradients
        let lapse_rate_per_m = params
            .elevation_lapse_rate
            .convert_to(PhysicalUnit::CelsiusPerMeter)
            .value;
        let temp_change_per_cell = lapse_rate_per_m * cell_size_m;

        if temp_change_per_cell > 5.0 {
            warnings.push(format!(
                "Large temperature changes per grid cell: {:.2}°C (consider finer resolution or gentler gradients)",
                temp_change_per_cell
            ));
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::core::scale::{DetailLevel, WorldScale};

    #[test]
    fn physical_quantity_conversions() {
        let distance = PhysicalQuantity::new(1.0, PhysicalUnit::Meters);
        let distance_mm = distance.convert_to(PhysicalUnit::Millimeters);
        assert!((distance_mm.value - 1000.0).abs() < 0.001);

        let time = PhysicalQuantity::new(1.0, PhysicalUnit::Hours);
        let time_s = time.convert_to(PhysicalUnit::Seconds);
        assert!((time_s.value - 3600.0).abs() < 0.001);
    }

    #[test]
    fn cfl_validation() {
        let params = DimensionalWaterFlowParameters::new(
            2.0,  // 2 m/s velocity
            10.0, // 10 mm/h rainfall
            5.0,  // 5 mm/h evaporation
            10.0, // 10m cell size
            1.0,  // 1s timestep
            0.01, // 1cm depth threshold
        );

        let cfl_result = params.validate_cfl_condition(0.5);
        // CFL = v*dt/dx = 2*1/10 = 0.2, which should be stable (< 0.5)
        assert!(cfl_result.is_stable);
        assert!((cfl_result.cfl_number - 0.2).abs() < 0.001);
    }

    #[test]
    fn dimensional_from_world_scale() {
        let scale = WorldScale::new(10.0, (100, 100), DetailLevel::Standard);
        let params = DimensionalAnalysis::from_world_scale(&scale, 1.0, 20.0, 10.0);

        // Cell size should be 100m (10km / 100 pixels)
        let cell_size = params.cell_size.convert_to(PhysicalUnit::Meters).value;
        assert!((cell_size - 100.0).abs() < 1.0);

        // Timestep should be conservative (CFL safe)
        let timestep = params.timestep.convert_to(PhysicalUnit::Seconds).value;
        assert!(timestep > 0.0 && timestep < 100.0); // Should be reasonable
    }

    #[test]
    fn rainfall_depth_calculation() {
        let params = DimensionalWaterFlowParameters::new(
            1.0,   // 1 m/s velocity
            36.0,  // 36 mm/h rainfall (0.01 mm/s)
            0.0,   // no evaporation
            100.0, // 100m cell size
            10.0,  // 10s timestep
            0.01,  // 1cm depth threshold
        );

        let rainfall_depth = params.rainfall_depth_per_timestep();
        let depth_m = rainfall_depth.convert_to(PhysicalUnit::Meters).value;

        // 36 mm/h = 0.01 mm/s = 0.00001 m/s
        // Over 10s = 0.0001 m = 0.1 mm
        assert!((depth_m - 0.0001).abs() < 0.000001);
    }

    #[test]
    fn dimensional_validation_warnings() {
        let params = DimensionalWaterFlowParameters::new(
            50.0,  // Unrealistically high velocity
            200.0, // Extreme rainfall
            0.0,   // no evaporation
            0.5,   // Very small cell
            1.0,   // 1s timestep
            0.01,  // 1cm depth threshold
        );

        let warnings = DimensionalAnalysis::validate_dimensional_consistency(&params);
        assert!(!warnings.is_empty()); // Should have warnings for extreme values

        // Should warn about high velocity
        assert!(warnings.iter().any(|w| w.contains("velocity")));
        // Should warn about extreme rainfall
        assert!(warnings.iter().any(|w| w.contains("rainfall")));
        // Should warn about CFL condition
        assert!(warnings.iter().any(|w| w.contains("CFL")));
    }

    #[test]
    fn climate_unit_conversions() {
        // Temperature conversions
        let temp_c = PhysicalQuantity::new(20.0, PhysicalUnit::Celsius);
        let temp_k = temp_c.convert_to(PhysicalUnit::Kelvin);
        assert!((temp_k.value - 293.15).abs() < 0.001);

        // Temperature gradient conversions
        let lapse_rate = PhysicalQuantity::new(6.5, PhysicalUnit::CelsiusPerKilometer);
        let lapse_rate_per_m = lapse_rate.convert_to(PhysicalUnit::CelsiusPerMeter);
        assert!((lapse_rate_per_m.value - 0.0065).abs() < 0.0001);

        // Velocity conversions
        let speed_ms = PhysicalQuantity::new(10.0, PhysicalUnit::MetersPerSecond);
        let speed_kmh = speed_ms.convert_to(PhysicalUnit::KilometersPerHour);
        assert!((speed_kmh.value - 36.0).abs() < 0.001);
    }

    #[test]
    fn dimensional_climate_parameters() {
        let params = DimensionalClimateParameters::new(
            15.0,   // 15°C base temperature
            0.0065, // Standard lapse rate
            20.0,   // 20°C seasonal variation
            0.8,    // 0.8°C per degree latitude
            100.0,  // 100m cell size
            10.0,   // 10km domain
        );

        // Test elevation temperature calculation
        let sea_level_temp = params.temperature_at_elevation(0.0);
        let mountain_temp = params.temperature_at_elevation(1000.0); // 1km elevation

        assert!((sea_level_temp.value - 15.0).abs() < 0.001);
        assert!((mountain_temp.value - 8.5).abs() < 0.001); // 15 - (1000 * 0.0065) = 8.5°C

        // Test seasonal variation
        let base_temp = PhysicalQuantity::new(10.0, PhysicalUnit::Celsius);
        let winter_temp = params.seasonal_temperature(base_temp, 0.0);
        let summer_temp = params.seasonal_temperature(base_temp, 1.0);

        assert!(winter_temp.value < base_temp.value);
        assert!(summer_temp.value > base_temp.value);
        assert!((summer_temp.value - winter_temp.value - 40.0).abs() < 0.001); // 2 * 20°C amplitude
    }

    #[test]
    fn climate_analysis_from_world_scale() {
        let scale = WorldScale::new(50.0, (500, 500), DetailLevel::Standard);
        let params = ClimateAnalysis::from_world_scale(&scale, 15.0, 0.0065, 25.0, 0.8);

        // Should derive correct cell size
        let cell_size_m = params.cell_size.convert_to(PhysicalUnit::Meters).value;
        assert!((cell_size_m - 100.0).abs() < 1.0); // 50km / 500 cells = 100m per cell

        // Should preserve physical parameters
        assert!((params.base_temperature.value - 15.0).abs() < 0.001);
        assert!((params.elevation_lapse_rate.value - 0.0065).abs() < 0.0001);
    }

    #[test]
    fn climate_validation_warnings() {
        let extreme_params = DimensionalClimateParameters::new(
            100.0,    // Extremely hot base temperature
            0.05,     // Extreme lapse rate
            80.0,     // Extreme seasonal variation
            5.0,      // Extreme latitude gradient
            1.0,      // 1m cell size
            100000.0, // Extremely large domain
        );

        let warnings = ClimateAnalysis::validate_climate_consistency(&extreme_params);
        assert!(!warnings.is_empty());

        // Should warn about extreme base temperature
        assert!(warnings.iter().any(|w| w.contains("temperature")));
        // Should warn about extreme lapse rate
        assert!(warnings.iter().any(|w| w.contains("lapse rate")));
        // Should warn about extreme seasonal variation
        assert!(warnings.iter().any(|w| w.contains("seasonal")));
    }
}
