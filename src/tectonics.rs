// ABOUTME: Tectonic plate simulation using Voronoi diagrams for realistic geological processes
// ABOUTME: Handles plate movement, boundary interactions, and elevation generation from tectonics

use rand::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlateType {
    Continental,
    Oceanic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryType {
    Divergent,  // Plates moving apart - ridges, rifts
    Convergent, // Plates moving together - mountains, trenches
    Transform,  // Plates sliding past - faults
}

#[derive(Debug, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn dot(&self, other: &Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn normalize(&self) -> Vec2 {
        let mag = self.magnitude();
        if mag > 0.0 {
            Vec2::new(self.x / mag, self.y / mag)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct TectonicPlate {
    pub id: usize,
    pub center: Vec2,
    pub plate_type: PlateType,
    pub velocity: Vec2,        // Movement direction and speed (cm/year scaled)
    pub age: f32,             // Age in millions of years
    pub density: f32,         // Relative density (affects subduction)
    pub base_elevation: f32,  // Base elevation for this plate
    pub crustal_thickness: f32, // Crustal thickness (affects isostatic elevation)
}

impl TectonicPlate {
    pub fn new(id: usize, center: Vec2, plate_type: PlateType, rng: &mut StdRng) -> Self {
        let (density, base_elevation, crustal_thickness) = match plate_type {
            PlateType::Continental => {
                // Continental plates have variable thickness (30-50km)
                let thickness = rng.gen_range(30.0..50.0);
                (2.7, 0.6, thickness)
            },
            PlateType::Oceanic => {
                // Oceanic plates have thinner, more uniform crust (5-10km)
                let thickness = rng.gen_range(5.0..10.0);
                (3.0, -0.5, thickness)
            },
        };

        // Random velocity vector (realistic plate speeds: 1-10 cm/year)
        let speed = rng.gen_range(0.01..0.05); // Scaled for simulation
        let direction = rng.gen_range(0.0..std::f32::consts::TAU);
        let velocity = Vec2::new(
            speed * direction.cos(),
            speed * direction.sin(),
        );

        Self {
            id,
            center,
            plate_type,
            velocity,
            age: rng.gen_range(10.0..200.0), // 10-200 million years
            density,
            base_elevation,
            crustal_thickness,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoronoiCell {
    pub plate_id: usize,
    pub distance: f32,
}

pub struct TectonicSystem {
    pub plates: Vec<TectonicPlate>,
    pub width: usize,
    pub height: usize,
    voronoi_map: Vec<Vec<VoronoiCell>>,
}

impl TectonicSystem {
    pub fn new(width: usize, height: usize, num_plates: usize, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut plates = Vec::new();

        // Generate plate centers using Poisson disk sampling for better distribution
        let plate_centers = Self::generate_plate_centers(width, height, num_plates, &mut rng);

        // Create plates with improved continental distribution
        // First, determine which plates will be continental (cluster them for realistic landmasses)
        let num_continental = (num_plates as f32 * 0.35).round() as usize; // Slightly more continental plates
        let mut continental_indices = Vec::new();
        
        // Pick a few "continental cores" and cluster continents around them
        let num_cores = (num_continental as f32 * 0.6).max(1.0) as usize;
        for _ in 0..num_cores {
            continental_indices.push(rng.gen_range(0..num_plates));
        }
        
        // Add nearby plates to continental clusters
        while continental_indices.len() < num_continental {
            let core_idx = continental_indices[rng.gen_range(0..continental_indices.len())];
            let core_center = &plate_centers[core_idx];
            
            // Find closest non-continental plate to this core
            let mut closest_idx = 0;
            let mut closest_dist = f32::INFINITY;
            
            for (i, center) in plate_centers.iter().enumerate() {
                if !continental_indices.contains(&i) {
                    let dx = center.x - core_center.x;
                    let dy = center.y - core_center.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < closest_dist {
                        closest_dist = dist;
                        closest_idx = i;
                    }
                }
            }
            
            if !continental_indices.contains(&closest_idx) {
                continental_indices.push(closest_idx);
            } else {
                // Fallback: pick random plate
                let mut attempts = 0;
                while attempts < 20 {
                    let idx = rng.gen_range(0..num_plates);
                    if !continental_indices.contains(&idx) {
                        continental_indices.push(idx);
                        break;
                    }
                    attempts += 1;
                }
                if attempts >= 20 { break; } // Avoid infinite loop
            }
        }
        
        // Create plates with determined types
        for (i, center) in plate_centers.into_iter().enumerate() {
            let plate_type = if continental_indices.contains(&i) {
                PlateType::Continental
            } else {
                PlateType::Oceanic
            };
            
            plates.push(TectonicPlate::new(i, center, plate_type, &mut rng));
        }

        let mut system = Self {
            plates,
            width,
            height,
            voronoi_map: vec![vec![VoronoiCell { plate_id: 0, distance: f32::INFINITY }; width]; height],
        };

        // Generate Voronoi diagram
        system.generate_voronoi_diagram();
        
        system
    }

    fn generate_plate_centers(width: usize, height: usize, num_plates: usize, rng: &mut StdRng) -> Vec<Vec2> {
        let mut centers = Vec::new();
        let min_distance = ((width * height) as f32 / num_plates as f32).sqrt() * 0.5;

        // Simple random placement with minimum distance constraint
        for _ in 0..num_plates {
            let mut attempts = 0;
            loop {
                let x = rng.gen_range(0.0..width as f32);
                let y = rng.gen_range(0.0..height as f32);
                let candidate = Vec2::new(x, y);

                // Check minimum distance to existing centers
                let too_close = centers.iter().any(|center: &Vec2| {
                    let dx = candidate.x - center.x;
                    let dy = candidate.y - center.y;
                    (dx * dx + dy * dy).sqrt() < min_distance
                });

                if !too_close || attempts > 50 {
                    centers.push(candidate);
                    break;
                }
                attempts += 1;
            }
        }

        centers
    }

    fn generate_voronoi_diagram(&mut self) {
        // For each cell in the grid, find the closest plate
        for y in 0..self.height {
            for x in 0..self.width {
                let point = Vec2::new(x as f32, y as f32);
                let mut closest_plate = 0;
                let mut closest_distance = f32::INFINITY;

                for (i, plate) in self.plates.iter().enumerate() {
                    let dx = point.x - plate.center.x;
                    let dy = point.y - plate.center.y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_plate = i;
                    }
                }

                self.voronoi_map[y][x] = VoronoiCell {
                    plate_id: closest_plate,
                    distance: closest_distance,
                };
            }
        }
    }

    pub fn get_plate_at(&self, x: usize, y: usize) -> Option<&TectonicPlate> {
        if x < self.width && y < self.height {
            let plate_id = self.voronoi_map[y][x].plate_id;
            self.plates.get(plate_id)
        } else {
            None
        }
    }

    pub fn get_elevation_at(&self, x: usize, y: usize) -> f32 {
        if let Some(plate) = self.get_plate_at(x, y) {
            // Start with base plate elevation
            let mut elevation = plate.base_elevation;

            // Add isostatic adjustment based on crustal thickness
            // Thicker crust "floats" higher due to isostatic equilibrium
            let isostatic_adjustment = (plate.crustal_thickness - 20.0) * 0.02; // 20km reference thickness
            elevation += isostatic_adjustment;

            // Add age-related subsidence for oceanic plates
            if plate.plate_type == PlateType::Oceanic {
                // Older oceanic plates cool and subside (thermal subsidence)
                let age_subsidence = -(plate.age * 0.001); // Older = deeper
                elevation += age_subsidence;
            }

            // Add boundary effects
            let boundary_effect = self.calculate_boundary_effect(x, y);
            elevation += boundary_effect;

            // Safety check: ensure elevation is finite and reasonable
            if elevation.is_finite() {
                elevation.clamp(-2.0, 2.0)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    fn calculate_boundary_effect(&self, x: usize, y: usize) -> f32 {
        let current_plate_id = self.voronoi_map[y][x].plate_id;
        let current_plate = &self.plates[current_plate_id];
        
        // Distance to nearest plate boundary (from Voronoi distance)
        let boundary_distance = self.voronoi_map[y][x].distance;
        
        // Safety check: handle infinity distances
        if !boundary_distance.is_finite() || boundary_distance > 1000.0 {
            return 0.0;
        }
        
        // Find the nearest different plate to determine boundary type
        let mut nearest_different_plate_id = current_plate_id;
        let mut min_distance_to_different = f32::INFINITY;
        
        // Search in expanding radius for different plate
        let search_radius = 10; // Larger search radius for more realistic effects
        for dy in -(search_radius as i32)..=(search_radius as i32) {
            for dx in -(search_radius as i32)..=(search_radius as i32) {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    let neighbor_plate_id = self.voronoi_map[ny as usize][nx as usize].plate_id;
                    
                    if neighbor_plate_id != current_plate_id {
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        if distance < min_distance_to_different {
                            min_distance_to_different = distance;
                            nearest_different_plate_id = neighbor_plate_id;
                        }
                    }
                }
            }
        }
        
        if nearest_different_plate_id == current_plate_id {
            return 0.0; // No nearby different plate found
        }
        
        let neighbor_plate = &self.plates[nearest_different_plate_id];
        
        // Calculate boundary interaction with improved distance falloff
        self.calculate_plate_interaction_elevation(current_plate, neighbor_plate, boundary_distance)
    }

    fn calculate_plate_interaction_elevation(&self, plate1: &TectonicPlate, plate2: &TectonicPlate, distance: f32) -> f32 {
        // Calculate relative velocity between plates
        let relative_velocity = Vec2::new(
            plate1.velocity.x - plate2.velocity.x,
            plate1.velocity.y - plate2.velocity.y,
        );

        // Determine boundary type based on relative motion
        let speed = relative_velocity.magnitude();
        let boundary_type = self.determine_boundary_type(plate1, plate2, &relative_velocity);

        // Improved distance falloff - effects extend much further from boundaries
        let max_effect_distance = 20.0; // Effects extend 20 pixels from boundary
        let distance_factor = if distance < max_effect_distance {
            // Exponential falloff for more realistic mountain building
            (-distance / (max_effect_distance * 0.3)).exp()
        } else {
            0.0
        };

        // Convergence strength based on relative velocity and crustal properties
        let convergence_strength = speed * 100.0; // Amplify the effect significantly

        match boundary_type {
            BoundaryType::Convergent => {
                // Mountain building - elevation depends on crust types and convergence rate
                let mountain_height = match (plate1.plate_type, plate2.plate_type) {
                    (PlateType::Continental, PlateType::Continental) => {
                        // Continental collision: Himalayas-style - highest mountains
                        1.5 + (plate1.crustal_thickness + plate2.crustal_thickness) * 0.02
                    },
                    (PlateType::Continental, PlateType::Oceanic) | (PlateType::Oceanic, PlateType::Continental) => {
                        // Subduction zone: Andes-style - high coastal mountains
                        1.0 + plate1.crustal_thickness.max(plate2.crustal_thickness) * 0.015
                    },
                    (PlateType::Oceanic, PlateType::Oceanic) => {
                        // Ocean-ocean convergence: Island arcs
                        0.6 + (plate1.age + plate2.age) * 0.002 // Older plates create higher islands
                    },
                };
                mountain_height * distance_factor * convergence_strength
            },
            BoundaryType::Divergent => {
                // Rift valleys and mid-ocean ridges
                let rift_depth = match (plate1.plate_type, plate2.plate_type) {
                    (PlateType::Continental, PlateType::Continental) => -0.3, // Continental rifts
                    _ => -0.1, // Mid-ocean ridges (less deep due to volcanic activity)
                };
                rift_depth * distance_factor * convergence_strength
            },
            BoundaryType::Transform => {
                // Transform faults - create linear valleys and ridges
                let fault_effect = 0.2 * (1.0 - 2.0 * (distance % 2.0)); // Alternating ridges/valleys
                fault_effect * distance_factor * convergence_strength * 0.5
            },
        }
    }

    fn determine_boundary_type(&self, plate1: &TectonicPlate, plate2: &TectonicPlate, relative_velocity: &Vec2) -> BoundaryType {
        // Calculate direction from plate1 center to plate2 center
        let direction = Vec2::new(
            plate2.center.x - plate1.center.x,
            plate2.center.y - plate1.center.y,
        ).normalize();

        // Dot product tells us if plates are moving toward/away from each other
        let dot_product = relative_velocity.dot(&direction);

        if dot_product > 0.01 {
            BoundaryType::Convergent // Moving toward each other
        } else if dot_product < -0.01 {
            BoundaryType::Divergent // Moving away from each other
        } else {
            BoundaryType::Transform // Moving parallel (sliding past)
        }
    }

    pub fn get_plate_info(&self, x: usize, y: usize) -> Option<(usize, PlateType, f32)> {
        if let Some(plate) = self.get_plate_at(x, y) {
            Some((plate.id, plate.plate_type, self.voronoi_map[y][x].distance))
        } else {
            None
        }
    }
}