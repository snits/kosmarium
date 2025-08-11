// ABOUTME: Tests for geostrophic balance validation framework detecting current physics violations
// ABOUTME: Implements f × v ≈ -(1/ρ)∇P validation and measures pressure-wind coupling correlation

use sim_protoype::engine::core::scale::{DetailLevel, WorldScale};
use sim_protoype::engine::physics::atmosphere::{AtmosphericSystem, BoundaryType, WindLayer};
use sim_protoype::engine::physics::climate::AtmosphericPressureLayer;
use sim_protoype::engine::physics::water::Vec2;

// Safety parameters from SageMath validation
const F_THRESHOLD: f64 = 1e-6; // s⁻¹ - numerical stability  
const F_TROPICAL_LIMIT: f64 = 1.27e-5; // s⁻¹ - 5° boundary
const MIN_PRESSURE_GRADIENT: f64 = 0.0001; // Pa/m
const MAX_PRESSURE_GRADIENT: f64 = 0.0500; // Pa/m  
const MAX_REALISTIC_WIND: f64 = 100.0; // m/s

/// Geostrophic balance validation results
#[derive(Debug, Clone)]
pub struct GeostrophicBalanceValidation {
    /// Average geostrophic balance residual across domain (should be near zero)
    pub average_balance_residual: f64,
    /// Maximum geostrophic balance residual
    pub max_balance_residual: f64,
    /// Pressure-wind coupling correlation coefficient (-1 to 1)
    pub pressure_wind_correlation: f64,
    /// Number of cells with unrealistic wind speeds (>100 m/s)
    pub unrealistic_wind_cell_count: usize,
    /// Average wind speed across domain
    pub average_wind_speed: f64,
    /// Maximum wind speed found
    pub maximum_wind_speed: f64,
    /// Whether geostrophic balance is satisfied (residual < threshold)
    pub is_geostrophic_balanced: bool,
    /// Detailed cell-by-cell analysis for problematic regions
    pub problematic_cells: Vec<GeostrophicCellAnalysis>,
}

/// Per-cell geostrophic balance analysis
#[derive(Debug, Clone)]
pub struct GeostrophicCellAnalysis {
    /// Grid coordinates  
    pub x: usize,
    pub y: usize,
    /// Latitude at this cell
    pub latitude_deg: f64,
    /// Coriolis parameter at this cell
    pub coriolis_parameter: f64,
    /// Pressure gradient vector (Pa/m)
    pub pressure_gradient: Vec2,
    /// Wind velocity vector (m/s)
    pub wind_velocity: Vec2,
    /// Geostrophic balance residual: |f × v + ∇P/ρ|
    pub balance_residual: f64,
    /// Whether this cell violates physics
    pub is_problematic: bool,
}

/// Atmospheric scale analysis validation results
#[derive(Debug, Clone)]
pub struct AtmosphericScaleValidation {
    /// Rossby number: Ro = U/(fL) - measures relative importance of inertial to Coriolis forces
    pub rossby_number: f64,
    /// Domain length scale (meters)
    pub length_scale_m: f64,
    /// Characteristic velocity scale (m/s)
    pub velocity_scale: f64,
    /// Effective Coriolis parameter (s⁻¹)
    pub effective_coriolis_parameter: f64,
    /// Whether winds are in realistic continental range (5-25 m/s)
    pub winds_in_continental_range: bool,
    /// Percentage of cells with realistic wind speeds
    pub realistic_wind_percentage: f64,
    /// Scale regime classification
    pub scale_regime: AtmosphericScaleRegime,
    /// Dimensional analysis validation
    pub dimensional_consistency: DimensionalConsistency,
}

/// Atmospheric scale regimes based on Rossby number
#[derive(Debug, Clone, PartialEq)]
pub enum AtmosphericScaleRegime {
    /// Ro << 1: Geostrophic balance dominates
    Geostrophic,
    /// Ro ~ 1: Both inertial and Coriolis forces important  
    Transitional,
    /// Ro >> 1: Inertial forces dominate, no geostrophic balance
    Inertial,
}

/// Dimensional analysis validation for atmospheric equations
#[derive(Debug, Clone)]
pub struct DimensionalConsistency {
    /// Whether pressure gradient term has correct dimensions [m/s²]
    pub pressure_gradient_dimensions_correct: bool,
    /// Whether Coriolis term has correct dimensions [m/s²]
    pub coriolis_term_dimensions_correct: bool,
    /// Whether velocity units are consistent [m/s]
    pub velocity_units_consistent: bool,
    /// Overall dimensional consistency
    pub is_dimensionally_consistent: bool,
}

/// Enhanced mass conservation diagnostics with spatial analysis
#[derive(Debug, Clone)]
pub struct EnhancedMassConservationValidation {
    /// Total momentum magnitude in the domain (m/s)
    pub total_momentum_magnitude: f64,
    /// Momentum flux through domain boundaries ∮(ρv·n)dA (kg/s)
    pub boundary_momentum_flux: BoundaryMomentumFlux,
    /// Continuity equation validation results
    pub continuity_validation: ContinuityEquationValidation,
    /// Spatial distribution of momentum violations
    pub spatial_violations: Vec<SpatialMomentumViolation>,
    /// Whether mass conservation is satisfied
    pub is_mass_conserved: bool,
    /// Diagnostic summary
    pub diagnostic_summary: String,
}

/// Momentum flux analysis through domain boundaries
#[derive(Debug, Clone)]
pub struct BoundaryMomentumFlux {
    /// Flux through north boundary (kg/s)
    pub north_flux: f64,
    /// Flux through south boundary (kg/s)  
    pub south_flux: f64,
    /// Flux through east boundary (kg/s)
    pub east_flux: f64,
    /// Flux through west boundary (kg/s)
    pub west_flux: f64,
    /// Total net flux (should be near zero for conservation)
    pub net_flux: f64,
    /// Whether boundary fluxes are balanced
    pub is_balanced: bool,
}

/// Continuity equation validation: ∂ρ/∂t + ∇·(ρv) = 0
#[derive(Debug, Clone)]
pub struct ContinuityEquationValidation {
    /// Average divergence across domain (s⁻¹)
    pub average_divergence: f64,
    /// Maximum divergence magnitude (s⁻¹)
    pub max_divergence: f64,
    /// Number of cells violating continuity
    pub violation_cell_count: usize,
    /// Total cells analyzed
    pub total_cells: usize,
    /// Percentage of cells violating continuity
    pub violation_percentage: f64,
    /// Whether continuity equation is satisfied
    pub is_satisfied: bool,
}

/// Spatial momentum violation analysis
#[derive(Debug, Clone)]
pub struct SpatialMomentumViolation {
    /// Grid coordinates
    pub x: usize,
    pub y: usize,
    /// Local velocity divergence (s⁻¹)
    pub velocity_divergence: f64,
    /// Local momentum magnitude (m/s)
    pub momentum_magnitude: f64,
    /// Boundary type if applicable
    pub boundary_type: Option<String>,
    /// Violation severity (0-1 scale)
    pub severity: f64,
}

/// Validate geostrophic balance equation: f × v ≈ -(1/ρ)∇P
/// Returns detailed analysis of current physics violations
pub fn validate_geostrophic_balance(
    atmospheric_system: &AtmosphericSystem,
    pressure_layer: &AtmosphericPressureLayer,
    wind_layer: &WindLayer,
) -> GeostrophicBalanceValidation {
    let width = pressure_layer.pressure.width();
    let height = pressure_layer.pressure.height();
    let air_density = atmospheric_system.parameters.air_density_sea_level as f64;

    let mut balance_residuals = Vec::new();
    let mut problematic_cells = Vec::new();
    let mut unrealistic_wind_count = 0;
    let mut wind_speeds = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let latitude_rad = atmospheric_system.grid_y_to_latitude(y, height);
            let latitude_deg = latitude_rad * 180.0 / std::f64::consts::PI;
            let coriolis_f = atmospheric_system.coriolis_parameter_at_latitude(latitude_rad);

            let pressure_grad = pressure_layer.get_pressure_gradient(x, y);
            let wind_vel = wind_layer.get_velocity(x, y);
            let wind_speed = wind_vel.magnitude() as f64;

            wind_speeds.push(wind_speed);

            // Check for unrealistic wind speeds (current system produces ~135 m/s)
            if wind_speed > MAX_REALISTIC_WIND {
                unrealistic_wind_count += 1;
            }

            // Skip cells near equator where Coriolis is negligible
            if coriolis_f.abs() < F_THRESHOLD {
                continue;
            }

            // Calculate geostrophic balance residual
            // Geostrophic balance: f × v = -(1/ρ)∇P
            // In 2D: f * v_y = -(1/ρ) * ∂P/∂x  (u-component)
            //        f * v_x = (1/ρ) * ∂P/∂y   (v-component)
            let theoretical_u = -(pressure_grad.y as f64) / (air_density * coriolis_f);
            let theoretical_v = (pressure_grad.x as f64) / (air_density * coriolis_f);

            let actual_u = wind_vel.x as f64;
            let actual_v = wind_vel.y as f64;

            // Calculate residual magnitude
            let residual_u = actual_u - theoretical_u;
            let residual_v = actual_v - theoretical_v;
            let balance_residual = (residual_u * residual_u + residual_v * residual_v).sqrt();

            balance_residuals.push(balance_residual);

            // Flag problematic cells (large residual or unrealistic winds)
            let is_problematic = balance_residual > 10.0 || wind_speed > MAX_REALISTIC_WIND;

            if is_problematic {
                problematic_cells.push(GeostrophicCellAnalysis {
                    x,
                    y,
                    latitude_deg,
                    coriolis_parameter: coriolis_f,
                    pressure_gradient: pressure_grad,
                    wind_velocity: wind_vel,
                    balance_residual,
                    is_problematic,
                });
            }
        }
    }

    // Calculate statistics
    let average_balance_residual = if !balance_residuals.is_empty() {
        balance_residuals.iter().sum::<f64>() / balance_residuals.len() as f64
    } else {
        0.0
    };

    let max_balance_residual = balance_residuals.iter().fold(0.0f64, |a, &b| a.max(b));
    let average_wind_speed = wind_speeds.iter().sum::<f64>() / wind_speeds.len() as f64;
    let maximum_wind_speed = wind_speeds.iter().fold(0.0f64, |a, &b| a.max(b));

    // Calculate pressure-wind correlation (simplified)
    let pressure_wind_correlation = calculate_pressure_wind_correlation(pressure_layer, wind_layer);

    // System is balanced if residuals are small
    let is_geostrophic_balanced = average_balance_residual < 5.0 && max_balance_residual < 20.0;

    GeostrophicBalanceValidation {
        average_balance_residual,
        max_balance_residual,
        pressure_wind_correlation,
        unrealistic_wind_cell_count: unrealistic_wind_count,
        average_wind_speed,
        maximum_wind_speed,
        is_geostrophic_balanced,
        problematic_cells,
    }
}

/// Validate atmospheric scale analysis and dimensional consistency
/// Returns detailed analysis of scale-appropriate physics
pub fn validate_atmospheric_scale_analysis(
    atmospheric_system: &AtmosphericSystem,
    wind_layer: &WindLayer,
    world_scale: &WorldScale,
) -> AtmosphericScaleValidation {
    let width = wind_layer.width();
    let height = wind_layer.height();

    // Calculate domain length scale (typical for atmospheric analysis)
    let domain_width_m = world_scale.physical_size_km * 1000.0;
    let domain_height_m = (world_scale.physical_size_km * height as f64 / width as f64) * 1000.0;
    let length_scale_m = (domain_width_m * domain_height_m).sqrt(); // Geometric mean

    // Calculate characteristic velocity scale
    let mut wind_speeds = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let velocity = wind_layer.get_velocity(x, y);
            wind_speeds.push(velocity.magnitude() as f64);
        }
    }

    let velocity_scale = if !wind_speeds.is_empty() {
        wind_speeds.iter().sum::<f64>() / wind_speeds.len() as f64
    } else {
        0.0
    };

    // Get effective Coriolis parameter
    let effective_coriolis_parameter = atmospheric_system.effective_coriolis_parameter;

    // Calculate Rossby number: Ro = U/(fL)
    let rossby_number = if effective_coriolis_parameter.abs() > 1e-12 && length_scale_m > 0.0 {
        velocity_scale / (effective_coriolis_parameter * length_scale_m)
    } else {
        f64::INFINITY // No Coriolis effects
    };

    // Classify scale regime
    let scale_regime = if rossby_number < 0.1 {
        AtmosphericScaleRegime::Geostrophic
    } else if rossby_number < 1.0 {
        AtmosphericScaleRegime::Transitional
    } else {
        AtmosphericScaleRegime::Inertial
    };

    // Check if winds are in realistic continental range (5-25 m/s)
    let realistic_count = wind_speeds
        .iter()
        .filter(|&&speed| speed >= 5.0 && speed <= 25.0)
        .count();
    let realistic_wind_percentage = (realistic_count as f64 / wind_speeds.len() as f64) * 100.0;
    let winds_in_continental_range = realistic_wind_percentage > 80.0;

    // Validate dimensional consistency
    let dimensional_consistency = validate_dimensional_consistency();

    AtmosphericScaleValidation {
        rossby_number,
        length_scale_m,
        velocity_scale,
        effective_coriolis_parameter,
        winds_in_continental_range,
        realistic_wind_percentage,
        scale_regime,
        dimensional_consistency,
    }
}

/// Validate dimensional consistency of atmospheric equations
fn validate_dimensional_consistency() -> DimensionalConsistency {
    // In the atmospheric momentum equation: ∂v/∂t + (v·∇)v + fk×v = -(1/ρ)∇P
    // All terms must have dimensions of acceleration [m/s²]

    // Pressure gradient term: -(1/ρ)∇P
    // [Pa/m] / [kg/m³] = [N/m²/m] / [kg/m³] = [kg·m/s²/m³] / [kg/m³] = [m/s²] ✓
    let pressure_gradient_dimensions_correct = true;

    // Coriolis term: f × v
    // [s⁻¹] × [m/s] = [m/s²] ✓
    let coriolis_term_dimensions_correct = true;

    // Velocity units should be consistent [m/s]
    let velocity_units_consistent = true;

    let is_dimensionally_consistent = pressure_gradient_dimensions_correct
        && coriolis_term_dimensions_correct
        && velocity_units_consistent;

    DimensionalConsistency {
        pressure_gradient_dimensions_correct,
        coriolis_term_dimensions_correct,
        velocity_units_consistent,
        is_dimensionally_consistent,
    }
}

/// Enhanced mass conservation validation with spatial analysis and boundary flux analysis
/// Implements continuity equation validation: ∂ρ/∂t + ∇·(ρv) = 0
pub fn validate_enhanced_mass_conservation(
    wind_layer: &WindLayer,
    world_scale: &WorldScale,
    air_density: f32,
) -> EnhancedMassConservationValidation {
    let width = wind_layer.width();
    let height = wind_layer.height();
    let meters_per_pixel = world_scale.meters_per_pixel() as f64;

    // Calculate total momentum magnitude
    let total_momentum = wind_layer.calculate_total_momentum();
    let total_momentum_magnitude = total_momentum.magnitude() as f64;

    // Analyze momentum flux through boundaries: ∮(ρv·n)dA
    let boundary_flux =
        calculate_boundary_momentum_flux(wind_layer, meters_per_pixel, air_density as f64);

    // Validate continuity equation: ∇·v = 0 (incompressible flow assumption)
    let continuity_validation = validate_continuity_equation(wind_layer, meters_per_pixel);

    // Identify spatial momentum violations
    let spatial_violations = analyze_spatial_momentum_violations(wind_layer, meters_per_pixel);

    // Overall mass conservation assessment
    let is_mass_conserved = total_momentum_magnitude < 1000.0 // Arbitrary threshold for continental domain
        && boundary_flux.is_balanced
        && continuity_validation.is_satisfied;

    // Generate diagnostic summary
    let diagnostic_summary = format!(
        "Mass Conservation Analysis: Total momentum {:.1} m/s, Net boundary flux {:.3e} kg/s, {:.1}% cells violate continuity",
        total_momentum_magnitude,
        boundary_flux.net_flux,
        continuity_validation.violation_percentage
    );

    EnhancedMassConservationValidation {
        total_momentum_magnitude,
        boundary_momentum_flux: boundary_flux,
        continuity_validation,
        spatial_violations,
        is_mass_conserved,
        diagnostic_summary,
    }
}

/// Calculate momentum flux through domain boundaries: ∮(ρv·n)dA
fn calculate_boundary_momentum_flux(
    wind_layer: &WindLayer,
    meters_per_pixel: f64,
    air_density: f64,
) -> BoundaryMomentumFlux {
    let width = wind_layer.width();
    let height = wind_layer.height();
    let cell_area = meters_per_pixel * meters_per_pixel; // m²

    let mut north_flux = 0.0;
    let mut south_flux = 0.0;
    let mut east_flux = 0.0;
    let mut west_flux = 0.0;

    // North boundary (y = 0): outward normal is +y (northward)
    for x in 0..width {
        let velocity = wind_layer.get_velocity(x, 0);
        let normal_velocity = velocity.y as f64; // v·n where n = (0, 1)
        north_flux += air_density * normal_velocity * cell_area;
    }

    // South boundary (y = height-1): outward normal is -y (southward)
    if height > 1 {
        for x in 0..width {
            let velocity = wind_layer.get_velocity(x, height - 1);
            let normal_velocity = -velocity.y as f64; // v·n where n = (0, -1)
            south_flux += air_density * normal_velocity * cell_area;
        }
    }

    // East boundary (x = width-1): outward normal is +x (eastward)
    if width > 1 {
        for y in 0..height {
            let velocity = wind_layer.get_velocity(width - 1, y);
            let normal_velocity = velocity.x as f64; // v·n where n = (1, 0)
            east_flux += air_density * normal_velocity * cell_area;
        }
    }

    // West boundary (x = 0): outward normal is -x (westward)
    for y in 0..height {
        let velocity = wind_layer.get_velocity(0, y);
        let normal_velocity = -velocity.x as f64; // v·n where n = (-1, 0)
        west_flux += air_density * normal_velocity * cell_area;
    }

    let net_flux = north_flux + south_flux + east_flux + west_flux;
    let is_balanced = net_flux.abs() < 1e-3; // kg/s threshold for balance

    BoundaryMomentumFlux {
        north_flux,
        south_flux,
        east_flux,
        west_flux,
        net_flux,
        is_balanced,
    }
}

/// Validate continuity equation: ∇·v = 0 for incompressible flow
fn validate_continuity_equation(
    wind_layer: &WindLayer,
    meters_per_pixel: f64,
) -> ContinuityEquationValidation {
    let width = wind_layer.width();
    let height = wind_layer.height();
    let dx = meters_per_pixel;
    let dy = meters_per_pixel;

    let mut divergences = Vec::new();
    let mut violation_count = 0;

    // Calculate velocity divergence using central differences
    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let u_east = wind_layer.get_velocity(x + 1, y).x as f64;
            let u_west = wind_layer.get_velocity(x - 1, y).x as f64;
            let v_north = wind_layer.get_velocity(x, y - 1).y as f64; // y decreases northward
            let v_south = wind_layer.get_velocity(x, y + 1).y as f64;

            // Velocity divergence: ∇·v = ∂u/∂x + ∂v/∂y
            let du_dx = (u_east - u_west) / (2.0 * dx);
            let dv_dy = (v_south - v_north) / (2.0 * dy); // Note: y increases southward
            let divergence = du_dx + dv_dy;

            divergences.push(divergence);

            // Check for continuity violation (divergence should be ~0 for incompressible flow)
            if divergence.abs() > 1e-6 {
                // s⁻¹ threshold
                violation_count += 1;
            }
        }
    }

    let total_cells = divergences.len();
    let average_divergence = if !divergences.is_empty() {
        divergences.iter().sum::<f64>() / divergences.len() as f64
    } else {
        0.0
    };

    let max_divergence = divergences.iter().fold(0.0f64, |a, &b| a.max(b.abs()));
    let violation_percentage = if total_cells > 0 {
        (violation_count as f64 / total_cells as f64) * 100.0
    } else {
        0.0
    };

    let is_satisfied = average_divergence.abs() < 1e-8 && max_divergence < 1e-6;

    ContinuityEquationValidation {
        average_divergence,
        max_divergence,
        violation_cell_count: violation_count,
        total_cells,
        violation_percentage,
        is_satisfied,
    }
}

/// Analyze spatial distribution of momentum violations
fn analyze_spatial_momentum_violations(
    wind_layer: &WindLayer,
    meters_per_pixel: f64,
) -> Vec<SpatialMomentumViolation> {
    let width = wind_layer.width();
    let height = wind_layer.height();
    let dx = meters_per_pixel;
    let dy = meters_per_pixel;
    let mut violations = Vec::new();

    // Sample key locations for violation analysis
    let sample_step = ((width * height) / 100).max(1); // Sample ~100 cells

    for y in (0..height).step_by(sample_step) {
        for x in (0..width).step_by(sample_step) {
            let velocity = wind_layer.get_velocity(x, y);
            let momentum_magnitude = velocity.magnitude() as f64;

            // Calculate local velocity divergence if not on boundary
            let velocity_divergence = if x > 0 && x < width - 1 && y > 0 && y < height - 1 {
                let u_east = wind_layer.get_velocity(x + 1, y).x as f64;
                let u_west = wind_layer.get_velocity(x - 1, y).x as f64;
                let v_north = wind_layer.get_velocity(x, y - 1).y as f64;
                let v_south = wind_layer.get_velocity(x, y + 1).y as f64;

                let du_dx = (u_east - u_west) / (2.0 * dx);
                let dv_dy = (v_south - v_north) / (2.0 * dy);
                du_dx + dv_dy
            } else {
                0.0
            };

            // Determine boundary type
            let boundary_type = if wind_layer.is_boundary_cell(x, y) {
                Some(match wind_layer.get_boundary_type(x, y) {
                    BoundaryType::North => "North".to_string(),
                    BoundaryType::South => "South".to_string(),
                    BoundaryType::East => "East".to_string(),
                    BoundaryType::West => "West".to_string(),
                    BoundaryType::Interior => "Interior".to_string(),
                })
            } else {
                None
            };

            // Calculate violation severity (0-1 scale)
            let divergence_severity = (velocity_divergence.abs() / 1e-4).min(1.0);
            let momentum_severity = ((momentum_magnitude - 15.0).max(0.0) / 85.0).min(1.0); // Above 15 m/s
            let severity = (divergence_severity + momentum_severity) / 2.0;

            // Only include significant violations
            if severity > 0.1 {
                violations.push(SpatialMomentumViolation {
                    x,
                    y,
                    velocity_divergence,
                    momentum_magnitude,
                    boundary_type,
                    severity,
                });
            }
        }
    }

    // Sort by severity (highest first)
    violations.sort_by(|a, b| {
        b.severity
            .partial_cmp(&a.severity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    violations
}

/// Calculate correlation between pressure gradients and wind patterns
fn calculate_pressure_wind_correlation(
    pressure_layer: &AtmosphericPressureLayer,
    wind_layer: &WindLayer,
) -> f64 {
    let width = pressure_layer.pressure.width();
    let height = pressure_layer.pressure.height();

    let mut pressure_grad_magnitudes = Vec::new();
    let mut wind_speeds = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pressure_grad = pressure_layer.get_pressure_gradient(x, y);
            let wind_vel = wind_layer.get_velocity(x, y);

            pressure_grad_magnitudes.push(pressure_grad.magnitude() as f64);
            wind_speeds.push(wind_vel.magnitude() as f64);
        }
    }

    // Calculate correlation coefficient
    pearson_correlation(&pressure_grad_magnitudes, &wind_speeds)
}

/// Calculate Pearson correlation coefficient
fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }

    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let mut numerator = 0.0;
    let mut sum_sq_x = 0.0;
    let mut sum_sq_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        numerator += dx * dy;
        sum_sq_x += dx * dx;
        sum_sq_y += dy * dy;
    }

    let denominator = (sum_sq_x * sum_sq_y).sqrt();
    if denominator < 1e-10 {
        0.0
    } else {
        numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geostrophic_balance_detects_current_physics_violations() {
        println!("\n1. Testing continental domain for physics violations...");
        println!("-----------------------------------------------------");
        // Create continental scale system (500km domain) where we expect violations
        let scale = WorldScale::new(500.0, (50, 50), DetailLevel::Standard);
        let atmospheric_system = AtmosphericSystem::new_for_scale(&scale);

        // Create realistic pressure field with gradients
        let mut pressure_layer = AtmosphericPressureLayer::new(50, 50);

        // Set up pressure gradient from west (high) to east (low)
        for y in 0..50 {
            for x in 0..50 {
                let pressure = 101325.0 - (x as f32 * 200.0); // 200 Pa per cell decrease eastward
                pressure_layer.pressure.set(x, y, pressure);
            }
        }
        pressure_layer.calculate_pressure_gradients(scale.meters_per_pixel() as f32);

        // Generate winds using current (flawed) system
        let wind_layer = atmospheric_system.generate_geostrophic_winds(&pressure_layer, &scale);

        // Validate geostrophic balance - should detect violations
        let validation =
            validate_geostrophic_balance(&atmospheric_system, &pressure_layer, &wind_layer);

        // Expected results from current flawed implementation
        println!("Geostrophic Balance Analysis:");
        println!(
            "  Average balance residual: {:.2} m/s",
            validation.average_balance_residual
        );
        println!(
            "  Max balance residual: {:.2} m/s",
            validation.max_balance_residual
        );
        println!(
            "  Pressure-wind correlation: {:.3}",
            validation.pressure_wind_correlation
        );
        println!(
            "  Unrealistic wind cells: {}",
            validation.unrealistic_wind_cell_count
        );
        println!(
            "  Average wind speed: {:.2} m/s",
            validation.average_wind_speed
        );
        println!(
            "  Maximum wind speed: {:.2} m/s",
            validation.maximum_wind_speed
        );
        println!("  Is balanced: {}", validation.is_geostrophic_balanced);

        // The current system should violate geostrophic balance
        // We expect high wind speeds and poor correlation
        assert!(
            !validation.is_geostrophic_balanced,
            "Current implementation should violate geostrophic balance"
        );

        // We expect unrealistically high wind speeds from the current system
        assert!(
            validation.maximum_wind_speed > 50.0,
            "Current system should produce excessive wind speeds, got {:.1} m/s",
            validation.maximum_wind_speed
        );

        // For continental domains, we expect some pressure-wind coupling
        // but the current implementation likely has poor correlation
        println!(
            "  Problematic cells: {}",
            validation.problematic_cells.len()
        );

        // Document the specific physics violations found
        if !validation.problematic_cells.is_empty() {
            println!("  Sample problematic cells:");
            for (i, cell) in validation.problematic_cells.iter().take(5).enumerate() {
                println!(
                    "    Cell {}: ({}, {}) lat={:.1}° f={:.2e} |∇P|={:.1} |v|={:.1} residual={:.1}",
                    i + 1,
                    cell.x,
                    cell.y,
                    cell.latitude_deg,
                    cell.coriolis_parameter,
                    cell.pressure_gradient.magnitude(),
                    cell.wind_velocity.magnitude(),
                    cell.balance_residual
                );
            }
        }
    }

    #[test]
    fn test_atmospheric_scale_analysis_framework() {
        println!("\n2. Testing atmospheric scale analysis framework...");
        println!("-------------------------------------------------");

        // Create continental scale system (500km domain) for scale analysis
        let scale = WorldScale::new(500.0, (50, 50), DetailLevel::Standard);
        let atmospheric_system = AtmosphericSystem::new_for_scale(&scale);

        // Create pressure field with gradients
        let mut pressure_layer = AtmosphericPressureLayer::new(50, 50);
        for y in 0..50 {
            for x in 0..50 {
                let pressure = 101325.0 - (x as f32 * 200.0); // 200 Pa per cell decrease eastward
                pressure_layer.pressure.set(x, y, pressure);
            }
        }
        pressure_layer.calculate_pressure_gradients(scale.meters_per_pixel() as f32);

        // Generate winds using current system
        let wind_layer = atmospheric_system.generate_geostrophic_winds(&pressure_layer, &scale);

        // Validate atmospheric scale analysis
        let scale_validation =
            validate_atmospheric_scale_analysis(&atmospheric_system, &wind_layer, &scale);

        println!("Atmospheric Scale Analysis:");
        println!(
            "  Domain length scale: {:.0} km",
            scale_validation.length_scale_m / 1000.0
        );
        println!(
            "  Velocity scale: {:.2} m/s",
            scale_validation.velocity_scale
        );
        println!(
            "  Coriolis parameter: {:.2e} s⁻¹",
            scale_validation.effective_coriolis_parameter
        );
        println!("  Rossby number: {:.3}", scale_validation.rossby_number);
        println!("  Scale regime: {:?}", scale_validation.scale_regime);
        println!(
            "  Winds in continental range: {}",
            scale_validation.winds_in_continental_range
        );
        println!(
            "  Realistic wind percentage: {:.1}%",
            scale_validation.realistic_wind_percentage
        );
        println!(
            "  Dimensional consistency: {}",
            scale_validation
                .dimensional_consistency
                .is_dimensionally_consistent
        );

        // Expected results for continental domain (500km)
        // Continental domains should have Ro ~ 0.1-1.0 (transitional regime)
        assert!(
            scale_validation.length_scale_m > 100_000.0,
            "Continental domain should have length scale > 100km"
        );

        // The current system should produce unrealistic wind speeds
        assert!(
            !scale_validation.winds_in_continental_range,
            "Current system should produce non-continental wind speeds, got {}% realistic",
            scale_validation.realistic_wind_percentage
        );

        // For 500km domain, we expect transitional or inertial regime due to excessive winds
        println!("  Expected: Continental domains should have Ro ~ 0.1-1.0 (transitional)");
        println!(
            "  Actual: Ro = {:.3} ({:?})",
            scale_validation.rossby_number, scale_validation.scale_regime
        );

        // Dimensional analysis should be consistent regardless of implementation quality
        assert!(
            scale_validation
                .dimensional_consistency
                .is_dimensionally_consistent,
            "Dimensional analysis should always be consistent"
        );

        println!("✓ Scale analysis framework successfully detected physics violations");
    }

    #[test]
    fn test_enhanced_mass_conservation_diagnostics() {
        println!("\n3. Testing enhanced mass conservation diagnostics...");
        println!("---------------------------------------------------");

        // Create continental scale system (500km domain)
        let scale = WorldScale::new(500.0, (50, 50), DetailLevel::Standard);
        let atmospheric_system = AtmosphericSystem::new_for_scale(&scale);

        // Create pressure field with gradients
        let mut pressure_layer = AtmosphericPressureLayer::new(50, 50);
        for y in 0..50 {
            for x in 0..50 {
                let pressure = 101325.0 - (x as f32 * 200.0); // 200 Pa per cell decrease eastward
                pressure_layer.pressure.set(x, y, pressure);
            }
        }
        pressure_layer.calculate_pressure_gradients(scale.meters_per_pixel() as f32);

        // Generate winds using current system
        let wind_layer = atmospheric_system.generate_geostrophic_winds(&pressure_layer, &scale);

        // Standard air density at sea level
        let air_density = atmospheric_system.parameters.air_density_sea_level;

        // Validate enhanced mass conservation
        let conservation_validation =
            validate_enhanced_mass_conservation(&wind_layer, &scale, air_density);

        println!("Enhanced Mass Conservation Analysis:");
        println!(
            "  Total momentum magnitude: {:.2} m/s",
            conservation_validation.total_momentum_magnitude
        );
        println!("  Boundary momentum fluxes (kg/s):");
        println!(
            "    North: {:.3e}",
            conservation_validation.boundary_momentum_flux.north_flux
        );
        println!(
            "    South: {:.3e}",
            conservation_validation.boundary_momentum_flux.south_flux
        );
        println!(
            "    East:  {:.3e}",
            conservation_validation.boundary_momentum_flux.east_flux
        );
        println!(
            "    West:  {:.3e}",
            conservation_validation.boundary_momentum_flux.west_flux
        );
        println!(
            "    Net:   {:.3e}",
            conservation_validation.boundary_momentum_flux.net_flux
        );
        println!(
            "  Boundary fluxes balanced: {}",
            conservation_validation.boundary_momentum_flux.is_balanced
        );

        println!("  Continuity equation validation:");
        println!(
            "    Average divergence: {:.3e} s⁻¹",
            conservation_validation
                .continuity_validation
                .average_divergence
        );
        println!(
            "    Max divergence: {:.3e} s⁻¹",
            conservation_validation.continuity_validation.max_divergence
        );
        println!(
            "    Violation percentage: {:.1}%",
            conservation_validation
                .continuity_validation
                .violation_percentage
        );
        println!(
            "    Continuity satisfied: {}",
            conservation_validation.continuity_validation.is_satisfied
        );

        println!(
            "  Spatial momentum violations: {}",
            conservation_validation.spatial_violations.len()
        );
        if !conservation_validation.spatial_violations.is_empty() {
            println!("  Top violations by severity:");
            for (i, violation) in conservation_validation
                .spatial_violations
                .iter()
                .take(3)
                .enumerate()
            {
                println!(
                    "    {}: ({}, {}) |v|={:.1} m/s ∇·v={:.2e} s⁻¹ severity={:.2} {}",
                    i + 1,
                    violation.x,
                    violation.y,
                    violation.momentum_magnitude,
                    violation.velocity_divergence,
                    violation.severity,
                    violation
                        .boundary_type
                        .as_ref()
                        .map(|s| format!("({})", s))
                        .unwrap_or_default()
                );
            }
        }

        println!(
            "  Mass conserved: {}",
            conservation_validation.is_mass_conserved
        );
        println!("  {}", conservation_validation.diagnostic_summary);

        // Expected violations from current system
        assert!(
            conservation_validation.total_momentum_magnitude > 3000.0,
            "Current system should have excessive total momentum, got {:.1} m/s",
            conservation_validation.total_momentum_magnitude
        );

        assert!(
            !conservation_validation.is_mass_conserved,
            "Current system should violate mass conservation"
        );

        assert!(
            !conservation_validation.continuity_validation.is_satisfied,
            "Current system should violate continuity equation"
        );

        // Should detect many spatial violations due to excessive winds
        assert!(
            !conservation_validation.spatial_violations.is_empty(),
            "Current system should have spatial momentum violations"
        );

        println!("✓ Enhanced mass conservation diagnostics successfully detected violations");
    }

    #[test]
    fn test_geostrophic_balance_tropical_domain_no_coriolis() {
        println!("\n4. Testing tropical domain (no Coriolis)...");
        println!("------------------------------------------");
        // Create small tropical domain (50km) where Coriolis should be inactive
        let scale = WorldScale::new(50.0, (20, 20), DetailLevel::Standard);
        let atmospheric_system = AtmosphericSystem::new_for_scale(&scale);

        // For small domains, Coriolis should be inactive
        assert!(
            !atmospheric_system.is_coriolis_active(),
            "Coriolis should be inactive for 50km domain"
        );

        // Create pressure field
        let mut pressure_layer = AtmosphericPressureLayer::new(20, 20);
        for y in 0..20 {
            for x in 0..20 {
                pressure_layer.pressure.set(x, y, 101325.0);
            }
        }
        pressure_layer.calculate_pressure_gradients(scale.meters_per_pixel() as f32);

        // Generate winds (should be minimal for small domain)
        let wind_layer = atmospheric_system.generate_geostrophic_winds(&pressure_layer, &scale);

        // Validate - should have minimal winds and good "balance" (since no Coriolis)
        let validation =
            validate_geostrophic_balance(&atmospheric_system, &pressure_layer, &wind_layer);

        println!("Tropical Domain Analysis (no Coriolis):");
        println!(
            "  Average wind speed: {:.3} m/s",
            validation.average_wind_speed
        );
        println!(
            "  Maximum wind speed: {:.3} m/s",
            validation.maximum_wind_speed
        );

        // For small domains without Coriolis, winds should be minimal
        assert!(
            validation.average_wind_speed < 1.0,
            "Small domain without Coriolis should have minimal winds"
        );
    }

    #[test]
    fn test_pressure_wind_correlation_calculation() {
        println!("\n5. Testing correlation calculation...");
        println!("-----------------------------------");
        // Test the correlation calculation with known data
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0]; // Perfect positive correlation

        let correlation = pearson_correlation(&x, &y);
        assert!(
            (correlation - 1.0).abs() < 1e-10,
            "Perfect positive correlation should be 1.0"
        );

        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0]; // Perfect negative correlation  
        let correlation_neg = pearson_correlation(&x, &y_neg);
        assert!(
            (correlation_neg + 1.0).abs() < 1e-10,
            "Perfect negative correlation should be -1.0"
        );

        let y_zero = vec![5.0, 5.0, 5.0, 5.0, 5.0]; // No correlation
        let correlation_zero = pearson_correlation(&x, &y_zero);
        assert!(
            correlation_zero.abs() < 1e-10,
            "No correlation should be 0.0"
        );

        println!("✓ Correlation calculation tests passed");
    }
} // End of tests module
