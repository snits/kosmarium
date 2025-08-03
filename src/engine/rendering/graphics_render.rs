// ABOUTME: Graphics rendering system using macroquad for atmospheric visualization
// ABOUTME: Handles wind vectors, pressure fields, and weather patterns with 2D graphics

use super::super::agents::biome::BiomeType;
use super::super::physics::atmosphere::{WeatherPattern, WeatherPatternType};
use crate::engine::Simulation;
use crate::engine::physics::climate::AtmosphericPressureLayer;
use macroquad::prelude::*;

// Layout constants for bounded viewport system
const LEFT_SIDEBAR_WIDTH: f32 = 160.0;
const RIGHT_SIDEBAR_WIDTH: f32 = 200.0;
const TOP_BAR_HEIGHT: f32 = 40.0;
const BOTTOM_BAR_HEIGHT: f32 = 30.0;

pub struct GraphicsRenderer {
    camera: Camera2D,
    viewport: Rect,
    display_mode: DisplayMode,
    zoom_level: f32,
    pan_offset: Vec2,
    simulation_paused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayMode {
    Elevation,
    Water,
    Pressure,
    Wind,
    Weather,
    Temperature,
    Biomes,
}

impl GraphicsRenderer {
    pub fn new(width: f32, height: f32) -> Self {
        // Calculate bounded viewport area (central map area minus UI sidebars)
        let viewport_x = LEFT_SIDEBAR_WIDTH;
        let viewport_y = TOP_BAR_HEIGHT;
        let viewport_width = width - LEFT_SIDEBAR_WIDTH - RIGHT_SIDEBAR_WIDTH;
        let viewport_height = height - TOP_BAR_HEIGHT - BOTTOM_BAR_HEIGHT;

        let viewport = Rect::new(viewport_x, viewport_y, viewport_width, viewport_height);

        // Initialize camera to center of the constrained viewport
        let mut camera = Camera2D::from_display_rect(viewport);
        camera.target = Vec2::new(
            viewport_x + viewport_width * 0.5,
            viewport_y + viewport_height * 0.5,
        );

        Self {
            camera,
            viewport,
            display_mode: DisplayMode::Elevation,
            zoom_level: 1.0,
            pan_offset: Vec2::new(0.0, 0.0),
            simulation_paused: false,
        }
    }

    pub fn render_simulation(&mut self, simulation: &mut Simulation) {
        clear_background(BLACK);

        // Use default camera for consistent coordinate system
        set_default_camera();

        match self.display_mode {
            DisplayMode::Elevation => self.render_elevation(simulation),
            DisplayMode::Water => self.render_water(simulation),
            DisplayMode::Pressure => self.render_pressure_field(simulation),
            DisplayMode::Wind => self.render_wind_field(simulation),
            DisplayMode::Weather => self.render_weather_patterns(simulation),
            DisplayMode::Temperature => self.render_temperature_field(simulation),
            DisplayMode::Biomes => self.render_biomes(simulation),
        }

        self.render_ui(simulation);
    }

    fn render_elevation(&self, simulation: &Simulation) {
        let cell_size = self.calculate_cell_size(simulation.get_width(), simulation.get_height());

        // Center the simulation data in the viewport with pan offset
        let total_width = simulation.get_width() as f32 * cell_size;
        let total_height = simulation.get_height() as f32 * cell_size;
        let offset_x = self.viewport.x + (self.viewport.w - total_width) * 0.5 + self.pan_offset.x;
        let offset_y = self.viewport.y + (self.viewport.h - total_height) * 0.5 + self.pan_offset.y;

        for y in 0..simulation.get_height() {
            for x in 0..simulation.get_width() {
                let elevation = simulation.get_elevation(x, y);
                let color = self.elevation_to_color(elevation);

                let world_x = offset_x + x as f32 * cell_size;
                let world_y = offset_y + y as f32 * cell_size;

                draw_rectangle(world_x, world_y, cell_size, cell_size, color);
            }
        }
    }

    fn render_water(&self, simulation: &Simulation) {
        // Render elevation as base
        self.render_elevation(simulation);

        // Overlay water layer if available
        let water_layer = simulation.get_water_layer();
        let cell_size = self.calculate_cell_size(water_layer.width(), water_layer.height());

        // Center the simulation data in the viewport (same as elevation mode)
        let total_width = water_layer.width() as f32 * cell_size;
        let total_height = water_layer.height() as f32 * cell_size;
        let offset_x = self.viewport.x + (self.viewport.w - total_width) * 0.5 + self.pan_offset.x;
        let offset_y = self.viewport.y + (self.viewport.h - total_height) * 0.5 + self.pan_offset.y;

        for y in 0..water_layer.height() {
            for x in 0..water_layer.width() {
                let water_depth = water_layer.get_water_depth(x, y);
                if water_depth > 0.0 {
                    let alpha = (water_depth * 255.0).min(200.0) as u8;
                    let water_color = Color::new(0.0, 0.4, 0.8, alpha as f32 / 255.0);

                    let world_x = offset_x + x as f32 * cell_size;
                    let world_y = offset_y + (water_layer.height() - 1 - y) as f32 * cell_size;
                    draw_rectangle(world_x, world_y, cell_size, cell_size, water_color);
                }
            }
        }
    }

    fn render_pressure_field(&self, simulation: &Simulation) {
        let pressure_layer = simulation.get_atmospheric_pressure_layer();
        let cell_size = self.calculate_cell_size(simulation.get_width(), simulation.get_height());

        // Center the simulation data in the viewport (same as elevation mode)
        let total_width = simulation.get_width() as f32 * cell_size;
        let total_height = simulation.get_height() as f32 * cell_size;
        let offset_x = self.viewport.x + (self.viewport.w - total_width) * 0.5 + self.pan_offset.x;
        let offset_y = self.viewport.y + (self.viewport.h - total_height) * 0.5 + self.pan_offset.y;

        // Find pressure range for color mapping
        let (min_pressure, max_pressure) = self.find_pressure_range(pressure_layer);

        for y in 0..simulation.get_height() {
            for x in 0..simulation.get_width() {
                let pressure = pressure_layer.get_pressure(x, y);
                let color = self.pressure_to_color(pressure, min_pressure, max_pressure);

                let world_x = offset_x + x as f32 * cell_size;
                let world_y = offset_y + (simulation.get_height() - 1 - y) as f32 * cell_size;

                draw_rectangle(world_x, world_y, cell_size, cell_size, color);
            }
        }
    }

    fn render_wind_field(&self, simulation: &Simulation) {
        // Render pressure field as background
        self.render_pressure_field(simulation);

        let wind_layer = simulation.get_wind_layer();
        let cell_size = self.calculate_cell_size(simulation.get_width(), simulation.get_height());
        let arrow_scale = cell_size * 0.8;

        // Center the simulation data in the viewport (same as elevation mode)
        let total_width = simulation.get_width() as f32 * cell_size;
        let total_height = simulation.get_height() as f32 * cell_size;
        let offset_x = self.viewport.x + (self.viewport.w - total_width) * 0.5 + self.pan_offset.x;
        let offset_y = self.viewport.y + (self.viewport.h - total_height) * 0.5 + self.pan_offset.y;

        // Sample wind vectors at lower resolution to avoid clutter
        let sample_rate = (cell_size / 10.0).max(1.0) as usize;

        // Debug: Check wind data ranges (only print once to avoid spam)
        static mut DEBUG_PRINTED: bool = false;
        unsafe {
            if !DEBUG_PRINTED {
                let mut max_speed = 0.0f32;
                let mut min_speed = f32::INFINITY;
                let mut total_speed = 0.0f32;
                let mut count = 0;

                for y in 0..simulation.get_height() {
                    for x in 0..simulation.get_width() {
                        let speed = wind_layer.get_speed(x, y);
                        max_speed = max_speed.max(speed);
                        min_speed = min_speed.min(speed);
                        total_speed += speed;
                        count += 1;
                    }
                }
                let avg_speed = total_speed / count as f32;
                println!(
                    "WIND DEBUG: min={:.3}, max={:.3}, avg={:.3}",
                    min_speed, max_speed, avg_speed
                );
                DEBUG_PRINTED = true;
            }
        }

        for y in (0..simulation.get_height()).step_by(sample_rate) {
            for x in (0..simulation.get_width()).step_by(sample_rate) {
                let velocity = wind_layer.get_velocity(x, y);
                let speed = wind_layer.get_speed(x, y);

                if speed > 0.1 {
                    // Only draw significant winds
                    let center_x = offset_x + x as f32 * cell_size + cell_size * 0.5;
                    let center_y = offset_y
                        + (simulation.get_height() - 1 - y) as f32 * cell_size
                        + cell_size * 0.5;

                    let arrow_length = (speed * arrow_scale).min(arrow_scale);
                    let end_x = center_x + velocity.x * arrow_length;
                    let end_y = center_y - velocity.y * arrow_length; // Flip Y direction to match coordinate system

                    let color = self.wind_speed_to_color(speed);

                    // Draw wind arrow
                    draw_line(center_x, center_y, end_x, end_y, 2.0, color);

                    // Draw arrowhead
                    self.draw_arrowhead(center_x, center_y, end_x, end_y, color);
                }
            }
        }
    }

    fn render_weather_patterns(&self, simulation: &Simulation) {
        // Render wind field as background
        self.render_wind_field(simulation);

        let weather_analysis = simulation.get_weather_analysis();
        let cell_size = self.calculate_cell_size(simulation.get_width(), simulation.get_height());

        // Center the simulation data in the viewport (same as elevation mode)
        let total_width = simulation.get_width() as f32 * cell_size;
        let total_height = simulation.get_height() as f32 * cell_size;
        let offset_x = self.viewport.x + (self.viewport.w - total_width) * 0.5 + self.pan_offset.x;
        let offset_y = self.viewport.y + (self.viewport.h - total_height) * 0.5 + self.pan_offset.y;

        for pattern in &weather_analysis.patterns {
            self.render_weather_pattern(
                pattern,
                cell_size,
                offset_x,
                offset_y,
                simulation.get_height(),
            );
        }
    }

    fn render_temperature_field(&self, simulation: &Simulation) {
        let temperature_layer = simulation.get_temperature_layer();
        let cell_size = self.calculate_cell_size(simulation.get_width(), simulation.get_height());

        // Center the simulation data in the viewport (same as elevation mode)
        let total_width = simulation.get_width() as f32 * cell_size;
        let total_height = simulation.get_height() as f32 * cell_size;
        let offset_x = self.viewport.x + (self.viewport.w - total_width) * 0.5 + self.pan_offset.x;
        let offset_y = self.viewport.y + (self.viewport.h - total_height) * 0.5 + self.pan_offset.y;

        // Find temperature range for color mapping
        let (min_temp, max_temp) = self.find_temperature_range(temperature_layer);

        for y in 0..simulation.get_height() {
            for x in 0..simulation.get_width() {
                let temperature = temperature_layer.get_temperature(x, y);
                let color = self.temperature_to_color(temperature, min_temp, max_temp);

                let world_x = offset_x + x as f32 * cell_size;
                let world_y = offset_y + (simulation.get_height() - 1 - y) as f32 * cell_size;

                draw_rectangle(world_x, world_y, cell_size, cell_size, color);
            }
        }
    }

    fn render_biomes(&self, simulation: &mut Simulation) {
        // Get dimensions before borrowing for biome map
        let width = simulation.get_width();
        let height = simulation.get_height();
        let cell_size = self.calculate_cell_size(width, height);

        // Center the simulation data in the viewport (same as elevation mode)
        let total_width = width as f32 * cell_size;
        let total_height = height as f32 * cell_size;
        let offset_x = self.viewport.x + (self.viewport.w - total_width) * 0.5 + self.pan_offset.x;
        let offset_y = self.viewport.y + (self.viewport.h - total_height) * 0.5 + self.pan_offset.y;

        // Get cached biome map
        let biome_map = simulation.generate_biome_map();

        for y in 0..height {
            for x in 0..width {
                let biome = biome_map.get(x, y);
                let (r, g, b) = biome.display_color();
                let color = Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);

                let world_x = offset_x + x as f32 * cell_size;
                let world_y = offset_y + (height - 1 - y) as f32 * cell_size;

                draw_rectangle(world_x, world_y, cell_size, cell_size, color);
            }
        }
    }

    fn render_weather_pattern(
        &self,
        pattern: &WeatherPattern,
        cell_size: f32,
        offset_x: f32,
        offset_y: f32,
        height: usize,
    ) {
        let center_x = offset_x + pattern.center.0 as f32 * cell_size;
        let center_y = offset_y + (height - 1 - pattern.center.1) as f32 * cell_size;
        let radius = pattern.radius as f32 * cell_size;

        let (color, thickness) = match pattern.pattern_type {
            WeatherPatternType::LowPressureSystem => (RED, 3.0),
            WeatherPatternType::HighPressureSystem => (BLUE, 3.0),
            WeatherPatternType::WindShear => (YELLOW, 2.0),
            WeatherPatternType::Calm => (GREEN, 1.0),
        };

        // Draw pattern boundary
        draw_circle_lines(center_x, center_y, radius, thickness, color);

        // Draw center marker
        draw_circle(center_x, center_y, 3.0, color);
    }

    fn render_ui(&self, simulation: &Simulation) {
        // Use default screen coordinates for UI
        set_default_camera();

        // Draw sidebar backgrounds for clarity
        draw_rectangle(
            0.0,
            0.0,
            LEFT_SIDEBAR_WIDTH,
            screen_height(),
            Color::new(0.1, 0.1, 0.1, 0.8),
        );
        draw_rectangle(
            screen_width() - RIGHT_SIDEBAR_WIDTH,
            0.0,
            RIGHT_SIDEBAR_WIDTH,
            screen_height(),
            Color::new(0.1, 0.1, 0.1, 0.8),
        );
        draw_rectangle(
            LEFT_SIDEBAR_WIDTH,
            0.0,
            screen_width() - LEFT_SIDEBAR_WIDTH - RIGHT_SIDEBAR_WIDTH,
            TOP_BAR_HEIGHT,
            Color::new(0.1, 0.1, 0.1, 0.8),
        );
        draw_rectangle(
            LEFT_SIDEBAR_WIDTH,
            screen_height() - BOTTOM_BAR_HEIGHT,
            screen_width() - LEFT_SIDEBAR_WIDTH - RIGHT_SIDEBAR_WIDTH,
            BOTTOM_BAR_HEIGHT,
            Color::new(0.1, 0.1, 0.1, 0.8),
        );

        // Left sidebar content - Mode info, zoom level, simulation status
        self.render_left_sidebar(simulation);

        // Right sidebar content - Color legends
        self.render_right_sidebar();

        // Top bar content - Mode indicator, time, simulation state
        self.render_top_bar(simulation);

        // Bottom bar content - Control instructions
        self.render_bottom_bar();
    }

    fn render_left_sidebar(&self, simulation: &Simulation) {
        let sidebar_x = 10.0;
        let mut y_pos = 50.0;
        let line_height = 20.0;

        // Title
        draw_text("MODE INFO", sidebar_x, y_pos, 16.0, WHITE);
        y_pos += line_height * 1.5;

        // Display mode
        let mode_text = format!("Mode: {:?}", self.display_mode);
        draw_text(&mode_text, sidebar_x, y_pos, 14.0, LIGHTGRAY);
        y_pos += line_height;

        // Mode switch info
        draw_text("(1-7 to switch)", sidebar_x, y_pos, 12.0, DARKGRAY);
        y_pos += line_height * 1.5;

        // Zoom level
        let zoom_text = format!("Zoom: {:.1}x", self.zoom_level);
        draw_text(&zoom_text, sidebar_x, y_pos, 14.0, LIGHTGRAY);
        y_pos += line_height * 2.0;

        // Simulation status section
        draw_text("SIMULATION", sidebar_x, y_pos, 16.0, WHITE);
        y_pos += line_height * 1.5;

        // Simulation time
        let sim_time = simulation.get_simulation_time();
        let time_text = format!(
            "Day {}, {:02}:{:02}",
            sim_time.days, sim_time.hours, sim_time.minutes
        );
        draw_text(&time_text, sidebar_x, y_pos, 14.0, LIGHTGRAY);
        y_pos += line_height;

        // Simulation state
        let sim_state = if self.simulation_paused {
            "PAUSED"
        } else {
            "RUNNING"
        };
        let sim_color = if self.simulation_paused {
            YELLOW
        } else {
            GREEN
        };
        draw_text(
            &format!("Status: {}", sim_state),
            sidebar_x,
            y_pos,
            14.0,
            sim_color,
        );
    }

    fn render_right_sidebar(&self) {
        // Color legend in right sidebar
        self.render_color_legend();
    }

    fn render_top_bar(&self, simulation: &Simulation) {
        let bar_y = 20.0;
        let section_width = (screen_width() - LEFT_SIDEBAR_WIDTH - RIGHT_SIDEBAR_WIDTH) / 3.0;

        // Mode indicator (left section)
        let mode_x = LEFT_SIDEBAR_WIDTH + 10.0;
        draw_text(
            &format!("Mode: {:?}", self.display_mode),
            mode_x,
            bar_y,
            16.0,
            WHITE,
        );

        // Time (center section)
        let sim_time = simulation.get_simulation_time();
        let time_text = format!(
            "Day {}, {:02}:{:02}",
            sim_time.days, sim_time.hours, sim_time.minutes
        );
        let time_x = LEFT_SIDEBAR_WIDTH + section_width;
        draw_text(&time_text, time_x, bar_y, 16.0, LIGHTGRAY);

        // Simulation state (right section)
        let sim_state = if self.simulation_paused {
            "PAUSED"
        } else {
            "RUNNING"
        };
        let sim_color = if self.simulation_paused {
            YELLOW
        } else {
            GREEN
        };
        let state_x = LEFT_SIDEBAR_WIDTH + section_width * 2.0;
        draw_text(
            &format!("Simulation: {}", sim_state),
            state_x,
            bar_y,
            16.0,
            sim_color,
        );
    }

    fn render_bottom_bar(&self) {
        let bar_y = screen_height() - 15.0;
        let instructions_x = LEFT_SIDEBAR_WIDTH + 10.0;

        // Control instructions
        draw_text(
            "WASD: Pan, Mouse Wheel: Zoom, R: Reset, SPACE: Pause/Play, 1-7: Display Mode, ESC: Quit",
            instructions_x,
            bar_y,
            14.0,
            LIGHTGRAY,
        );
    }

    fn render_color_legend(&self) {
        let legend_x = screen_width() - RIGHT_SIDEBAR_WIDTH + 10.0;
        let mut legend_y = 50.0;
        let legend_spacing = 20.0;

        // Title for color legend
        draw_text("COLOR LEGEND", legend_x, legend_y, 16.0, WHITE);
        legend_y += legend_spacing * 1.5;

        match self.display_mode {
            DisplayMode::Elevation => {
                draw_text("Elevation:", legend_x, legend_y, 14.0, LIGHTGRAY);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, BLUE, "Water", 12.0);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, SKYBLUE, "Coast", 12.0);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, GREEN, "Plains", 12.0);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, YELLOW, "Hills", 12.0);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, RED, "Mountains", 12.0);
            }
            DisplayMode::Pressure => {
                draw_text("Pressure:", legend_x, legend_y, 14.0, LIGHTGRAY);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, BLUE, "Low Pressure", 12.0);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, RED, "High Pressure", 12.0);
            }
            DisplayMode::Wind => {
                draw_text("Wind:", legend_x, legend_y, 14.0, LIGHTGRAY);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, BLUE, "Low Pressure", 12.0);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, RED, "High Pressure", 12.0);
                legend_y += legend_spacing;
                draw_text("White arrows = wind", legend_x, legend_y, 12.0, WHITE);
            }
            DisplayMode::Weather => {
                draw_text("Weather Systems:", legend_x, legend_y, 14.0, LIGHTGRAY);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, RED, "Low Pressure", 12.0);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, BLUE, "High Pressure", 12.0);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, YELLOW, "Wind Shear", 12.0);
                legend_y += legend_spacing;
                self.draw_legend_item(legend_x, legend_y, GREEN, "Calm Zone", 12.0);
            }
            DisplayMode::Temperature => {
                draw_text("Temperature:", legend_x, legend_y, 14.0, LIGHTGRAY);
                legend_y += legend_spacing;
                self.draw_legend_item(
                    legend_x,
                    legend_y,
                    Color::new(0.0, 0.0, 1.0, 1.0),
                    "Cold",
                    12.0,
                );
                legend_y += legend_spacing;
                self.draw_legend_item(
                    legend_x,
                    legend_y,
                    Color::new(1.0, 0.0, 0.0, 1.0),
                    "Hot",
                    12.0,
                );
            }
            DisplayMode::Water => {
                draw_text("Water Depth:", legend_x, legend_y, 14.0, LIGHTGRAY);
                legend_y += legend_spacing;
                self.draw_legend_item(
                    legend_x,
                    legend_y,
                    Color::new(0.0, 0.4, 0.8, 0.3),
                    "Shallow",
                    12.0,
                );
                legend_y += legend_spacing;
                self.draw_legend_item(
                    legend_x,
                    legend_y,
                    Color::new(0.0, 0.4, 0.8, 0.8),
                    "Deep",
                    12.0,
                );
            }
            DisplayMode::Biomes => {
                draw_text("Biomes:", legend_x, legend_y, 14.0, LIGHTGRAY);
                legend_y += legend_spacing;

                // Show common biome types with their colors
                let biome_entries = [
                    (BiomeType::Ocean, "Ocean"),
                    (BiomeType::Lake, "Lake"),
                    (BiomeType::Grassland, "Grassland"),
                    (BiomeType::TemperateForest, "Forest"),
                    (BiomeType::Desert, "Desert"),
                    (BiomeType::Tundra, "Tundra"),
                ];

                for (biome_type, label) in biome_entries.iter() {
                    let (r, g, b) = biome_type.display_color();
                    let color =
                        Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
                    self.draw_legend_item(legend_x, legend_y, color, label, 12.0);
                    legend_y += legend_spacing;
                }
            }
        }
    }

    fn draw_legend_item(&self, x: f32, y: f32, color: Color, text: &str, font_size: f32) {
        // Draw color square
        draw_rectangle(x, y - 8.0, 12.0, 12.0, color);
        // Draw text
        draw_text(text, x + 18.0, y + 4.0, font_size, WHITE);
    }

    // Helper methods for color mapping
    fn elevation_to_color(&self, elevation: f32) -> Color {
        match elevation {
            e if e < 0.2 => BLUE,    // Water
            e if e < 0.4 => SKYBLUE, // Coast
            e if e < 0.6 => GREEN,   // Plains
            e if e < 0.8 => YELLOW,  // Hills
            _ => RED,                // Mountains
        }
    }

    fn pressure_to_color(&self, pressure: f32, min_p: f32, max_p: f32) -> Color {
        let normalized = (pressure - min_p) / (max_p - min_p);
        Color::new(normalized, 0.2, 1.0 - normalized, 0.8)
    }

    fn temperature_to_color(&self, temperature: f32, min_t: f32, max_t: f32) -> Color {
        let normalized = (temperature - min_t) / (max_t - min_t);
        Color::new(normalized, 0.0, 1.0 - normalized, 0.8)
    }

    fn wind_speed_to_color(&self, speed: f32) -> Color {
        let intensity = (speed * 2.0).min(1.0);
        Color::new(1.0, 1.0 - intensity, 1.0 - intensity, 0.9)
    }

    fn calculate_cell_size(&self, width: usize, height: usize) -> f32 {
        let viewport_width = self.viewport.w;
        let viewport_height = self.viewport.h;

        let base_cell_size = (viewport_width / width as f32).min(viewport_height / height as f32);
        base_cell_size * self.zoom_level
    }

    fn find_pressure_range(&self, pressure_layer: &AtmosphericPressureLayer) -> (f32, f32) {
        let mut min_p = f32::INFINITY;
        let mut max_p = f32::NEG_INFINITY;

        for y in 0..pressure_layer.height() {
            for x in 0..pressure_layer.width() {
                let pressure = pressure_layer.get_pressure(x, y);
                min_p = min_p.min(pressure);
                max_p = max_p.max(pressure);
            }
        }

        (min_p, max_p)
    }

    fn find_temperature_range(
        &self,
        temperature_layer: &super::super::physics::climate::TemperatureLayer,
    ) -> (f32, f32) {
        let mut min_t = f32::INFINITY;
        let mut max_t = f32::NEG_INFINITY;

        for y in 0..temperature_layer.height() {
            for x in 0..temperature_layer.width() {
                let temperature = temperature_layer.get_temperature(x, y);
                min_t = min_t.min(temperature);
                max_t = max_t.max(temperature);
            }
        }

        (min_t, max_t)
    }

    fn draw_arrowhead(&self, start_x: f32, start_y: f32, end_x: f32, end_y: f32, color: Color) {
        let dx = end_x - start_x;
        let dy = end_y - start_y;
        let length = (dx * dx + dy * dy).sqrt();

        if length > 0.0 {
            let unit_x = dx / length;
            let unit_y = dy / length;

            let head_length = 5.0;
            let head_width = 3.0;

            let back_x = end_x - unit_x * head_length;
            let back_y = end_y - unit_y * head_length;

            let perp_x = -unit_y * head_width;
            let perp_y = unit_x * head_width;

            draw_line(end_x, end_y, back_x + perp_x, back_y + perp_y, 2.0, color);
            draw_line(end_x, end_y, back_x - perp_x, back_y - perp_y, 2.0, color);
        }
    }

    // Input handling
    pub fn should_tick_simulation(&self) -> bool {
        !self.simulation_paused
    }

    pub fn handle_input(&mut self) {
        // Display mode switching
        if is_key_pressed(KeyCode::Key1) {
            self.display_mode = DisplayMode::Elevation;
        }
        if is_key_pressed(KeyCode::Key2) {
            self.display_mode = DisplayMode::Water;
        }
        if is_key_pressed(KeyCode::Key3) {
            self.display_mode = DisplayMode::Pressure;
        }
        if is_key_pressed(KeyCode::Key4) {
            self.display_mode = DisplayMode::Wind;
        }
        if is_key_pressed(KeyCode::Key5) {
            self.display_mode = DisplayMode::Weather;
        }
        if is_key_pressed(KeyCode::Key6) {
            self.display_mode = DisplayMode::Temperature;
        }
        if is_key_pressed(KeyCode::Key7) {
            self.display_mode = DisplayMode::Biomes;
        }

        // Simulation control
        if is_key_pressed(KeyCode::Space) {
            self.simulation_paused = !self.simulation_paused;
        }

        // Reset zoom and center view
        if is_key_pressed(KeyCode::R) {
            self.zoom_level = 1.0;
            self.pan_offset = Vec2::new(0.0, 0.0);
        }

        // Pan movement
        let pan_speed = 10.0;
        if is_key_down(KeyCode::W) {
            self.pan_offset.y += pan_speed;
        }
        if is_key_down(KeyCode::S) {
            self.pan_offset.y -= pan_speed;
        }
        if is_key_down(KeyCode::A) {
            self.pan_offset.x += pan_speed;
        }
        if is_key_down(KeyCode::D) {
            self.pan_offset.x -= pan_speed;
        }

        // Zoom with mouse wheel (more gradual)
        let (_x, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            let zoom_factor = 1.05; // More gradual zoom
            let zoom_delta = if scroll_y > 0.0 {
                zoom_factor
            } else {
                1.0 / zoom_factor
            };

            self.zoom_level *= zoom_delta;
            self.zoom_level = self.zoom_level.clamp(0.2, 5.0); // Tighter zoom range
        }
    }

    pub fn handle_resize(&mut self) {
        // Handle window resize - recalculate constrained viewport dimensions
        let current_width = screen_width();
        let current_height = screen_height();

        // Calculate new constrained viewport
        let new_viewport_x = LEFT_SIDEBAR_WIDTH;
        let new_viewport_y = TOP_BAR_HEIGHT;
        let new_viewport_width = current_width - LEFT_SIDEBAR_WIDTH - RIGHT_SIDEBAR_WIDTH;
        let new_viewport_height = current_height - TOP_BAR_HEIGHT - BOTTOM_BAR_HEIGHT;

        let new_viewport = Rect::new(
            new_viewport_x,
            new_viewport_y,
            new_viewport_width,
            new_viewport_height,
        );

        // Check if viewport actually changed
        if (self.viewport.w - new_viewport_width).abs() > 1.0
            || (self.viewport.h - new_viewport_height).abs() > 1.0
        {
            // Update viewport
            self.viewport = new_viewport;

            // Update camera display rect to match new constrained viewport
            self.camera = Camera2D::from_display_rect(new_viewport);
            self.camera.target = Vec2::new(
                new_viewport_x + new_viewport_width * 0.5,
                new_viewport_y + new_viewport_height * 0.5,
            );
            self.camera.zoom = Vec2::new(self.zoom_level, self.zoom_level);
        }
    }
}
