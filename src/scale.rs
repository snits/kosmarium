// ABOUTME: Core scaling architecture for scale-aware world generation systems
// ABOUTME: Provides WorldScale context and ScaleAware trait for consistent parameter derivation

/// Represents the scale context for world generation
/// Separates physical scale (real-world size) from resolution scale (output detail)
#[derive(Clone, Debug)]
pub struct WorldScale {
    /// Physical size the map represents in kilometers
    pub physical_size_km: f64,
    /// Output resolution (width, height)
    pub resolution: (u32, u32),
    /// Target detail level for generation quality
    pub detail_level: DetailLevel,
}

impl WorldScale {
    /// Create a new world scale context
    pub fn new(physical_size_km: f64, resolution: (u32, u32), detail_level: DetailLevel) -> Self {
        Self {
            physical_size_km,
            resolution,
            detail_level,
        }
    }

    /// Get the real-world distance represented by each pixel in meters
    pub fn meters_per_pixel(&self) -> f64 {
        (self.physical_size_km * 1000.0) / self.resolution.0.max(self.resolution.1) as f64
    }

    /// Get how many pixels represent one kilometer
    pub fn pixels_per_km(&self) -> f64 {
        self.resolution.0.max(self.resolution.1) as f64 / self.physical_size_km
    }

    /// Get total number of cells in the map
    pub fn total_cells(&self) -> u32 {
        self.resolution.0 * self.resolution.1
    }

    /// Get the scale factor relative to a reference resolution
    pub fn scale_factor_from_reference(&self, reference_resolution: (u32, u32)) -> f64 {
        let reference_cells = (reference_resolution.0 * reference_resolution.1) as f64;
        let current_cells = self.total_cells() as f64;
        reference_cells / current_cells
    }
}

/// Quality/performance trade-off levels for generation
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DetailLevel {
    /// Fast generation with basic features only
    Preview,
    /// Balanced quality and performance
    Standard,
    /// High detail with more complex features (slower)
    High,
}

/// Trait for types that can derive scale-appropriate parameters
pub trait ScaleAware {
    /// Derive parameters appropriate for the given world scale
    fn derive_parameters(&self, scale: &WorldScale) -> Self;
}

/// Standard reference scale used for parameter calibration
/// Most default parameters are tuned for this scale
pub const REFERENCE_SCALE: (u32, u32) = (240, 120);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_scale_calculations() {
        let scale = WorldScale::new(10.0, (1000, 500), DetailLevel::Standard);

        // Should be 10 meters per pixel for largest dimension
        assert!((scale.meters_per_pixel() - 10.0).abs() < 0.1);

        // Should be 100 pixels per km
        assert!((scale.pixels_per_km() - 100.0).abs() < 0.1);

        assert_eq!(scale.total_cells(), 500_000);
    }

    #[test]
    fn scale_factor_calculation() {
        let scale = WorldScale::new(10.0, (480, 240), DetailLevel::Standard);
        let reference = REFERENCE_SCALE;

        // 480x240 = 115,200 cells vs 240x120 = 28,800 cells = 4x larger
        let factor = scale.scale_factor_from_reference(reference);
        assert!(
            (factor - 0.25).abs() < 0.01,
            "Expected ~0.25, got {}",
            factor
        );
    }
}
