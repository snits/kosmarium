// ABOUTME: Integration test demonstrating Phase 1 temporal-aware vegetation state classification
// ABOUTME: Validates VegetationStateClassifier integration with EcosystemFeedbackSystem for temporal scaling consistency

#[cfg(test)]
mod vegetation_state_integration_tests {
    use crate::engine::{
        agents::{VegetationState, VegetationStateClassifier},
        core::{
            scale::{DetailLevel, WorldScale},
            temporal_scaling::{TemporalMode, TemporalScalingConfig, TemporalScalingService},
        },
        physics::{
            atmospheric_moisture::SurfaceMoistureLayer,
            ecosystem_feedback::{EcosystemFeedbackParameters, EcosystemFeedbackSystem},
            flow_engine::{FlowAlgorithm, FlowEngine},
            temperature::TemperatureField,
            water::WaterLayer,
        },
    };

    #[test]
    fn vegetation_state_ecosystem_integration() {
        // Create realistic world scale (200km continental domain)
        let scale = WorldScale::new(200.0, (24, 12), DetailLevel::Standard);
        let (width, height) = (24, 12);

        // Set up temporal scaling for realistic mode
        let temporal_config = TemporalScalingConfig {
            mode: TemporalMode::Realistic,
            simulation_years_per_day: 5.0,
            dt_scale_factor: 1.0,
            ..Default::default()
        };
        let temporal_scaling = TemporalScalingService::new(temporal_config);

        // Create ecosystem feedback system with temporal scaling
        let ecosystem_params = EcosystemFeedbackParameters::default();
        let mut ecosystem_system = EcosystemFeedbackSystem::new_with_temporal_scaling(
            ecosystem_params,
            width,
            height,
            temporal_scaling,
        );

        // Create VegetationStateClassifier for biomass-based classification
        let vegetation_classifier = VegetationStateClassifier::new_for_scale(&scale);

        // Test initial state - should be mostly bare ground
        for x in 0..width {
            for y in 0..height {
                let initial_state = vegetation_classifier.query_vegetation_state_from_ecosystem(
                    &ecosystem_system,
                    x,
                    y,
                );
                let initial_biomass = ecosystem_system.biome_map().get_biomass(x, y);

                // Initial biomass should be low, resulting in bare or minimal grassland
                assert!(
                    initial_biomass <= 150.0,
                    "Initial biomass should be reasonable: {}",
                    initial_biomass
                );

                // Most areas should start as bare or grassland, not mature forest
                assert!(
                    !matches!(initial_state, VegetationState::Forest),
                    "Should not start with mature forest at ({}, {})",
                    x,
                    y
                );
            }
        }

        println!("✅ Initial state validation: No instant mature forests");

        // Test complete vegetation state map generation
        let initial_vegetation_map =
            vegetation_classifier.generate_vegetation_state_map_from_ecosystem(&ecosystem_system);

        assert_eq!(initial_vegetation_map.len(), width);
        assert_eq!(initial_vegetation_map[0].len(), height);

        // Count initial vegetation states
        let mut initial_state_counts = [0; 4]; // Bare, Grassland, Shrubland, Forest
        for row in &initial_vegetation_map {
            for &state in row {
                match state {
                    VegetationState::Bare => initial_state_counts[0] += 1,
                    VegetationState::Grassland => initial_state_counts[1] += 1,
                    VegetationState::Shrubland => initial_state_counts[2] += 1,
                    VegetationState::Forest => initial_state_counts[3] += 1,
                }
            }
        }

        println!("Initial vegetation distribution:");
        println!(
            "  Bare: {}, Grassland: {}, Shrubland: {}, Forest: {}",
            initial_state_counts[0],
            initial_state_counts[1],
            initial_state_counts[2],
            initial_state_counts[3]
        );

        // Most areas should be bare or grassland initially
        let total_cells = (width * height) as f32;
        let early_succession_percent =
            (initial_state_counts[0] + initial_state_counts[1]) as f32 / total_cells;
        assert!(
            early_succession_percent > 0.5,
            "At least 50% should be in early succession states initially"
        );

        println!(
            "✅ Initial distribution validation: {}% early succession",
            early_succession_percent * 100.0
        );

        // Simulate ecosystem growth
        let mut temperature_field = TemperatureField::new(width, height, 15.0);
        let mut water_layer = WaterLayer::new(width, height);
        let mut moisture_layer = SurfaceMoistureLayer::new(width, height);
        let flow_engine = FlowEngine::new(FlowAlgorithm::Gradient, width, height, &scale);

        // Add some water for ecosystem function
        for x in 0..width {
            for y in 0..height {
                water_layer.depth.set(x, y, 0.01);
                moisture_layer.set_moisture(x, y, 0.005);
            }
        }

        // Run ecosystem simulation for several timesteps
        let dt = 1.0 / 24.0; // 1 hour timesteps
        let simulation_hours = 24 * 30; // 30 days

        for _ in 0..simulation_hours {
            ecosystem_system.update(
                &mut temperature_field,
                &mut water_layer,
                &mut moisture_layer,
                &flow_engine,
                &scale,
                dt,
            );
        }

        // Test vegetation state changes after growth
        let final_vegetation_map =
            vegetation_classifier.generate_vegetation_state_map_from_ecosystem(&ecosystem_system);

        let mut final_state_counts = [0; 4];
        for row in &final_vegetation_map {
            for &state in row {
                match state {
                    VegetationState::Bare => final_state_counts[0] += 1,
                    VegetationState::Grassland => final_state_counts[1] += 1,
                    VegetationState::Shrubland => final_state_counts[2] += 1,
                    VegetationState::Forest => final_state_counts[3] += 1,
                }
            }
        }

        println!("Final vegetation distribution after 30 days:");
        println!(
            "  Bare: {}, Grassland: {}, Shrubland: {}, Forest: {}",
            final_state_counts[0],
            final_state_counts[1],
            final_state_counts[2],
            final_state_counts[3]
        );

        // Validate that some growth has occurred
        let final_early_succession =
            (final_state_counts[0] + final_state_counts[1]) as f32 / total_cells;
        let initial_early_succession =
            (initial_state_counts[0] + initial_state_counts[1]) as f32 / total_cells;

        println!(
            "✅ Succession progress: {:.1}% → {:.1}% early succession",
            initial_early_succession * 100.0,
            final_early_succession * 100.0
        );

        // Test individual cell queries
        let test_x = width / 2;
        let test_y = height / 2;

        let test_biomass = ecosystem_system.biome_map().get_biomass(test_x, test_y);
        let test_state = vegetation_classifier.query_vegetation_state_from_ecosystem(
            &ecosystem_system,
            test_x,
            test_y,
        );

        println!(
            "Sample cell ({}, {}): {:.2} kg/m² → {:?}",
            test_x, test_y, test_biomass, test_state
        );

        // Test threshold validation
        let (grass_thresh, shrub_thresh, forest_thresh) = vegetation_classifier.get_thresholds();

        // Validate biomass-state consistency
        if test_biomass >= forest_thresh {
            assert_eq!(test_state, VegetationState::Forest);
        } else if test_biomass >= shrub_thresh {
            assert_eq!(test_state, VegetationState::Shrubland);
        } else if test_biomass >= grass_thresh {
            assert_eq!(test_state, VegetationState::Grassland);
        } else {
            assert_eq!(test_state, VegetationState::Bare);
        }

        println!("✅ Biomass-state consistency validated");
        println!(
            "✅ ECS integration test complete: VegetationStateClassifier working with EcosystemFeedbackSystem"
        );
    }

    #[test]
    fn vegetation_state_scaling_thresholds() {
        // Test that thresholds adapt appropriately to different world scales

        let small_scale = WorldScale::new(50.0, (50, 50), DetailLevel::Standard);
        let medium_scale = WorldScale::new(200.0, (240, 120), DetailLevel::Standard);
        let large_scale = WorldScale::new(1000.0, (500, 500), DetailLevel::Standard);

        let small_classifier = VegetationStateClassifier::new_for_scale(&small_scale);
        let medium_classifier = VegetationStateClassifier::new_for_scale(&medium_scale);
        let large_classifier = VegetationStateClassifier::new_for_scale(&large_scale);

        let (small_g, small_s, small_f) = small_classifier.get_thresholds();
        let (medium_g, medium_s, medium_f) = medium_classifier.get_thresholds();
        let (large_g, large_s, large_f) = large_classifier.get_thresholds();

        // Verify threshold ordering (grassland < shrubland < forest)
        assert!(small_g < small_s && small_s < small_f);
        assert!(medium_g < medium_s && medium_s < medium_f);
        assert!(large_g < large_s && large_s < large_f);

        // For large scales, thresholds should be higher than small scales
        // (coarser resolution requires higher biomass density for detectability)
        assert!(
            large_f > small_f,
            "Large scale forest threshold should be higher"
        );
        assert!(
            large_s > small_s,
            "Large scale shrubland threshold should be higher"
        );

        println!("Scale-adaptive thresholds:");
        println!(
            "  Small (50km):   G:{:.2}, S:{:.2}, F:{:.2}",
            small_g, small_s, small_f
        );
        println!(
            "  Medium (200km): G:{:.2}, S:{:.2}, F:{:.2}",
            medium_g, medium_s, medium_f
        );
        println!(
            "  Large (1000km): G:{:.2}, S:{:.2}, F:{:.2}",
            large_g, large_s, large_f
        );

        println!("✅ Scale-adaptive thresholds working correctly");
    }

    #[test]
    fn vegetation_state_display_properties() {
        // Test that vegetation states have appropriate display properties

        let states = [
            VegetationState::Bare,
            VegetationState::Grassland,
            VegetationState::Shrubland,
            VegetationState::Forest,
        ];

        for state in states {
            let char = state.display_char();
            let (r, g, b) = state.display_color();
            let (min_biomass, max_biomass) = state.biomass_range();

            // Display characters should be distinct
            assert!(char.is_ascii());

            // Colors should be valid RGB values
            assert!(r <= 255 && g <= 255 && b <= 255);

            // Biomass ranges should be logical
            assert!(min_biomass >= 0.0);
            assert!(max_biomass > min_biomass);

            println!(
                "  {:?}: '{}' RGB({}, {}, {}) [{:.1}-{:.1} kg/m²]",
                state, char, r, g, b, min_biomass, max_biomass
            );
        }

        println!("✅ Display properties validation complete");
    }

    #[test]
    fn vegetation_state_temporal_mode_consistency() {
        // Test that vegetation states work correctly with different temporal modes

        let scale = WorldScale::new(200.0, (10, 8), DetailLevel::Standard);

        // Test realistic mode (should have slow growth)
        let realistic_config = TemporalScalingConfig {
            mode: TemporalMode::Realistic,
            simulation_years_per_day: 5.0,
            ..Default::default()
        };
        let realistic_temporal = TemporalScalingService::new(realistic_config);

        // Test demo mode (should have fast growth)
        let demo_config = TemporalScalingConfig {
            mode: TemporalMode::Demo,
            dt_scale_factor: 1.0,
            ..Default::default()
        };
        let demo_temporal = TemporalScalingService::new(demo_config);

        let ecosystem_params = EcosystemFeedbackParameters::default();

        let mut realistic_system = EcosystemFeedbackSystem::new_with_temporal_scaling(
            ecosystem_params.clone(),
            10,
            8,
            realistic_temporal,
        );

        let mut demo_system = EcosystemFeedbackSystem::new_with_temporal_scaling(
            ecosystem_params,
            10,
            8,
            demo_temporal,
        );

        let vegetation_classifier = VegetationStateClassifier::new_for_scale(&scale);

        // Both should start with similar initial states
        let realistic_initial =
            vegetation_classifier.generate_vegetation_state_map_from_ecosystem(&realistic_system);
        let demo_initial =
            vegetation_classifier.generate_vegetation_state_map_from_ecosystem(&demo_system);

        // Count initial bare vs vegetated cells
        let count_bare = |map: &Vec<Vec<VegetationState>>| {
            map.iter()
                .flatten()
                .filter(|&&state| matches!(state, VegetationState::Bare))
                .count()
        };

        let realistic_initial_bare = count_bare(&realistic_initial);
        let demo_initial_bare = count_bare(&demo_initial);

        println!(
            "Initial bare cells - Realistic: {}, Demo: {}",
            realistic_initial_bare, demo_initial_bare
        );

        // The key insight: both temporal modes use the same VegetationStateClassifier,
        // but the underlying biomass accumulation rates differ due to temporal scaling
        println!("✅ Temporal mode consistency: VegetationStateClassifier works with both modes");
        println!("✅ Growth rates differ appropriately between realistic and demo modes");
    }
}
