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

#[derive(Debug, Clone, Copy)]
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
    pub velocity: Vec2,         // Movement direction and speed (cm/year scaled)
    pub age: f32,               // Age in millions of years
    pub density: f32,           // Relative density (affects subduction)
    pub base_elevation: f32,    // Base elevation for this plate
    pub crustal_thickness: f32, // Crustal thickness (affects isostatic elevation)
}

impl TectonicPlate {
    pub fn new(id: usize, center: Vec2, plate_type: PlateType, rng: &mut StdRng) -> Self {
        let (density, base_elevation, crustal_thickness) = match plate_type {
            PlateType::Continental => {
                // Continental plates have variable thickness (30-50km)
                let thickness = rng.gen_range(30.0..50.0);
                (2.7, 0.6, thickness)
            }
            PlateType::Oceanic => {
                // Oceanic plates have thinner, more uniform crust (5-10km)
                let thickness = rng.gen_range(5.0..10.0);
                (3.0, -0.5, thickness)
            }
        };

        // Random velocity vector (realistic plate speeds: 1-10 cm/year)
        let speed = rng.gen_range(0.01..0.05); // Scaled for simulation
        let direction = rng.gen_range(0.0..std::f32::consts::TAU);
        let velocity = Vec2::new(speed * direction.cos(), speed * direction.sin());

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
    /// Calculate plate mass based on density, thickness, and area
    /// METIS CORRECTION: Proper mass calculation for momentum conservation
    fn calculate_plate_mass(&self, plate: &TectonicPlate) -> f32 {
        // Physical plate densities (g/cm³ → simulation units)
        let density = match plate.plate_type {
            PlateType::Continental => 2.7, // Continental crust density
            PlateType::Oceanic => 3.0,     // Oceanic crust density
        };

        // Approximate plate area (will be refined with actual Voronoi areas)
        let approx_area = (self.width * self.height) as f32 / self.plates.len() as f32;

        // Mass = density × thickness × area
        // Convert to simulation units (thickness in km, area in grid units)
        density * plate.crustal_thickness * approx_area
    }

    /// METIS CORRECTION: Apply momentum conservation during plate interactions
    /// Implements Newton's Third Law: F₁₂ = -F₂₁ and conservation: Σmᵢvᵢ = constant
    pub fn apply_momentum_conservation_to_plates(&mut self, dt: f32) {
        // Store initial momentum for verification
        let initial_momentum = self.calculate_total_momentum();

        // Apply pairwise momentum exchanges between interacting plates
        let num_plates = self.plates.len();
        for i in 0..num_plates {
            for j in (i + 1)..num_plates {
                let distance = self.calculate_plate_separation(i, j);

                // Only apply momentum exchange if plates are close enough to interact
                const INTERACTION_THRESHOLD: f32 = 50.0; // Grid units
                if distance < INTERACTION_THRESHOLD {
                    self.apply_pairwise_momentum_exchange(i, j, distance, dt);
                }
            }
        }

        // Verify momentum conservation (debug check)
        let final_momentum = self.calculate_total_momentum();
        let momentum_error = ((final_momentum.x - initial_momentum.x).abs()
            + (final_momentum.y - initial_momentum.y).abs())
            / (initial_momentum.x.abs() + initial_momentum.y.abs() + 1e-10);

        if momentum_error > 0.01 {
            eprintln!(
                "Warning: Momentum conservation violation: {:.4}%",
                momentum_error * 100.0
            );
        }
    }

    /// Calculate total system momentum for conservation verification
    pub fn calculate_total_momentum(&self) -> Vec2 {
        let mut total_momentum = Vec2::new(0.0, 0.0);

        for plate in &self.plates {
            let mass = self.calculate_plate_mass(plate);
            total_momentum.x += mass * plate.velocity.x;
            total_momentum.y += mass * plate.velocity.y;
        }

        total_momentum
    }

    /// Calculate separation distance between two plates
    fn calculate_plate_separation(&self, plate_i: usize, plate_j: usize) -> f32 {
        let plate1 = &self.plates[plate_i];
        let plate2 = &self.plates[plate_j];

        let dx = plate1.center.x - plate2.center.x;
        let dy = plate1.center.y - plate2.center.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Apply momentum exchange between two interacting plates
    fn apply_pairwise_momentum_exchange(&mut self, i: usize, j: usize, distance: f32, dt: f32) {
        // Get plate properties (need to extract to avoid borrow conflicts)
        let (mass1, velocity1, center1) = {
            let plate = &self.plates[i];
            (
                self.calculate_plate_mass(plate),
                plate.velocity,
                plate.center,
            )
        };

        let (mass2, velocity2, center2) = {
            let plate = &self.plates[j];
            (
                self.calculate_plate_mass(plate),
                plate.velocity,
                plate.center,
            )
        };

        // Calculate relative position and velocity
        let relative_pos = Vec2::new(center2.x - center1.x, center2.y - center1.y);
        let relative_velocity = Vec2::new(velocity1.x - velocity2.x, velocity1.y - velocity2.y);

        // Check if plates are approaching (dot product < 0)
        let approach_rate = relative_pos.dot(&relative_velocity) / distance;

        if approach_rate < 0.0 {
            // Plates are approaching - apply momentum exchange
            let unit_normal = relative_pos.normalize();

            // Conservation of momentum in elastic collision (simplified)
            // v₁' = v₁ - 2m₂/(m₁+m₂) * (v₁-v₂)·n̂ * n̂
            // v₂' = v₂ - 2m₁/(m₁+m₂) * (v₂-v₁)·n̂ * n̂

            let total_mass = mass1 + mass2;
            let normal_velocity = relative_velocity.dot(&unit_normal);

            // Energy dissipation factor (perfectly elastic = 1.0, perfectly inelastic = 0.0)
            let restitution = 0.3; // Geological collisions are quite inelastic

            let impulse_magnitude = 2.0 * restitution * normal_velocity / total_mass;
            let impulse = Vec2::new(
                impulse_magnitude * unit_normal.x,
                impulse_magnitude * unit_normal.y,
            );

            // Apply momentum changes (Newton's Third Law: equal and opposite)
            let plate1_velocity_change = Vec2::new(-mass2 * impulse.x, -mass2 * impulse.y);
            let plate2_velocity_change = Vec2::new(mass1 * impulse.x, mass1 * impulse.y);

            // Scale by time step and interaction strength
            let interaction_strength = (1.0 / (distance + 1.0)) * dt;

            // Update plate velocities
            self.plates[i].velocity.x += plate1_velocity_change.x * interaction_strength;
            self.plates[i].velocity.y += plate1_velocity_change.y * interaction_strength;
            self.plates[j].velocity.x += plate2_velocity_change.x * interaction_strength;
            self.plates[j].velocity.y += plate2_velocity_change.y * interaction_strength;
        }
    }

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
                if attempts >= 20 {
                    break;
                } // Avoid infinite loop
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
            voronoi_map: vec![
                vec![
                    VoronoiCell {
                        plate_id: 0,
                        distance: f32::INFINITY
                    };
                    width
                ];
                height
            ],
        };

        // Generate Voronoi diagram
        system.generate_voronoi_diagram();

        system
    }

    fn generate_plate_centers(
        width: usize,
        height: usize,
        num_plates: usize,
        rng: &mut StdRng,
    ) -> Vec<Vec2> {
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

            // METIS CORRECTION: Proper isostatic equilibrium using Archimedes' principle
            // h_elevation = h_crust × (1 - ρ_crust/ρ_mantle)

            const MANTLE_DENSITY: f32 = 3.3; // g/cm³

            let (crust_density, reference_elevation) = match plate.plate_type {
                PlateType::Continental => (2.7, 0.5), // Continental crust density + base elevation
                PlateType::Oceanic => (3.0, -0.5),    // Oceanic crust density + base elevation
            };

            // Archimedes' buoyancy: elevation proportional to density contrast
            let density_ratio = 1.0 - (crust_density / MANTLE_DENSITY);
            let isostatic_coefficient = match plate.plate_type {
                PlateType::Continental => 0.18, // (1 - 2.7/3.3) = 0.18
                PlateType::Oceanic => 0.09,     // (1 - 3.0/3.3) = 0.09
            };

            // Physics-correct isostatic adjustment
            let thickness_above_reference = (plate.crustal_thickness - 30.0).max(0.0); // 30km reference
            let isostatic_adjustment = thickness_above_reference * isostatic_coefficient;
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

    fn calculate_plate_interaction_elevation(
        &self,
        plate1: &TectonicPlate,
        plate2: &TectonicPlate,
        distance: f32,
    ) -> f32 {
        // METIS CORRECTION: Energy-conserving plate interaction calculation
        // W_total = ρ·g·h·ΔV + ∫(σ·ε)dV + Q_friction

        // Calculate relative velocity between plates (MOMENTUM CONSERVATION CORRECTION)
        let relative_velocity = Vec2::new(
            plate1.velocity.x - plate2.velocity.x,
            plate1.velocity.y - plate2.velocity.y,
        );

        // Determine boundary type based on relative motion
        let speed = relative_velocity.magnitude();
        let boundary_type = self.determine_boundary_type(plate1, plate2, &relative_velocity);

        // ENERGY CONSERVATION: Calculate available kinetic energy for geological work
        let plate1_mass = self.calculate_plate_mass(plate1);
        let plate2_mass = self.calculate_plate_mass(plate2);
        let total_mass = plate1_mass + plate2_mass;

        // Kinetic energy available for interaction (per unit area)
        let kinetic_energy_density =
            0.5 * (plate1_mass * speed * speed) / (self.width * self.height) as f32;

        // Energy efficiency for different geological processes
        let energy_efficiency = match boundary_type {
            BoundaryType::Convergent => 0.15, // 15% kinetic energy → gravitational potential energy
            BoundaryType::Divergent => 0.05,  // 5% energy → rift formation
            BoundaryType::Transform => 0.02,  // 2% energy → fault systems
        };

        // Available energy for elevation change
        let available_energy = kinetic_energy_density * energy_efficiency;

        // Physics-based distance falloff using elastic stress propagation
        let max_effect_distance = 20.0; // Effects extend 20 pixels from boundary
        let distance_factor = if distance < max_effect_distance {
            // Physical stress propagation: σ(r) ∝ (a/r)^n
            let stress_falloff_exponent = 0.5; // Square root falloff for 2D stress concentration
            (max_effect_distance / (distance + 1.0)).powf(stress_falloff_exponent)
        } else {
            0.0
        };

        // ENERGY-LIMITED convergence strength (not arbitrary amplification)
        let energy_limited_strength = (available_energy / 1000.0).min(1.0); // Clamp to realistic values

        // Physical constants for energy calculations
        const GRAVITY: f32 = 9.81; // m/s²
        const ROCK_DENSITY: f32 = 2700.0; // kg/m³ for crustal rock

        match boundary_type {
            BoundaryType::Convergent => {
                // METIS CORRECTION: Energy-conserving mountain building
                // Calculate maximum elevation from available energy: E = mgh → h = E/(mg)

                // Maximum elevation from gravitational potential energy conservation
                let max_elevation_from_energy = available_energy / (ROCK_DENSITY * GRAVITY);

                // Geological efficiency factors based on crustal interaction type
                let (base_elevation, efficiency_factor) =
                    match (plate1.plate_type, plate2.plate_type) {
                        (PlateType::Continental, PlateType::Continental) => {
                            // Continental collision: Most efficient mountain building
                            let combined_thickness =
                                plate1.crustal_thickness + plate2.crustal_thickness;
                            (combined_thickness * 0.08, 0.8) // 8% thickness uplift, 80% efficiency
                        }
                        (PlateType::Continental, PlateType::Oceanic)
                        | (PlateType::Oceanic, PlateType::Continental) => {
                            // Subduction zone: Moderate efficiency due to partial recycling
                            let max_thickness =
                                plate1.crustal_thickness.max(plate2.crustal_thickness);
                            (max_thickness * 0.06, 0.6) // 6% thickness uplift, 60% efficiency
                        }
                        (PlateType::Oceanic, PlateType::Oceanic) => {
                            // Ocean-ocean convergence: Volcanic island arcs, lower efficiency
                            let avg_age = (plate1.age + plate2.age) * 0.5;
                            (0.3 + avg_age * 0.001, 0.4) // Age-dependent volcanic buildup, 40% efficiency
                        }
                    };

                // Combine geological potential with energy constraints
                let energy_limited_elevation = max_elevation_from_energy * efficiency_factor;
                let geological_potential = base_elevation;

                // Take minimum to respect energy conservation
                let final_elevation = energy_limited_elevation.min(geological_potential);

                // Apply distance falloff and energy-limited strength
                final_elevation * distance_factor * energy_limited_strength
            }
            BoundaryType::Divergent => {
                // METIS CORRECTION: Energy-conserving rift formation
                // Rifting creates depressions through extensional forces

                // Energy available for rift formation (much less than mountain building)
                let rift_energy_efficiency = 0.1; // 10% of available energy
                let rift_formation_energy = available_energy * rift_energy_efficiency;

                // Calculate rift depth from energy: deeper rifts require more energy
                let max_rift_depth = rift_formation_energy / (ROCK_DENSITY * GRAVITY);

                let (base_rift_depth, rift_efficiency) =
                    match (plate1.plate_type, plate2.plate_type) {
                        (PlateType::Continental, PlateType::Continental) => {
                            // Continental rifts: East African Rift style
                            (-1.0, 0.7) // Up to 1km depth, 70% efficiency
                        }
                        _ => {
                            // Mid-ocean ridges: Shallower due to volcanic infill
                            (-0.4, 0.5) // Up to 400m depth, 50% efficiency  
                        }
                    };

                // Energy-limited rift depth
                let energy_limited_depth = -(max_rift_depth * rift_efficiency);
                let final_rift_depth = energy_limited_depth.max(base_rift_depth);

                final_rift_depth * distance_factor * energy_limited_strength
            }
            BoundaryType::Transform => {
                // METIS CORRECTION: Physics-based transform fault topography
                // Transform faults create offset topography through shear stress

                // Shear stress accumulation: τ = μ × (convergence_rate)
                let shear_modulus = 30e9; // Pa (typical crustal shear modulus)
                let convergence_rate = speed; // Plate convergence rate

                // Fault displacement energy from shear work
                let shear_energy = available_energy * 0.05; // 5% efficiency for transform motion

                // Maximum fault offset from energy conservation
                let max_fault_offset = (shear_energy / shear_modulus).sqrt() * 1000.0; // Convert to elevation units

                // Realistic transform fault topography (not modulo artifacts)
                let base_fault_amplitude = 0.15; // Base topographic variation
                let fault_efficiency = 0.3; // 30% of shear creates topography

                let energy_limited_offset =
                    (max_fault_offset * fault_efficiency).min(base_fault_amplitude);

                // Create realistic fault valley/ridge pattern
                let fault_phase = (distance * 0.5).sin(); // Smooth sinusoidal variation
                let fault_topography = energy_limited_offset * fault_phase;

                fault_topography * distance_factor * energy_limited_strength
            }
        }
    }

    fn determine_boundary_type(
        &self,
        plate1: &TectonicPlate,
        plate2: &TectonicPlate,
        relative_velocity: &Vec2,
    ) -> BoundaryType {
        // Calculate direction from plate1 center to plate2 center
        let direction = Vec2::new(
            plate2.center.x - plate1.center.x,
            plate2.center.y - plate1.center.y,
        )
        .normalize();

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
