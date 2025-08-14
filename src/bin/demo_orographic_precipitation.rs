// ABOUTME: Demonstration of orographic precipitation coupling - shows terrain-driven rainfall patterns
// ABOUTME: Creates mountain ridge scenario and demonstrates windward/leeward precipitation effects

use sim_prototype::engine::{
    core::{
        heightmap::HeightMap,
        math::Vec2,
        scale::{DetailLevel, WorldScale},
    },
    physics::{
        atmospheric_moisture::AtmosphericMoistureSystem,
        climate::ClimateSystem,
        flow_engine::{FlowAlgorithm, FlowEngine},
        orographic_precipitation::{OrographicParameters, OrographicPrecipitationSystem},
        water::WaterLayer,
    },
};

fn main() {
    println!("🏔️  Orographic Precipitation Coupling Demonstration");
    println!("===================================================");

    // Create mountain ridge domain: 20km x 20km at 1km resolution
    let scale = WorldScale::new(20.0, (20, 20), DetailLevel::Standard);
    println!(
        "Domain: {:.0}km x {:.0}km at {:.0}m resolution",
        scale.physical_size_km,
        scale.physical_size_km,
        scale.meters_per_pixel()
    );

    // Create west-to-east mountain ridge terrain
    let mut terrain_data = vec![vec![0.0; 20]; 20];
    for x in 0..20 {
        for y in 0..20 {
            // Create mountain ridge running north-south at x=10-12
            let distance_from_ridge = if x >= 8 && x <= 12 {
                // Mountain ridge - elevation increases toward center
                let ridge_position = (x as f32 - 10.0).abs(); // 0 at center, 2 at edges
                let ridge_height = (1.0 - ridge_position / 2.0).max(0.0); // 1.0 at center, 0.0 at edges
                0.4 + ridge_height * 0.5 // Base elevation 0.4, peak at 0.9
            } else {
                // Plains and valleys
                let valley_depth = if x < 8 {
                    0.2 + (x as f32) * 0.02 // Gradual rise from west
                } else {
                    0.4 - ((x - 12) as f32) * 0.015 // Gradual descent to east
                };
                valley_depth.max(0.1)
            };
            terrain_data[x][y] = distance_from_ridge;
        }
    }

    let heightmap = HeightMap::from_nested(terrain_data);

    // Print terrain profile (west to east at y=10)
    println!("\nTerrain Profile (West → East):");
    print!("  ");
    for x in 0..20 {
        if x % 5 == 0 {
            print!("{:>4}", x);
        } else {
            print!("   .");
        }
    }
    println!();
    print!("  ");
    for x in 0..20 {
        let elevation = heightmap.get(x, 10);
        let symbol = if elevation > 0.7 {
            "^^^"
        } else if elevation > 0.5 {
            "+++"
        } else if elevation > 0.3 {
            "---"
        } else {
            "..."
        };
        print!("{}", symbol);
    }
    println!(" (elevation)");

    // Initialize atmospheric systems
    let mut atmospheric_moisture = AtmosphericMoistureSystem::new_for_scale(&scale, 20, 20);
    let water_layer = WaterLayer::new(20, 20);
    atmospheric_moisture.initialize_from_terrain(&heightmap, &water_layer);

    // Set moderate humidity across domain
    for x in 0..20 {
        for y in 0..20 {
            atmospheric_moisture
                .surface_moisture
                .set_humidity(x, y, 50.0);
        }
    }

    // Create steady westerly wind (5 m/s eastward)
    let mut flow_engine = FlowEngine::new(FlowAlgorithm::Gradient, 20, 20, &scale);
    for x in 0..20 {
        for y in 0..20 {
            flow_engine
                .velocity_field
                .set_velocity(x, y, Vec2::new(5.0, 0.0));
        }
    }

    let climate_system = ClimateSystem::new_for_scale(&scale);

    // Create and test orographic precipitation system
    let parameters = OrographicParameters::default();
    let mut orographic_system = OrographicPrecipitationSystem::new(parameters);

    println!("\nOrographic Parameters:");
    println!(
        "  Lifting Condensation Level: {:.0}m",
        orographic_system.parameters.lifting_condensation_level
    );
    println!(
        "  Dry Lapse Rate: {:.1}°C/100m",
        orographic_system.parameters.dry_lapse_rate * 100.0
    );
    println!(
        "  Moist Lapse Rate: {:.1}°C/100m",
        orographic_system.parameters.moist_lapse_rate * 100.0
    );
    println!(
        "  Minimum Wind Speed: {:.1} m/s",
        orographic_system.parameters.min_wind_speed
    );
    println!(
        "  Precipitation Efficiency: {:.0}%",
        orographic_system.parameters.precipitation_efficiency * 100.0
    );

    // Update orographic system
    orographic_system.update(
        &heightmap,
        &flow_engine,
        &mut atmospheric_moisture,
        &climate_system,
        &scale,
        0.5, // 30-minute time step
    );

    println!("\n🌧️  Orographic Precipitation Effects:");

    if orographic_system.has_active_effects() {
        let effects = orographic_system.get_effects().unwrap();

        // Show precipitation multiplier profile (west to east)
        println!("\nPrecipitation Enhancement (West → East):");
        print!("  ");
        for x in 0..20 {
            if x % 5 == 0 {
                print!("{:>5}", x);
            } else {
                print!("    .");
            }
        }
        println!();

        for y_sample in [5, 10, 15] {
            print!("{:2}: ", y_sample);
            for x in 0..20 {
                let multiplier = effects.get_precipitation_multiplier(x, y_sample);
                let symbol = if multiplier > 1.5 {
                    "+++++" // Strong enhancement
                } else if multiplier > 1.2 {
                    " +++ " // Moderate enhancement
                } else if multiplier > 0.8 {
                    " === " // Normal
                } else if multiplier > 0.5 {
                    " --- " // Rain shadow
                } else {
                    "  .  " // Strong shadow
                };
                print!("{}", symbol);
            }
            println!(" (y={})", y_sample);
        }

        // Analysis of effects
        let mut windward_enhancement: f32 = 0.0;
        let mut leeward_reduction: f32 = 100.0;
        let mut peak_effects: f32 = 0.0;

        for x in 0..20 {
            for y in 0..20 {
                let multiplier = effects.get_precipitation_multiplier(x, y);

                if x >= 6 && x <= 8 {
                    // Windward slopes
                    windward_enhancement = windward_enhancement.max(multiplier);
                }
                if x >= 13 && x <= 15 {
                    // Leeward slopes
                    leeward_reduction = leeward_reduction.min(multiplier);
                }
                if x >= 9 && x <= 11 {
                    // Mountain peaks
                    peak_effects = peak_effects.max(multiplier);
                }
            }
        }

        println!("\n📊 Effect Analysis:");
        println!(
            "  Maximum windward enhancement: {:.2}x",
            windward_enhancement
        );
        println!("  Mountain peak effects: {:.2}x", peak_effects);
        println!("  Minimum leeward reduction: {:.2}x", leeward_reduction);

        if windward_enhancement > 1.1 {
            println!("  ✓ Windward precipitation enhancement detected");
        }
        if leeward_reduction < 0.9 {
            println!("  ✓ Leeward rain shadow effects detected");
        }

        // Show vertical air motion
        println!("\n🌬️  Vertical Air Motion (Orographic Lifting):");
        let mut max_uplift: f32 = 0.0;
        let mut max_downdraft: f32 = 0.0;

        for x in 0..20 {
            for y in 0..20 {
                let vertical_vel = effects.get_vertical_velocity(x, y);
                if vertical_vel > max_uplift {
                    max_uplift = vertical_vel;
                }
                if vertical_vel < max_downdraft {
                    max_downdraft = vertical_vel;
                }
            }
        }

        println!("  Maximum upward velocity: {:.3} m/s", max_uplift);
        println!("  Maximum downward velocity: {:.3} m/s", max_downdraft);

        if max_uplift > 0.1 {
            println!("  ✓ Orographic lifting detected on windward slopes");
        }
    } else {
        println!("  ❌ No orographic effects detected (check wind speed and terrain)");
    }

    println!("\n✨ Orographic Precipitation Coupling Demonstration Complete!");
    println!("   This demonstrates the fifth cross-system physics coupling enabled");
    println!("   by Phase 2 architecture consolidation: terrain → wind → precipitation");
}
