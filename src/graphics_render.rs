// ABOUTME: Graphics rendering system using macroquad for atmospheric visualization
// ABOUTME: Handles wind vectors, pressure fields, and weather patterns with 2D graphics

use crate::atmosphere::{WeatherAnalysis, WeatherPattern, WeatherPatternType};
use crate::climate::{AtmosphericPressureLayer, TemperatureLayer};
use crate::sim::Simulation;
use macroquad::prelude::*;

pub struct GraphicsRenderer {
    camera: Camera2D,
    viewport: Rect,
    display_mode: DisplayMode,
    zoom_level: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayMode {
    Terrain,
    Water,
    Pressure,
    Wind,
    Weather,
    Temperature,
}

impl GraphicsRenderer {
    pub fn new(width: f32, height: f32) -> Self {
        let camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, width, height));

        Self {
            camera,
            viewport: Rect::new(0.0, 0.0, width, height),
            display_mode: DisplayMode::Terrain,
            zoom_level: 1.0,
        }
    }

    pub fn render_simulation(&mut self, simulation: &Simulation) {
        clear_background(BLACK);

        set_camera(&self.camera);

        match self.display_mode {
            DisplayMode::Terrain => self.render_terrain(simulation),
            DisplayMode::Water => self.render_water(simulation),
            DisplayMode::Pressure => self.render_pressure_field(simulation),
            DisplayMode::Wind => self.render_wind_field(simulation),
            DisplayMode::Weather => self.render_weather_patterns(simulation),
            DisplayMode::Temperature => self.render_temperature_field(simulation),
        }

        self.render_ui(simulation);
    }

    fn render_terrain(&self, simulation: &Simulation) {
        let cell_size = self.calculate_cell_size(simulation.get_width(), simulation.get_height());

        for y in 0..simulation.get_height() {
            for x in 0..simulation.get_width() {
                let elevation = simulation.get_elevation(x, y);
                let color = self.elevation_to_color(elevation);

                let world_x = x as f32 * cell_size;
                let world_y = y as f32 * cell_size;

                draw_rectangle(world_x, world_y, cell_size, cell_size, color);
            }
        }
    }

    fn render_water(&self, simulation: &Simulation) {
        // Render terrain as base
        self.render_terrain(simulation);

        // Overlay water layer if available
        let water_layer = simulation.get_water_layer();
        let cell_size = self.calculate_cell_size(water_layer.width(), water_layer.height());

        for y in 0..water_layer.height() {
            for x in 0..water_layer.width() {
                let water_depth = water_layer.get_water_depth(x, y);
                if water_depth > 0.0 {
                    let alpha = (water_depth * 255.0).min(200.0) as u8;
                    let water_color = Color::new(0.0, 0.4, 0.8, alpha as f32 / 255.0);

                    let world_x = x as f32 * cell_size;
                    let world_y = y as f32 * cell_size;

                    draw_rectangle(world_x, world_y, cell_size, cell_size, water_color);
                }
            }
        }
    }

    fn render_pressure_field(&self, simulation: &Simulation) {
        let pressure_layer = simulation.get_atmospheric_pressure_layer();
        let cell_size = self.calculate_cell_size(simulation.get_width(), simulation.get_height());

        // Find pressure range for color mapping
        let (min_pressure, max_pressure) = self.find_pressure_range(pressure_layer);

        for y in 0..simulation.get_height() {
            for x in 0..simulation.get_width() {
                let pressure = pressure_layer.get_pressure(x, y);
                let color = self.pressure_to_color(pressure, min_pressure, max_pressure);

                let world_x = x as f32 * cell_size;
                let world_y = y as f32 * cell_size;

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

        // Sample wind vectors at lower resolution to avoid clutter
        let sample_rate = (cell_size / 10.0).max(1.0) as usize;

        for y in (0..simulation.get_height()).step_by(sample_rate) {
            for x in (0..simulation.get_width()).step_by(sample_rate) {
                let velocity = wind_layer.get_velocity(x, y);
                let speed = wind_layer.get_speed(x, y);

                if speed > 0.1 {
                    // Only draw significant winds
                    let center_x = x as f32 * cell_size + cell_size * 0.5;
                    let center_y = y as f32 * cell_size + cell_size * 0.5;

                    let arrow_length = (speed * arrow_scale).min(arrow_scale);
                    let end_x = center_x + velocity.x * arrow_length;
                    let end_y = center_y + velocity.y * arrow_length;

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

        for pattern in &weather_analysis.patterns {
            self.render_weather_pattern(pattern, cell_size);
        }
    }

    fn render_temperature_field(&self, simulation: &Simulation) {
        let temperature_layer = simulation.get_temperature_layer();
        let cell_size = self.calculate_cell_size(simulation.get_width(), simulation.get_height());

        // Find temperature range for color mapping
        let (min_temp, max_temp) = self.find_temperature_range(temperature_layer);

        for y in 0..simulation.get_height() {
            for x in 0..simulation.get_width() {
                let temperature = temperature_layer.get_temperature(x, y);
                let color = self.temperature_to_color(temperature, min_temp, max_temp);

                let world_x = x as f32 * cell_size;
                let world_y = y as f32 * cell_size;

                draw_rectangle(world_x, world_y, cell_size, cell_size, color);
            }
        }
    }

    fn render_weather_pattern(&self, pattern: &WeatherPattern, cell_size: f32) {
        let center_x = pattern.center.0 as f32 * cell_size;
        let center_y = pattern.center.1 as f32 * cell_size;
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
        set_default_camera();

        // Display mode indicator
        let mode_text = format!("Mode: {:?} (1-6 to switch)", self.display_mode);
        draw_text(&mode_text, 10.0, 30.0, 20.0, WHITE);

        // Zoom level
        let zoom_text = format!("Zoom: {:.1}x", self.zoom_level);
        draw_text(&zoom_text, 10.0, 55.0, 20.0, WHITE);

        // Instructions
        draw_text(
            "WASD: Pan, Mouse Wheel: Zoom, 1-6: Display Mode",
            10.0,
            screen_height() - 20.0,
            16.0,
            LIGHTGRAY,
        );
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
        let viewport_width = self.viewport.w / self.zoom_level;
        let viewport_height = self.viewport.h / self.zoom_level;

        (viewport_width / width as f32).min(viewport_height / height as f32)
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
        temperature_layer: &crate::climate::TemperatureLayer,
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
    pub fn handle_input(&mut self) {
        // Display mode switching
        if is_key_pressed(KeyCode::Key1) {
            self.display_mode = DisplayMode::Terrain;
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

        // Camera movement
        let pan_speed = 10.0 / self.zoom_level;
        if is_key_down(KeyCode::W) {
            self.camera.target.y -= pan_speed;
        }
        if is_key_down(KeyCode::S) {
            self.camera.target.y += pan_speed;
        }
        if is_key_down(KeyCode::A) {
            self.camera.target.x -= pan_speed;
        }
        if is_key_down(KeyCode::D) {
            self.camera.target.x += pan_speed;
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

            // Update camera zoom
            self.camera.zoom = Vec2::new(self.zoom_level, self.zoom_level);
        }
    }
}
