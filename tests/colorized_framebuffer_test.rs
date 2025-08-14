// ABOUTME: Test suite for colorized ASCII framebuffer functionality
// ABOUTME: Validates ANSI color integration with existing framebuffer system

use sim_prototype::engine::agents::biome::BiomeType;
use sim_prototype::engine::core::heightmap::HeightMap;
use sim_prototype::engine::physics::worldgen::{
    DiamondSquareConfig, DiamondSquareGenerator, TerrainGenerator,
};
use sim_prototype::engine::rendering::ansi_colors::{
    ANSI_RESET, AnsiColor, biome_to_ansi_color, colorize_char, elevation_to_ansi_color,
    pressure_to_ansi_color, temperature_to_ansi_color,
};
use sim_prototype::engine::rendering::{AsciiFramebuffer, FramebufferConfig, VisualizationLayer};
use sim_prototype::engine::sim::Simulation;

#[test]
fn test_colorized_framebuffer_creation() {
    // Test that we can create a framebuffer with color support
    let config = FramebufferConfig {
        layers: vec![VisualizationLayer::Elevation],
        buffer_size: 1,
        panel_width: 20,
        panel_height: 10,
        show_timestamps: false,
        highlight_changes: false,
        subsample_rate: 1,
    };

    let framebuffer = AsciiFramebuffer::new(config);
    assert_eq!(framebuffer.frame_count(), 0);
}

#[test]
fn test_elevation_colorized_output() {
    // This test should PASS now that we've implemented colorized output

    // Create a simple simulation
    let generator = DiamondSquareGenerator::new(42);
    let config = DiamondSquareConfig::default();
    let heightmap = generator.generate(32, 32, &config);
    let mut sim = Simulation::new(heightmap);
    sim.tick(); // Generate some data

    // Create framebuffer configured for elevation
    let config = FramebufferConfig {
        layers: vec![VisualizationLayer::Elevation],
        buffer_size: 1,
        panel_width: 10,
        panel_height: 10,
        show_timestamps: false,
        highlight_changes: false,
        subsample_rate: 1,
    };

    let mut framebuffer = AsciiFramebuffer::new(config);
    let frame = framebuffer.capture_frame(&sim);
    framebuffer.add_frame(frame);

    // Get the formatted output
    let output = framebuffer.format_frame_colorized(framebuffer.latest_frame().unwrap());

    // This should contain ANSI color codes
    assert!(
        output.contains("\x1b["),
        "Output should contain ANSI escape sequences"
    );
    assert!(
        output.contains(ANSI_RESET),
        "Output should contain ANSI reset sequences"
    );

    // Should contain elevation characters with colors
    // Based on the elevation mapping:
    // e < -0.5 => '~' + Blue
    // e < 0.0 => '.' + Blue
    // e < 0.2 => ',' + Blue
    // e < 0.4 => '^' + Cyan
    // e < 0.6 => '#' + Green
    // e < 0.8 => '@' + Yellow
    // _ => '%' + Red

    let coastal_areas = colorize_char(',', AnsiColor::Blue); // Beach/coast with blue
    let low_hills = colorize_char('^', AnsiColor::Cyan); // Low hills with cyan  
    let hills = colorize_char('#', AnsiColor::Green); // Hills with green
    let mountains = colorize_char('@', AnsiColor::Yellow); // Mountains with yellow

    // At least one colored character should be present
    assert!(
        output.contains(&coastal_areas)
            || output.contains(&low_hills)
            || output.contains(&hills)
            || output.contains(&mountains),
        "Output should contain colorized elevation characters"
    );
}

#[test]
fn test_multi_layer_colorized_output() {
    // This test should PASS now that we've implemented multi-layer colorization

    let generator = DiamondSquareGenerator::new(123);
    let config = DiamondSquareConfig::default();
    let heightmap = generator.generate(16, 16, &config);
    let mut sim = Simulation::new(heightmap);
    for _ in 0..3 {
        sim.tick();
    } // Generate more varied data

    let config = FramebufferConfig {
        layers: vec![
            VisualizationLayer::Elevation,
            VisualizationLayer::Pressure,
            VisualizationLayer::Temperature,
        ],
        buffer_size: 1,
        panel_width: 8,
        panel_height: 8,
        show_timestamps: false,
        highlight_changes: false,
        subsample_rate: 1,
    };

    let mut framebuffer = AsciiFramebuffer::new(config);
    let frame = framebuffer.capture_frame(&sim);
    framebuffer.add_frame(frame);

    let output = framebuffer.format_frame_colorized(framebuffer.latest_frame().unwrap());

    // Should contain different color schemes for different layers
    assert!(
        output.contains("ELEVATION"),
        "Should contain elevation header"
    );
    assert!(
        output.contains("PRESSURE"),
        "Should contain pressure header"
    );
    assert!(
        output.contains("TEMPERATURE"),
        "Should contain temperature header"
    );

    // Should have lots of ANSI codes due to multiple colored layers
    let ansi_count = output.matches("\x1b[").count();
    assert!(
        ansi_count >= 10,
        "Should contain many ANSI color codes for multiple layers"
    );
}

#[test]
fn test_biome_color_consistency() {
    // Test that biome colors match the graphics frontend semantics
    assert_eq!(biome_to_ansi_color(BiomeType::Ocean), AnsiColor::Blue);
    assert_eq!(
        biome_to_ansi_color(BiomeType::Desert),
        AnsiColor::BrightYellow
    );
    assert_eq!(
        biome_to_ansi_color(BiomeType::TemperateForest),
        AnsiColor::BrightGreen
    );
    assert_eq!(biome_to_ansi_color(BiomeType::Ice), AnsiColor::BrightWhite);
}

#[test]
fn test_pressure_color_range() {
    // Test pressure color mapping matches graphics frontend logic
    let min_p = 1000.0;
    let max_p = 1020.0;

    // Low pressure should be blue
    assert_eq!(
        pressure_to_ansi_color(1002.0, min_p, max_p),
        AnsiColor::BrightBlue
    );

    // High pressure should be red
    assert_eq!(pressure_to_ansi_color(1018.0, min_p, max_p), AnsiColor::Red);

    // Average pressure should be white/neutral
    assert_eq!(
        pressure_to_ansi_color(1010.0, min_p, max_p),
        AnsiColor::White
    );
}

#[test]
fn test_color_format_functions() {
    let colored_text = colorize_char('A', AnsiColor::Red);
    assert_eq!(colored_text, "\x1b[31mA\x1b[0m");

    let blue_char = colorize_char('#', AnsiColor::Blue);
    assert_eq!(blue_char, "\x1b[34m#\x1b[0m");
}
