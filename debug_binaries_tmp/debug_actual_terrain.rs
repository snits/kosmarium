// ABOUTME: Debug script to examine actual terrain generation output
// ABOUTME: Shows what elevation values are generated and where they're located

use std::path::Path;

// Copy the necessary code from the main project
mod worldgen {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    pub trait TerrainGenerator {
        type Config: Clone + Default;
        fn generate(&self, width: usize, height: usize, config: &Self::Config) -> Vec<Vec<f32>>;
        fn name(&self) -> &'static str;
        fn supports_arbitrary_dimensions(&self) -> bool;
    }

    #[derive(Clone, Debug)]
    pub struct DiamondSquareConfig {
        pub initial_corners: [f32; 4],
        pub roughness: f32,
        pub persistence: f32,
        pub wrap_edges: bool,
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

    pub struct DiamondSquareGenerator {
        seed: u64,
    }

    impl DiamondSquareGenerator {
        pub fn new(seed: u64) -> Self {
            Self { seed }
        }

        fn generate_power_of_two(&self, size: usize, config: &DiamondSquareConfig) -> Vec<Vec<f32>> {
            let mut rng = StdRng::seed_from_u64(self.seed);
            let mut map = vec![vec![0.0; size]; size];
            let max_index = size - 1;

            map[0][0] = config.initial_corners[0];
            map[0][max_index] = config.initial_corners[1];
            map[max_index][0] = config.initial_corners[2];
            map[max_index][max_index] = config.initial_corners[3];

            let mut step_size = size - 1;
            let mut scale = config.roughness;

            while step_size > 1 {
                let half_step = step_size / 2;

                for y in (half_step..size).step_by(step_size) {
                    for x in (half_step..size).step_by(step_size) {
                        let avg = self.diamond_average(&map, x, y, half_step, config.wrap_edges, size);
                        let noise = rng.gen_range(-scale..scale);
                        map[y][x] = avg + noise;
                    }
                }

                for y in (0..size).step_by(half_step) {
                    let offset = if (y / half_step) % 2 == 0 { half_step } else { 0 };
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

        fn diamond_average(&self, map: &[Vec<f32>], x: usize, y: usize, half_step: usize, _wrap: bool, _size: usize) -> f32 {
            let tl = map[y - half_step][x - half_step];
            let tr = map[y - half_step][x + half_step];
            let bl = map[y + half_step][x - half_step];
            let br = map[y + half_step][x + half_step];
            (tl + tr + bl + br) / 4.0
        }

        fn square_average(&self, map: &[Vec<f32>], x: usize, y: usize, half_step: usize, _wrap: bool, size: usize) -> f32 {
            let mut sum = 0.0;
            let mut count = 0;

            let coords = [
                (x, y.wrapping_sub(half_step)),
                (x.wrapping_sub(half_step), y),
                (x + half_step, y),
                (x, y + half_step),
            ];

            for (cx, cy) in coords {
                if cx < size && cy < size {
                    sum += map[cy][cx];
                    count += 1;
                }
            }

            if count > 0 { sum / count as f32 } else { 0.0 }
        }
    }

    impl TerrainGenerator for DiamondSquareGenerator {
        type Config = DiamondSquareConfig;

        fn generate(&self, width: usize, height: usize, config: &Self::Config) -> Vec<Vec<f32>> {
            let size = width.max(height).next_power_of_two();
            let full_map = self.generate_power_of_two(size, config);
            
            let mut result = vec![vec![0.0; width]; height];
            for y in 0..height {
                for x in 0..width {
                    result[y][x] = full_map[y][x];
                }
            }
            result
        }

        fn name(&self) -> &'static str { "Diamond-Square" }
        fn supports_arbitrary_dimensions(&self) -> bool { true }
    }
}

fn main() {
    use worldgen::*;
    
    println!("=== ACTUAL TERRAIN GENERATION DEBUG ===");
    
    // Generate a small terrain to examine
    let generator = DiamondSquareGenerator::new(12345);
    let config = DiamondSquareConfig::default();
    let terrain = generator.generate(8, 8, &config);
    
    println!("Generated 8x8 terrain:");
    for (y, row) in terrain.iter().enumerate() {
        print!("y={}: ", y);
        for (x, &elevation) in row.iter().enumerate() {
            print!("{:5.2} ", elevation);
        }
        println!();
    }
    println!();
    
    // Analyze elevation distribution by row
    println!("Row analysis:");
    for (y, row) in terrain.iter().enumerate() {
        let avg_elevation: f32 = row.iter().sum::<f32>() / row.len() as f32;
        let min_elevation = row.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_elevation = row.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        println!("Row {}: avg={:.3}, min={:.3}, max={:.3}", y, avg_elevation, min_elevation, max_elevation);
    }
    println!();
    
    // Count water cells (< 0.2) by row
    println!("Water distribution (elevation < 0.2):");
    for (y, row) in terrain.iter().enumerate() {
        let water_count = row.iter().filter(|&&e| e < 0.2).count();
        println!("Row {}: {} water cells out of {}", y, water_count, row.len());
    }
}