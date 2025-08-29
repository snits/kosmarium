// Debug flow accumulation issue
use kosmarium::engine::physics::drainage::{FlowDirectionMap, FlowAccumulationMap};
use kosmarium::engine::core::heightmap::HeightMap;

fn main() {
    println!("Debugging flow accumulation calculation...");
    
    // Simple test case: flow left to right
    let heightmap = HeightMap::from_nested(vec![
        vec![1.0, 0.8, 0.6],
        vec![1.0, 0.8, 0.6], 
        vec![1.0, 0.8, 0.6],
    ]);
    
    let flow_directions = FlowDirectionMap::from_heightmap(&heightmap);
    let flow_accumulation = FlowAccumulationMap::from_flow_directions(&flow_directions);
    
    println!("Flow directions:");
    for y in 0..3 {
        for x in 0..3 {
            let dir = flow_directions.get(x, y);
            let elev = heightmap.get(x, y);
            print!("({:.1},{:?}) ", elev, dir);
        }
        println!();
    }
    
    println!("\nFlow accumulation:");
    let mut total = 0.0;
    for y in 0..3 {
        for x in 0..3 {
            let acc = flow_accumulation.get(x, y);
            print!("{:.1} ", acc);
            total += acc;
        }
        println!();
    }
    
    println!("\nTotal accumulation: {:.1} (expected: 9.0)", total);
    println!("Each cell should contribute 1 unit area");
}