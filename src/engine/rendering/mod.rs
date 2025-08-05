// ABOUTME: Rendering and visualization systems - ASCII, TUI, and graphics rendering
// ABOUTME: Provides multiple visualization modes for simulation data and user interaction

pub mod graphics_render;
pub mod render;
pub mod tui;
pub mod ascii_framebuffer;

// Re-export rendering functions
pub use graphics_render::GraphicsRenderer;
pub use render::{ascii_render, ascii_render_biomes};
pub use tui::run_tui;
pub use ascii_framebuffer::{AsciiFramebuffer, FramebufferConfig, VisualizationLayer};
