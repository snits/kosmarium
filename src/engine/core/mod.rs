// ABOUTME: Core engine foundation - fundamental data structures and scaling systems
// ABOUTME: Provides basic building blocks for all other engine components

pub mod cache_system;
pub mod dimensional;
pub mod heightmap;
pub mod math;
pub mod optimized_heightmap;
pub mod physics_grid;
pub mod scale;
pub mod temporal_performance;
pub mod temporal_scaling;
pub mod unified_temporal_scaling;

// Re-export key types for convenience
pub use physics_grid::PhysicsGrid;
pub use scale::{DetailLevel, WorldScale};
pub use temporal_performance::{
    PerformanceSummary, TemporalPerformanceMonitor, TemporalScalingTimer,
};
pub use temporal_scaling::{TemporalMode, TemporalScalingConfig, TemporalScalingService};
pub use unified_temporal_scaling::{TemporalScale, TemporalScaleBuilder};
