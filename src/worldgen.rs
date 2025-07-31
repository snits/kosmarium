// sim-prototype/src/worldgen.rs

use crate::geological_evolution::{GeologicalEvolution, GeologicalEvolutionConfig};
use crate::tectonics::TectonicSystem;
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

/// Configuration for tectonic-based terrain generation with layered detail
#[derive(Clone, Debug)]
pub struct TectonicConfig {
    pub num_plates: usize,          // Number of tectonic plates (5-15 typical)
    pub surface_detail: f32,        // Amount of surface detail to add (0.0-1.0)
    pub mountain_scale: f32,        // Scale factor for mountain heights
    pub ocean_depth_scale: f32,     // Scale factor for ocean depths
    pub continental_roughness: f32, // Diamond-Square roughness for continental areas
    pub oceanic_roughness: f32,     // Diamond-Square roughness for oceanic areas
    pub detail_persistence: f32,    // How detail amplitude decreases with scale
    pub tectonic_influence: f32,    // How much tectonic base affects final terrain (0.0-1.0)
    pub coastal_blending: f32,      // Distance over which to blend continental/oceanic detail

    // Geological evolution settings
    pub enable_geological_evolution: bool, // Whether to run geological time scale evolution
    pub geological_evolution_config: Option<GeologicalEvolutionConfig>,
}

impl Default for TectonicConfig {
    fn default() -> Self {
        Self {
            num_plates: 8,
            surface_detail: 0.6, // Increased for more realistic detail
            mountain_scale: 1.0,
            ocean_depth_scale: 1.0,
            continental_roughness: 0.7, // Higher roughness for varied continental terrain
            oceanic_roughness: 0.3,     // Lower roughness for smoother ocean floors
            detail_persistence: 0.5,    // Standard fractal persistence
            tectonic_influence: 0.7,    // Strong tectonic foundation with fractal detail
            coastal_blending: 15.0,     // Blend detail types over 15 pixels

            // Geological evolution defaults
            enable_geological_evolution: true, // Enable by default for realistic terrain
            geological_evolution_config: Some(GeologicalEvolutionConfig::default()),
        }
    }
}

/// Tectonic plate-based terrain generator using Voronoi diagrams
pub struct TectonicGenerator {
    seed: u64,
}

impl TectonicGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }
}

impl TerrainGenerator for TectonicGenerator {
    type Config = TectonicConfig;

    fn generate(&self, width: usize, height: usize, config: &Self::Config) -> Vec<Vec<f32>> {
        // Create tectonic system
        let tectonic_system = TectonicSystem::new(width, height, config.num_plates, self.seed);

        // Generate base elevation from tectonics
        let mut tectonic_base = vec![vec![0.0; width]; height];
        let mut plate_type_map = vec![vec![false; width]; height]; // true = continental, false = oceanic

        for y in 0..height {
            for x in 0..width {
                let base_elevation = tectonic_system.get_elevation_at(x, y);
                tectonic_base[y][x] = base_elevation * config.mountain_scale;

                // Track plate type for terrain-aware detail generation
                if let Some(plate) = tectonic_system.get_plate_at(x, y) {
                    plate_type_map[y][x] =
                        matches!(plate.plate_type, crate::tectonics::PlateType::Continental);
                }
            }
        }

        // Apply geological evolution if enabled
        let evolved_base = if config.enable_geological_evolution {
            if let Some(ref evo_config) = config.geological_evolution_config {
                let geological_evolution = GeologicalEvolution::new(
                    evo_config.clone(),
                    self.seed + 100, // Offset seed for geological processes
                );

                let evolution_results = geological_evolution
                    .evolve_terrain(tectonic_base.clone(), Some(&tectonic_system));

                if evo_config.verbose_logging {
                    println!("Geological evolution completed:");
                    println!("  Iterations: {}", evolution_results.stats.total_iterations);
                    println!(
                        "  Average elevation change: {:.4}",
                        evolution_results.stats.average_elevation_change
                    );
                    println!(
                        "  River network length: {:.1}",
                        evolution_results.stats.river_network_length
                    );
                }

                evolution_results.evolved_heightmap
            } else {
                tectonic_base.clone()
            }
        } else {
            tectonic_base.clone()
        };

        // Generate terrain-aware fractal detail if requested
        let heightmap = if config.surface_detail > 0.0 {
            self.generate_layered_detail(&evolved_base, &plate_type_map, width, height, config)
        } else {
            evolved_base
        };

        // Apply final normalization
        let mut final_heightmap = heightmap;
        self.normalize_map(&mut final_heightmap);

        final_heightmap
    }

    fn name(&self) -> &'static str {
        "Layered Tectonic"
    }

    fn supports_arbitrary_dimensions(&self) -> bool {
        true
    }
}

impl TectonicGenerator {
    /// Generate layered terrain detail combining tectonic foundation with terrain-aware fractal noise
    fn generate_layered_detail(
        &self,
        tectonic_base: &[Vec<f32>],
        plate_type_map: &[Vec<bool>],
        width: usize,
        height: usize,
        config: &TectonicConfig,
    ) -> Vec<Vec<f32>> {
        // Generate continental detail (high roughness for varied terrain)
        let continental_generator = DiamondSquareGenerator::new(self.seed + 1);
        let continental_config = DiamondSquareConfig {
            initial_corners: [0.0, 0.0, 0.0, 0.0],
            roughness: config.continental_roughness,
            persistence: config.detail_persistence,
            wrap_edges: false,
        };
        let continental_detail = continental_generator.generate(width, height, &continental_config);

        // Generate oceanic detail (lower roughness for smoother ocean floors)
        let oceanic_generator = DiamondSquareGenerator::new(self.seed + 2);
        let oceanic_config = DiamondSquareConfig {
            initial_corners: [0.0, 0.0, 0.0, 0.0],
            roughness: config.oceanic_roughness,
            persistence: config.detail_persistence,
            wrap_edges: false,
        };
        let oceanic_detail = oceanic_generator.generate(width, height, &oceanic_config);

        // Create distance field for coastal blending
        let coastal_distance_field =
            self.calculate_coastal_distance_field(plate_type_map, width, height);

        // Combine tectonic base with terrain-aware detail
        let mut layered_heightmap = vec![vec![0.0; width]; height];

        for y in 0..height {
            for x in 0..width {
                let tectonic_elevation = tectonic_base[y][x];
                let is_continental = plate_type_map[y][x];
                let coastal_distance = coastal_distance_field[y][x];

                // Select appropriate detail based on terrain type and coastal proximity
                let detail_value = self.blend_terrain_detail(
                    continental_detail[y][x],
                    oceanic_detail[y][x],
                    is_continental,
                    coastal_distance,
                    config.coastal_blending,
                );

                // Scale detail based on tectonic elevation (more detail on higher terrain)
                let elevation_factor = self.calculate_elevation_detail_factor(tectonic_elevation);
                let scaled_detail = (detail_value - 0.5) * config.surface_detail * elevation_factor;

                // Combine tectonic foundation with fractal detail (additive approach)
                let combined_elevation = tectonic_elevation + scaled_detail;

                // Final safety check: ensure result is finite and reasonable for OpenGL
                layered_heightmap[y][x] = if combined_elevation.is_finite() {
                    combined_elevation.clamp(-10.0, 10.0) // Reasonable elevation bounds
                } else {
                    0.0 // Safe fallback
                };
            }
        }

        layered_heightmap
    }

    /// Calculate distance field from continental/oceanic boundaries for coastal blending
    fn calculate_coastal_distance_field(
        &self,
        plate_type_map: &[Vec<bool>],
        width: usize,
        height: usize,
    ) -> Vec<Vec<f32>> {
        let mut distance_field = vec![vec![f32::INFINITY; width]; height];

        // Find all boundary pixels (where continental meets oceanic)
        let mut boundary_pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let current_type = plate_type_map[y][x];

                // Check if this pixel is near a boundary
                let mut is_boundary = false;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;

                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            let neighbor_type = plate_type_map[ny as usize][nx as usize];
                            if neighbor_type != current_type {
                                is_boundary = true;
                                break;
                            }
                        }
                    }
                    if is_boundary {
                        break;
                    }
                }

                if is_boundary {
                    boundary_pixels.push((x, y));
                    distance_field[y][x] = 0.0;
                }
            }
        }

        // Simple distance propagation (could be optimized with proper distance transform)
        for _ in 0..50 {
            // Iterate to propagate distances
            let mut changed = false;
            for y in 0..height {
                for x in 0..width {
                    let current_dist = distance_field[y][x];

                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }

                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;

                            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                                let neighbor_dist = distance_field[ny as usize][nx as usize];

                                // Defensive programming: ensure neighbor distance is finite
                                if neighbor_dist.is_finite() {
                                    let step_distance = ((dx * dx + dy * dy) as f32).sqrt();
                                    let new_dist = neighbor_dist + step_distance;

                                    // Ensure new distance is finite and reasonable
                                    if new_dist.is_finite() && new_dist < current_dist {
                                        distance_field[y][x] = new_dist;
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }

        // Final safety pass: replace any remaining infinity values with large but finite distances
        let max_reasonable_distance = (width.max(height) as f32) * 2.0; // Diagonal of map * 2
        for row in distance_field.iter_mut() {
            for distance in row.iter_mut() {
                if !distance.is_finite() || *distance > max_reasonable_distance {
                    *distance = max_reasonable_distance;
                }
            }
        }

        distance_field
    }

    /// Blend continental and oceanic detail based on terrain type and coastal distance
    fn blend_terrain_detail(
        &self,
        continental_detail: f32,
        oceanic_detail: f32,
        is_continental: bool,
        coastal_distance: f32,
        blending_distance: f32,
    ) -> f32 {
        // Defensive programming: validate all inputs for OpenGL safety
        let safe_continental = if continental_detail.is_finite() {
            continental_detail
        } else {
            0.5
        };
        let safe_oceanic = if oceanic_detail.is_finite() {
            oceanic_detail
        } else {
            0.5
        };
        let safe_distance = if coastal_distance.is_finite() && coastal_distance >= 0.0 {
            coastal_distance
        } else {
            0.0
        };

        // Handle zero or invalid blending distance to prevent division by zero
        if blending_distance <= 0.0 || !blending_distance.is_finite() {
            // No blending - return appropriate detail type
            return if is_continental {
                safe_continental
            } else {
                safe_oceanic
            };
        }

        if safe_distance >= blending_distance {
            // Far from coast - use pure terrain type detail
            if is_continental {
                safe_continental
            } else {
                safe_oceanic
            }
        } else {
            // Near coast - blend the two detail types
            let blend_factor = safe_distance / blending_distance;

            // Ensure blend_factor is in valid range [0, 1]
            let safe_blend_factor = blend_factor.clamp(0.0, 1.0);

            let result = if is_continental {
                // Continental side - blend from continental to oceanic
                safe_continental * safe_blend_factor + safe_oceanic * (1.0 - safe_blend_factor)
            } else {
                // Oceanic side - blend from oceanic to continental
                safe_oceanic * safe_blend_factor + safe_continental * (1.0 - safe_blend_factor)
            };

            // Final safety check: ensure result is finite
            if result.is_finite() { result } else { 0.5 }
        }
    }

    /// Calculate how much detail to apply based on tectonic elevation
    fn calculate_elevation_detail_factor(&self, tectonic_elevation: f32) -> f32 {
        // Defensive programming: validate input for OpenGL safety
        let safe_elevation = if tectonic_elevation.is_finite() {
            tectonic_elevation.clamp(-10.0, 10.0) // Reasonable elevation bounds
        } else {
            0.0
        };

        // More detail on higher elevations (mountains get more rugged)
        // Less detail in deep ocean (smoother abyssal plains)
        let result = if safe_elevation > 0.0 {
            // Continental areas - more detail at higher elevations
            1.0 + safe_elevation * 0.5
        } else {
            // Oceanic areas - less detail at greater depths
            (1.0 + safe_elevation * 0.3).max(0.3)
        };

        // Final safety check: ensure result is finite and positive
        if result.is_finite() && result > 0.0 {
            result.clamp(0.1, 5.0) // Reasonable detail factor bounds
        } else {
            1.0 // Safe default
        }
    }

    fn normalize_map(&self, heightmap: &mut Vec<Vec<f32>>) {
        let mut min_val = f32::INFINITY;
        let mut max_val = f32::NEG_INFINITY;

        // Find min/max
        for row in heightmap.iter() {
            for &val in row.iter() {
                min_val = min_val.min(val);
                max_val = max_val.max(val);
            }
        }

        println!(
            "Layered terrain - Min: {:.3}, Max: {:.3}, Range: {:.3}",
            min_val,
            max_val,
            max_val - min_val
        );

        // Normalize to -0.5 to 1.0 range (more land than water)
        let range = max_val - min_val;
        if range > 0.0 {
            for row in heightmap.iter_mut() {
                for val in row.iter_mut() {
                    *val = (*val - min_val) / range * 1.5 - 0.5;
                }
            }
        }
    }
}
