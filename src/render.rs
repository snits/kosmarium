// sim-prototype/src/render.rs

use crate::sim::Simulation;
use crossterm::{style::{Stylize, PrintStyledContent}, execute};
use std::io::{stdout, Write};

pub fn ascii_render(sim: &Simulation) {
    let mut stdout = stdout();

    for row in &sim.heightmap {
        for &val in row {
            let symbol = match val {
                x if x < 0.2 => '.'.blue(),
                x if x < 0.4 => '~'.cyan(),
                x if x < 0.6 => '^'.green(),
                x if x < 0.8 => '#' .yellow(),
                _ => '@'.red(),
            };
            let _ = execute!(stdout, PrintStyledContent(symbol));
        }
        let _ = writeln!(stdout);
    }
}

