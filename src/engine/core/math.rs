// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Unified mathematical types and utilities for cross-system data sharing
// ABOUTME: Consolidated from duplicate Vec2 implementations in water.rs and tectonics.rs

/// 2D vector type for physics calculations
/// Unified across all physics systems to enable cross-system data sharing
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// Create a new Vec2
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a zero vector
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Calculate magnitude (length) of the vector
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Calculate magnitude squared (avoids sqrt for performance)
    pub fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Normalize the vector (unit vector)
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self::new(self.x / mag, self.y / mag)
        } else {
            Self::zero()
        }
    }

    /// Dot product with another vector
    pub fn dot(&self, other: &Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Cross product magnitude (for 2D vectors, returns scalar)
    pub fn cross(&self, other: &Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }

    /// Scale the vector by a scalar
    pub fn scale(&self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }

    /// Add another vector
    pub fn add(&self, other: &Vec2) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    /// Subtract another vector
    pub fn subtract(&self, other: &Vec2) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, scalar: f32) -> Vec2 {
        self.scale(scalar)
    }
}

impl std::ops::Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, vec: Vec2) -> Vec2 {
        vec.scale(self)
    }
}

/// Mathematical constants and utility functions
pub mod constants {
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const E: f32 = std::f32::consts::E;

    /// Convert degrees to radians
    pub fn deg_to_rad(degrees: f32) -> f32 {
        degrees * PI / 180.0
    }

    /// Convert radians to degrees  
    pub fn rad_to_deg(radians: f32) -> f32 {
        radians * 180.0 / PI
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_basic_operations() {
        let v1 = Vec2::new(3.0, 4.0);
        let v2 = Vec2::new(1.0, 2.0);

        assert_eq!(v1.magnitude(), 5.0);
        assert_eq!(v1.magnitude_squared(), 25.0);
        assert_eq!(v1.dot(&v2), 11.0);
        assert_eq!(v1.cross(&v2), 2.0);

        let v3 = v1 + v2;
        assert_eq!(v3, Vec2::new(4.0, 6.0));

        let v4 = v1 - v2;
        assert_eq!(v4, Vec2::new(2.0, 2.0));

        let v5 = v1 * 2.0;
        assert_eq!(v5, Vec2::new(6.0, 8.0));
    }

    #[test]
    fn test_vec2_normalization() {
        let v = Vec2::new(3.0, 4.0);
        let normalized = v.normalize();
        assert!((normalized.magnitude() - 1.0).abs() < 1e-6);

        let zero = Vec2::zero();
        assert_eq!(zero.normalize(), Vec2::zero());
    }
}
