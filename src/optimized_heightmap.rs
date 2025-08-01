// ABOUTME: Memory-optimized heightmap data structure with flat storage for cache efficiency
// ABOUTME: Provides coordinate mapping utilities and spatial access patterns for performance

/// Memory-optimized heightmap using flat storage for better cache locality
#[derive(Clone, Debug)]
pub struct FlatHeightmap {
    data: Vec<f32>,
    width: usize,
    height: usize,
}

impl FlatHeightmap {
    /// Create new heightmap with specified dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0.0; width * height],
            width,
            height,
        }
    }

    /// Create from existing Vec<Vec<f32>> data
    pub fn from_nested(nested: Vec<Vec<f32>>) -> Self {
        let height = nested.len();
        let width = if height > 0 { nested[0].len() } else { 0 };
        let mut data = Vec::with_capacity(width * height);

        for row in nested {
            data.extend(row);
        }

        Self {
            data,
            width,
            height,
        }
    }

    /// Convert to Vec<Vec<f32>> format for compatibility
    pub fn to_nested(&self) -> Vec<Vec<f32>> {
        let mut result = Vec::with_capacity(self.height);
        for y in 0..self.height {
            let start = y * self.width;
            let end = start + self.width;
            result.push(self.data[start..end].to_vec());
        }
        result
    }

    /// Get elevation at coordinates (bounds-checked)
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.data[y * self.width + x]
        } else {
            0.0
        }
    }

    /// Set elevation at coordinates (bounds-checked)
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            self.data[y * self.width + x] = value;
        }
    }

    /// Get elevation at coordinates (unchecked for performance-critical code)
    #[inline]
    pub unsafe fn get_unchecked(&self, x: usize, y: usize) -> f32 {
        unsafe { *self.data.get_unchecked(y * self.width + x) }
    }

    /// Set elevation at coordinates (unchecked for performance-critical code)
    #[inline]
    pub unsafe fn set_unchecked(&mut self, x: usize, y: usize, value: f32) {
        unsafe {
            *self.data.get_unchecked_mut(y * self.width + x) = value;
        }
    }

    /// Get raw data slice for bulk operations
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Get mutable raw data slice for bulk operations
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Get dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Get total number of cells
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if heightmap is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Iterator over all elevation values with coordinates
    pub fn iter_with_coords(&self) -> impl Iterator<Item = (usize, usize, f32)> + '_ {
        self.data.iter().enumerate().map(move |(idx, &value)| {
            let y = idx / self.width;
            let x = idx % self.width;
            (x, y, value)
        })
    }

    /// Iterator over neighbors of a cell (8-connected)
    pub fn neighbors(&self, x: usize, y: usize) -> NeighborIterator {
        NeighborIterator::new(self, x, y)
    }
}

/// Iterator for 8-connected neighbors of a cell
pub struct NeighborIterator<'a> {
    heightmap: &'a FlatHeightmap,
    center_x: usize,
    center_y: usize,
    current_offset: usize,
}

impl<'a> NeighborIterator<'a> {
    fn new(heightmap: &'a FlatHeightmap, x: usize, y: usize) -> Self {
        Self {
            heightmap,
            center_x: x,
            center_y: y,
            current_offset: 0,
        }
    }
}

impl<'a> Iterator for NeighborIterator<'a> {
    type Item = (usize, usize, f32);

    fn next(&mut self) -> Option<Self::Item> {
        const OFFSETS: [(i32, i32); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        while self.current_offset < OFFSETS.len() {
            let (dx, dy) = OFFSETS[self.current_offset];
            self.current_offset += 1;

            let nx = self.center_x as i32 + dx;
            let ny = self.center_y as i32 + dy;

            if nx >= 0 && ny >= 0 {
                let nx = nx as usize;
                let ny = ny as usize;

                if nx < self.heightmap.width && ny < self.heightmap.height {
                    let value = self.heightmap.get(nx, ny);
                    return Some((nx, ny, value));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_heightmap_basic_operations() {
        let mut heightmap = FlatHeightmap::new(3, 2);

        // Test dimensions
        assert_eq!(heightmap.dimensions(), (3, 2));
        assert_eq!(heightmap.len(), 6);

        // Test get/set
        heightmap.set(1, 0, 0.5);
        assert_eq!(heightmap.get(1, 0), 0.5);

        // Test bounds checking
        assert_eq!(heightmap.get(10, 10), 0.0);
        heightmap.set(10, 10, 1.0); // Should not panic
    }

    #[test]
    fn conversion_roundtrip() {
        let nested = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let flat = FlatHeightmap::from_nested(nested.clone());
        let converted_back = flat.to_nested();

        assert_eq!(nested, converted_back);
    }

    #[test]
    fn neighbor_iteration() {
        let mut heightmap = FlatHeightmap::new(3, 3);

        // Set center and neighbors
        heightmap.set(1, 1, 5.0);
        heightmap.set(0, 0, 1.0);
        heightmap.set(2, 2, 9.0);

        let neighbors: Vec<_> = heightmap.neighbors(1, 1).collect();
        assert_eq!(neighbors.len(), 8); // All 8 neighbors should be found

        // Check that corner values are included
        assert!(neighbors.contains(&(0, 0, 1.0)));
        assert!(neighbors.contains(&(2, 2, 9.0)));
    }

    #[test]
    fn cache_efficiency_test() {
        let size = 1000;
        let mut heightmap = FlatHeightmap::new(size, size);

        // Sequential access should be cache-friendly
        let start = std::time::Instant::now();
        for y in 0..size {
            for x in 0..size {
                heightmap.set(x, y, (x + y) as f32);
            }
        }
        let sequential_time = start.elapsed();

        // Verify values were set correctly
        assert_eq!(heightmap.get(10, 20), 30.0);

        println!("Sequential access time: {:?}", sequential_time);
        // This test demonstrates cache-friendly access patterns
    }
}
