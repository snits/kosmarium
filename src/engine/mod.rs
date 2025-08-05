// ABOUTME: Engine module organization - core simulation engine components
// ABOUTME: Provides clean internal structure for engine subsystems

pub mod agents;
pub mod core;
pub mod diagnostics;
pub mod physics;
pub mod rendering;

// Main simulation struct - keep at engine level
pub mod sim;
pub use diagnostics::SimulationDiagnostics;
pub use sim::{RainfallScaling, Simulation, WaterFlowParameters, WaterFlowSystem};
