// sim-prototype/src/render.rs

use crate::sim::Simulation;
use crossterm::{
    execute,
    style::{PrintStyledContent, Stylize},
};
use std::io::{Write, stdout};

pub fn ascii_render(sim: &Simulation) {
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
