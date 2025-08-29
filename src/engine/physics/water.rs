// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Water layer data structure for simulation water flow and accumulation
// ABOUTME: Provides high-performance storage for water depth, velocity, and sediment data

use super::super::core::heightmap::{HeightMap, Vec2Map};

#[derive(Clone, Debug, PartialEq)]
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
    pub depth: HeightMap,    // Water depth at each cell (primary buffer)
    depth_buffer: HeightMap, // Secondary buffer for double-buffering optimization
    pub velocity: Vec2Map,   // Flow direction and speed
    pub sediment: HeightMap, // Carried sediment for erosion
    width: usize,
    height: usize,
}

impl WaterLayer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            depth: HeightMap::new(width, height, 0.0),
            depth_buffer: HeightMap::new(width, height, 0.0),
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

    /// Get mutable reference to the depth buffer for double-buffering optimization
    pub fn get_depth_buffer_mut(&mut self) -> &mut HeightMap {
        &mut self.depth_buffer
    }

    /// Swap the primary depth buffer with the secondary buffer (ping-pong optimization)
    pub fn swap_depth_buffers(&mut self) {
        std::mem::swap(&mut self.depth, &mut self.depth_buffer);
    }

    /// Clear the depth buffer (prepare for next iteration)
    pub fn clear_depth_buffer(&mut self) {
        self.depth_buffer.fill(0.0);
    }

    /// Copy current depth to depth buffer for double-buffering
    pub fn copy_depth_to_buffer(&mut self) {
        self.depth_buffer.copy_from(&self.depth);
    }
}
