// ABOUTME: TUI implementation with ratatui for interactive terrain exploration
// ABOUTME: Provides scrollable viewport system for navigating large generated maps

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::io;
use std::time::{Duration, Instant};

use crate::sim::Simulation;

/// Viewport for navigating the world map
#[derive(Debug, Clone)]
pub struct Viewport {
    /// World coordinates (top-left corner of viewport)
    pub world_x: i32,
    pub world_y: i32,
    /// Terminal dimensions for rendering
    pub view_width: usize,
    pub view_height: usize,
}

impl Viewport {
    pub fn new(view_width: usize, view_height: usize) -> Self {
        Self {
            world_x: 0,
            world_y: 0,
            view_width,
            view_height,
        }
    }

    /// Move viewport by given offset, clamping to world bounds
    pub fn move_by(&mut self, dx: i32, dy: i32, world_width: usize, world_height: usize) {
        self.world_x = (self.world_x + dx)
            .max(0)
            .min((world_width as i32) - (self.view_width as i32));
        self.world_y = (self.world_y + dy)
            .max(0)
            .min((world_height as i32) - (self.view_height as i32));
    }

    /// Extract visible portion of heightmap for rendering with zoom support
    pub fn extract_visible_region(&self, heightmap: &[Vec<f32>], zoom_level: u32) -> Vec<Vec<f32>> {
        let world_height = heightmap.len();
        let world_width = if world_height > 0 {
            heightmap[0].len()
        } else {
            0
        };

        let start_y = self.world_y.max(0) as usize;
        let start_x = self.world_x.max(0) as usize;

        let mut visible = Vec::new();

        for view_y in 0..self.view_height {
            let mut row = Vec::new();

            for view_x in 0..self.view_width {
                // Calculate world coordinates based on zoom level
                let world_x = start_x + (view_x * zoom_level as usize);
                let world_y = start_y + (view_y * zoom_level as usize);

                // Sample from heightmap (with bounds checking)
                if world_y < world_height && world_x < world_width {
                    row.push(heightmap[world_y][world_x]);
                } else {
                    row.push(0.0); // Default value for out-of-bounds
                }
            }
            visible.push(row);
        }

        visible
    }

    /// Extract visible portion of water layer for rendering with zoom support
    pub fn extract_visible_water(
        &self,
        water_layer: &crate::sim::WaterLayer,
        zoom_level: u32,
    ) -> Vec<Vec<f32>> {
        let world_height = water_layer.depth.len();
        let world_width = if world_height > 0 {
            water_layer.depth[0].len()
        } else {
            0
        };

        let start_y = self.world_y.max(0) as usize;
        let start_x = self.world_x.max(0) as usize;

        let mut visible = Vec::new();

        for view_y in 0..self.view_height {
            let mut row = Vec::new();

            for view_x in 0..self.view_width {
                // Calculate world coordinates based on zoom level
                let world_x = start_x + (view_x * zoom_level as usize);
                let world_y = start_y + (view_y * zoom_level as usize);

                // Sample from water depth (with bounds checking)
                if world_y < world_height && world_x < world_width {
                    row.push(water_layer.depth[world_y][world_x]);
                } else {
                    row.push(0.0); // Default value for out-of-bounds
                }
            }
            visible.push(row);
        }

        visible
    }

    /// Extract visible portion of water velocity for flow direction arrows
    pub fn extract_visible_water_velocity(
        &self,
        water_layer: &crate::sim::WaterLayer,
        zoom_level: u32,
    ) -> Vec<Vec<crate::sim::Vec2>> {
        let world_height = water_layer.velocity.len();
        let world_width = if world_height > 0 {
            water_layer.velocity[0].len()
        } else {
            0
        };

        let start_y = self.world_y.max(0) as usize;
        let start_x = self.world_x.max(0) as usize;

        let mut visible = Vec::new();

        for view_y in 0..self.view_height {
            let mut row = Vec::new();

            for view_x in 0..self.view_width {
                // Calculate world coordinates based on zoom level
                let world_x = start_x + (view_x * zoom_level as usize);
                let world_y = start_y + (view_y * zoom_level as usize);

                // Sample from water velocity (with bounds checking)
                if world_y < world_height && world_x < world_width {
                    row.push(water_layer.velocity[world_y][world_x].clone());
                } else {
                    row.push(crate::sim::Vec2::zero()); // Default value for out-of-bounds
                }
            }
            visible.push(row);
        }

        visible
    }
}

/// TUI application state
pub struct TuiApp {
    pub simulation: Simulation,
    pub viewport: Viewport,
    pub should_quit: bool,
    pub zoom_level: u32,  // 1 = 1:1, 2 = 1:2, 4 = 1:4, etc.
    pub paused: bool,     // Whether simulation is paused
    pub show_water: bool, // Whether to visualize water layer
}

impl TuiApp {
    pub fn new(simulation: Simulation) -> Self {
        // Start with reasonable viewport size (will be updated based on terminal)
        let viewport = Viewport::new(80, 24);

        Self {
            simulation,
            viewport,
            should_quit: false,
            zoom_level: 1,    // Start at 1:1 zoom
            paused: false,    // Start with simulation running
            show_water: true, // Start with water visualization enabled
        }
    }

    /// Get terrain info at current cursor position
    pub fn get_cursor_terrain_info(&self) -> (f32, &'static str, &'static str) {
        let world_height = self.simulation.heightmap.len();
        let world_width = if world_height > 0 {
            self.simulation.heightmap[0].len()
        } else {
            0
        };

        // Calculate center of viewport (cursor position)
        let cursor_x = (self.viewport.world_x + (self.viewport.view_width as i32 / 2)) as usize;
        let cursor_y = (self.viewport.world_y + (self.viewport.view_height as i32 / 2)) as usize;

        // Get elevation at cursor (with bounds checking)
        let elevation = if cursor_y < world_height && cursor_x < world_width {
            self.simulation.heightmap[cursor_y][cursor_x]
        } else {
            0.0 // Default for out-of-bounds
        };

        // Determine terrain type and symbol
        let (terrain_type, symbol) = match elevation {
            x if x < 0.2 => ("Deep Water", "·"),
            x if x < 0.4 => ("Shallow/Coast", "~"),
            x if x < 0.6 => ("Plains", "-"),
            x if x < 0.8 => ("Hills", "^"),
            _ => ("Mountains", "▲"),
        };

        (elevation, terrain_type, symbol)
    }

    /// Handle keyboard input events
    pub fn handle_key_event(&mut self, key_code: KeyCode) {
        let movement_speed = 1; // Could be made configurable for fast navigation

        match key_code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            // WASD navigation
            KeyCode::Char('w') | KeyCode::Up => {
                self.viewport.move_by(
                    0,
                    -movement_speed,
                    self.simulation.heightmap[0].len(),
                    self.simulation.heightmap.len(),
                );
            }
            KeyCode::Char('s') | KeyCode::Down => {
                self.viewport.move_by(
                    0,
                    movement_speed,
                    self.simulation.heightmap[0].len(),
                    self.simulation.heightmap.len(),
                );
            }
            KeyCode::Char('a') | KeyCode::Left => {
                self.viewport.move_by(
                    -movement_speed,
                    0,
                    self.simulation.heightmap[0].len(),
                    self.simulation.heightmap.len(),
                );
            }
            KeyCode::Char('d') | KeyCode::Right => {
                self.viewport.move_by(
                    movement_speed,
                    0,
                    self.simulation.heightmap[0].len(),
                    self.simulation.heightmap.len(),
                );
            }
            // Fast movement with Shift (future enhancement)
            KeyCode::Char('W') => {
                self.viewport.move_by(
                    0,
                    -5,
                    self.simulation.heightmap[0].len(),
                    self.simulation.heightmap.len(),
                );
            }
            KeyCode::Char('S') => {
                self.viewport.move_by(
                    0,
                    5,
                    self.simulation.heightmap[0].len(),
                    self.simulation.heightmap.len(),
                );
            }
            KeyCode::Char('A') => {
                self.viewport.move_by(
                    -5,
                    0,
                    self.simulation.heightmap[0].len(),
                    self.simulation.heightmap.len(),
                );
            }
            KeyCode::Char('D') => {
                self.viewport.move_by(
                    5,
                    0,
                    self.simulation.heightmap[0].len(),
                    self.simulation.heightmap.len(),
                );
            }
            // Zoom controls
            KeyCode::Char('=') | KeyCode::Char('+') => {
                if self.zoom_level > 1 {
                    self.zoom_level /= 2; // Zoom in (1:4 -> 1:2 -> 1:1)
                }
            }
            KeyCode::Char('-') => {
                if self.zoom_level < 4 {
                    self.zoom_level *= 2; // Zoom out (1:1 -> 1:2 -> 1:4)
                }
            }
            // Simulation controls
            KeyCode::Char(' ') => {
                self.paused = !self.paused; // Toggle pause/resume
            }
            KeyCode::Char('r') => {
                if !self.paused {
                    self.simulation.tick(); // Manual single step when running
                }
            }
            KeyCode::Char('t') => {
                if self.paused {
                    self.simulation.tick(); // Manual tick when paused
                }
            }
            KeyCode::Char('v') => {
                self.show_water = !self.show_water; // Toggle water visualization
            }
            // Add water at cursor position for testing
            KeyCode::Char('f') => {
                let cursor_x =
                    (self.viewport.world_x + (self.viewport.view_width as i32 / 2)) as usize;
                let cursor_y =
                    (self.viewport.world_y + (self.viewport.view_height as i32 / 2)) as usize;
                self.simulation.add_water_at(cursor_x, cursor_y, 0.1);
            }
            _ => {}
        }
    }

    /// Update viewport size based on available terminal area
    pub fn update_viewport_size(&mut self, render_area: Rect) {
        // Account for borders, status bar, and mini-map space
        let minimap_width = 22; // 20 chars + 2 for borders
        let available_width = (render_area.width.saturating_sub(2 + minimap_width)) as usize;
        let available_height = (render_area.height.saturating_sub(3)) as usize; // 2 for borders + 1 for status

        self.viewport.view_width = available_width;
        self.viewport.view_height = available_height;
    }
}

/// Convert velocity vector to directional arrow character
fn velocity_to_arrow(velocity: &crate::sim::Vec2) -> char {
    let magnitude = velocity.magnitude();
    if magnitude < 0.01 {
        return ' '; // No significant flow
    }

    // Determine primary direction based on dominant component
    let angle = velocity.y.atan2(velocity.x);
    let angle_degrees = angle.to_degrees();

    // Convert to 8-directional arrows
    match angle_degrees {
        a if a >= -22.5 && a < 22.5 => '→',    // East
        a if a >= 22.5 && a < 67.5 => '↗',     // Northeast
        a if a >= 67.5 && a < 112.5 => '↑',    // North
        a if a >= 112.5 && a < 157.5 => '↖',   // Northwest
        a if a >= 157.5 || a < -157.5 => '←',  // West
        a if a >= -157.5 && a < -112.5 => '↙', // Southwest
        a if a >= -112.5 && a < -67.5 => '↓',  // South
        a if a >= -67.5 && a < -22.5 => '↘',   // Southeast
        _ => '·',                              // Fallback
    }
}

/// Render the visible terrain with optional water overlay using ASCII characters with colors and cursor
fn render_terrain_with_water(
    app: &TuiApp,
    visible_heightmap: &[Vec<f32>],
    show_cursor: bool,
    cursor_x: usize,
    cursor_y: usize,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Extract visible water data if showing water
    let (visible_water, visible_velocity) = if app.show_water {
        (
            Some(
                app.viewport
                    .extract_visible_water(&app.simulation.water, app.zoom_level),
            ),
            Some(
                app.viewport
                    .extract_visible_water_velocity(&app.simulation.water, app.zoom_level),
            ),
        )
    } else {
        (None, None)
    };

    for (row_idx, row) in visible_heightmap.iter().enumerate() {
        let mut spans = Vec::new();

        for (col_idx, &terrain_val) in row.iter().enumerate() {
            // Check if this is the cursor position
            let is_cursor = show_cursor && row_idx == cursor_y && col_idx == cursor_x;

            let (symbol, _color, style) = if is_cursor {
                // Cursor: bright white dot with dark background
                (
                    '●',
                    Color::White,
                    Style::default().fg(Color::White).bg(Color::Black),
                )
            } else {
                // Get water depth and velocity at this position if water overlay is enabled
                let water_depth = if let Some(water_data) = &visible_water {
                    if row_idx < water_data.len() && col_idx < water_data[row_idx].len() {
                        water_data[row_idx][col_idx]
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                let water_velocity = if let Some(velocity_data) = &visible_velocity {
                    if row_idx < velocity_data.len() && col_idx < velocity_data[row_idx].len() {
                        &velocity_data[row_idx][col_idx]
                    } else {
                        &crate::sim::Vec2::zero()
                    }
                } else {
                    &crate::sim::Vec2::zero()
                };

                // Render water if present, otherwise terrain
                // Use scale-aware water visualization thresholds based on evaporation_threshold
                let base_threshold = app.simulation.water_system.evaporation_threshold;
                if app.show_water && water_depth > base_threshold {
                    // Enhanced water visualization with flow arrows prioritized across all depths
                    let arrow = velocity_to_arrow(water_velocity);
                    let has_significant_flow = arrow != ' ' && arrow != '·';

                    // Scale all water depth thresholds proportionally to the evaporation threshold
                    // Use more conservative multipliers for better visual gradation
                    let (water_symbol, water_color, bg_color) = match water_depth {
                        x if x > base_threshold * 8.0 => ('■', Color::White, Color::Blue), // Very deep pools - white on blue
                        x if x > base_threshold * 6.0 => ('≋', Color::LightBlue, Color::Blue), // Deep flowing water - light blue on blue
                        x if x > base_threshold * 4.0 => {
                            // Medium water - show flow direction arrows where meaningful
                            if has_significant_flow {
                                (arrow, Color::White, Color::Cyan) // Flow arrows in medium water - white on cyan
                            } else {
                                ('~', Color::White, Color::Cyan) // Static medium water - white on cyan
                            }
                        }
                        x if x > base_threshold * 2.5 => ('·', Color::Blue, Color::LightBlue), // Shallow water - blue on light blue
                        x if x > base_threshold * 1.5 => ('░', Color::Cyan, Color::LightCyan), // Light moisture - cyan on light cyan
                        _ => ('▒', Color::Gray, Color::White), // Trace moisture - gray on white
                    };
                    (
                        water_symbol,
                        water_color,
                        Style::default().fg(water_color).bg(bg_color),
                    )
                } else {
                    // Normal terrain (base terrain + any deposited sediment)
                    let (terrain_symbol, terrain_color) = match terrain_val {
                        x if x < 0.2 => ('.', Color::Blue),   // Deep water
                        x if x < 0.4 => ('~', Color::Cyan),   // Shallow water/coastline
                        x if x < 0.6 => ('-', Color::Green),  // Plains/flatlands
                        x if x < 0.8 => ('^', Color::Yellow), // Hills/foothills
                        _ => ('▲', Color::Red),               // Mountains/peaks
                    };
                    (
                        terrain_symbol,
                        terrain_color,
                        Style::default().fg(terrain_color),
                    )
                }
            };

            spans.push(Span::styled(symbol.to_string(), style));
        }
        lines.push(Line::from(spans));
    }

    lines
}

/// Render mini-map with viewport indicator and optional water overlay
fn render_minimap_with_viewport(
    heightmap: &[Vec<f32>],
    water_layer: Option<&crate::sim::WaterLayer>,
    show_water: bool,
    viewport: &Viewport,
    minimap_width: usize,
    minimap_height: usize,
    evaporation_threshold: f32,
) -> Vec<Line<'static>> {
    let world_height = heightmap.len();
    let world_width = if world_height > 0 {
        heightmap[0].len()
    } else {
        0
    };

    let mut minimap_lines = Vec::new();

    // Calculate viewport bounds in minimap coordinates
    let viewport_left = (viewport.world_x as usize * minimap_width) / world_width;
    let viewport_right =
        ((viewport.world_x as usize + viewport.view_width) * minimap_width) / world_width;
    let viewport_top = (viewport.world_y as usize * minimap_height) / world_height;
    let viewport_bottom =
        ((viewport.world_y as usize + viewport.view_height) * minimap_height) / world_height;

    for minimap_y in 0..minimap_height {
        let mut spans = Vec::new();

        for minimap_x in 0..minimap_width {
            // Sample from the full heightmap
            let world_x = (minimap_x * world_width) / minimap_width;
            let world_y = (minimap_y * world_height) / minimap_height;

            let height_val = heightmap[world_y.min(world_height - 1)][world_x.min(world_width - 1)];

            // Get water depth if water layer is available and enabled
            let water_depth = if show_water && water_layer.is_some() {
                let water = water_layer.unwrap();
                if world_y < water.depth.len() && world_x < water.depth[0].len() {
                    water.depth[world_y][world_x]
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Check if this cell is within the viewport
            let in_viewport = minimap_x >= viewport_left
                && minimap_x <= viewport_right
                && minimap_y >= viewport_top
                && minimap_y <= viewport_bottom;

            // Check if this is the center/current position
            let is_center = minimap_x == (viewport_left + viewport_right) / 2
                && minimap_y == (viewport_top + viewport_bottom) / 2;

            let (symbol, _color, style) = if is_center {
                // Mark exact center with a bright cursor
                (
                    '●',
                    Color::White,
                    Style::default().fg(Color::White).bg(Color::Black),
                )
            } else if in_viewport {
                // Highlight viewport area - show water if present, otherwise terrain
                let (display_symbol, fg_color, bg_color) =
                    if show_water && water_depth > evaporation_threshold {
                        let symbol = match water_depth {
                            x if x > evaporation_threshold * 8.0 => '■',
                            x if x > evaporation_threshold * 6.0 => '≋',
                            x if x > evaporation_threshold * 4.0 => '~',
                            x if x > evaporation_threshold * 2.5 => '·',
                            x if x > evaporation_threshold * 1.5 => '░',
                            _ => '▒',
                        };
                        (symbol, Color::White, Color::Gray) // Keep viewport highlighting visible
                    } else {
                        let symbol = match height_val {
                            x if x < 0.2 => '·',
                            x if x < 0.4 => '~',
                            x if x < 0.6 => '-',
                            x if x < 0.8 => '^',
                            _ => '▲',
                        };
                        (symbol, Color::White, Color::Gray) // Keep viewport highlighting visible
                    };
                (
                    display_symbol,
                    fg_color,
                    Style::default().fg(fg_color).bg(bg_color),
                )
            } else {
                // Normal mini-map - show water if present, otherwise terrain
                if show_water && water_depth > evaporation_threshold {
                    // Get velocity for flow direction (minimap doesn't need arrows, but could show flow intensity)
                    let (water_symbol, water_color, bg_color) = match water_depth {
                        x if x > evaporation_threshold * 8.0 => ('■', Color::White, Color::Blue),
                        x if x > evaporation_threshold * 6.0 => {
                            ('≋', Color::LightBlue, Color::Blue)
                        }
                        x if x > evaporation_threshold * 4.0 => ('~', Color::White, Color::Cyan),
                        x if x > evaporation_threshold * 2.5 => {
                            ('·', Color::Blue, Color::LightBlue)
                        }
                        x if x > evaporation_threshold * 1.5 => {
                            ('░', Color::Cyan, Color::LightCyan)
                        }
                        _ => ('▒', Color::Gray, Color::White),
                    };
                    (
                        water_symbol,
                        water_color,
                        Style::default().fg(water_color).bg(bg_color),
                    )
                } else {
                    let (terrain_symbol, terrain_color) = match height_val {
                        x if x < 0.2 => ('·', Color::Blue),
                        x if x < 0.4 => ('~', Color::Cyan),
                        x if x < 0.6 => ('-', Color::Green),
                        x if x < 0.8 => ('^', Color::Yellow),
                        _ => ('▲', Color::Red),
                    };
                    (
                        terrain_symbol,
                        terrain_color,
                        Style::default().fg(terrain_color),
                    )
                }
            };

            spans.push(Span::styled(symbol.to_string(), style));
        }

        minimap_lines.push(Line::from(spans));
    }

    minimap_lines
}

/// Main UI rendering function
pub fn ui(f: &mut Frame, app: &mut TuiApp) {
    // Update viewport size based on terminal
    let size = f.size();
    app.update_viewport_size(size);

    // Create main layout: content area + status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Content area
            Constraint::Length(1), // Status bar
        ])
        .split(size);

    // Create horizontal layout: terrain view + sidebar
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),     // Main terrain view
            Constraint::Length(22), // Sidebar (20 chars + 2 for borders)
        ])
        .split(main_chunks[0]);

    // Create vertical layout for sidebar: mini-map + legend
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14), // Mini-map (12 lines + 2 for borders)
            Constraint::Min(0),     // Legend
        ])
        .split(content_chunks[1]);

    // Extract visible terrain region with zoom
    let visible_heightmap = app
        .viewport
        .extract_visible_region(&app.simulation.heightmap, app.zoom_level);

    // Calculate cursor position in the visible region (center of viewport)
    let cursor_x = app.viewport.view_width / 2;
    let cursor_y = app.viewport.view_height / 2;

    let terrain_lines =
        render_terrain_with_water(app, &visible_heightmap, true, cursor_x, cursor_y);

    // Main terrain viewport
    let terrain_paragraph = Paragraph::new(terrain_lines)
        .block(
            Block::default()
                .title("Terrain Explorer")
                .borders(Borders::ALL),
        )
        .style(Style::default());

    f.render_widget(terrain_paragraph, content_chunks[0]);

    // Mini-map with viewport indicator and water overlay
    let minimap_lines = render_minimap_with_viewport(
        &app.simulation.heightmap,
        Some(&app.simulation.water),
        app.show_water,
        &app.viewport,
        20, // minimap width
        12, // minimap height
        app.simulation.water_system.evaporation_threshold,
    );

    let minimap_paragraph = Paragraph::new(minimap_lines)
        .block(Block::default().title("Map").borders(Borders::ALL))
        .style(Style::default());

    f.render_widget(minimap_paragraph, sidebar_chunks[0]);

    // Elevation legend
    let legend_lines = vec![
        Line::from(vec![
            Span::styled("·", Style::default().fg(Color::Blue)),
            Span::raw(" Deep Water"),
        ]),
        Line::from(vec![
            Span::styled("~", Style::default().fg(Color::Cyan)),
            Span::raw(" Shallow/Coast"),
        ]),
        Line::from(vec![
            Span::styled("-", Style::default().fg(Color::Green)),
            Span::raw(" Plains"),
        ]),
        Line::from(vec![
            Span::styled("^", Style::default().fg(Color::Yellow)),
            Span::raw(" Hills"),
        ]),
        Line::from(vec![
            Span::styled("▲", Style::default().fg(Color::Red)),
            Span::raw(" Mountains"),
        ]),
    ];

    let legend_paragraph = Paragraph::new(legend_lines)
        .block(Block::default().title("Legend").borders(Borders::ALL))
        .style(Style::default());

    f.render_widget(legend_paragraph, sidebar_chunks[1]);

    // Status bar with navigation info, terrain data, and simulation controls
    let _world_width = app.simulation.heightmap[0].len();
    let _world_height = app.simulation.heightmap.len();
    let (elevation, terrain_type, symbol) = app.get_cursor_terrain_info();
    let total_water = app.simulation.water.get_total_water();

    let status_text = format!(
        "Pos: ({}, {}) | Zoom: 1:{} | {} {} ({:.3}) | Water: {:.1} | Tick: {} | {} | WASD=Move SPC=Pause F=AddWater V=ToggleWater Q=Quit",
        app.viewport.world_x,
        app.viewport.world_y,
        app.zoom_level,
        symbol,
        terrain_type,
        elevation,
        total_water,
        app.simulation.tick_count,
        if app.paused { "PAUSED" } else { "RUNNING" }
    );

    let status_paragraph = Paragraph::new(status_text).style(Style::default().fg(Color::Gray));

    f.render_widget(status_paragraph, main_chunks[1]);
}

/// Run the TUI application
pub fn run_tui(simulation: Simulation) -> Result<(), Box<dyn std::error::Error>> {
    // Check if we're in a proper terminal
    if !atty::is(atty::Stream::Stdout) {
        return Err("TUI mode requires a terminal. Try running with --ascii flag instead.".into());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = TuiApp::new(simulation);

    // Event handling loop with optimized timing
    let mut needs_redraw = true;
    let mut last_redraw = Instant::now();
    let mut last_sim_tick = Instant::now();
    let min_frame_time = Duration::from_millis(33); // ~30fps, more responsive than 60fps for terminal
    let sim_tick_interval = Duration::from_millis(100); // ~10 simulation ticks per second

    loop {
        // Run simulation tick if not paused and enough time has passed
        if !app.paused && last_sim_tick.elapsed() >= sim_tick_interval {
            app.simulation.tick();
            last_sim_tick = Instant::now();
            needs_redraw = true; // Redraw after simulation update
        }

        // Only redraw if needed and enough time has passed
        if needs_redraw && last_redraw.elapsed() >= min_frame_time {
            terminal.draw(|f| ui(f, &mut app))?;
            needs_redraw = false;
            last_redraw = std::time::Instant::now();
        }

        // Handle events immediately without tokio complexity
        if event::poll(Duration::from_millis(1))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.handle_key_event(key.code);
                    needs_redraw = true; // Mark for redraw only when something changes
                }
                Event::Resize(_, _) => {
                    needs_redraw = true;
                }
                _ => {}
            }
        }

        // Check for exit condition
        if app.should_quit {
            break;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
