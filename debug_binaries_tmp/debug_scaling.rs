fn main() {
    let reference_cells = 240.0 * 120.0; // 28,800
    let large_cells = 1024.0 * 512.0; // 524,288

    let scale_factor = reference_cells / large_cells;
    let base_rainfall = 0.002;
    let effective_rainfall = base_rainfall * scale_factor;

    println!("Reference cells: {}", reference_cells);
    println!("Large map cells: {}", large_cells);
    println!("Scale factor: {}", scale_factor);
    println!("Base rainfall: {}", base_rainfall);
    println!("Effective rainfall: {}", effective_rainfall);
    println!("Ratio: {:.6}", effective_rainfall / base_rainfall);
}
