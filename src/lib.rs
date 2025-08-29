// ABOUTME: Library interface for Kosmarium planetary physics simulation engine
// ABOUTME: Exposes clean public API for external use while keeping internal organization

pub mod engine;

// Re-export key engine components for library users
pub use engine::{RainfallScaling, Simulation, WaterFlowParameters, WaterFlowSystem};

// Re-export applications for convenience
pub mod applications {
    pub use crate::engine::*;
}
