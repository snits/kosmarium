// sim-prototype/src/sim.rs

pub struct Simulation {
    pub heightmap: Vec<Vec<f32>>,
    // TODO: add agents, biome map, etc.
}

impl Simulation {
    pub fn new(heightmap: Vec<Vec<f32>>) -> Self {
        Self { heightmap }
    }
}
