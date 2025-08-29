// Debug coordinate transformation between nested Vec and PhysicsGrid

use kosmarium::engine::core::PhysicsGrid;

#[test]
fn test_coordinate_transformation_debugging() {
    println!("Testing coordinate transformation between Vec<Vec<T>> and PhysicsGrid<T>");

    // Create test data with known pattern
    let nested = vec![
        vec![10, 11, 12], // y=0: [10, 11, 12]
        vec![20, 21, 22], // y=1: [20, 21, 22]
        vec![30, 31, 32], // y=2: [30, 31, 32]
    ];

    println!("Original nested Vec:");
    for (y, row) in nested.iter().enumerate() {
        println!("  y={}: {:?}", y, row);
    }

    // Convert to PhysicsGrid
    let grid = PhysicsGrid::from_nested(nested.clone());

    println!("\nPhysicsGrid access comparison:");
    for y in 0..3 {
        for x in 0..3 {
            let nested_value = nested[y][x];
            let grid_value = *grid.get(x, y);
            println!(
                "  ({}, {}): nested={}, grid={}, match={}",
                x,
                y,
                nested_value,
                grid_value,
                nested_value == grid_value
            );

            assert_eq!(
                nested_value, grid_value,
                "Coordinate transformation failed at ({}, {})",
                x, y
            );
        }
    }

    // Convert back to nested
    let back_to_nested = grid.to_nested();
    println!("\nConverted back to nested:");
    for (y, row) in back_to_nested.iter().enumerate() {
        println!("  y={}: {:?}", y, row);
    }

    // Test equality
    assert_eq!(nested, back_to_nested, "Round-trip conversion failed");
    println!("\n✓ Round-trip conversion successful");
}
