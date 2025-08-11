// ABOUTME: ANSI color mapping system for colorized ASCII framebuffer output
// ABOUTME: Provides semantic color coding matching graphics frontend for AI agent consumption

use super::super::agents::biome::BiomeType;

/// ANSI color codes for terminal output
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum AnsiColor {
    // Standard colors
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,

    // Bright colors
    BrightBlack = 90,
    BrightRed = 91,
    BrightGreen = 92,
    BrightYellow = 93,
    BrightBlue = 94,
    BrightMagenta = 95,
    BrightCyan = 96,
    BrightWhite = 97,
}

impl AnsiColor {
    /// Convert to ANSI escape sequence for foreground color
    pub fn fg(self) -> String {
        format!("\x1b[{}m", self as u8)
    }

    /// Convert to ANSI escape sequence for background color
    pub fn bg(self) -> String {
        format!("\x1b[{}m", (self as u8) + 10)
    }
}

/// ANSI color reset sequence
pub const ANSI_RESET: &str = "\x1b[0m";

/// Color mapping for elevation data (matching graphics frontend)
pub fn elevation_to_ansi_color(elevation: f32) -> AnsiColor {
    match elevation {
        e if e < 0.2 => AnsiColor::Blue,   // Water (BLUE)
        e if e < 0.4 => AnsiColor::Cyan,   // Coast (SKYBLUE -> Cyan)
        e if e < 0.6 => AnsiColor::Green,  // Plains (GREEN)
        e if e < 0.8 => AnsiColor::Yellow, // Hills (YELLOW)
        _ => AnsiColor::Red,               // Mountains (RED)
    }
}

/// Color mapping for pressure data (blue = low, red = high)
pub fn pressure_to_ansi_color(pressure: f32, min_pressure: f32, max_pressure: f32) -> AnsiColor {
    let range = max_pressure - min_pressure;
    if range < 0.1 {
        return AnsiColor::White; // Minimal variation
    }

    let normalized = (pressure - min_pressure) / range;
    match normalized {
        n if n < 0.2 => AnsiColor::BrightBlue, // Very low pressure
        n if n < 0.4 => AnsiColor::Blue,       // Low pressure
        n if n < 0.6 => AnsiColor::White,      // Average pressure
        n if n < 0.8 => AnsiColor::Yellow,     // High pressure
        _ => AnsiColor::Red,                   // Very high pressure
    }
}

/// Color mapping for temperature data (blue = cold, red = hot)
pub fn temperature_to_ansi_color(temperature: f32, min_temp: f32, max_temp: f32) -> AnsiColor {
    let normalized = (temperature - min_temp) / (max_temp - min_temp);
    match normalized {
        n if n < 0.2 => AnsiColor::BrightBlue, // Very cold
        n if n < 0.4 => AnsiColor::Blue,       // Cold
        n if n < 0.6 => AnsiColor::White,      // Moderate
        n if n < 0.8 => AnsiColor::Yellow,     // Warm
        _ => AnsiColor::Red,                   // Hot
    }
}

/// Color mapping for wind speed (blue = calm, red = strong)
pub fn wind_speed_to_ansi_color(speed: f32) -> AnsiColor {
    // Use logarithmic scaling like graphics frontend
    let log_speed = speed.max(1.0).ln();
    let min_log = 16.0f32.ln(); // ~2.77
    let max_log = 1000.0f32.ln(); // ~6.91

    let intensity = ((log_speed - min_log) / (max_log - min_log)).clamp(0.0, 1.0);

    match intensity {
        i if i < 0.25 => AnsiColor::Blue,   // Calm
        i if i < 0.5 => AnsiColor::Green,   // Light breeze
        i if i < 0.75 => AnsiColor::Yellow, // Strong wind
        _ => AnsiColor::Red,                // Very strong wind
    }
}

/// Color mapping for biome types
pub fn biome_to_ansi_color(biome: BiomeType) -> AnsiColor {
    match biome {
        BiomeType::Ocean => AnsiColor::Blue,
        BiomeType::Lake => AnsiColor::Cyan,
        BiomeType::River => AnsiColor::BrightCyan,
        BiomeType::Wetland => AnsiColor::BrightBlue,
        BiomeType::Grassland => AnsiColor::Green,
        BiomeType::Savanna => AnsiColor::BrightGreen,
        BiomeType::Shrubland => AnsiColor::Yellow,
        BiomeType::TemperateForest => AnsiColor::BrightGreen,
        BiomeType::Tundra => AnsiColor::BrightBlack,
        BiomeType::Desert => AnsiColor::BrightYellow,
        BiomeType::RainForest => AnsiColor::Green,
        BiomeType::BorealForest => AnsiColor::BrightGreen,
        BiomeType::Alpine => AnsiColor::White,
        BiomeType::Ice => AnsiColor::BrightWhite,
    }
}

/// Color mapping for water depth
pub fn water_depth_to_ansi_color(depth: f32, threshold: f32) -> AnsiColor {
    match depth {
        d if d < threshold => AnsiColor::BrightBlack, // Dry (dark)
        d if d < threshold * 5.0 => AnsiColor::BrightBlue, // Trace water
        d if d < threshold * 20.0 => AnsiColor::Blue, // Shallow water
        d if d < threshold * 50.0 => AnsiColor::Cyan, // Deep water
        _ => AnsiColor::BrightCyan,                   // Very deep water
    }
}

/// Color mapping for sediment levels
pub fn sediment_to_ansi_color(sediment: f32, threshold: f32) -> AnsiColor {
    match sediment {
        s if s < threshold => AnsiColor::BrightBlack, // No sediment (dark)
        s if s < threshold * 2.0 => AnsiColor::Yellow, // Light sediment
        s if s < threshold * 5.0 => AnsiColor::BrightYellow, // Medium sediment
        s if s < threshold * 10.0 => AnsiColor::Red,  // Heavy sediment
        _ => AnsiColor::BrightRed,                    // Very heavy sediment
    }
}

/// Combined wind colorization: speed intensity with directional hue modulation
pub fn wind_to_ansi_color(velocity: (f32, f32)) -> AnsiColor {
    let speed = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();

    // For very low wind speeds, use calm color
    if speed < 1.0 {
        return AnsiColor::BrightBlack; // Calm/no wind
    }

    // Calculate wind direction (in radians)
    let angle = velocity.1.atan2(velocity.0);
    let angle_deg = ((angle.to_degrees() + 360.0) % 360.0) / 45.0; // 0-8 sectors

    // Base speed intensity color
    let base_speed_color = wind_speed_to_ansi_color(speed);

    // Modulate base color by direction for visual distinction
    match angle_deg as i32 {
        0 | 8 => base_speed_color, // East: keep base color
        1 => match base_speed_color {
            // Northeast: shift toward yellow/bright
            AnsiColor::Blue => AnsiColor::Cyan,
            AnsiColor::Green => AnsiColor::BrightGreen,
            AnsiColor::Yellow => AnsiColor::BrightYellow,
            AnsiColor::Red => AnsiColor::BrightRed,
            c => c,
        },
        2 => match base_speed_color {
            // North: shift toward cool colors
            AnsiColor::Green => AnsiColor::Cyan,
            AnsiColor::Yellow => AnsiColor::Green,
            AnsiColor::Red => AnsiColor::Yellow,
            c => c,
        },
        3 => match base_speed_color {
            // Northwest: shift toward blue/cyan
            AnsiColor::Green => AnsiColor::Blue,
            AnsiColor::Yellow => AnsiColor::Cyan,
            AnsiColor::Red => AnsiColor::Green,
            c => c,
        },
        4 => match base_speed_color {
            // West: shift toward magenta/purple
            AnsiColor::Blue => AnsiColor::Magenta,
            AnsiColor::Green => AnsiColor::Blue,
            AnsiColor::Yellow => AnsiColor::Magenta,
            AnsiColor::Red => AnsiColor::BrightMagenta,
            c => c,
        },
        5 => match base_speed_color {
            // Southwest: shift warm
            AnsiColor::Blue => AnsiColor::Red,
            AnsiColor::Green => AnsiColor::Yellow,
            AnsiColor::Yellow => AnsiColor::Red,
            c => c,
        },
        6 => match base_speed_color {
            // South: enhance intensity
            AnsiColor::Blue => AnsiColor::BrightBlue,
            AnsiColor::Green => AnsiColor::BrightGreen,
            AnsiColor::Yellow => AnsiColor::BrightYellow,
            AnsiColor::Red => AnsiColor::BrightRed,
            c => c,
        },
        7 => match base_speed_color {
            // Southeast: shift toward yellow/orange
            AnsiColor::Blue => AnsiColor::Green,
            AnsiColor::Green => AnsiColor::Yellow,
            AnsiColor::Red => AnsiColor::BrightYellow,
            c => c,
        },
        _ => base_speed_color, // Default
    }
}

/// Format a character with color for terminal output
pub fn colorize_char(ch: char, color: AnsiColor) -> String {
    format!("{}{}{}", color.fg(), ch, ANSI_RESET)
}

/// Format a character with background color for terminal output
pub fn colorize_char_bg(ch: char, color: AnsiColor) -> String {
    format!("{}{}{}", color.bg(), ch, ANSI_RESET)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevation_colors() {
        assert_eq!(elevation_to_ansi_color(0.1), AnsiColor::Blue); // Water
        assert_eq!(elevation_to_ansi_color(0.3), AnsiColor::Cyan); // Coast
        assert_eq!(elevation_to_ansi_color(0.5), AnsiColor::Green); // Plains
        assert_eq!(elevation_to_ansi_color(0.7), AnsiColor::Yellow); // Hills
        assert_eq!(elevation_to_ansi_color(0.9), AnsiColor::Red); // Mountains
    }

    #[test]
    fn test_pressure_colors() {
        let min_p = 1000.0;
        let max_p = 1020.0;

        assert_eq!(
            pressure_to_ansi_color(1002.0, min_p, max_p),
            AnsiColor::BrightBlue
        );
        assert_eq!(
            pressure_to_ansi_color(1010.0, min_p, max_p),
            AnsiColor::White
        );
        assert_eq!(pressure_to_ansi_color(1018.0, min_p, max_p), AnsiColor::Red);
    }

    #[test]
    fn test_biome_colors() {
        assert_eq!(biome_to_ansi_color(BiomeType::Ocean), AnsiColor::Blue);
        assert_eq!(
            biome_to_ansi_color(BiomeType::Desert),
            AnsiColor::BrightYellow
        );
        assert_eq!(
            biome_to_ansi_color(BiomeType::TemperateForest),
            AnsiColor::BrightGreen
        );
    }

    #[test]
    fn test_colorize_functions() {
        let colored = colorize_char('A', AnsiColor::Red);
        assert!(colored.contains("\x1b[31m")); // Red foreground
        assert!(colored.contains("\x1b[0m")); // Reset

        let bg_colored = colorize_char_bg('B', AnsiColor::Blue);
        assert!(bg_colored.contains("\x1b[44m")); // Blue background
    }
}
