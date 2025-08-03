// sim-prototype/src/render.rs

use super::super::agents::biome::BiomeMap;
use crate::engine::Simulation;
use crossterm::{
    execute,
    style::{Color::Rgb, PrintStyledContent, Stylize},
};
use std::io::{Write, stdout};

pub fn ascii_render(sim: &Simulation) {
    ascii_render_elevation(sim);
}

/// Original elevation-based rendering (kept for compatibility)
pub fn ascii_render_elevation(sim: &Simulation) {
    let mut stdout = stdout();

    for y in 0..sim.heightmap.height() {
        for x in 0..sim.heightmap.width() {
            let val = sim.heightmap.get(x, y);
            let symbol = match val {
                x if x < 0.2 => '.'.blue(),   // Deep water
                x if x < 0.4 => '~'.cyan(),   // Shallow water/coastline
                x if x < 0.6 => '-'.green(),  // Plains/flatlands
                x if x < 0.8 => '^'.yellow(), // Hills/foothills
                _ => '▲'.red(),               // Mountains/peaks
            };
            let _ = execute!(stdout, PrintStyledContent(symbol));
        }
        let _ = writeln!(stdout);
    }
}

/// Biome-based rendering using Whittaker classification
pub fn ascii_render_biomes(biome_map: &BiomeMap) {
    let mut stdout = stdout();

    for y in 0..biome_map.height() {
        for x in 0..biome_map.width() {
            let biome = biome_map.get(x, y);
            let char = biome.display_char();
            let (r, g, b) = biome.display_color();

            let symbol = char.with(Rgb { r, g, b });
            let _ = execute!(stdout, PrintStyledContent(symbol));
        }
        let _ = writeln!(stdout);
    }
}
