// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Temperature field management for atmospheric and surface thermal simulation
// ABOUTME: Provides temperature storage and manipulation for climate coupling systems

/// Temperature field for thermal simulation
#[derive(Clone, Debug)]
pub struct TemperatureField {
    /// Temperature values in Celsius
    temperatures: Vec<Vec<f32>>,
    width: usize,
    height: usize,
}

impl TemperatureField {
    /// Create new temperature field with uniform initial temperature
    pub fn new(width: usize, height: usize, initial_temperature: f32) -> Self {
        Self {
            temperatures: vec![vec![initial_temperature; height]; width],
            width,
            height,
        }
    }

    /// Set temperature at position
    pub fn set_temperature(&mut self, x: usize, y: usize, temperature: f32) {
        if x < self.width && y < self.height {
            self.temperatures[x][y] = temperature;
        }
    }

    /// Get temperature at position
    pub fn get_temperature(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.temperatures[x][y]
        } else {
            15.0 // Default temperature
        }
    }

    /// Get dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Calculate average temperature across field
    pub fn average_temperature(&self) -> f32 {
        let mut total = 0.0;
        let mut count = 0;

        for x in 0..self.width {
            for y in 0..self.height {
                total += self.temperatures[x][y];
                count += 1;
            }
        }

        if count > 0 {
            total / count as f32
        } else {
            15.0
        }
    }

    /// Find temperature range (min, max)
    pub fn temperature_range(&self) -> (f32, f32) {
        let mut min_temp = f32::INFINITY;
        let mut max_temp = f32::NEG_INFINITY;

        for x in 0..self.width {
            for y in 0..self.height {
                let temp = self.temperatures[x][y];
                min_temp = min_temp.min(temp);
                max_temp = max_temp.max(temp);
            }
        }

        (min_temp, max_temp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_field_creation() {
        let field = TemperatureField::new(10, 10, 20.0);

        assert_eq!(field.dimensions(), (10, 10));
        assert_eq!(field.get_temperature(5, 5), 20.0);
        assert_eq!(field.average_temperature(), 20.0);
    }

    #[test]
    fn test_temperature_modification() {
        let mut field = TemperatureField::new(5, 5, 15.0);

        field.set_temperature(2, 2, 25.0);
        assert_eq!(field.get_temperature(2, 2), 25.0);
        assert_eq!(field.get_temperature(1, 1), 15.0);
    }

    #[test]
    fn test_temperature_range() {
        let mut field = TemperatureField::new(3, 3, 20.0);
        field.set_temperature(0, 0, 10.0);
        field.set_temperature(2, 2, 30.0);

        let (min, max) = field.temperature_range();
        assert_eq!(min, 10.0);
        assert_eq!(max, 30.0);
    }

    #[test]
    fn test_bounds_checking() {
        let field = TemperatureField::new(5, 5, 20.0);

        // Out of bounds should return default
        assert_eq!(field.get_temperature(10, 10), 15.0);
    }
}
