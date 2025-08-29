// ABOUTME: Demo binary showcasing thermal circulation coupling system
// ABOUTME: Demonstrates temperature-driven atmospheric flow patterns and visualization

use kosmarium::engine::{
    core::{
        PhysicsGrid,
        heightmap::HeightMap,
        scale::{DetailLevel, WorldScale},
    },
    physics::{
        atmosphere::AtmosphericSystem,
        climate::{AtmosphericPressureLayer, ClimateSystem, TemperatureLayer},
        flow_engine::FlowEngine,
        thermal_circulation::{
            ThermalCirculationEffects, ThermalCirculationParameters, ThermalCirculationSystem,
        },
        water::Vec2,
    },
    rendering::ascii_render,
};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌡️  Thermal Circulation Coupling Demo");
    println!("=====================================");
    println!();

    // Create world scale and dimensions
    let width = 40;
    let height = 30;
    let scale = WorldScale::new(2000.0, (width, height), DetailLevel::Standard);

    println!("🗺️  World Setup:");
    println!("   Dimensions: {}x{} cells", width, height);
    println!("   Scale: {:.1} meters/pixel", scale.meters_per_pixel());
    println!(
        "   Total area: {:.1} km²",
        (scale.meters_per_pixel() * width as f64 * scale.meters_per_pixel() * height as f64)
            / 1_000_000.0
    );
    println!();

    // Create demo scenarios
    demo_continental_thermal_circulation(width as usize, height as usize, &scale)?;
    demo_island_heat_effects(width as usize, height as usize, &scale)?;
    demo_seasonal_thermal_patterns(width as usize, height as usize, &scale)?;

    println!("✅ Thermal circulation demonstrations completed!");
    println!();
    println!("🔬 Key Physics Demonstrated:");
    println!("   • Temperature-driven buoyancy forces (F = ρ*g*β*ΔT)");
    println!("   • Pressure gradient responses to thermal anomalies");
    println!("   • Convection cell formation and circulation patterns");
    println!("   • Thermal diffusion smoothing for numerical stability");
    println!("   • Integration with unified FlowEngine architecture");

    Ok(())
}

fn demo_continental_thermal_circulation(
    width: usize,
    height: usize,
    scale: &WorldScale,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 Demo 1: Continental Thermal Circulation");
    println!("   Scenario: Land-sea thermal contrast driving circulation");
    println!();

    // Create heightmap with land and sea
    let mut heightmap_data = vec![vec![0.0; height]; width];
    for x in 0..width {
        for y in 0..height {
            let elevation = if x < width / 3 {
                -50.0 // Ocean on the left
            } else {
                100.0 + (x as f32 - width as f32 / 3.0) * 2.0 // Rising land to the right
            };
            heightmap_data[x][y] = elevation;
        }
    }
    let _heightmap = HeightMap::from_nested(heightmap_data);

    // Create temperature layer with land-sea contrast
    let mut temp_layer = TemperatureLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let temperature = if x < width / 3 {
                18.0 // Cooler ocean
            } else {
                25.0 + (x as f32 - width as f32 / 3.0) / 10.0 // Warmer land, getting hotter inland
            };
            temp_layer.temperature.set(x, y, temperature);
        }
    }

    // Set up simulation systems
    let mut flow_engine = FlowEngine::for_climate(width, height, scale);
    let mut pressure_layer = AtmosphericPressureLayer {
        pressure: PhysicsGrid::new(width, height, 101325.0),
        pressure_gradient: PhysicsGrid::new(width, height, Vec2::new(0.0, 0.0)),
    };
    let climate_system = ClimateSystem::new_for_scale(scale);

    // Initialize thermal circulation with continental parameters
    let mut params = ThermalCirculationParameters::default();
    params.reference_temperature_difference = 8.0; // Sensitive to land-sea contrasts
    params.buoyancy_coefficient = 0.05; // Enhanced buoyancy for demo
    let mut thermal_system = ThermalCirculationSystem::new(params);

    // Run thermal circulation simulation
    println!("   🔥 Generating thermal circulation patterns...");
    thermal_system.update(
        &temp_layer,
        &mut flow_engine,
        &mut pressure_layer,
        &climate_system,
        scale,
        300.0, // 5-minute integration for continental patterns
    );

    // Display results
    display_thermal_analysis(&thermal_system, width, height, "Continental")?;
    display_temperature_field(&temp_layer, width, height)?;

    println!("   📊 Analysis: Ocean-land thermal contrast creates circulation cells");
    println!("               Warm land creates low pressure, driving onshore flow");
    println!();

    Ok(())
}

fn demo_island_heat_effects(
    width: usize,
    height: usize,
    scale: &WorldScale,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏝️  Demo 2: Island Heat Island Effects");
    println!("   Scenario: Urban heat island creating local circulation");
    println!();

    // Create temperature layer with central hotspot (urban heat island)
    let mut temp_layer = TemperatureLayer::new(width, height);
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let base_temp = 22.0;

    for x in 0..width {
        for y in 0..height {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            let max_radius = 8.0;

            // Create heat island: hot center, cooler surroundings
            let temperature = if distance < max_radius {
                let heat_factor = 1.0 - (distance / max_radius);
                base_temp + heat_factor * 8.0 // Up to 8°C warmer in center
            } else {
                base_temp - 2.0 // Cooler rural surroundings
            };

            temp_layer.temperature.set(x, y, temperature);
        }
    }

    // Set up simulation systems
    let mut flow_engine = FlowEngine::for_climate(width, height, scale);
    let mut pressure_layer = AtmosphericPressureLayer {
        pressure: PhysicsGrid::new(width, height, 101325.0),
        pressure_gradient: PhysicsGrid::new(width, height, Vec2::new(0.0, 0.0)),
    };
    let climate_system = ClimateSystem::new_for_scale(scale);

    // Initialize thermal circulation optimized for heat islands
    let mut params = ThermalCirculationParameters::default();
    params.reference_temperature_difference = 5.0; // Sensitive to urban temperature differences
    params.buoyancy_coefficient = 0.08; // Strong buoyancy response
    params.convection_aspect_ratio = 1.5; // Compact convection cells
    let mut thermal_system = ThermalCirculationSystem::new(params);

    // Run thermal circulation simulation
    println!("   🏙️  Simulating urban heat island circulation...");
    thermal_system.update(
        &temp_layer,
        &mut flow_engine,
        &mut pressure_layer,
        &climate_system,
        scale,
        600.0, // 10-minute integration for urban-scale patterns
    );

    // Display results
    display_thermal_analysis(&thermal_system, width, height, "Heat Island")?;
    display_convection_patterns(&thermal_system, width, height)?;

    println!("   📊 Analysis: Central heating creates strong updrafts and convergence");
    println!("               Classic heat island circulation with inward surface flow");
    println!();

    Ok(())
}

fn demo_seasonal_thermal_patterns(
    width: usize,
    height: usize,
    scale: &WorldScale,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌱 Demo 3: Seasonal Thermal Patterns");
    println!("   Scenario: Complex seasonal temperature distribution");
    println!();

    // Create complex seasonal temperature pattern
    let mut temp_layer = TemperatureLayer::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let lat_factor = (y as f32 / height as f32) * 2.0 - 1.0; // -1 to +1 (south to north)
            let elevation_factor = (x as f32 / width as f32) * 0.5; // 0 to 0.5 (west to east)

            // Complex temperature pattern: latitude + seasonal + elevation effects
            let base_temp = 15.0;
            let latitude_effect = lat_factor * -10.0; // Cooler in the north
            let seasonal_effect = if y < height / 2 { 5.0 } else { -3.0 }; // South warm, north cool
            let elevation_effect = elevation_factor * -8.0; // Cooler at higher elevation

            let temperature = base_temp + latitude_effect + seasonal_effect + elevation_effect;
            temp_layer.temperature.set(x, y, temperature);
        }
    }

    // Set up simulation systems
    let mut flow_engine = FlowEngine::for_climate(width, height, scale);
    let mut pressure_layer = AtmosphericPressureLayer {
        pressure: PhysicsGrid::new(width, height, 101325.0),
        pressure_gradient: PhysicsGrid::new(width, height, Vec2::new(0.0, 0.0)),
    };
    let climate_system = ClimateSystem::new_for_scale(scale);

    // Initialize thermal circulation for seasonal patterns
    let mut params = ThermalCirculationParameters::default();
    params.reference_temperature_difference = 12.0; // Large seasonal temperature ranges
    params.thermal_diffusion_rate = 0.2; // Moderate smoothing for seasonal patterns
    let mut thermal_system = ThermalCirculationSystem::new(params);

    // Run thermal circulation simulation
    println!("   🍂 Computing seasonal thermal circulation...");
    thermal_system.update(
        &temp_layer,
        &mut flow_engine,
        &mut pressure_layer,
        &climate_system,
        scale,
        1800.0, // 30-minute integration for regional patterns
    );

    // Display results
    display_thermal_analysis(&thermal_system, width, height, "Seasonal")?;
    display_pressure_field(&pressure_layer, width, height)?;

    println!("   📊 Analysis: Complex temperature gradients create multiple circulation cells");
    println!("               Seasonal patterns drive large-scale atmospheric dynamics");
    println!();

    Ok(())
}

fn display_thermal_analysis(
    thermal_system: &ThermalCirculationSystem,
    width: usize,
    height: usize,
    scenario: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !thermal_system.has_active_effects() {
        println!("   ⚠️  No thermal effects generated");
        return Ok(());
    }

    let effects = thermal_system.get_effects().unwrap();

    // Calculate thermal circulation statistics
    let mut max_gradient = 0.0f32;
    let mut max_velocity = 0.0f32;
    let mut max_buoyancy = 0.0f32;
    let mut total_thermal_energy = 0.0f64;
    let mut rising_cells = 0;
    let mut sinking_cells = 0;

    for x in 0..width {
        for y in 0..height {
            let gradient = effects.get_temperature_gradient(x, y);
            let velocity = effects.get_thermal_velocity(x, y);
            let buoyancy = effects.get_buoyancy_force(x, y);
            let convection = effects.get_convection_cell(x, y);

            max_gradient = max_gradient.max(gradient);
            max_velocity = max_velocity.max(velocity.magnitude());
            max_buoyancy = max_buoyancy.max(buoyancy.abs());
            let magnitude = velocity.magnitude();
            total_thermal_energy += (magnitude * magnitude) as f64;

            if convection > 0.1 {
                rising_cells += 1;
            } else if convection < -0.1 {
                sinking_cells += 1;
            }
        }
    }

    println!("   📈 {} Thermal Analysis:", scenario);
    println!("      Max temperature gradient: {:.4} °C/m", max_gradient);
    println!("      Max thermal velocity: {:.2} m/s", max_velocity);
    println!("      Max buoyancy force: {:.2} N/kg", max_buoyancy);
    println!(
        "      Total thermal energy: {:.1} J/kg",
        total_thermal_energy
    );
    println!(
        "      Convection cells: {} rising, {} sinking",
        rising_cells, sinking_cells
    );

    Ok(())
}

fn display_temperature_field(
    temp_layer: &TemperatureLayer,
    width: usize,
    height: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🌡️  Temperature Field (°C):");

    // Find temperature range for scaling
    let mut min_temp = f32::INFINITY;
    let mut max_temp = f32::NEG_INFINITY;

    for x in 0..width {
        for y in 0..height {
            let temp = temp_layer.get_temperature(x, y);
            min_temp = min_temp.min(temp);
            max_temp = max_temp.max(temp);
        }
    }

    // Display temperature field with color coding
    for y in 0..height.min(20) {
        // Limit display size
        print!("      ");
        for x in 0..width.min(40) {
            let temp = temp_layer.get_temperature(x, y);
            let normalized = if max_temp > min_temp {
                (temp - min_temp) / (max_temp - min_temp)
            } else {
                0.5
            };

            let symbol = if normalized < 0.2 {
                "❄️" // Very cold
            } else if normalized < 0.4 {
                "🟦" // Cold
            } else if normalized < 0.6 {
                "🟨" // Moderate
            } else if normalized < 0.8 {
                "🟧" // Warm
            } else {
                "🔥" // Hot
            };

            print!("{}", symbol);
        }
        println!();
    }

    println!("      Range: {:.1}°C to {:.1}°C", min_temp, max_temp);
    println!();

    Ok(())
}

fn display_convection_patterns(
    thermal_system: &ThermalCirculationSystem,
    width: usize,
    height: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if !thermal_system.has_active_effects() {
        return Ok(());
    }

    let effects = thermal_system.get_effects().unwrap();

    println!("   ⬆️  Convection Pattern:");
    println!("      ⬆️ = Rising air, ⬇️ = Sinking air, ➡️ = Neutral");

    for y in 0..height.min(15) {
        // Limit display size
        print!("      ");
        for x in 0..width.min(30) {
            let convection = effects.get_convection_cell(x, y);
            let symbol = if convection > 0.1 {
                "⬆️"
            } else if convection < -0.1 {
                "⬇️"
            } else {
                "➡️"
            };
            print!("{}", symbol);
        }
        println!();
    }
    println!();

    Ok(())
}

fn display_pressure_field(
    pressure_layer: &AtmosphericPressureLayer,
    width: usize,
    height: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🌬️  Pressure Field (relative to standard):");

    let standard_pressure = 101325.0;

    for y in 0..height.min(15) {
        // Limit display size
        print!("      ");
        for x in 0..width.min(30) {
            let pressure = pressure_layer.pressure.get(x, y);
            let pressure_diff = pressure - standard_pressure;

            let symbol = if pressure_diff < -100.0 {
                "🔵" // Very low pressure
            } else if pressure_diff < -20.0 {
                "🟦" // Low pressure
            } else if pressure_diff < 20.0 {
                "⚪" // Normal pressure
            } else if pressure_diff < 100.0 {
                "🟧" // High pressure
            } else {
                "🔴" // Very high pressure
            };

            print!("{}", symbol);
        }
        println!();
    }
    println!();

    Ok(())
}
