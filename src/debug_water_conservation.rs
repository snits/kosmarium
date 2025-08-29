// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Diagnostic tool for debugging water conservation issues in simulation
// ABOUTME: Traces water mass, flow velocities, and accumulation patterns over time

use crate::engine::core::heightmap::HeightMap;
use crate::engine::core::scale::{DetailLevel, WorldScale};
use crate::engine::sim::Simulation;

#[derive(Debug, Clone)]
pub struct WaterConservationDiagnostics {
    pub tick: u64,
    pub total_water_before: f32,
    pub total_water_after: f32,
    pub water_added_rainfall: f32,
    pub water_removed_evaporation: f32,
    pub water_removed_boundaries: f32,
    pub max_velocity_magnitude: f32,
    pub avg_velocity_magnitude: f32,
    pub cells_with_high_velocity: usize,
    pub cells_with_water: usize,
    pub largest_water_accumulation: f32,
    pub water_distribution_histogram: Vec<(f32, usize)>, // (depth_range, count)
}

pub struct WaterConservationTracker {
    diagnostics_history: Vec<WaterConservationDiagnostics>,
    velocity_threshold_warning: f32,
    accumulation_threshold_warning: f32,
}

impl WaterConservationTracker {
    pub fn new() -> Self {
        Self {
            diagnostics_history: Vec::new(),
            velocity_threshold_warning: 0.5, // Warn if velocities exceed this
            accumulation_threshold_warning: 0.1, // Warn if single cell exceeds this water depth
        }
    }

    /// Track water conservation for one simulation tick
    pub fn track_tick(&mut self, sim: &mut Simulation) -> WaterConservationDiagnostics {
        let water_before = sim.water.get_total_water();
        let velocity_stats = self.analyze_velocities(&sim.water);

        // Run the tick
        sim.tick();

        let water_after = sim.water.get_total_water();
        let accumulation_stats = self.analyze_water_distribution(&sim.water);

        // Calculate expected water changes
        let rainfall_per_tick = sim.water_system.effective_rainfall_rate
            * (sim.water.width() * sim.water.height()) as f32;

        // Estimate evaporation (rough calculation - actual is temperature dependent)
        let avg_evaporation_rate = sim.water_system.parameters.evaporation_rate;
        let estimated_evaporation = water_before * avg_evaporation_rate;

        let diagnostics = WaterConservationDiagnostics {
            tick: sim.tick_count,
            total_water_before: water_before,
            total_water_after: water_after,
            water_added_rainfall: rainfall_per_tick,
            water_removed_evaporation: estimated_evaporation,
            water_removed_boundaries: 0.0, // TODO: Track boundary losses
            max_velocity_magnitude: velocity_stats.max_velocity,
            avg_velocity_magnitude: velocity_stats.avg_velocity,
            cells_with_high_velocity: velocity_stats.high_velocity_cells,
            cells_with_water: accumulation_stats.cells_with_water,
            largest_water_accumulation: accumulation_stats.max_accumulation,
            water_distribution_histogram: accumulation_stats.histogram,
        };

        self.diagnostics_history.push(diagnostics.clone());
        diagnostics
    }

    /// Analyze velocity field for potential CFL violations and artificial clamping effects
    fn analyze_velocities(
        &self,
        water: &crate::engine::physics::water::WaterLayer,
    ) -> VelocityStats {
        let mut max_velocity = 0.0f32;
        let mut total_velocity = 0.0f32;
        let mut velocity_count = 0;
        let mut high_velocity_cells = 0;

        for y in 0..water.height() {
            for x in 0..water.width() {
                let (vx, vy) = water.velocity.get(x, y);
                let velocity_mag = (vx * vx + vy * vy).sqrt();

                if velocity_mag > 0.0 {
                    max_velocity = max_velocity.max(velocity_mag);
                    total_velocity += velocity_mag;
                    velocity_count += 1;

                    if velocity_mag > self.velocity_threshold_warning {
                        high_velocity_cells += 1;
                    }
                }
            }
        }

        VelocityStats {
            max_velocity,
            avg_velocity: if velocity_count > 0 {
                total_velocity / velocity_count as f32
            } else {
                0.0
            },
            high_velocity_cells,
        }
    }

    /// Analyze water distribution patterns for artificial accumulation
    fn analyze_water_distribution(
        &self,
        water: &crate::engine::physics::water::WaterLayer,
    ) -> AccumulationStats {
        let mut max_accumulation = 0.0f32;
        let mut cells_with_water = 0;
        let mut histogram = vec![0usize; 20]; // 20 bins from 0.0 to 0.2 depth

        for y in 0..water.height() {
            for x in 0..water.width() {
                let depth = water.depth.get(x, y);

                if depth > 0.0 {
                    cells_with_water += 1;
                    max_accumulation = max_accumulation.max(depth);

                    // Update histogram (0.01 increments up to 0.2)
                    let bin = ((depth / 0.01).floor() as usize).min(19);
                    histogram[bin] += 1;
                }
            }
        }

        // Convert histogram to (range, count) pairs
        let histogram_ranges: Vec<(f32, usize)> = histogram
            .into_iter()
            .enumerate()
            .map(|(i, count)| (i as f32 * 0.01, count))
            .filter(|(_, count)| *count > 0)
            .collect();

        AccumulationStats {
            max_accumulation,
            cells_with_water,
            histogram: histogram_ranges,
        }
    }

    /// Generate comprehensive diagnostics report
    pub fn generate_report(&self, last_n_ticks: usize) -> String {
        if self.diagnostics_history.is_empty() {
            return "No diagnostics data collected yet.".to_string();
        }

        let recent_data: Vec<&WaterConservationDiagnostics> = self
            .diagnostics_history
            .iter()
            .rev()
            .take(last_n_ticks)
            .collect();

        let mut report = String::new();
        report.push_str("=== WATER CONSERVATION DIAGNOSTIC REPORT ===\n\n");

        // Conservation analysis
        self.analyze_conservation(&mut report, &recent_data);

        // Velocity analysis
        self.analyze_velocity_patterns(&mut report, &recent_data);

        // Accumulation patterns
        self.analyze_accumulation_patterns(&mut report, &recent_data);

        // Stability warnings
        self.generate_warnings(&mut report, &recent_data);

        report
    }

    fn analyze_conservation(&self, report: &mut String, data: &[&WaterConservationDiagnostics]) {
        report.push_str("--- WATER MASS CONSERVATION ---\n");

        if data.len() >= 2 {
            let first = data.last().unwrap(); // Oldest in recent data
            let last = data.first().unwrap(); // Most recent

            let net_change = last.total_water_after - first.total_water_before;
            let expected_rainfall = data.iter().map(|d| d.water_added_rainfall).sum::<f32>();
            let expected_evaporation = data
                .iter()
                .map(|d| d.water_removed_evaporation)
                .sum::<f32>();
            let expected_net = expected_rainfall - expected_evaporation;

            report.push_str(&format!("Time period: {} ticks\n", data.len()));
            report.push_str(&format!("Actual net change: {:.6}\n", net_change));
            report.push_str(&format!("Expected net change: {:.6}\n", expected_net));
            report.push_str(&format!(
                "Conservation error: {:.6} ({:.3}%)\n",
                net_change - expected_net,
                if expected_net.abs() > 0.0 {
                    100.0 * (net_change - expected_net) / expected_net.abs()
                } else {
                    0.0
                }
            ));

            // Check for systematic drift
            if (net_change - expected_net).abs() > expected_net.abs() * 0.1 {
                report.push_str("⚠️  WARNING: Significant conservation violation detected!\n");
            }
        }

        report.push_str("\n");
    }

    fn analyze_velocity_patterns(
        &self,
        report: &mut String,
        data: &[&WaterConservationDiagnostics],
    ) {
        report.push_str("--- VELOCITY FIELD ANALYSIS ---\n");

        let max_velocity = data
            .iter()
            .map(|d| d.max_velocity_magnitude)
            .fold(0.0f32, f32::max);
        let avg_max_velocity =
            data.iter().map(|d| d.max_velocity_magnitude).sum::<f32>() / data.len() as f32;
        let avg_avg_velocity =
            data.iter().map(|d| d.avg_velocity_magnitude).sum::<f32>() / data.len() as f32;

        report.push_str(&format!("Maximum velocity observed: {:.4}\n", max_velocity));
        report.push_str(&format!(
            "Average maximum velocity: {:.4}\n",
            avg_max_velocity
        ));
        report.push_str(&format!(
            "Average velocity magnitude: {:.4}\n",
            avg_avg_velocity
        ));

        // Check for velocity clamping at 1.0 (artificial limit)
        if max_velocity >= 0.99 {
            report.push_str("⚠️  WARNING: Velocities may be artificially clamped at 1.0!\n");
        }

        // Check for CFL stability issues
        if max_velocity > 0.5 {
            report.push_str("⚠️  WARNING: High velocities may violate CFL stability conditions!\n");
        }

        report.push_str("\n");
    }

    fn analyze_accumulation_patterns(
        &self,
        report: &mut String,
        data: &[&WaterConservationDiagnostics],
    ) {
        report.push_str("--- WATER ACCUMULATION PATTERNS ---\n");

        let max_accumulation = data
            .iter()
            .map(|d| d.largest_water_accumulation)
            .fold(0.0f32, f32::max);
        let avg_cells_with_water =
            data.iter().map(|d| d.cells_with_water).sum::<usize>() / data.len();

        report.push_str(&format!(
            "Maximum single-cell accumulation: {:.6}\n",
            max_accumulation
        ));
        report.push_str(&format!(
            "Average cells with water: {}\n",
            avg_cells_with_water
        ));

        // Check for excessive accumulation
        if max_accumulation > self.accumulation_threshold_warning {
            report
                .push_str("⚠️  WARNING: Excessive water accumulation in single cells detected!\n");
        }

        // Analyze trend in accumulation
        if data.len() >= 10 {
            let recent_max = data
                .iter()
                .take(5)
                .map(|d| d.largest_water_accumulation)
                .fold(0.0f32, f32::max);
            let older_max = data
                .iter()
                .skip(5)
                .map(|d| d.largest_water_accumulation)
                .fold(0.0f32, f32::max);

            if recent_max > older_max * 1.5 {
                report
                    .push_str("⚠️  WARNING: Water accumulation is increasing rapidly over time!\n");
            }
        }

        report.push_str("\n");
    }

    fn generate_warnings(&self, report: &mut String, data: &[&WaterConservationDiagnostics]) {
        report.push_str("--- STABILITY WARNINGS ---\n");

        let mut warnings = Vec::new();

        // Check for lake buildup pattern (what Jerry reported)
        let recent_max_accumulation = data
            .iter()
            .take(5)
            .map(|d| d.largest_water_accumulation)
            .fold(0.0f32, f32::max);
        if recent_max_accumulation > 0.05 {
            warnings.push("Lake buildup pattern detected - may lead to 'flip' behavior");
        }

        // Check for boundary condition issues
        let has_boundary_losses = data.iter().any(|d| d.water_removed_boundaries > 0.0);
        if !has_boundary_losses && data.iter().any(|d| d.max_velocity_magnitude > 0.1) {
            warnings.push(
                "High velocities but no boundary water loss - may indicate reflection issues",
            );
        }

        // Check for numerical instability
        let velocity_variation = {
            let velocities: Vec<f32> = data.iter().map(|d| d.max_velocity_magnitude).collect();
            if velocities.len() >= 3 {
                let mut max_change = 0.0f32;
                for i in 1..velocities.len() {
                    max_change = max_change.max((velocities[i] - velocities[i - 1]).abs());
                }
                max_change
            } else {
                0.0
            }
        };

        if velocity_variation > 0.2 {
            warnings.push("Large velocity fluctuations detected - possible numerical instability");
        }

        if warnings.is_empty() {
            report.push_str("No major stability warnings detected.\n");
        } else {
            for warning in warnings {
                report.push_str(&format!("⚠️  {}\n", warning));
            }
        }

        report.push_str("\n");
    }
}

#[derive(Debug)]
struct VelocityStats {
    max_velocity: f32,
    avg_velocity: f32,
    high_velocity_cells: usize,
}

#[derive(Debug)]
struct AccumulationStats {
    max_accumulation: f32,
    cells_with_water: usize,
    histogram: Vec<(f32, usize)>,
}

/// Test function to run diagnostics on the problematic 512x256 resolution
pub fn test_512x256_conservation() {
    println!("Testing water conservation at 512x256 resolution...\n");

    // Create the problematic setup
    let heightmap = HeightMap::new(512, 256, 0.5); // Flat terrain to isolate the issue
    let world_scale = WorldScale::new(100.0, (512, 256), DetailLevel::Standard);
    let mut sim = Simulation::_new_with_scale(heightmap, world_scale);

    let mut tracker = WaterConservationTracker::new();

    // Run simulation for many ticks to observe the buildup pattern
    println!("Running 50 ticks to observe water buildup pattern...");
    for tick in 0..50 {
        let diagnostics = tracker.track_tick(&mut sim);

        if tick % 10 == 0 {
            println!(
                "Tick {}: Total water = {:.6}, Max accumulation = {:.6}, Max velocity = {:.4}",
                diagnostics.tick,
                diagnostics.total_water_after,
                diagnostics.largest_water_accumulation,
                diagnostics.max_velocity_magnitude
            );
        }
    }

    println!("\n{}", tracker.generate_report(50));
}

/// Test different resolutions to compare conservation behavior
pub fn test_resolution_scaling_conservation() {
    println!("Testing water conservation across different resolutions...\n");

    let resolutions = vec![
        (240, 120, "Reference"),
        (480, 240, "2x Scale"),
        (512, 256, "Problematic"),
        (1024, 512, "Large Scale"),
    ];

    for (width, height, label) in resolutions {
        println!("--- Testing {}x{} ({}) ---", width, height, label);

        let heightmap = HeightMap::new(width, height, 0.5);
        let world_scale =
            WorldScale::new(100.0, (width as u32, height as u32), DetailLevel::Standard);
        let mut sim = Simulation::_new_with_scale(heightmap, world_scale);

        let mut tracker = WaterConservationTracker::new();

        // Run for 20 ticks
        for _ in 0..20 {
            tracker.track_tick(&mut sim);
        }

        let report = tracker.generate_report(20);
        // Print only the conservation section
        let conservation_section: Vec<&str> = report
            .lines()
            .skip_while(|line| !line.starts_with("--- WATER MASS CONSERVATION ---"))
            .take_while(|line| !line.starts_with("--- VELOCITY FIELD ANALYSIS ---"))
            .collect();

        for line in conservation_section {
            println!("{}", line);
        }
        println!();
    }
}
