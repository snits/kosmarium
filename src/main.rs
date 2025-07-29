// sim-prototype/src/main.rs

mod worldgen;
mod sim;
mod render;

use worldgen::DiamondSquareGenerator;
use sim::Simulation;
use render::ascii_render;

fn main() {
    // Step 1: Generate the map
    let generator = DiamondSquareGenerator::new(12345);
    let heightmap = generator.generate(60, 30); // width x height

    // Step 2: Run simulation setup (placeholder for now)
    let sim = Simulation::new(heightmap);

    // Step 3: Render the current world state
    ascii_render(&sim);
}

