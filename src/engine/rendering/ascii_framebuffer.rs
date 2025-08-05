// ABOUTME: ASCII framebuffer system for multi-layer temporal visualization of simulation data
// ABOUTME: Provides real-time monitoring with configurable layers, change detection, and frame buffering

use super::super::agents::biome::BiomeType;
use super::super::sim::Simulation;
use std::collections::VecDeque;

/// Available visualization layers for ASCII framebuffer
#[derive(Debug, Clone, PartialEq)]
pub enum VisualizationLayer {
    Elevation,
    Water,
    Biomes,
    Temperature,
    Pressure,
    Wind,
    Flow,
    Changes,
    Sediment,
}

impl VisualizationLayer {
    /// Parse layer from string for CLI arguments
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "elevation" | "elev" | "height" => Some(Self::Elevation),
            "water" | "depth" => Some(Self::Water),
            "biomes" | "biome" => Some(Self::Biomes),
            "temperature" | "temp" => Some(Self::Temperature),
            "pressure" | "press" => Some(Self::Pressure),
            "wind" => Some(Self::Wind),
            "flow" | "velocity" => Some(Self::Flow),
            "changes" | "diff" => Some(Self::Changes),
            "sediment" | "sed" => Some(Self::Sediment),
            _ => None,
        }
    }

    /// Get display name for headers
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Elevation => "ELEVATION",
            Self::Water => "WATER",
            Self::Biomes => "BIOMES",
            Self::Temperature => "TEMPERATURE",
            Self::Pressure => "PRESSURE",
            Self::Wind => "WIND",
            Self::Flow => "FLOW",
            Self::Changes => "CHANGES",
            Self::Sediment => "SEDIMENT",
        }
    }
}

/// Configuration for ASCII framebuffer rendering
#[derive(Debug, Clone)]
pub struct FramebufferConfig {
    /// Layers to display
    pub layers: Vec<VisualizationLayer>,
    /// Number of frames to buffer for temporal analysis
    pub buffer_size: usize,
    /// Width of each layer panel (0 = auto-size)
    pub panel_width: usize,
    /// Height of each layer panel (0 = auto-size)
    pub panel_height: usize,
    /// Show frame numbers and timestamps
    pub show_timestamps: bool,
    /// Highlight changes between frames
    pub highlight_changes: bool,
    /// Subsample rate for large maps (1 = every cell, 2 = every other cell, etc.)
    pub subsample_rate: usize,
}

impl Default for FramebufferConfig {
    fn default() -> Self {
        Self {
            layers: vec![
                VisualizationLayer::Elevation,
                VisualizationLayer::Water,
                VisualizationLayer::Biomes,
            ],
            buffer_size: 10,
            panel_width: 0,  // Auto-size
            panel_height: 0, // Auto-size
            show_timestamps: true,
            highlight_changes: false,
            subsample_rate: 1,
        }
    }
}

/// Single ASCII frame containing all layer data
#[derive(Debug, Clone)]
pub struct AsciiFrame {
    /// Frame number
    pub frame_number: usize,
    /// Simulation time/iteration when frame was captured
    pub simulation_time: u64,
    /// ASCII representations for each layer
    pub layer_data: Vec<LayerFrame>,
    /// Frame dimensions (width, height)
    pub dimensions: (usize, usize),
}

/// ASCII data for a single visualization layer
#[derive(Debug, Clone)]
pub struct LayerFrame {
    /// Which layer this represents
    pub layer_type: VisualizationLayer,
    /// ASCII character data (row-major order)
    pub chars: Vec<Vec<char>>,
    /// Color codes for each character (optional)
    pub colors: Vec<Vec<u8>>,
}

/// ASCII framebuffer system with temporal buffering
pub struct AsciiFramebuffer {
    /// Configuration
    config: FramebufferConfig,
    /// Circular buffer of frames
    frame_buffer: VecDeque<AsciiFrame>,
    /// Current frame number
    current_frame: usize,
    /// Cached previous frame for change detection
    previous_frame: Option<AsciiFrame>,
}

impl AsciiFramebuffer {
    /// Create new ASCII framebuffer with configuration
    pub fn new(config: FramebufferConfig) -> Self {
        let mut frame_buffer = VecDeque::new();
        frame_buffer.reserve(config.buffer_size);

        Self {
            config,
            frame_buffer,
            current_frame: 0,
            previous_frame: None,
        }
    }

    /// Capture current simulation state as ASCII frame
    pub fn capture_frame(&mut self, simulation: &Simulation) -> AsciiFrame {
        let width = simulation.get_width();
        let height = simulation.get_height();

        // Calculate display dimensions with subsampling
        let display_width = (width + self.config.subsample_rate - 1) / self.config.subsample_rate;
        let display_height = (height + self.config.subsample_rate - 1) / self.config.subsample_rate;

        // Apply panel size limits if specified
        let final_width = if self.config.panel_width > 0 {
            self.config.panel_width.min(display_width)
        } else {
            display_width.min(80) // Reasonable terminal width
        };
        let final_height = if self.config.panel_height > 0 {
            self.config.panel_height.min(display_height)
        } else {
            display_height.min(24) // Reasonable terminal height
        };

        let mut layer_data = Vec::new();

        // Generate ASCII data for each requested layer
        for layer_type in &self.config.layers {
            let layer_frame = self.generate_layer_frame(
                simulation,
                layer_type.clone(),
                final_width,
                final_height,
                width,
                height,
            );
            layer_data.push(layer_frame);
        }

        let frame = AsciiFrame {
            frame_number: self.current_frame,
            simulation_time: simulation.tick_count,
            layer_data,
            dimensions: (final_width, final_height),
        };

        self.current_frame += 1;
        frame
    }

    /// Add frame to buffer and maintain size limit
    pub fn add_frame(&mut self, frame: AsciiFrame) {
        // Store previous frame for change detection
        if !self.frame_buffer.is_empty() {
            self.previous_frame = self.frame_buffer.back().cloned();
        }

        // Add new frame
        self.frame_buffer.push_back(frame);

        // Maintain buffer size limit
        while self.frame_buffer.len() > self.config.buffer_size {
            self.frame_buffer.pop_front();
        }
    }

    /// Generate ASCII representation for a specific layer
    fn generate_layer_frame(
        &self,
        simulation: &Simulation,
        layer_type: VisualizationLayer,
        display_width: usize,
        display_height: usize,
        sim_width: usize,
        sim_height: usize,
    ) -> LayerFrame {
        let mut chars = vec![vec![' '; display_width]; display_height];
        let mut colors = vec![vec![0u8; display_width]; display_height];

        match layer_type {
            VisualizationLayer::Elevation => {
                self.generate_elevation_layer(
                    simulation,
                    &mut chars,
                    &mut colors,
                    display_width,
                    display_height,
                    sim_width,
                    sim_height,
                );
            }
            VisualizationLayer::Water => {
                self.generate_water_layer(
                    simulation,
                    &mut chars,
                    &mut colors,
                    display_width,
                    display_height,
                    sim_width,
                    sim_height,
                );
            }
            VisualizationLayer::Biomes => {
                self.generate_biomes_layer(
                    simulation,
                    &mut chars,
                    &mut colors,
                    display_width,
                    display_height,
                    sim_width,
                    sim_height,
                );
            }
            VisualizationLayer::Temperature => {
                self.generate_temperature_layer(
                    simulation,
                    &mut chars,
                    &mut colors,
                    display_width,
                    display_height,
                    sim_width,
                    sim_height,
                );
            }
            VisualizationLayer::Pressure => {
                self.generate_pressure_layer(
                    simulation,
                    &mut chars,
                    &mut colors,
                    display_width,
                    display_height,
                    sim_width,
                    sim_height,
                );
            }
            VisualizationLayer::Wind => {
                self.generate_wind_layer(
                    simulation,
                    &mut chars,
                    &mut colors,
                    display_width,
                    display_height,
                    sim_width,
                    sim_height,
                );
            }
            VisualizationLayer::Flow => {
                self.generate_flow_layer(
                    simulation,
                    &mut chars,
                    &mut colors,
                    display_width,
                    display_height,
                    sim_width,
                    sim_height,
                );
            }
            VisualizationLayer::Changes => {
                self.generate_changes_layer(&mut chars, &mut colors, display_width, display_height);
            }
            VisualizationLayer::Sediment => {
                self.generate_sediment_layer(
                    simulation,
                    &mut chars,
                    &mut colors,
                    display_width,
                    display_height,
                    sim_width,
                    sim_height,
                );
            }
        }

        LayerFrame {
            layer_type,
            chars,
            colors,
        }
    }

    /// Generate elevation layer ASCII
    fn generate_elevation_layer(
        &self,
        simulation: &Simulation,
        chars: &mut Vec<Vec<char>>,
        _colors: &mut Vec<Vec<u8>>,
        display_width: usize,
        display_height: usize,
        sim_width: usize,
        sim_height: usize,
    ) {
        for y in 0..display_height {
            for x in 0..display_width {
                // Map display coordinates to simulation coordinates with subsampling
                let sim_x = (x * sim_width) / display_width;
                let sim_y = (y * sim_height) / display_height;

                let elevation = simulation.get_elevation(sim_x, sim_y);
                chars[y][x] = match elevation {
                    e if e < -0.5 => '~', // Deep water
                    e if e < 0.0 => '.',  // Shallow water
                    e if e < 0.2 => ',',  // Beach/coast
                    e if e < 0.4 => '^',  // Low hills
                    e if e < 0.6 => '#',  // Hills
                    e if e < 0.8 => '@',  // Mountains
                    _ => '%',             // High peaks
                };
            }
        }
    }

    /// Generate water layer ASCII
    fn generate_water_layer(
        &self,
        simulation: &Simulation,
        chars: &mut Vec<Vec<char>>,
        _colors: &mut Vec<Vec<u8>>,
        display_width: usize,
        display_height: usize,
        sim_width: usize,
        sim_height: usize,
    ) {
        let water_system = simulation.get_water_system();
        let threshold = water_system.evaporation_threshold;

        for y in 0..display_height {
            for x in 0..display_width {
                let sim_x = (x * sim_width) / display_width;
                let sim_y = (y * sim_height) / display_height;

                // Get water depth (need to access water layer directly)
                let water_depth = if sim_x < sim_width && sim_y < sim_height {
                    simulation.water.depth.get(sim_x, sim_y)
                } else {
                    0.0
                };

                chars[y][x] = match water_depth {
                    d if d < threshold => '.',        // Dry
                    d if d < threshold * 5.0 => ':',  // Trace water
                    d if d < threshold * 20.0 => '~', // Shallow water
                    d if d < threshold * 50.0 => '#', // Deep water
                    _ => '@',                         // Very deep water
                };
            }
        }
    }

    /// Generate biomes layer ASCII
    fn generate_biomes_layer(
        &self,
        simulation: &Simulation,
        chars: &mut Vec<Vec<char>>,
        _colors: &mut Vec<Vec<u8>>,
        display_width: usize,
        display_height: usize,
        sim_width: usize,
        sim_height: usize,
    ) {
        let biome_map = simulation.generate_biome_map_basic();

        for y in 0..display_height {
            for x in 0..display_width {
                let sim_x = (x * sim_width) / display_width;
                let sim_y = (y * sim_height) / display_height;

                let biome = if sim_x < biome_map.width() && sim_y < biome_map.height() {
                    biome_map.get(sim_x, sim_y)
                } else {
                    BiomeType::Ocean
                };

                chars[y][x] = match biome {
                    BiomeType::Ocean => '~',
                    BiomeType::Lake => '=',
                    BiomeType::River => '-',
                    BiomeType::Wetland => '*',
                    BiomeType::Grassland => 'G',
                    BiomeType::Savanna => 'S',
                    BiomeType::Shrubland => 's',
                    BiomeType::TemperateForest => 'F',
                    BiomeType::Tundra => 'T',
                    BiomeType::Desert => 'D',
                    BiomeType::RainForest => 'R',
                    BiomeType::BorealForest => 'B',
                    BiomeType::Alpine => 'A',
                    BiomeType::Ice => 'I',
                };
            }
        }
    }

    /// Generate temperature layer ASCII
    fn generate_temperature_layer(
        &self,
        simulation: &Simulation,
        chars: &mut Vec<Vec<char>>,
        _colors: &mut Vec<Vec<u8>>,
        display_width: usize,
        display_height: usize,
        sim_width: usize,
        sim_height: usize,
    ) {
        let temp_layer = simulation.get_temperature_layer();

        for y in 0..display_height {
            for x in 0..display_width {
                let sim_x = (x * sim_width) / display_width;
                let sim_y = (y * sim_height) / display_height;

                let temperature = if sim_x < sim_width && sim_y < sim_height {
                    temp_layer.get_temperature(sim_x, sim_y)
                } else {
                    0.0
                };

                chars[y][x] = match temperature {
                    t if t < -10.0 => '■', // Very cold
                    t if t < 0.0 => '▓',   // Cold
                    t if t < 10.0 => '▒',  // Cool
                    t if t < 20.0 => '░',  // Mild
                    t if t < 30.0 => '.',  // Warm
                    t if t < 40.0 => '+',  // Hot
                    _ => '#',              // Very hot
                };
            }
        }
    }

    /// Generate pressure layer ASCII
    fn generate_pressure_layer(
        &self,
        simulation: &Simulation,
        chars: &mut Vec<Vec<char>>,
        _colors: &mut Vec<Vec<u8>>,
        display_width: usize,
        display_height: usize,
        sim_width: usize,
        sim_height: usize,
    ) {
        let pressure_layer = simulation.get_pressure_layer();

        // Calculate pressure range for normalization
        let mut min_pressure = f32::INFINITY;
        let mut max_pressure = f32::NEG_INFINITY;

        for row in &pressure_layer.pressure {
            for &p in row {
                min_pressure = min_pressure.min(p);
                max_pressure = max_pressure.max(p);
            }
        }

        let pressure_range = max_pressure - min_pressure;

        for y in 0..display_height {
            for x in 0..display_width {
                let sim_x = (x * sim_width) / display_width;
                let sim_y = (y * sim_height) / display_height;

                let pressure = pressure_layer.get_pressure(sim_x, sim_y);
                let normalized = if pressure_range > 0.0 {
                    (pressure - min_pressure) / pressure_range
                } else {
                    0.5
                };

                chars[y][x] = match normalized {
                    n if n < 0.2 => '-', // Low pressure
                    n if n < 0.4 => '.', // Below average
                    n if n < 0.6 => '0', // Average
                    n if n < 0.8 => '+', // Above average
                    _ => '#',            // High pressure
                };
            }
        }
    }

    /// Generate wind layer ASCII
    fn generate_wind_layer(
        &self,
        simulation: &Simulation,
        chars: &mut Vec<Vec<char>>,
        _colors: &mut Vec<Vec<u8>>,
        display_width: usize,
        display_height: usize,
        sim_width: usize,
        sim_height: usize,
    ) {
        let wind_layer = simulation.get_wind_layer();

        for y in 0..display_height {
            for x in 0..display_width {
                let sim_x = (x * sim_width) / display_width;
                let sim_y = (y * sim_height) / display_height;

                let velocity = wind_layer.get_velocity(sim_x, sim_y);
                let speed = velocity.magnitude();

                if speed < 1.0 {
                    chars[y][x] = '.'; // Calm
                } else {
                    // Convert wind direction to arrow character
                    let angle = velocity.y.atan2(velocity.x);
                    let angle_deg = angle.to_degrees();
                    let normalized_angle = ((angle_deg + 360.0) % 360.0) / 45.0;

                    chars[y][x] = match normalized_angle as i32 {
                        0 => '→', // East
                        1 => '↗', // Northeast
                        2 => '↑', // North
                        3 => '↖', // Northwest
                        4 => '←', // West
                        5 => '↙', // Southwest
                        6 => '↓', // South
                        7 => '↘', // Southeast
                        _ => '→', // Default to east
                    };
                }
            }
        }
    }

    /// Generate flow layer ASCII (water velocity)
    fn generate_flow_layer(
        &self,
        simulation: &Simulation,
        chars: &mut Vec<Vec<char>>,
        _colors: &mut Vec<Vec<u8>>,
        display_width: usize,
        display_height: usize,
        sim_width: usize,
        sim_height: usize,
    ) {
        for y in 0..display_height {
            for x in 0..display_width {
                let sim_x = (x * sim_width) / display_width;
                let sim_y = (y * sim_height) / display_width;

                let velocity = if sim_x < sim_width && sim_y < sim_height {
                    simulation.water.velocity.get(sim_x, sim_y)
                } else {
                    (0.0, 0.0)
                };

                let speed = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();

                if speed < 0.001 {
                    chars[y][x] = '.'; // No flow
                } else {
                    // Convert flow direction to arrow character
                    let angle = velocity.1.atan2(velocity.0);
                    let angle_deg = angle.to_degrees();
                    let normalized_angle = ((angle_deg + 360.0) % 360.0) / 45.0;

                    chars[y][x] = match normalized_angle as i32 {
                        0 => '→', // East
                        1 => '↗', // Northeast
                        2 => '↑', // North
                        3 => '↖', // Northwest
                        4 => '←', // West
                        5 => '↙', // Southwest
                        6 => '↓', // South
                        7 => '↘', // Southeast
                        _ => '→', // Default to east
                    };
                }
            }
        }
    }

    /// Generate changes layer (difference from previous frame)
    fn generate_changes_layer(
        &self,
        chars: &mut Vec<Vec<char>>,
        _colors: &mut Vec<Vec<u8>>,
        display_width: usize,
        display_height: usize,
    ) {
        // For now, just mark as placeholder
        // In a full implementation, this would compare with previous frame
        for y in 0..display_height {
            for x in 0..display_width {
                chars[y][x] = if (x + y) % 4 == 0 { '▲' } else { '.' };
            }
        }
    }

    /// Generate sediment layer ASCII
    fn generate_sediment_layer(
        &self,
        simulation: &Simulation,
        chars: &mut Vec<Vec<char>>,
        _colors: &mut Vec<Vec<u8>>,
        display_width: usize,
        display_height: usize,
        sim_width: usize,
        sim_height: usize,
    ) {
        let threshold = simulation.get_water_system().evaporation_threshold * 10.0;

        for y in 0..display_height {
            for x in 0..display_width {
                let sim_x = (x * sim_width) / display_width;
                let sim_y = (y * sim_height) / display_height;

                let sediment = if sim_x < sim_width && sim_y < sim_height {
                    simulation.water.sediment.get(sim_x, sim_y)
                } else {
                    0.0
                };

                chars[y][x] = match sediment {
                    s if s < threshold => '.',        // No sediment
                    s if s < threshold * 2.0 => ':',  // Light sediment
                    s if s < threshold * 5.0 => '+',  // Medium sediment
                    s if s < threshold * 10.0 => '#', // Heavy sediment
                    _ => '@',                         // Very heavy sediment
                };
            }
        }
    }

    /// Format frame for display with multi-layer layout
    pub fn format_frame(&self, frame: &AsciiFrame) -> String {
        let mut output = String::new();

        if self.config.show_timestamps {
            output.push_str(&format!(
                "=== FRAME {:03} (t={:>6} ticks) ===\n",
                frame.frame_number, frame.simulation_time
            ));
        }

        let (width, height) = frame.dimensions;
        let layers_per_row = (self.config.layers.len()).min(4); // Max 4 layers per row
        let rows_needed = (self.config.layers.len() + layers_per_row - 1) / layers_per_row;

        for row in 0..rows_needed {
            // Headers
            for col in 0..layers_per_row {
                let layer_idx = row * layers_per_row + col;
                if layer_idx < frame.layer_data.len() {
                    let layer_name = frame.layer_data[layer_idx].layer_type.display_name();
                    output.push_str(&format!("{:<12} ", layer_name));
                }
            }
            output.push('\n');

            // Layer content
            for y in 0..height {
                for col in 0..layers_per_row {
                    let layer_idx = row * layers_per_row + col;
                    if layer_idx < frame.layer_data.len() {
                        let layer = &frame.layer_data[layer_idx];
                        for x in 0..width {
                            if x < layer.chars[y].len() {
                                output.push(layer.chars[y][x]);
                            } else {
                                output.push(' ');
                            }
                        }
                        output.push_str("  "); // Space between layers
                    }
                }
                output.push('\n');
            }
            output.push('\n'); // Space between rows
        }

        output
    }

    /// Get most recent frame
    pub fn latest_frame(&self) -> Option<&AsciiFrame> {
        self.frame_buffer.back()
    }

    /// Get frame by index (0 = oldest, len-1 = newest)
    pub fn get_frame(&self, index: usize) -> Option<&AsciiFrame> {
        self.frame_buffer.get(index)
    }

    /// Get number of buffered frames
    pub fn frame_count(&self) -> usize {
        self.frame_buffer.len()
    }
}
