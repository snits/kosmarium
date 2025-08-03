#[cfg(test)]
mod debug_cache {
    use crate::cache_system::SimulationCache;
    use crate::optimized_heightmap::FlatHeightmap;

    #[test]
    fn debug_cache_key_generation() {
        let cache = SimulationCache::new();
        
        let heightmap1 = FlatHeightmap::new(3, 3);
        let mut heightmap2 = FlatHeightmap::new(3, 3);
        let mut heightmap3 = FlatHeightmap::new(3, 3);
        
        // Make heightmaps different
        heightmap2.set(1, 1, 0.5);
        heightmap3.set(1, 1, 0.8);
        
        let hash1 = cache.hash_heightmap(&heightmap1);
        let hash2 = cache.hash_heightmap(&heightmap2);
        let hash3 = cache.hash_heightmap(&heightmap3);
        
        println\!("Hash1: {}", hash1);
        println\!("Hash2: {}", hash2);
        println\!("Hash3: {}", hash3);
        
        assert_ne\!(hash1, hash2);
        assert_ne\!(hash1, hash3);
        assert_ne\!(hash2, hash3);
    }
}
EOF < /dev/null