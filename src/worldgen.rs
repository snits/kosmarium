// sim-prototype/src/worldgen.rs

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Core trait for terrain generation algorithms
pub trait TerrainGenerator {
    type Config: Clone + Default;

    fn generate(&self, width: usize, height: usize, config: &Self::Config) -> Vec<Vec<f32>>;
    fn name(&self) -> &'static str;
    fn supports_arbitrary_dimensions(&self) -> bool;
}

/// Configuration parameters for Diamond-Square generation
#[derive(Clone, Debug)]
pub struct DiamondSquareConfig {
    pub initial_corners: [f32; 4], // [top-left, top-right, bottom-left, bottom-right]
    pub roughness: f32,            // Controls terrain roughness (0.0-1.0)
    pub persistence: f32,          // How much randomness decreases each iteration
    pub wrap_edges: bool,          // Whether to treat edges as wrapping
}

impl Default for DiamondSquareConfig {
    fn default() -> Self {
        Self {
            initial_corners: [0.5, 0.5, 0.5, 0.5],
            roughness: 0.5,
            persistence: 0.5,
            wrap_edges: false,
        }
    }
}

/// Diamond-Square terrain generator
pub struct DiamondSquareGenerator {
    seed: u64,
}

impl DiamondSquareGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate terrain on a power-of-2 sized grid using Diamond-Square algorithm
    fn generate_power_of_two(&self, size: usize, config: &DiamondSquareConfig) -> Vec<Vec<f32>> {
        let mut rng = StdRng::seed_from_u64(self.seed);
        let mut map = vec![vec![0.0; size]; size];
        let max_index = size - 1;

        // Initialize corners
        map[0][0] = config.initial_corners[0];
        map[0][max_index] = config.initial_corners[1];
        map[max_index][0] = config.initial_corners[2];
        map[max_index][max_index] = config.initial_corners[3];

        let mut step_size = size - 1;
        let mut scale = config.roughness;

        while step_size > 1 {
            let half_step = step_size / 2;

            // Diamond step
            for y in (half_step..size).step_by(step_size) {
                for x in (half_step..size).step_by(step_size) {
                    let avg = self.diamond_average(&map, x, y, half_step, config.wrap_edges, size);
                    let noise = rng.gen_range(-scale..scale);
                    map[y][x] = avg + noise;
                }
            }

            // Square step
            for y in (0..size).step_by(half_step) {
                let offset = if (y / half_step) % 2 == 0 {
                    half_step
                } else {
                    0
                };
                for x in (offset..size).step_by(step_size) {
                    let avg = self.square_average(&map, x, y, half_step, config.wrap_edges, size);
                    let noise = rng.gen_range(-scale..scale);
                    map[y][x] = avg + noise;
                }
            }

            step_size /= 2;
            scale *= config.persistence;
        }

        map
    }

    fn diamond_average(
        &self,
        map: &[Vec<f32>],
        x: usize,
        y: usize,
        half_step: usize,
        wrap: bool,
        size: usize,
    ) -> f32 {
        let points = [
            (x.wrapping_sub(half_step), y.wrapping_sub(half_step)),
            (x + half_step, y.wrapping_sub(half_step)),
            (x.wrapping_sub(half_step), y + half_step),
            (x + half_step, y + half_step),
        ];

        let mut sum = 0.0;
        let mut count = 0;

        for (px, py) in points {
            if wrap || (px < size && py < size) {
                sum += map[py % size][px % size];
                count += 1;
            }
        }

        if count > 0 { sum / count as f32 } else { 0.0 }
    }

    fn square_average(
        &self,
        map: &[Vec<f32>],
        x: usize,
        y: usize,
        half_step: usize,
        wrap: bool,
        size: usize,
    ) -> f32 {
        let points = [
            (x, y.wrapping_sub(half_step)),
            (x.wrapping_sub(half_step), y),
            (x + half_step, y),
            (x, y + half_step),
        ];

        let mut sum = 0.0;
        let mut count = 0;

        for (px, py) in points {
            if wrap || (px < size && py < size) {
                sum += map[py % size][px % size];
                count += 1;
            }
        }

        if count > 0 { sum / count as f32 } else { 0.0 }
    }

    /// Sample a larger grid down to requested dimensions
    fn sample_to_dimensions(
        &self,
        full_map: Vec<Vec<f32>>,
        width: usize,
        height: usize,
    ) -> Vec<Vec<f32>> {
        let full_size = full_map.len();
        let mut result = vec![vec![0.0; width]; height];

        for y in 0..height {
            for x in 0..width {
                let src_x = (x * (full_size - 1)) / (width - 1).max(1);
                let src_y = (y * (full_size - 1)) / (height - 1).max(1);
                result[y][x] = full_map[src_y.min(full_size - 1)][src_x.min(full_size - 1)];
            }
        }

        result
    }

    /// Normalize map values to 0.0-1.0 range
    fn normalize_map(&self, map: &mut Vec<Vec<f32>>) {
        let min = map.iter().flatten().cloned().fold(f32::INFINITY, f32::min);
        let max = map
            .iter()
            .flatten()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        let mean = map.iter().flatten().cloned().sum::<f32>() / (map.len() * map[0].len()) as f32;

        // Diagnostic: Show raw elevation distribution before normalization
        println!(
            "Raw elevation - Min: {:.3}, Max: {:.3}, Mean: {:.3}, Range: {:.3}",
            min,
            max,
            mean,
            max - min
        );

        if max > min {
            for row in map.iter_mut() {
                for val in row.iter_mut() {
                    *val = (*val - min) / (max - min);
                }
            }
        }
    }
}

impl TerrainGenerator for DiamondSquareGenerator {
    type Config = DiamondSquareConfig;

    fn generate(&self, width: usize, height: usize, config: &Self::Config) -> Vec<Vec<f32>> {
        // Handle arbitrary dimensions by generating on power-of-2 grid then sampling
        let power_size = (width.max(height).next_power_of_two()).max(8);
        let full_map = self.generate_power_of_two(power_size, config);

        // Sample down to requested dimensions
        let mut result = self.sample_to_dimensions(full_map, width, height);

        // Normalize to 0.0-1.0 range
        self.normalize_map(&mut result);

        result
    }

    fn name(&self) -> &'static str {
        "Diamond-Square"
    }

    fn supports_arbitrary_dimensions(&self) -> bool {
        true // Via sampling
    }
}
