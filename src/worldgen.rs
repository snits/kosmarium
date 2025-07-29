// sim-prototype/src/worldgen.rs

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub struct DiamondSquareGenerator {
    seed: u64,
}

impl DiamondSquareGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn generate(&self, width: usize, height: usize) -> Vec<Vec<f32>> {
        // Placeholder: random noise instead of real Diamond-Square
        let mut rng = StdRng::seed_from_u64(self.seed);
        let mut map = vec![vec![0.0; width]; height];
        for row in map.iter_mut() {
            for val in row.iter_mut() {
                *val = rng.gen_range(0.0..1.0);
            }
        }
        map
    }
}
