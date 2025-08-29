// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Rendering and visualization systems - ASCII, TUI, and graphics rendering
// ABOUTME: Provides multiple visualization modes for simulation data and user interaction

pub mod ansi_colors;
pub mod ascii_framebuffer;
pub mod graphics_render;
pub mod multi_viewport;
pub mod render;
pub mod tui;

// Re-export rendering functions
pub use ascii_framebuffer::{AsciiFramebuffer, FramebufferConfig, VisualizationLayer};
pub use graphics_render::GraphicsRenderer;
pub use render::{ascii_render, ascii_render_biomes};
pub use tui::run_tui;
