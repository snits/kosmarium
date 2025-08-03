// ABOUTME: Water layer data structure for simulation water flow and accumulation
// ABOUTME: Provides high-performance storage for water depth, velocity, and sediment data

use super::super::core::heightmap::{HeightMap, Vec2Map};

#[derive(Clone, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

#[derive(Clone, Debug)]
pub struct WaterLayer {
    pub depth: HeightMap,    // Water depth at each cell
    pub velocity: Vec2Map,   // Flow direction and speed
    pub sediment: HeightMap, // Carried sediment for erosion
    width: usize,
    height: usize,
}

impl WaterLayer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            depth: HeightMap::new(width, height, 0.0),
            velocity: Vec2Map::new(width, height),
            sediment: HeightMap::new(width, height, 0.0),
            width,
            height,
        }
    }

    pub fn get_total_water(&self) -> f32 {
        self.depth.iter().sum()
    }

    pub fn add_water(&mut self, x: usize, y: usize, amount: f32) {
        if x < self.width && y < self.height {
            let current = self.depth.get(x, y);
            self.depth.set(x, y, current + amount);
        }
    }

    /// Get water depth at specific coordinates
    pub fn get_water_depth(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.depth.get(x, y)
        } else {
            0.0
        }
    }

    /// Get width of water layer
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get height of water layer
    pub fn height(&self) -> usize {
        self.height
    }
}
