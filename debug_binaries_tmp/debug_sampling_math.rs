fn main() {
    println!("=== Diamond-Square Sampling Mathematics Debug ===");

    // Simulate the exact sampling math used in sample_to_dimensions
    // Test with problematic case: 240x120 target dimensions

    let target_width = 240;
    let target_height = 120;

    // Diamond-Square generates on power-of-2 grids
    let power_size = ((target_width.max(target_height) as u32).next_power_of_two() as usize).max(8);
    println!("Target dimensions: {}x{}", target_width, target_height);
    println!("Power-of-2 source size: {}x{}", power_size, power_size);

    println!("\n=== Sampling coordinate calculation ===");

    // Check what source coordinates the first few target positions map to
    println!("Target (x,y) -> Source (src_x, src_y):");

    for target_y in 0..5 {
        for target_x in 0..5 {
            let src_x = (target_x * (power_size - 1)) / (target_width - 1).max(1);
            let src_y = (target_y * (power_size - 1)) / (target_height - 1).max(1);

            println!(
                "({:3},{:3}) -> ({:3},{:3})",
                target_x, target_y, src_x, src_y
            );
        }
    }

    // Check the exact calculation for (0,0)
    println!("\n=== Detailed (0,0) calculation ===");
    let src_x_00 = (0 * (power_size - 1)) / (target_width - 1).max(1);
    let src_y_00 = (0 * (power_size - 1)) / (target_height - 1).max(1);
    println!("(0,0) maps to source ({}, {})", src_x_00, src_y_00);

    // Test corner mappings
    println!("\n=== Corner mapping analysis ===");
    let corners = vec![
        (0, 0, "top-left"),
        (target_width - 1, 0, "top-right"),
        (0, target_height - 1, "bottom-left"),
        (target_width - 1, target_height - 1, "bottom-right"),
    ];

    for (x, y, name) in corners {
        let src_x = (x * (power_size - 1)) / (target_width - 1).max(1);
        let src_y = (y * (power_size - 1)) / (target_height - 1).max(1);
        println!(
            "{}: target({},{}) -> source({},{})",
            name, x, y, src_x, src_y
        );

        // Check if this is exactly a corner of the source grid
        let is_source_corner =
            (src_x == 0 || src_x == power_size - 1) && (src_y == 0 || src_y == power_size - 1);
        println!("  Is source corner: {}", is_source_corner);
    }

    // Test with other problematic dimensions we found
    println!("\n=== Other dimension tests ===");
    let test_dimensions = vec![
        (240, 120), // Known problematic
        (16, 16),   // Known working
        (15, 15),   // Known working
        (32, 32),   // Known working
    ];

    for (w, h) in test_dimensions {
        let ps = ((w.max(h) as u32).next_power_of_two() as usize).max(8);
        let src_x = (0 * (ps - 1)) / (w - 1).max(1);
        let src_y = (0 * (ps - 1)) / (h - 1).max(1);

        println!(
            "{}x{} (power_size={}): (0,0) -> ({},{})",
            w, h, ps, src_x, src_y
        );
    }

    // Check if there's a specific pattern in the initial corner assignment
    println!("\n=== Initial corner assignment investigation ===");
    println!("In Diamond-Square, initial corners are assigned to:");
    println!("  map[0][0] = initial_corners[0] (top-left)");
    println!("  map[0][max_index] = initial_corners[1] (top-right)");
    println!("  map[max_index][0] = initial_corners[2] (bottom-left)");
    println!("  map[max_index][max_index] = initial_corners[3] (bottom-right)");

    println!(
        "\nFor power_size = {}, max_index = {}",
        power_size,
        power_size - 1
    );
    println!(
        "Initial corners are at: (0,0), (0,{}), ({},0), ({},{})",
        power_size - 1,
        power_size - 1,
        power_size - 1,
        power_size - 1
    );

    // This suggests the issue: when sampling (0,0) from target to source,
    // if it maps to a source corner that was initialized with a high value,
    // and the normalization makes that corner the maximum, we get the artifact.

    println!("\n=== Hypothesis ===");
    println!("The (0,0) artifact occurs when:");
    println!("1. Target (0,0) maps to source (0,0) - the top-left corner");
    println!("2. The initial_corners[0] value contributes to making that position");
    println!("   relatively high after Diamond-Square generation");
    println!("3. The normalization process makes that position the maximum");
    println!("4. Larger grids seem more susceptible, possibly due to how the");
    println!("   Diamond-Square algorithm evolves from the initial corners");
}
