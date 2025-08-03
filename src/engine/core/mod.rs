// ABOUTME: Core engine foundation - fundamental data structures and scaling systems
// ABOUTME: Provides basic building blocks for all other engine components

pub mod cache_system;
pub mod dimensional;
pub mod heightmap;
pub mod optimized_heightmap;
pub mod scale;

// Re-export key types for convenience
pub use dimensional::{PhysicalQuantity, PhysicalUnit};
pub use heightmap::HeightMap;
pub use scale::{DetailLevel, REFERENCE_SCALE, ScaleAware, WorldScale};
