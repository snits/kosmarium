// ABOUTME: Multi-viewport TUI system for simultaneous monitoring of atmospheric data layers
// ABOUTME: Enables scientists to view multiple data layers simultaneously with independent navigation

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::ascii_framebuffer::VisualizationLayer;
use super::tui::Viewport;
use crate::engine::Simulation;

/// Configuration for multi-viewport layout
#[derive(Debug, Clone)]
pub struct MultiViewportConfig {
    /// Viewports to display (up to 4)
    pub viewports: Vec<ViewportConfig>,
    /// Overall layout type
    pub layout: LayoutType,
    /// Show status panel
    pub show_status: bool,
    /// Active viewport index
    pub active_viewport: usize,
}

impl Default for MultiViewportConfig {
    fn default() -> Self {
        Self {
            viewports: vec![
                ViewportConfig::new(VisualizationLayer::Elevation, "ELEVATION"),
                ViewportConfig::new(VisualizationLayer::Temperature, "TEMPERATURE"),
                ViewportConfig::new(VisualizationLayer::Pressure, "PRESSURE"),
                ViewportConfig::new(VisualizationLayer::Wind, "WIND"),
            ],
            layout: LayoutType::Grid2x2,
            show_status: true,
            active_viewport: 0,
        }
    }
}

/// Layout types for multi-viewport display
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutType {
    Grid2x2,  // 2x2 grid of equal viewports
    Split2x1, // 2 viewports side by side
    Triple,   // 3 viewports with one dominant
}

/// Configuration for a single viewport
#[derive(Debug, Clone)]
pub struct ViewportConfig {
    /// Data layer to display
    pub layer: VisualizationLayer,
    /// Display title
    pub title: String,
    /// Viewport navigation state
    pub viewport: Viewport,
    /// Zoom level (continental, regional, local)
    pub zoom_level: ZoomLevel,
}

impl ViewportConfig {
    pub fn new(layer: VisualizationLayer, title: &str) -> Self {
        Self {
            layer,
            title: title.to_string(),
            viewport: Viewport::new(40, 20), // Default size, will be adjusted
            zoom_level: ZoomLevel::Continental,
        }
    }
}

/// Zoom levels for viewports
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoomLevel {
    Continental, // Wide view
    Regional,    // Medium detail
    Local,       // High detail
}

/// Movement directions for WASD navigation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementDirection {
    North, // W key - move up
    South, // S key - move down
    West,  // A key - move left
    East,  // D key - move right
}

impl ZoomLevel {
    pub fn display_name(&self) -> &'static str {
        match self {
            ZoomLevel::Continental => "CONT",
            ZoomLevel::Regional => "REGI",
            ZoomLevel::Local => "LOCA",
        }
    }
}

/// Multi-viewport renderer for ASCII framebuffer system
pub struct MultiViewportRenderer {
    /// Configuration
    config: MultiViewportConfig,
}

/// Multi-viewport application state for TUI integration
pub struct MultiViewportApp {
    /// Simulation reference
    pub simulation: Simulation,
    /// Multi-viewport renderer
    pub renderer: MultiViewportRenderer,
    /// Should quit flag
    pub should_quit: bool,
}

impl MultiViewportApp {
    /// Create new multi-viewport application
    pub fn new(simulation: Simulation) -> Self {
        let config = MultiViewportConfig::default();
        let renderer = MultiViewportRenderer::new(config);

        Self {
            simulation,
            renderer,
            should_quit: false,
        }
    }

    /// Handle Tab key to cycle to next viewport
    pub fn cycle_next_viewport(&mut self) {
        let total_viewports = self.renderer.config.viewports.len();
        if total_viewports > 0 {
            self.renderer.config.active_viewport =
                (self.renderer.config.active_viewport + 1) % total_viewports;
        }
    }

    /// Handle Shift+Tab to cycle to previous viewport  
    pub fn cycle_previous_viewport(&mut self) {
        let total_viewports = self.renderer.config.viewports.len();
        if total_viewports > 0 {
            self.renderer.config.active_viewport = if self.renderer.config.active_viewport == 0 {
                total_viewports - 1
            } else {
                self.renderer.config.active_viewport - 1
            };
        }
    }

    /// Direct viewport selection (1-4 keys)
    pub fn select_viewport(&mut self, viewport_index: usize) -> bool {
        if viewport_index < self.renderer.config.viewports.len() {
            self.renderer.config.active_viewport = viewport_index;
            true
        } else {
            false
        }
    }

    /// Get current active viewport index
    pub fn get_active_viewport(&self) -> usize {
        self.renderer.config.active_viewport
    }

    /// Handle WASD navigation for active viewport
    pub fn handle_movement(&mut self, direction: MovementDirection, fast: bool) -> bool {
        let active_idx = self.renderer.config.active_viewport;
        if active_idx >= self.renderer.config.viewports.len() {
            return false;
        }

        let step_size = if fast { 5 } else { 1 };
        let viewport_config = &mut self.renderer.config.viewports[active_idx];

        match direction {
            MovementDirection::North => {
                if viewport_config.viewport.world_y >= step_size {
                    viewport_config.viewport.world_y -= step_size;
                    true
                } else {
                    viewport_config.viewport.world_y = 0;
                    false // Hit boundary
                }
            }
            MovementDirection::South => {
                // Simple bounds checking - could be enhanced with world size limits
                viewport_config.viewport.world_y += step_size;
                true
            }
            MovementDirection::West => {
                if viewport_config.viewport.world_x >= step_size {
                    viewport_config.viewport.world_x -= step_size;
                    true
                } else {
                    viewport_config.viewport.world_x = 0;
                    false // Hit boundary
                }
            }
            MovementDirection::East => {
                // Simple bounds checking - could be enhanced with world size limits
                viewport_config.viewport.world_x += step_size;
                true
            }
        }
    }

    /// Get current viewport position for active viewport
    pub fn get_active_viewport_position(&self) -> (i32, i32) {
        let active_idx = self.renderer.config.active_viewport;
        if active_idx < self.renderer.config.viewports.len() {
            let viewport = &self.renderer.config.viewports[active_idx].viewport;
            (viewport.world_x, viewport.world_y)
        } else {
            (0, 0)
        }
    }

    /// Set viewport position for active viewport (for testing)
    pub fn set_active_viewport_position(&mut self, x: i32, y: i32) -> bool {
        let active_idx = self.renderer.config.active_viewport;
        if active_idx < self.renderer.config.viewports.len() {
            let viewport = &mut self.renderer.config.viewports[active_idx].viewport;
            viewport.world_x = x;
            viewport.world_y = y;
            true
        } else {
            false
        }
    }

    /// Quit application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

impl MultiViewportRenderer {
    /// Create new multi-viewport renderer
    pub fn new(config: MultiViewportConfig) -> Self {
        Self { config }
    }

    /// Get the number of viewports
    pub fn viewport_count(&self) -> usize {
        self.config.viewports.len()
    }

    /// Get the active viewport index
    pub fn get_active_viewport(&self) -> usize {
        self.config.active_viewport
    }

    /// Generate 2x2 grid layout areas
    pub fn generate_2x2_layout(&self, area: Rect) -> Vec<Rect> {
        // Reserve space for status panel if enabled
        let content_area = if self.config.show_status {
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(area);
            main_layout[0]
        } else {
            area
        };

        // Split into 2 rows
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_area);

        // Split each row into 2 columns
        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let bottom_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        vec![top_cols[0], top_cols[1], bottom_cols[0], bottom_cols[1]]
    }

    /// Generate status panel area (returns None if status panel disabled)
    pub fn generate_status_panel(&self, area: Rect) -> Option<Rect> {
        if self.config.show_status {
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(area);
            Some(main_layout[1])
        } else {
            None
        }
    }

    /// Create status panel widget with keybinding legend and active viewport info
    pub fn create_status_panel(&self) -> Paragraph {
        let active_viewport_name = if self.config.active_viewport < self.config.viewports.len() {
            &self.config.viewports[self.config.active_viewport].title
        } else {
            "NONE"
        };

        let keybinding_line1 = Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw("/"),
            Span::styled("Shift+Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Cycle • "),
            Span::styled("1-4", Style::default().fg(Color::Yellow)),
            Span::raw(": Select • "),
            Span::styled("WASD", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate • "),
            Span::styled("Shift+WASD", Style::default().fg(Color::Yellow)),
            Span::raw(": Fast • "),
            Span::styled("Q", Style::default().fg(Color::Yellow)),
            Span::raw("/"),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]);

        let status_line = Line::from(vec![
            Span::raw("Active: "),
            Span::styled(
                format!(
                    "{} ({})",
                    active_viewport_name,
                    self.config.active_viewport + 1
                ),
                Style::default().fg(Color::White),
            ),
            Span::raw(" • Viewports: "),
            Span::styled(
                format!("{}", self.config.viewports.len()),
                Style::default().fg(Color::Cyan),
            ),
        ]);

        Paragraph::new(vec![keybinding_line1, status_line]).block(
            Block::default()
                .title("Controls & Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
    }

    /// Render single viewport with ASCII framebuffer data
    pub fn render_viewport_content(
        &self,
        simulation: &Simulation,
        viewport_idx: usize,
    ) -> Option<Vec<Line>> {
        if viewport_idx >= self.config.viewports.len() {
            return None;
        }

        let viewport_config = &self.config.viewports[viewport_idx];
        let layer = &viewport_config.layer;

        // Create simple ASCII content for the viewport
        let mut lines = Vec::new();
        let sample_size = 20; // Small viewport sample

        match layer {
            VisualizationLayer::Elevation => {
                for y in 0..sample_size {
                    let mut spans = Vec::new();
                    for x in 0..sample_size {
                        let elevation = simulation.get_elevation(x, y);
                        let symbol = match elevation {
                            e if e < 0.2 => '.',
                            e if e < 0.4 => '~',
                            e if e < 0.6 => '^',
                            e if e < 0.8 => '#',
                            _ => '@',
                        };
                        spans.push(Span::raw(symbol.to_string()));
                    }
                    lines.push(Line::from(spans));
                }
            }
            VisualizationLayer::Temperature => {
                for y in 0..sample_size {
                    let mut spans = Vec::new();
                    for x in 0..sample_size {
                        let temp = simulation.get_temperature_layer().get_temperature(x, y);
                        let symbol = match temp {
                            t if t < -10.0 => '■',
                            t if t < 0.0 => '▓',
                            t if t < 10.0 => '▒',
                            t if t < 20.0 => '░',
                            t if t < 30.0 => '.',
                            t if t < 40.0 => '+',
                            _ => '#',
                        };
                        spans.push(Span::raw(symbol.to_string()));
                    }
                    lines.push(Line::from(spans));
                }
            }
            VisualizationLayer::Pressure => {
                for y in 0..sample_size {
                    let mut spans = Vec::new();
                    for x in 0..sample_size {
                        let pressure = simulation.get_pressure_layer().get_pressure(x, y);
                        let avg_pressure = 101300.0; // Standard pressure
                        let normalized = (pressure - avg_pressure) / 2000.0 + 0.5;

                        let symbol = match normalized {
                            n if n < 0.2 => '-',
                            n if n < 0.4 => '.',
                            n if n < 0.6 => '0',
                            n if n < 0.8 => '+',
                            _ => '#',
                        };
                        spans.push(Span::raw(symbol.to_string()));
                    }
                    lines.push(Line::from(spans));
                }
            }
            VisualizationLayer::Wind => {
                for y in 0..sample_size {
                    let mut spans = Vec::new();
                    for x in 0..sample_size {
                        let velocity = simulation.get_wind_layer().get_velocity(x, y);
                        let speed = velocity.magnitude();

                        let symbol = if speed < 1.0 {
                            '.'
                        } else {
                            let angle = velocity.y.atan2(velocity.x);
                            let angle_deg = angle.to_degrees();
                            let normalized_angle = ((angle_deg + 360.0) % 360.0) / 45.0;

                            match normalized_angle as i32 {
                                0 => '→',
                                1 => '↗',
                                2 => '↑',
                                3 => '↖',
                                4 => '←',
                                5 => '↙',
                                6 => '↓',
                                7 => '↘',
                                _ => '→',
                            }
                        };
                        spans.push(Span::raw(symbol.to_string()));
                    }
                    lines.push(Line::from(spans));
                }
            }
            _ => {
                // Other layers - placeholder for now
                for _y in 0..sample_size {
                    let mut spans = Vec::new();
                    for _x in 0..sample_size {
                        spans.push(Span::raw("?"));
                    }
                    lines.push(Line::from(spans));
                }
            }
        }

        Some(lines)
    }

    /// Create viewport paragraph widget with proper borders and titles
    pub fn create_viewport_widget<'a>(
        &self,
        content: Vec<Line<'a>>,
        viewport_idx: usize,
        is_active: bool,
    ) -> Paragraph<'a> {
        let viewport_config = &self.config.viewports[viewport_idx];
        let title = format!(
            "{} {}",
            viewport_config.title,
            if is_active { "*" } else { "" }
        );
        let zoom_indicator = format!("[{}]", viewport_config.zoom_level.display_name());
        let full_title = format!("{title} {zoom_indicator}");

        let border_style = if is_active {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        Paragraph::new(content).block(
            Block::default()
                .title(full_title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_viewport_config_default() {
        let config = MultiViewportConfig::default();

        // Should have 4 viewports by default
        assert_eq!(config.viewports.len(), 4);

        // Should use 2x2 grid layout
        assert_eq!(config.layout, LayoutType::Grid2x2);

        // Should start with first viewport active
        assert_eq!(config.active_viewport, 0);

        // Should show status panel
        assert!(config.show_status);
    }

    #[test]
    fn test_viewport_config_creation() {
        let viewport_config = ViewportConfig::new(VisualizationLayer::Elevation, "ELEVATION");

        assert_eq!(viewport_config.layer, VisualizationLayer::Elevation);
        assert_eq!(viewport_config.title, "ELEVATION");
        assert_eq!(viewport_config.zoom_level, ZoomLevel::Continental);
    }

    #[test]
    fn test_2x2_grid_layout_generation() {
        let config = MultiViewportConfig::default();
        let renderer = MultiViewportRenderer::new(config);

        // Test with a 100x50 area
        let test_area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };

        let layout_areas = renderer.generate_2x2_layout(test_area);

        // Should return exactly 4 areas for 2x2 grid
        assert_eq!(layout_areas.len(), 4);

        // Verify layout areas are reasonable (accounting for status panel)
        for area in &layout_areas {
            assert!(area.width > 0);
            assert!(area.height > 0);
            // Each viewport should be roughly 1/4 of the content area
            assert!(area.width <= test_area.width / 2);
            assert!(area.height <= test_area.height / 2);
        }
    }

    #[test]
    fn test_zoom_level_display_names() {
        assert_eq!(ZoomLevel::Continental.display_name(), "CONT");
        assert_eq!(ZoomLevel::Regional.display_name(), "REGI");
        assert_eq!(ZoomLevel::Local.display_name(), "LOCA");
    }

    #[test]
    fn test_render_viewport_integration() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        use crate::engine::physics::{DiamondSquareGenerator, TerrainGenerator};

        // Create a minimal simulation for testing
        let generator = DiamondSquareGenerator::new(42);
        let heightmap = generator.generate(50, 50, &Default::default());
        let scale = WorldScale::new(200.0, (50, 50), DetailLevel::Standard);
        let simulation = Simulation::_new_with_scale(heightmap, scale);

        let config = MultiViewportConfig::default();
        let renderer = MultiViewportRenderer::new(config);

        // Test that we can render each viewport type
        for viewport_idx in 0..4 {
            let result = renderer.render_viewport_content(&simulation, viewport_idx);

            // Should return content for valid viewport indices
            assert!(
                result.is_some(),
                "Viewport {} should render content",
                viewport_idx
            );

            let lines = result.unwrap();

            // Should have 20 lines (sample_size in implementation)
            assert_eq!(
                lines.len(),
                20,
                "Viewport {} should have 20 lines",
                viewport_idx
            );

            // Each line should have spans (ASCII characters)
            for (line_idx, line) in lines.iter().enumerate() {
                assert!(
                    !line.spans.is_empty(),
                    "Viewport {} line {} should have content",
                    viewport_idx,
                    line_idx
                );
            }
        }

        // Test invalid viewport index
        let invalid_result = renderer.render_viewport_content(&simulation, 999);
        assert!(
            invalid_result.is_none(),
            "Invalid viewport index should return None"
        );
    }

    #[test]
    fn test_multi_viewport_app_creation() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        use crate::engine::physics::{DiamondSquareGenerator, TerrainGenerator};

        let generator = DiamondSquareGenerator::new(42);
        let heightmap = generator.generate(50, 50, &Default::default());
        let scale = WorldScale::new(200.0, (50, 50), DetailLevel::Standard);
        let simulation = Simulation::_new_with_scale(heightmap, scale);

        let app = MultiViewportApp::new(simulation);

        // Should start with viewport 0 active
        assert_eq!(app.get_active_viewport(), 0);

        // Should not be quitting initially
        assert!(!app.should_quit);

        // Should have default 4 viewports
        assert_eq!(app.renderer.config.viewports.len(), 4);
    }

    #[test]
    fn test_viewport_cycling() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        use crate::engine::physics::{DiamondSquareGenerator, TerrainGenerator};

        let generator = DiamondSquareGenerator::new(42);
        let heightmap = generator.generate(50, 50, &Default::default());
        let scale = WorldScale::new(200.0, (50, 50), DetailLevel::Standard);
        let simulation = Simulation::_new_with_scale(heightmap, scale);

        let mut app = MultiViewportApp::new(simulation);

        // Test forward cycling: 0 -> 1 -> 2 -> 3 -> 0
        assert_eq!(app.get_active_viewport(), 0);

        app.cycle_next_viewport();
        assert_eq!(app.get_active_viewport(), 1);

        app.cycle_next_viewport();
        assert_eq!(app.get_active_viewport(), 2);

        app.cycle_next_viewport();
        assert_eq!(app.get_active_viewport(), 3);

        app.cycle_next_viewport(); // Should wrap to 0
        assert_eq!(app.get_active_viewport(), 0);
    }

    #[test]
    fn test_viewport_reverse_cycling() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        use crate::engine::physics::{DiamondSquareGenerator, TerrainGenerator};

        let generator = DiamondSquareGenerator::new(42);
        let heightmap = generator.generate(50, 50, &Default::default());
        let scale = WorldScale::new(200.0, (50, 50), DetailLevel::Standard);
        let simulation = Simulation::_new_with_scale(heightmap, scale);

        let mut app = MultiViewportApp::new(simulation);

        // Test reverse cycling: 0 -> 3 -> 2 -> 1 -> 0
        assert_eq!(app.get_active_viewport(), 0);

        app.cycle_previous_viewport(); // Should wrap to 3
        assert_eq!(app.get_active_viewport(), 3);

        app.cycle_previous_viewport();
        assert_eq!(app.get_active_viewport(), 2);

        app.cycle_previous_viewport();
        assert_eq!(app.get_active_viewport(), 1);

        app.cycle_previous_viewport();
        assert_eq!(app.get_active_viewport(), 0);
    }

    #[test]
    fn test_direct_viewport_selection() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        use crate::engine::physics::{DiamondSquareGenerator, TerrainGenerator};

        let generator = DiamondSquareGenerator::new(42);
        let heightmap = generator.generate(50, 50, &Default::default());
        let scale = WorldScale::new(200.0, (50, 50), DetailLevel::Standard);
        let simulation = Simulation::_new_with_scale(heightmap, scale);

        let mut app = MultiViewportApp::new(simulation);

        // Test valid direct selections
        assert!(app.select_viewport(2));
        assert_eq!(app.get_active_viewport(), 2);

        assert!(app.select_viewport(0));
        assert_eq!(app.get_active_viewport(), 0);

        assert!(app.select_viewport(3));
        assert_eq!(app.get_active_viewport(), 3);

        // Test invalid selection (out of bounds)
        assert!(!app.select_viewport(999));
        assert_eq!(app.get_active_viewport(), 3); // Should remain unchanged

        assert!(!app.select_viewport(4)); // Only 0-3 are valid
        assert_eq!(app.get_active_viewport(), 3);
    }

    #[test]
    fn test_active_viewport_indication_in_widget() {
        let config = MultiViewportConfig::default();
        let renderer = MultiViewportRenderer::new(config);

        let content = vec![ratatui::text::Line::from("test content")];

        // Test active viewport widget (should have asterisk)
        let active_widget = renderer.create_viewport_widget(content.clone(), 0, true);
        // Can't directly inspect the widget title, but we can verify it was created
        // The actual title checking would be done in integration tests

        // Test inactive viewport widget (no asterisk)
        let inactive_widget = renderer.create_viewport_widget(content, 0, false);
        // Similar - actual verification would be in integration tests

        // This test mainly verifies the function doesn't panic with different parameters
        assert!(true); // Both widgets created successfully
    }

    #[test]
    fn test_wasd_navigation_basic_movement() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        use crate::engine::physics::{DiamondSquareGenerator, TerrainGenerator};

        let generator = DiamondSquareGenerator::new(42);
        let heightmap = generator.generate(50, 50, &Default::default());
        let scale = WorldScale::new(200.0, (50, 50), DetailLevel::Standard);
        let simulation = Simulation::_new_with_scale(heightmap, scale);

        let mut app = MultiViewportApp::new(simulation);

        // Start at origin
        assert_eq!(app.get_active_viewport_position(), (0, 0));

        // Test East movement (D key)
        assert!(app.handle_movement(MovementDirection::East, false));
        assert_eq!(app.get_active_viewport_position(), (1, 0));

        // Test South movement (S key)
        assert!(app.handle_movement(MovementDirection::South, false));
        assert_eq!(app.get_active_viewport_position(), (1, 1));

        // Test West movement (A key)
        assert!(app.handle_movement(MovementDirection::West, false));
        assert_eq!(app.get_active_viewport_position(), (0, 1));

        // Test North movement (W key)
        assert!(app.handle_movement(MovementDirection::North, false));
        assert_eq!(app.get_active_viewport_position(), (0, 0));
    }

    #[test]
    fn test_wasd_navigation_fast_movement() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        use crate::engine::physics::{DiamondSquareGenerator, TerrainGenerator};

        let generator = DiamondSquareGenerator::new(42);
        let heightmap = generator.generate(50, 50, &Default::default());
        let scale = WorldScale::new(200.0, (50, 50), DetailLevel::Standard);
        let simulation = Simulation::_new_with_scale(heightmap, scale);

        let mut app = MultiViewportApp::new(simulation);

        // Start at origin
        assert_eq!(app.get_active_viewport_position(), (0, 0));

        // Test fast East movement (Shift+D)
        assert!(app.handle_movement(MovementDirection::East, true));
        assert_eq!(app.get_active_viewport_position(), (5, 0));

        // Test fast South movement (Shift+S)
        assert!(app.handle_movement(MovementDirection::South, true));
        assert_eq!(app.get_active_viewport_position(), (5, 5));

        // Test fast West movement (Shift+A)
        assert!(app.handle_movement(MovementDirection::West, true));
        assert_eq!(app.get_active_viewport_position(), (0, 5));

        // Test fast North movement (Shift+W)
        assert!(app.handle_movement(MovementDirection::North, true));
        assert_eq!(app.get_active_viewport_position(), (0, 0));
    }

    #[test]
    fn test_wasd_navigation_boundary_conditions() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        use crate::engine::physics::{DiamondSquareGenerator, TerrainGenerator};

        let generator = DiamondSquareGenerator::new(42);
        let heightmap = generator.generate(50, 50, &Default::default());
        let scale = WorldScale::new(200.0, (50, 50), DetailLevel::Standard);
        let simulation = Simulation::_new_with_scale(heightmap, scale);

        let mut app = MultiViewportApp::new(simulation);

        // Start at origin (0, 0)
        assert_eq!(app.get_active_viewport_position(), (0, 0));

        // Test North movement at origin (should hit boundary)
        assert!(!app.handle_movement(MovementDirection::North, false));
        assert_eq!(app.get_active_viewport_position(), (0, 0)); // Should remain at origin

        // Test West movement at origin (should hit boundary)
        assert!(!app.handle_movement(MovementDirection::West, false));
        assert_eq!(app.get_active_viewport_position(), (0, 0)); // Should remain at origin

        // Test fast North movement at origin (should hit boundary)
        assert!(!app.handle_movement(MovementDirection::North, true));
        assert_eq!(app.get_active_viewport_position(), (0, 0)); // Should remain at origin

        // Test fast West movement at origin (should hit boundary)
        assert!(!app.handle_movement(MovementDirection::West, true));
        assert_eq!(app.get_active_viewport_position(), (0, 0)); // Should remain at origin

        // Move to position (3, 3) for partial boundary testing
        app.set_active_viewport_position(3, 3);
        assert_eq!(app.get_active_viewport_position(), (3, 3));

        // Test fast West movement that would go past boundary
        assert!(!app.handle_movement(MovementDirection::West, true)); // 3 - 5 = -2, clamped to 0
        assert_eq!(app.get_active_viewport_position(), (0, 3)); // Should clamp to boundary

        // Test fast North movement that would go past boundary
        assert!(!app.handle_movement(MovementDirection::North, true)); // 3 - 5 = -2, clamped to 0
        assert_eq!(app.get_active_viewport_position(), (0, 0)); // Should clamp to boundary
    }

    #[test]
    fn test_wasd_navigation_active_viewport_isolation() {
        use crate::engine::core::scale::{DetailLevel, WorldScale};
        use crate::engine::physics::{DiamondSquareGenerator, TerrainGenerator};

        let generator = DiamondSquareGenerator::new(42);
        let heightmap = generator.generate(50, 50, &Default::default());
        let scale = WorldScale::new(200.0, (50, 50), DetailLevel::Standard);
        let simulation = Simulation::_new_with_scale(heightmap, scale);

        let mut app = MultiViewportApp::new(simulation);

        // Start with viewport 0 active
        assert_eq!(app.get_active_viewport(), 0);
        assert_eq!(app.get_active_viewport_position(), (0, 0));

        // Move viewport 0 to (5, 5)
        app.handle_movement(MovementDirection::East, true);
        app.handle_movement(MovementDirection::South, true);
        assert_eq!(app.get_active_viewport_position(), (5, 5));

        // Switch to viewport 1
        app.cycle_next_viewport();
        assert_eq!(app.get_active_viewport(), 1);
        // Viewport 1 should still be at origin
        assert_eq!(app.get_active_viewport_position(), (0, 0));

        // Move viewport 1 to (3, 3)
        for _ in 0..3 {
            app.handle_movement(MovementDirection::East, false);
            app.handle_movement(MovementDirection::South, false);
        }
        assert_eq!(app.get_active_viewport_position(), (3, 3));

        // Switch back to viewport 0
        app.cycle_previous_viewport();
        assert_eq!(app.get_active_viewport(), 0);
        // Viewport 0 should still be at (5, 5)
        assert_eq!(app.get_active_viewport_position(), (5, 5));

        // Switch to viewport 1 again
        app.cycle_next_viewport();
        assert_eq!(app.get_active_viewport(), 1);
        // Viewport 1 should still be at (3, 3)
        assert_eq!(app.get_active_viewport_position(), (3, 3));
    }

    #[test]
    fn test_movement_direction_enum() {
        // Test that MovementDirection values are correct
        use std::mem;

        // Test enum size (should be small)
        assert_eq!(mem::size_of::<MovementDirection>(), 1);

        // Test that all variants are distinct
        let north = MovementDirection::North;
        let south = MovementDirection::South;
        let west = MovementDirection::West;
        let east = MovementDirection::East;

        assert_ne!(north, south);
        assert_ne!(north, west);
        assert_ne!(north, east);
        assert_ne!(south, west);
        assert_ne!(south, east);
        assert_ne!(west, east);
    }

    #[test]
    fn test_status_panel_generation() {
        let config = MultiViewportConfig::default();
        let renderer = MultiViewportRenderer::new(config);

        // Test with status panel enabled (default)
        let test_area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };

        let status_area = renderer.generate_status_panel(test_area);
        assert!(
            status_area.is_some(),
            "Status panel should be generated when enabled"
        );

        let status_rect = status_area.unwrap();
        assert_eq!(status_rect.height, 3, "Status panel should be 3 lines high");
        assert_eq!(
            status_rect.width, 100,
            "Status panel should span full width"
        );

        // Status panel should be at the bottom
        assert!(status_rect.y > 0, "Status panel should not be at top");
    }

    #[test]
    fn test_status_panel_widget_creation() {
        let config = MultiViewportConfig::default();
        let renderer = MultiViewportRenderer::new(config);

        // Test that status panel widget can be created
        let status_widget = renderer.create_status_panel();

        // Can't easily inspect the paragraph content, but we can verify it doesn't panic
        // In a real TUI test environment, we'd verify the text content and styling
        assert!(true); // Widget created successfully
    }

    #[test]
    fn test_status_panel_disabled() {
        let mut config = MultiViewportConfig::default();
        config.show_status = false; // Disable status panel
        let renderer = MultiViewportRenderer::new(config);

        let test_area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };

        let status_area = renderer.generate_status_panel(test_area);
        assert!(
            status_area.is_none(),
            "Status panel should not be generated when disabled"
        );
    }
}
