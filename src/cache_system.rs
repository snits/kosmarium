// ABOUTME: Caching system for expensive calculations in geological simulation
// ABOUTME: Implements temperature field caching, change detection, and selective recomputation

use crate::climate::TemperatureLayer;
use crate::optimized_heightmap::FlatHeightmap;
use std::collections::HashMap;

/// Cache entry for temperature calculations
#[derive(Debug, Clone)]
struct TemperatureCacheEntry {
    temperature_layer: TemperatureLayer,
    terrain_hash: u64,
    creation_iteration: u64,
    last_accessed: u64,
}

/// Intelligent caching system for expensive simulation calculations
pub struct SimulationCache {
    // Temperature field cache
    temperature_cache: HashMap<u64, TemperatureCacheEntry>,

    // Configuration
    temperature_cache_lifetime: u64,
    max_cache_entries: usize,
    terrain_change_threshold: f32,

    // Statistics
    cache_hits: u64,
    cache_misses: u64,
    current_iteration: u64,
}

impl SimulationCache {
    pub fn new() -> Self {
        Self {
            temperature_cache: HashMap::new(),
            temperature_cache_lifetime: 500, // Cache valid for 500 iterations
            max_cache_entries: 10,
            terrain_change_threshold: 0.01, // 1% terrain change invalidates cache
            cache_hits: 0,
            cache_misses: 0,
            current_iteration: 0,
        }
    }

    /// Get cached temperature layer or compute if needed
    pub fn get_temperature_layer<F>(
        &mut self,
        heightmap: &FlatHeightmap,
        season: f32,
        compute_fn: F,
    ) -> TemperatureLayer
    where
        F: FnOnce(&FlatHeightmap) -> TemperatureLayer,
    {
        let terrain_hash = self.hash_heightmap(heightmap);
        let cache_key = self.compute_cache_key(terrain_hash, season);

        // Check if we have a valid cache entry
        let cache_hit = if let Some(entry) = self.temperature_cache.get(&cache_key) {
            self.is_cache_entry_valid(entry, terrain_hash)
        } else {
            false
        };

        if cache_hit {
            if let Some(entry) = self.temperature_cache.get_mut(&cache_key) {
                entry.last_accessed = self.current_iteration;
                self.cache_hits += 1;
                return entry.temperature_layer.clone();
            }
        }

        // Cache miss - compute temperature layer
        self.cache_misses += 1;
        let temperature_layer = compute_fn(heightmap);

        // Store in cache
        let cache_entry = TemperatureCacheEntry {
            temperature_layer: temperature_layer.clone(),
            terrain_hash,
            creation_iteration: self.current_iteration,
            last_accessed: self.current_iteration,
        };

        self.temperature_cache.insert(cache_key, cache_entry);

        // Clean up old cache entries
        self.cleanup_cache();

        temperature_layer
    }

    /// Check if terrain has changed significantly since cache entry
    pub fn has_terrain_changed_significantly(
        &self,
        old_heightmap: &FlatHeightmap,
        new_heightmap: &FlatHeightmap,
    ) -> bool {
        if old_heightmap.len() != new_heightmap.len() {
            return true;
        }

        let old_data = old_heightmap.data();
        let new_data = new_heightmap.data();

        let mut total_change = 0.0;
        let mut max_change: f32 = 0.0;

        for (old_val, new_val) in old_data.iter().zip(new_data.iter()) {
            let change = (new_val - old_val).abs();
            total_change += change;
            max_change = max_change.max(change);
        }

        let average_change = total_change / old_data.len() as f32;

        // Significant change if average exceeds threshold OR any single cell changes dramatically
        average_change > self.terrain_change_threshold
            || max_change > self.terrain_change_threshold * 10.0
    }

    /// Advance iteration counter and update cache statistics
    pub fn advance_iteration(&mut self) {
        self.current_iteration += 1;

        // Periodic cache cleanup
        if self.current_iteration % 100 == 0 {
            self.cleanup_cache();
        }
    }

    /// Get cache performance statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        let total_requests = self.cache_hits + self.cache_misses;
        let hit_rate = if total_requests > 0 {
            self.cache_hits as f32 / total_requests as f32
        } else {
            0.0
        };

        CacheStats {
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            hit_rate,
            cache_size: self.temperature_cache.len(),
            current_iteration: self.current_iteration,
        }
    }

    /// Reset cache and statistics
    pub fn reset(&mut self) {
        self.temperature_cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.current_iteration = 0;
    }

    /// Configure cache parameters
    pub fn configure(&mut self, lifetime: u64, max_entries: usize, change_threshold: f32) {
        self.temperature_cache_lifetime = lifetime;
        self.max_cache_entries = max_entries;
        self.terrain_change_threshold = change_threshold;
    }

    // Private methods

    fn hash_heightmap(&self, heightmap: &FlatHeightmap) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Sample heightmap at regular intervals for performance
        let (width, height) = heightmap.dimensions();
        let sample_rate = 4; // Sample every 4th cell

        for y in (0..height).step_by(sample_rate) {
            for x in (0..width).step_by(sample_rate) {
                let value = heightmap.get(x, y);
                // Hash the bits directly for consistent hashing
                value.to_bits().hash(&mut hasher);
            }
        }

        hasher.finish()
    }

    fn compute_cache_key(&self, terrain_hash: u64, season: f32) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        terrain_hash.hash(&mut hasher);

        // Discretize season to avoid cache misses from tiny seasonal changes
        let discretized_season = (season * 100.0) as u32;
        discretized_season.hash(&mut hasher);

        hasher.finish()
    }

    fn is_cache_entry_valid(
        &self,
        entry: &TemperatureCacheEntry,
        current_terrain_hash: u64,
    ) -> bool {
        // Check age
        let age = self.current_iteration - entry.creation_iteration;
        if age > self.temperature_cache_lifetime {
            return false;
        }

        // Check terrain consistency
        if entry.terrain_hash != current_terrain_hash {
            return false;
        }

        true
    }

    fn cleanup_cache(&mut self) {
        // Remove expired entries
        let current_iter = self.current_iteration;
        let lifetime = self.temperature_cache_lifetime;

        self.temperature_cache.retain(|_, entry| {
            let age = current_iter - entry.creation_iteration;
            age <= lifetime
        });

        // If still too many entries, remove least recently used
        if self.temperature_cache.len() > self.max_cache_entries {
            let keys_to_remove: Vec<_> = {
                let mut entries: Vec<_> = self.temperature_cache.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.last_accessed);

                let entries_to_remove = self.temperature_cache.len() - self.max_cache_entries;
                entries
                    .iter()
                    .take(entries_to_remove)
                    .map(|(key, _)| (*key).clone())
                    .collect()
            };

            for key in keys_to_remove {
                self.temperature_cache.remove(&key);
            }
        }
    }
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_rate: f32,
    pub cache_size: usize,
    pub current_iteration: u64,
}

impl CacheStats {
    pub fn computational_savings(&self) -> f32 {
        if self.cache_hits == 0 {
            return 0.0;
        }

        // Estimate computational savings (temperature calculation is expensive)
        self.cache_hits as f32 / (self.cache_hits + self.cache_misses) as f32
    }

    pub fn total_requests(&self) -> u64 {
        self.cache_hits + self.cache_misses
    }
}

/// Optimized climate system with intelligent caching
pub struct CachedClimateSystem {
    cache: SimulationCache,
    base_climate_system: crate::climate::ClimateSystem,
}

impl CachedClimateSystem {
    pub fn new(base_system: crate::climate::ClimateSystem) -> Self {
        Self {
            cache: SimulationCache::new(),
            base_climate_system: base_system,
        }
    }

    /// Get temperature layer with caching
    pub fn get_cached_temperature_layer(&mut self, heightmap: &FlatHeightmap) -> TemperatureLayer {
        let season = self.base_climate_system.current_season;

        self.cache.get_temperature_layer(heightmap, season, |hm| {
            // Convert FlatHeightmap to Vec<Vec<f32>> for compatibility
            let nested_heightmap = hm.to_nested();
            self.base_climate_system
                .generate_temperature_layer(&nested_heightmap)
        })
    }

    /// Advance iteration and update caches
    pub fn advance_iteration(&mut self) {
        self.cache.advance_iteration();
        self.base_climate_system.tick();
    }

    /// Get cache performance statistics
    pub fn get_performance_stats(&self) -> CacheStats {
        self.cache.get_cache_stats()
    }

    /// Check if significant terrain changes require cache invalidation
    pub fn should_invalidate_cache(
        &self,
        old_heightmap: &FlatHeightmap,
        new_heightmap: &FlatHeightmap,
    ) -> bool {
        self.cache
            .has_terrain_changed_significantly(old_heightmap, new_heightmap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_basic_functionality() {
        let mut cache = SimulationCache::new();
        let heightmap = FlatHeightmap::new(10, 10);

        let mut call_count = 0;
        let compute_fn = |_: &FlatHeightmap| {
            call_count += 1;
            TemperatureLayer::new(10, 10)
        };

        // First call should miss cache
        let _temp1 = cache.get_temperature_layer(&heightmap, 0.5, &compute_fn);
        assert_eq!(call_count, 1);

        // Second call with same parameters should hit cache
        let _temp2 = cache.get_temperature_layer(&heightmap, 0.5, &compute_fn);
        assert_eq!(call_count, 1); // No additional computation

        let stats = cache.get_cache_stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[test]
    fn terrain_change_detection() {
        let cache = SimulationCache::new();
        let mut heightmap1 = FlatHeightmap::new(5, 5);
        let mut heightmap2 = FlatHeightmap::new(5, 5);

        // Small changes should not trigger cache invalidation
        heightmap1.set(2, 2, 0.5);
        heightmap2.set(2, 2, 0.505); // 0.5% change

        assert!(!cache.has_terrain_changed_significantly(&heightmap1, &heightmap2));

        // Large changes should trigger cache invalidation
        heightmap2.set(2, 2, 0.6); // 10% change
        assert!(cache.has_terrain_changed_significantly(&heightmap1, &heightmap2));
    }

    #[test]
    fn cache_expiration() {
        let mut cache = SimulationCache::new();
        cache.configure(5, 10, 0.01); // Very short lifetime

        let heightmap = FlatHeightmap::new(3, 3);

        let mut call_count = 0;
        let compute_fn = |_: &FlatHeightmap| {
            call_count += 1;
            TemperatureLayer::new(3, 3)
        };

        // First call
        let _temp1 = cache.get_temperature_layer(&heightmap, 0.5, &compute_fn);
        assert_eq!(call_count, 1);

        // Advance time past cache lifetime
        for _ in 0..10 {
            cache.advance_iteration();
        }

        // Should recompute due to expiration
        let _temp2 = cache.get_temperature_layer(&heightmap, 0.5, &compute_fn);
        assert_eq!(call_count, 2);
    }

    #[test]
    fn cache_lru_eviction() {
        let mut cache = SimulationCache::new();
        cache.configure(1000, 2, 0.01); // Max 2 entries

        let heightmap1 = FlatHeightmap::new(3, 3);
        let mut heightmap2 = FlatHeightmap::new(3, 3);
        let mut heightmap3 = FlatHeightmap::new(3, 3);

        // Make heightmaps different
        heightmap2.set(1, 1, 0.5);
        heightmap3.set(1, 1, 0.8);

        let mut call_count = 0;
        let compute_fn = |_: &FlatHeightmap| {
            call_count += 1;
            TemperatureLayer::new(3, 3)
        };

        // Fill cache
        let _temp1 = cache.get_temperature_layer(&heightmap1, 0.5, &compute_fn);
        let _temp2 = cache.get_temperature_layer(&heightmap2, 0.5, &compute_fn);
        assert_eq!(call_count, 2);

        // This should evict least recently used entry
        let _temp3 = cache.get_temperature_layer(&heightmap3, 0.5, &compute_fn);
        assert_eq!(call_count, 3);

        // First heightmap should now be evicted, causing recomputation
        let _temp1_again = cache.get_temperature_layer(&heightmap1, 0.5, &compute_fn);
        assert_eq!(call_count, 4);
    }
}
