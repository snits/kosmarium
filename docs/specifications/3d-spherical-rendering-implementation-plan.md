# 3D Spherical Rendering Implementation Specification
*Engineering-Ready Plan for Physics-Correct Atmospheric Visualization*

## Executive Summary

This specification provides a comprehensive implementation plan for transitioning from 2D rectangular rendering to 3D spherical visualization, addressing the fundamental atmospheric physics violations identified by the atmospheric-physicist. The plan leverages the existing scale-aware architecture while introducing proper spherical coordinate handling and planetary-scale 3D rendering capabilities.

**MISSION**: Create a 3D spherical rendering system that enables realistic atmospheric simulation by eliminating artificial boundary artifacts through proper planetary geometry.

**KEY DELIVERABLES**:
- Spherical coordinate system with proper physics integration
- 3D visualization architecture for continental-to-planetary scale domains
- Multi-layer atmospheric data rendering (pressure, wind, temperature, moisture)
- Real-time performance system handling domains from 100km to 40,000km
- Intuitive 3D navigation interface for planetary simulation

## 1. Current System Analysis & Extension Points

### 1.1 Existing Architecture Strengths

**Scale-Aware Foundation** ✅
- `WorldScale` provides robust physical scaling (1m to 40,000km domains)
- `ScaleAware` trait enables parameter derivation across scales  
- Dimensional analysis system with proper physical units
- Proven CFL timestep management for numerical stability

**Multi-Layer Rendering System** ✅
- `GraphicsRenderer` with 7 display modes (elevation, water, pressure, wind, weather, temperature, biomes)
- Bounded viewport system with UI sidebars ready for 3D extension
- Color mapping functions for atmospheric data visualization
- Real-time input handling with zoom/pan controls

**Atmospheric Physics Integration** ✅ 
- Proper geostrophic wind calculation with Coriolis effects
- Thermodynamic coupling (P-ρ-T relationships)
- Scale-aware atmospheric parameter derivation
- Multi-layer atmospheric state (pressure, wind, temperature)

### 1.2 Critical Gaps Requiring Spherical Geometry

**Boundary Condition Problems** ❌
- Rectangular boundaries violate atmospheric physics (confirmed by atmospheric-physicist)
- Artificial momentum accumulation at domain edges
- Non-physical pressure clamping preventing realistic weather systems
- "Ghost in machine" horizontal wind band artifacts

**Coordinate System Limitations** ❌
- 2D Cartesian coordinates incompatible with planetary atmospheric dynamics
- No proper handling of Coriolis parameter variation with latitude
- Missing atmospheric wave propagation across realistic geometry
- Inability to simulate global-scale atmospheric phenomena

## 2. Spherical Coordinate System Architecture

### 2.1 Mathematical Framework

**Spherical Coordinate Definitions**:
```rust
// New module: src/engine/rendering/spherical_coordinates.rs
use crate::engine::core::dimensional::PhysicalQuantity;
use crate::engine::core::scale::WorldScale;

/// Spherical coordinate system for planetary-scale rendering
#[derive(Debug, Clone, Copy)]
pub struct SphericalCoordinates {
    /// Longitude in radians (-π to π, 0 = Greenwich meridian)
    pub longitude: f64,
    /// Latitude in radians (-π/2 to π/2, 0 = equator, π/2 = north pole)
    pub latitude: f64,  
    /// Altitude in meters above sea level
    pub altitude: f64,
    /// Planetary radius in meters (Earth = 6.371e6 m)
    pub planet_radius: f64,
}

impl SphericalCoordinates {
    /// Standard Earth-like planetary parameters
    pub const EARTH_RADIUS_M: f64 = 6.371e6;
    
    /// Convert from Cartesian coordinates to spherical
    pub fn from_cartesian(x: f64, y: f64, z: f64) -> Self {
        let r = (x*x + y*y + z*z).sqrt();
        let latitude = (z / r).asin();
        let longitude = y.atan2(x);
        let altitude = r - Self::EARTH_RADIUS_M;
        
        Self {
            longitude,
            latitude, 
            altitude,
            planet_radius: Self::EARTH_RADIUS_M,
        }
    }
    
    /// Convert to Cartesian coordinates (for 3D rendering)
    pub fn to_cartesian(&self) -> (f64, f64, f64) {
        let r = self.planet_radius + self.altitude;
        let x = r * self.latitude.cos() * self.longitude.cos();
        let y = r * self.latitude.cos() * self.longitude.sin(); 
        let z = r * self.latitude.sin();
        (x, y, z)
    }
    
    /// Calculate great circle distance to another point (meters)
    pub fn distance_to(&self, other: &SphericalCoordinates) -> f64 {
        let dlat = other.latitude - self.latitude;
        let dlon = other.longitude - self.longitude;
        
        let a = (dlat/2.0).sin().powi(2) + 
                self.latitude.cos() * other.latitude.cos() * 
                (dlon/2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0-a).sqrt());
        
        self.planet_radius * c
    }
    
    /// Convert grid indices to spherical coordinates for domain
    pub fn from_grid_indices(
        x: usize,
        y: usize, 
        width: usize,
        height: usize,
        domain_bounds: SphericalDomain,
    ) -> Self {
        let lon_range = domain_bounds.east_longitude - domain_bounds.west_longitude;
        let lat_range = domain_bounds.north_latitude - domain_bounds.south_latitude;
        
        let longitude = domain_bounds.west_longitude + (x as f64 / width as f64) * lon_range;
        let latitude = domain_bounds.south_latitude + (y as f64 / height as f64) * lat_range;
        
        Self {
            longitude,
            latitude,
            altitude: 0.0,
            planet_radius: Self::EARTH_RADIUS_M,
        }
    }
}
```

### 2.2 Domain Definition System

**Spherical Domain Boundaries**:
```rust
/// Defines a spherical domain for atmospheric simulation
#[derive(Debug, Clone)]
pub struct SphericalDomain {
    /// Western boundary longitude (radians)
    pub west_longitude: f64,
    /// Eastern boundary longitude (radians)  
    pub east_longitude: f64,
    /// Southern boundary latitude (radians)
    pub south_latitude: f64,
    /// Northern boundary latitude (radians)
    pub north_latitude: f64,
    /// Minimum altitude (meters above sea level)
    pub min_altitude: f64,
    /// Maximum altitude (meters above sea level) 
    pub max_altitude: f64,
    /// Domain classification for appropriate boundary conditions
    pub domain_type: DomainType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DomainType {
    /// Continental-scale domain (100-2000km) with lateral boundaries
    Continental,
    /// Hemispheric domain (5000-20000km) with polar treatment
    Hemispheric, 
    /// Global domain (40000km) with periodic longitude boundaries
    Global,
}

impl SphericalDomain {
    /// Create continental domain centered at given coordinates
    pub fn continental_centered_at(
        center_longitude: f64,
        center_latitude: f64, 
        extent_km: f64,
    ) -> Self {
        let angular_extent = extent_km / (SphericalCoordinates::EARTH_RADIUS_M / 1000.0);
        let half_extent = angular_extent / 2.0;
        
        Self {
            west_longitude: center_longitude - half_extent,
            east_longitude: center_longitude + half_extent,
            south_latitude: center_latitude - half_extent,
            north_latitude: center_latitude + half_extent,
            min_altitude: 0.0,
            max_altitude: 15000.0, // Atmospheric boundary layer
            domain_type: DomainType::Continental,
        }
    }
    
    /// Create global domain for planetary simulation
    pub fn global() -> Self {
        Self {
            west_longitude: -std::f64::consts::PI,
            east_longitude: std::f64::consts::PI,
            south_latitude: -std::f64::consts::PI / 2.0,
            north_latitude: std::f64::consts::PI / 2.0,
            min_altitude: 0.0,
            max_altitude: 15000.0,
            domain_type: DomainType::Global,
        }
    }
    
    /// Calculate physical extent in kilometers
    pub fn physical_extent_km(&self) -> (f64, f64) {
        let lon_extent_km = (self.east_longitude - self.west_longitude) * 
                            SphericalCoordinates::EARTH_RADIUS_M / 1000.0;
        let lat_extent_km = (self.north_latitude - self.south_latitude) * 
                            SphericalCoordinates::EARTH_RADIUS_M / 1000.0;
        (lon_extent_km, lat_extent_km)
    }
}
```

### 2.3 Coordinate Transformation System

**Grid to Spherical Mapping**:
```rust
/// Handles coordinate transformations between grid indices and spherical coordinates
pub struct CoordinateTransform {
    pub domain: SphericalDomain,
    pub grid_resolution: (usize, usize),
    pub scale: WorldScale,
}

impl CoordinateTransform {
    pub fn new(domain: SphericalDomain, resolution: (usize, usize), scale: WorldScale) -> Self {
        Self {
            domain,
            grid_resolution: resolution,
            scale,
        }
    }
    
    /// Convert grid cell (i,j) to spherical coordinates
    pub fn grid_to_spherical(&self, i: usize, j: usize) -> SphericalCoordinates {
        SphericalCoordinates::from_grid_indices(
            i, j, 
            self.grid_resolution.0, 
            self.grid_resolution.1,
            self.domain,
        )
    }
    
    /// Convert spherical coordinates to grid cell (with bounds checking)
    pub fn spherical_to_grid(&self, coords: SphericalCoordinates) -> Option<(usize, usize)> {
        if !self.domain.contains(coords) {
            return None;
        }
        
        let lon_fraction = (coords.longitude - self.domain.west_longitude) /
                          (self.domain.east_longitude - self.domain.west_longitude);
        let lat_fraction = (coords.latitude - self.domain.south_latitude) /
                          (self.domain.north_latitude - self.domain.south_latitude);
                          
        let i = (lon_fraction * self.grid_resolution.0 as f64) as usize;
        let j = (lat_fraction * self.grid_resolution.1 as f64) as usize;
        
        Some((i.min(self.grid_resolution.0 - 1), j.min(self.grid_resolution.1 - 1)))
    }
    
    /// Calculate proper Coriolis parameter for given coordinates
    pub fn coriolis_parameter(&self, coords: SphericalCoordinates) -> f64 {
        const EARTH_ROTATION_RATE: f64 = 7.27e-5; // rad/s
        2.0 * EARTH_ROTATION_RATE * coords.latitude.sin()
    }
    
    /// Calculate metric factors for numerical derivatives
    pub fn metric_factors(&self, coords: SphericalCoordinates) -> (f64, f64) {
        let r = coords.planet_radius + coords.altitude;
        let dx_per_dlon = r * coords.latitude.cos(); // meters per radian longitude
        let dy_per_dlat = r;                        // meters per radian latitude
        (dx_per_dlon, dy_per_dlat)
    }
}

impl SphericalDomain {
    fn contains(&self, coords: SphericalCoordinates) -> bool {
        coords.longitude >= self.west_longitude &&
        coords.longitude <= self.east_longitude &&
        coords.latitude >= self.south_latitude &&
        coords.latitude <= self.north_latitude &&
        coords.altitude >= self.min_altitude &&
        coords.altitude <= self.max_altitude
    }
}
```

## 3. 3D Visualization Architecture

### 3.1 3D Camera System

**Spherical Camera for Planetary Visualization**:
```rust
// New module: src/engine/rendering/spherical_camera.rs  
use macroquad::prelude::*;

/// 3D camera system optimized for spherical planetary visualization
pub struct SphericalCamera {
    /// Camera position in spherical coordinates  
    pub position: SphericalCoordinates,
    /// Target position camera is looking at
    pub target: SphericalCoordinates,
    /// Up vector in world space
    pub up: Vec3,
    /// Field of view in radians
    pub fov: f32,
    /// Aspect ratio (width/height)
    pub aspect_ratio: f32,
    /// Near clipping plane (meters)
    pub near_plane: f32,
    /// Far clipping plane (meters)  
    pub far_plane: f32,
    /// Current view mode
    pub view_mode: CameraMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraMode {
    /// Orbital view around planet (typical for global domains)
    Orbital,
    /// Surface-relative view (for continental domains)
    Surface,  
    /// Atmospheric layer view (for weather pattern analysis)
    Atmospheric,
    /// Cross-section view (for vertical atmospheric structure)
    CrossSection,
}

impl SphericalCamera {
    /// Create camera for continental domain visualization
    pub fn for_continental_domain(domain: &SphericalDomain) -> Self {
        let (extent_lon, extent_lat) = domain.physical_extent_km();
        let domain_center_lon = (domain.west_longitude + domain.east_longitude) / 2.0;
        let domain_center_lat = (domain.south_latitude + domain.north_latitude) / 2.0;
        
        // Position camera above domain center at appropriate altitude
        let camera_altitude = (extent_lon.max(extent_lat) * 1000.0 * 0.5) as f64; // 50% above domain
        let camera_position = SphericalCoordinates {
            longitude: domain_center_lon,
            latitude: domain_center_lat + 0.2, // Slightly north for better viewing angle
            altitude: camera_altitude,
            planet_radius: SphericalCoordinates::EARTH_RADIUS_M,
        };
        
        let target = SphericalCoordinates {
            longitude: domain_center_lon,
            latitude: domain_center_lat,
            altitude: 0.0,
            planet_radius: SphericalCoordinates::EARTH_RADIUS_M,
        };
        
        Self {
            position: camera_position,
            target,
            up: Vec3::new(0.0, 0.0, 1.0), // Z-up coordinate system
            fov: std::f32::consts::PI / 4.0, // 45 degree FOV
            aspect_ratio: 16.0 / 9.0,
            near_plane: 100.0, // 100m near plane
            far_plane: (camera_altitude * 3.0) as f32, // 3x camera altitude
            view_mode: CameraMode::Surface,
        }
    }
    
    /// Create camera for global planetary visualization
    pub fn for_global_domain() -> Self {
        let camera_position = SphericalCoordinates {
            longitude: 0.0,
            latitude: std::f64::consts::PI / 6.0, // 30°N viewing angle
            altitude: SphericalCoordinates::EARTH_RADIUS_M * 2.0, // 2 Earth radii altitude
            planet_radius: SphericalCoordinates::EARTH_RADIUS_M,
        };
        
        let target = SphericalCoordinates {
            longitude: 0.0,
            latitude: 0.0,
            altitude: 0.0,
            planet_radius: SphericalCoordinates::EARTH_RADIUS_M,
        };
        
        Self {
            position: camera_position,
            target,
            up: Vec3::new(0.0, 0.0, 1.0),
            fov: std::f32::consts::PI / 3.0, // 60 degree FOV for planetary view
            aspect_ratio: 16.0 / 9.0,
            near_plane: 1000.0, // 1km near plane
            far_plane: (SphericalCoordinates::EARTH_RADIUS_M * 5.0) as f32,
            view_mode: CameraMode::Orbital,
        }
    }
    
    /// Generate view matrix for 3D rendering
    pub fn view_matrix(&self) -> Mat4 {
        let camera_cart = self.position.to_cartesian();
        let target_cart = self.target.to_cartesian();
        
        let eye = Vec3::new(camera_cart.0 as f32, camera_cart.1 as f32, camera_cart.2 as f32);
        let target = Vec3::new(target_cart.0 as f32, target_cart.1 as f32, target_cart.2 as f32);
        
        Mat4::look_at_rh(eye, target, self.up)
    }
    
    /// Generate projection matrix for 3D rendering
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov, self.aspect_ratio, self.near_plane, self.far_plane)
    }
    
    /// Update camera based on user input
    pub fn handle_input(&mut self, domain: &SphericalDomain) {
        match self.view_mode {
            CameraMode::Orbital => self.handle_orbital_input(),
            CameraMode::Surface => self.handle_surface_input(domain),
            CameraMode::Atmospheric => self.handle_atmospheric_input(),
            CameraMode::CrossSection => self.handle_cross_section_input(),
        }
    }
    
    fn handle_orbital_input(&mut self) {
        let rotation_speed = 0.02; // radians per frame
        let zoom_speed = 1.1;
        
        // Longitude rotation (left/right)
        if is_key_down(KeyCode::A) {
            self.position.longitude -= rotation_speed;
        }
        if is_key_down(KeyCode::D) {
            self.position.longitude += rotation_speed;
        }
        
        // Latitude rotation (up/down) with pole constraints
        if is_key_down(KeyCode::W) {
            self.position.latitude = (self.position.latitude + rotation_speed)
                .min(std::f64::consts::PI / 2.0 - 0.1);
        }
        if is_key_down(KeyCode::S) {
            self.position.latitude = (self.position.latitude - rotation_speed)
                .max(-std::f64::consts::PI / 2.0 + 0.1);
        }
        
        // Altitude zoom (mouse wheel or Q/E)
        let (_x, scroll_y) = mouse_wheel();
        if scroll_y > 0.0 || is_key_down(KeyCode::Q) {
            self.position.altitude /= zoom_speed;
            self.position.altitude = self.position.altitude.max(self.position.planet_radius * 0.1);
        }
        if scroll_y < 0.0 || is_key_down(KeyCode::E) {
            self.position.altitude *= zoom_speed;
            self.position.altitude = self.position.altitude.min(self.position.planet_radius * 10.0);
        }
    }
    
    fn handle_surface_input(&mut self, domain: &SphericalDomain) {
        let pan_speed = 0.005; // radians per frame for surface view
        let altitude_speed = 1.1;
        
        // Pan across domain surface
        if is_key_down(KeyCode::A) {
            self.target.longitude = (self.target.longitude - pan_speed)
                .max(domain.west_longitude);
        }
        if is_key_down(KeyCode::D) {
            self.target.longitude = (self.target.longitude + pan_speed)
                .min(domain.east_longitude);
        }
        if is_key_down(KeyCode::W) {
            self.target.latitude = (self.target.latitude + pan_speed)
                .min(domain.north_latitude);
        }
        if is_key_down(KeyCode::S) {
            self.target.latitude = (self.target.latitude - pan_speed)
                .max(domain.south_latitude);
        }
        
        // Altitude adjustment
        let (_x, scroll_y) = mouse_wheel();
        if scroll_y > 0.0 {
            self.position.altitude /= altitude_speed;
            self.position.altitude = self.position.altitude.max(100.0); // Minimum 100m
        }
        if scroll_y < 0.0 {
            self.position.altitude *= altitude_speed;
            let (extent_x, extent_y) = domain.physical_extent_km();
            self.position.altitude = self.position.altitude
                .min(extent_x.max(extent_y) * 1000.0 * 2.0); // Max 2x domain extent
        }
        
        // Keep camera positioned relative to target
        self.position.longitude = self.target.longitude;
        self.position.latitude = self.target.latitude;
    }
    
    fn handle_atmospheric_input(&mut self) {
        // Similar to surface input but with vertical layer focus
        // Allow altitude layers selection for atmospheric analysis
        unimplemented!("Atmospheric camera mode")
    }
    
    fn handle_cross_section_input(&mut self) {
        // Cross-section slice through atmosphere for vertical structure analysis
        unimplemented!("Cross-section camera mode")
    }
}
```

### 3.2 Spherical Mesh Generation

**Adaptive Spherical Grid System**:
```rust
// New module: src/engine/rendering/spherical_mesh.rs
use macroquad::prelude::*;

/// Generates 3D meshes for spherical domain visualization
pub struct SphericalMeshGenerator {
    pub domain: SphericalDomain,
    pub transform: CoordinateTransform,
    pub mesh_resolution: MeshResolution,
}

#[derive(Debug, Clone, Copy)]
pub struct MeshResolution {
    /// Number of longitude divisions
    pub longitude_segments: usize,
    /// Number of latitude divisions  
    pub latitude_segments: usize,
    /// Number of altitude layers for 3D atmospheric visualization
    pub altitude_layers: usize,
}

impl MeshResolution {
    /// Adaptive resolution based on domain size and performance targets
    pub fn adaptive_for_domain(domain: &SphericalDomain) -> Self {
        let (extent_lon_km, extent_lat_km) = domain.physical_extent_km();
        let max_extent_km = extent_lon_km.max(extent_lat_km);
        
        let (lon_segments, lat_segments) = match max_extent_km {
            km if km < 100.0 => (64, 32),   // High detail for local domains
            km if km < 1000.0 => (128, 64), // Medium detail for regional domains  
            km if km < 5000.0 => (256, 128), // Continental domains
            _ => (512, 256),                // Global/hemispheric domains
        };
        
        Self {
            longitude_segments: lon_segments,
            latitude_segments: lat_segments,
            altitude_layers: 8, // Standard atmospheric layers
        }
    }
}

impl SphericalMeshGenerator {
    pub fn new(
        domain: SphericalDomain, 
        transform: CoordinateTransform, 
        resolution: MeshResolution
    ) -> Self {
        Self {
            domain,
            transform,
            mesh_resolution: resolution,
        }
    }
    
    /// Generate surface mesh for terrain/elevation visualization
    pub fn generate_surface_mesh(&self, elevation_data: &HeightMap) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut colors = Vec::new();
        
        // Generate vertices for surface mesh
        for j in 0..=self.mesh_resolution.latitude_segments {
            for i in 0..=self.mesh_resolution.longitude_segments {
                let lon_fraction = i as f64 / self.mesh_resolution.longitude_segments as f64;
                let lat_fraction = j as f64 / self.mesh_resolution.latitude_segments as f64;
                
                let longitude = self.domain.west_longitude + 
                               lon_fraction * (self.domain.east_longitude - self.domain.west_longitude);
                let latitude = self.domain.south_latitude + 
                              lat_fraction * (self.domain.north_latitude - self.domain.south_latitude);
                
                // Sample elevation data
                let grid_x = (lon_fraction * (elevation_data.width() - 1) as f64) as usize;
                let grid_y = (lat_fraction * (elevation_data.height() - 1) as f64) as usize;
                let elevation_m = elevation_data.get(grid_x, grid_y) as f64 * 8000.0; // Scale to realistic elevations
                
                let coords = SphericalCoordinates {
                    longitude,
                    latitude,
                    altitude: elevation_m,
                    planet_radius: SphericalCoordinates::EARTH_RADIUS_M,
                };
                
                let (x, y, z) = coords.to_cartesian();
                vertices.push(Vertex {
                    position: Vec3::new(x as f32, y as f32, z as f32),
                    uv: Vec2::new(lon_fraction as f32, lat_fraction as f32),
                    color: self.elevation_to_color(elevation_data.get(grid_x, grid_y)),
                });
            }
        }
        
        // Generate triangle indices for mesh
        for j in 0..self.mesh_resolution.latitude_segments {
            for i in 0..self.mesh_resolution.longitude_segments {
                let base_idx = j * (self.mesh_resolution.longitude_segments + 1) + i;
                
                // Two triangles per quad
                indices.extend_from_slice(&[
                    base_idx as u16,
                    (base_idx + self.mesh_resolution.longitude_segments + 1) as u16,
                    (base_idx + 1) as u16,
                    
                    (base_idx + 1) as u16,
                    (base_idx + self.mesh_resolution.longitude_segments + 1) as u16,
                    (base_idx + self.mesh_resolution.longitude_segments + 2) as u16,
                ]);
            }
        }
        
        Mesh {
            vertices,
            indices,
        }
    }
    
    /// Generate atmospheric layer mesh for pressure/wind visualization
    pub fn generate_atmospheric_layer_mesh(
        &self, 
        atmospheric_data: &AtmosphericPressureLayer,
        altitude_m: f64,
    ) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Generate vertices at specified altitude with atmospheric data colors
        for j in 0..=self.mesh_resolution.latitude_segments {
            for i in 0..=self.mesh_resolution.longitude_segments {
                let lon_fraction = i as f64 / self.mesh_resolution.longitude_segments as f64;
                let lat_fraction = j as f64 / self.mesh_resolution.latitude_segments as f64;
                
                let longitude = self.domain.west_longitude + 
                               lon_fraction * (self.domain.east_longitude - self.domain.west_longitude);
                let latitude = self.domain.south_latitude + 
                              lat_fraction * (self.domain.north_latitude - self.domain.south_latitude);
                
                let coords = SphericalCoordinates {
                    longitude,
                    latitude,
                    altitude: altitude_m,
                    planet_radius: SphericalCoordinates::EARTH_RADIUS_M,
                };
                
                let (x, y, z) = coords.to_cartesian();
                
                // Sample atmospheric data
                let grid_x = (lon_fraction * (atmospheric_data.width() - 1) as f64) as usize;
                let grid_y = (lat_fraction * (atmospheric_data.height() - 1) as f64) as usize;
                let pressure = atmospheric_data.get_pressure(grid_x, grid_y);
                
                vertices.push(Vertex {
                    position: Vec3::new(x as f32, y as f32, z as f32),
                    uv: Vec2::new(lon_fraction as f32, lat_fraction as f32),
                    color: self.pressure_to_color(pressure),
                });
            }
        }
        
        // Generate triangle indices (same topology as surface mesh)
        for j in 0..self.mesh_resolution.latitude_segments {
            for i in 0..self.mesh_resolution.longitude_segments {
                let base_idx = j * (self.mesh_resolution.longitude_segments + 1) + i;
                
                indices.extend_from_slice(&[
                    base_idx as u16,
                    (base_idx + self.mesh_resolution.longitude_segments + 1) as u16,
                    (base_idx + 1) as u16,
                    
                    (base_idx + 1) as u16,
                    (base_idx + self.mesh_resolution.longitude_segments + 1) as u16,
                    (base_idx + self.mesh_resolution.longitude_segments + 2) as u16,
                ]);
            }
        }
        
        Mesh {
            vertices,
            indices,
        }
    }
    
    /// Generate 3D wind vector field visualization
    pub fn generate_wind_vectors(&self, wind_layer: &WindLayer) -> Vec<WindVector3D> {
        let mut wind_vectors = Vec::new();
        
        let sample_rate = match self.domain.domain_type {
            DomainType::Continental => 8,   // Every 8th grid cell
            DomainType::Hemispheric => 16,  // Every 16th grid cell
            DomainType::Global => 32,       // Every 32nd grid cell
        };
        
        for j in (0..wind_layer.height()).step_by(sample_rate) {
            for i in (0..wind_layer.width()).step_by(sample_rate) {
                let velocity = wind_layer.get_velocity(i, j);
                let speed = wind_layer.get_speed(i, j);
                
                if speed < 0.5 { continue; } // Skip very low wind speeds
                
                let coords = self.transform.grid_to_spherical(i, j);
                let (x, y, z) = coords.to_cartesian();
                
                // Convert wind velocity from surface-relative to 3D Cartesian
                let wind_3d = self.surface_wind_to_3d_cartesian(velocity, coords);
                
                wind_vectors.push(WindVector3D {
                    position: Vec3::new(x as f32, y as f32, z as f32),
                    velocity: wind_3d,
                    magnitude: speed,
                    color: self.wind_speed_to_color(speed),
                });
            }
        }
        
        wind_vectors
    }
    
    fn surface_wind_to_3d_cartesian(
        &self, 
        surface_velocity: Vec2, 
        coords: SphericalCoordinates
    ) -> Vec3 {
        // Convert surface wind vector (east, north) to 3D Cartesian coordinates
        // This requires proper transformation accounting for sphere geometry
        
        let cos_lat = coords.latitude.cos();
        let sin_lat = coords.latitude.sin(); 
        let cos_lon = coords.longitude.cos();
        let sin_lon = coords.longitude.sin();
        
        // East direction vector in Cartesian coordinates
        let east_x = -sin_lon;
        let east_y = cos_lon;
        let east_z = 0.0;
        
        // North direction vector in Cartesian coordinates  
        let north_x = -sin_lat * cos_lon;
        let north_y = -sin_lat * sin_lon;
        let north_z = cos_lat;
        
        Vec3::new(
            (surface_velocity.x * east_x + surface_velocity.y * north_x) as f32,
            (surface_velocity.x * east_y + surface_velocity.y * north_y) as f32, 
            (surface_velocity.x * east_z + surface_velocity.y * north_z) as f32,
        )
    }
    
    fn elevation_to_color(&self, elevation: f32) -> Color {
        match elevation {
            e if e < 0.2 => BLUE,    // Water
            e if e < 0.4 => SKYBLUE, // Coast
            e if e < 0.6 => GREEN,   // Plains
            e if e < 0.8 => YELLOW,  // Hills
            _ => RED,                // Mountains
        }
    }
    
    fn pressure_to_color(&self, pressure: f32) -> Color {
        // Map pressure to blue (low) to red (high) color scale
        let normalized = ((pressure - 950.0) / (1050.0 - 950.0)).clamp(0.0, 1.0);
        Color::new(normalized, 0.2, 1.0 - normalized, 0.8)
    }
    
    fn wind_speed_to_color(&self, speed: f32) -> Color {
        // Map wind speed to color intensity
        let intensity = (speed / 30.0).clamp(0.0, 1.0); // 30 m/s max reference speed
        Color::new(intensity, 1.0 - intensity * 0.5, 0.0, 0.9)
    }
}

#[derive(Debug, Clone)]
pub struct WindVector3D {
    pub position: Vec3,
    pub velocity: Vec3,
    pub magnitude: f32,
    pub color: Color,
}
```

## 4. Multi-Layer Rendering System

### 4.1 Atmospheric Data Layer Architecture

**Layered Atmospheric Rendering**:
```rust
// New module: src/engine/rendering/atmospheric_layers.rs

/// Multi-layer atmospheric visualization system
pub struct AtmosphericLayerRenderer {
    pub layers: Vec<AtmosphericRenderLayer>,
    pub active_layers: HashSet<LayerType>,
    pub opacity_settings: HashMap<LayerType, f32>,
    pub mesh_generator: SphericalMeshGenerator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerType {
    /// Surface terrain and elevation
    Surface,
    /// Water bodies and ocean
    Water,
    /// Atmospheric pressure field
    Pressure,
    /// Wind vector field
    Wind,
    /// Temperature distribution
    Temperature,
    /// Humidity/moisture content
    Humidity,
    /// Weather systems (low/high pressure centers)
    WeatherSystems,
    /// Biome classification
    Biomes,
}

#[derive(Debug, Clone)]
pub struct AtmosphericRenderLayer {
    pub layer_type: LayerType,
    pub altitude_m: f64,
    pub mesh: Option<Mesh>,
    pub wind_vectors: Vec<WindVector3D>,
    pub opacity: f32,
    pub is_visible: bool,
    pub update_frequency: UpdateFrequency,
    pub last_update_frame: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum UpdateFrequency {
    /// Update every frame (for rapidly changing data)
    EveryFrame,
    /// Update every N frames (for stable data)
    EveryNFrames(u32),
    /// Update only when data changes
    OnDataChange,
}

impl AtmosphericLayerRenderer {
    pub fn new(
        domain: SphericalDomain, 
        transform: CoordinateTransform, 
        resolution: MeshResolution
    ) -> Self {
        let mesh_generator = SphericalMeshGenerator::new(domain, transform, resolution);
        
        let mut layers = Vec::new();
        let mut active_layers = HashSet::new();
        let mut opacity_settings = HashMap::new();
        
        // Initialize standard atmospheric layers
        layers.push(AtmosphericRenderLayer {
            layer_type: LayerType::Surface,
            altitude_m: 0.0,
            mesh: None,
            wind_vectors: Vec::new(),
            opacity: 1.0,
            is_visible: true,
            update_frequency: UpdateFrequency::OnDataChange,
            last_update_frame: 0,
        });
        
        layers.push(AtmosphericRenderLayer {
            layer_type: LayerType::Pressure,
            altitude_m: 1000.0, // 1km altitude for atmospheric visualization
            mesh: None,
            wind_vectors: Vec::new(),
            opacity: 0.7,
            is_visible: false, // Start hidden
            update_frequency: UpdateFrequency::EveryNFrames(5),
            last_update_frame: 0,
        });
        
        layers.push(AtmosphericRenderLayer {
            layer_type: LayerType::Wind,
            altitude_m: 500.0, // 500m altitude for wind vectors
            mesh: None,
            wind_vectors: Vec::new(),
            opacity: 0.9,
            is_visible: false, // Start hidden
            update_frequency: UpdateFrequency::EveryFrame,
            last_update_frame: 0,
        });
        
        // Initialize active layers and opacity settings
        active_layers.insert(LayerType::Surface);
        opacity_settings.insert(LayerType::Surface, 1.0);
        opacity_settings.insert(LayerType::Pressure, 0.7);
        opacity_settings.insert(LayerType::Wind, 0.9);
        opacity_settings.insert(LayerType::Temperature, 0.8);
        opacity_settings.insert(LayerType::Water, 0.6);
        
        Self {
            layers,
            active_layers,
            opacity_settings,
            mesh_generator,
        }
    }
    
    /// Update layer data from simulation state
    pub fn update_layers(&mut self, simulation: &Simulation, current_frame: u64) {
        for layer in &mut self.layers {
            if self.should_update_layer(layer, current_frame) {
                self.update_layer_data(layer, simulation);
                layer.last_update_frame = current_frame;
            }
        }
    }
    
    fn should_update_layer(&self, layer: &AtmosphericRenderLayer, current_frame: u64) -> bool {
        match layer.update_frequency {
            UpdateFrequency::EveryFrame => true,
            UpdateFrequency::EveryNFrames(n) => (current_frame - layer.last_update_frame) >= n as u64,
            UpdateFrequency::OnDataChange => {
                // TODO: Implement data change detection
                (current_frame - layer.last_update_frame) > 60 // Update every 60 frames as fallback
            }
        }
    }
    
    fn update_layer_data(&mut self, layer: &mut AtmosphericRenderLayer, simulation: &Simulation) {
        match layer.layer_type {
            LayerType::Surface => {
                layer.mesh = Some(self.mesh_generator.generate_surface_mesh(simulation.get_heightmap()));
            },
            LayerType::Pressure => {
                layer.mesh = Some(self.mesh_generator.generate_atmospheric_layer_mesh(
                    simulation.get_atmospheric_pressure_layer(), 
                    layer.altitude_m
                ));
            },
            LayerType::Wind => {
                layer.wind_vectors = self.mesh_generator.generate_wind_vectors(
                    simulation.get_wind_layer()
                );
            },
            LayerType::Temperature => {
                // Generate temperature layer mesh
                // TODO: Implement temperature layer mesh generation
            },
            _ => {
                // TODO: Implement other layer types
            }
        }
    }
    
    /// Render all active layers with proper depth sorting
    pub fn render(&self, camera: &SphericalCamera) {
        let view_matrix = camera.view_matrix();
        let projection_matrix = camera.projection_matrix();
        
        // Sort layers by altitude for proper transparency rendering
        let mut sorted_layers: Vec<&AtmosphericRenderLayer> = self.layers
            .iter()
            .filter(|layer| layer.is_visible && self.active_layers.contains(&layer.layer_type))
            .collect();
        
        sorted_layers.sort_by(|a, b| a.altitude_m.partial_cmp(&b.altitude_m).unwrap());
        
        // Render opaque layers first (back to front)
        for layer in &sorted_layers {
            if layer.opacity >= 1.0 {
                self.render_layer(layer, &view_matrix, &projection_matrix);
            }
        }
        
        // Render transparent layers (front to back for alpha blending)
        for layer in sorted_layers.iter().rev() {
            if layer.opacity < 1.0 {
                self.render_layer(layer, &view_matrix, &projection_matrix);
            }
        }
    }
    
    fn render_layer(&self, layer: &AtmosphericRenderLayer, view_matrix: &Mat4, projection_matrix: &Mat4) {
        // Set shader parameters
        let mvp_matrix = *projection_matrix * *view_matrix;
        
        // Render mesh if available
        if let Some(mesh) = &layer.mesh {
            self.render_mesh(mesh, &mvp_matrix, layer.opacity);
        }
        
        // Render wind vectors if available
        if !layer.wind_vectors.is_empty() {
            self.render_wind_vectors(&layer.wind_vectors, &mvp_matrix);
        }
    }
    
    fn render_mesh(&self, mesh: &Mesh, mvp_matrix: &Mat4, opacity: f32) {
        // Use macroquad's 3D rendering with custom shader for atmospheric data
        gl_use_material(self.get_atmospheric_material(opacity));
        
        for i in (0..mesh.indices.len()).step_by(3) {
            let v1 = mesh.vertices[mesh.indices[i] as usize];
            let v2 = mesh.vertices[mesh.indices[i + 1] as usize];  
            let v3 = mesh.vertices[mesh.indices[i + 2] as usize];
            
            draw_triangle_3d(v1.position, v2.position, v3.position, v1.color);
        }
        
        gl_use_default_material();
    }
    
    fn render_wind_vectors(&self, wind_vectors: &[WindVector3D], mvp_matrix: &Mat4) {
        for vector in wind_vectors {
            let end_position = vector.position + vector.velocity * 0.001; // Scale for visibility
            
            // Draw wind arrow as line
            draw_line_3d(vector.position, end_position, vector.color);
            
            // Draw arrowhead
            self.draw_3d_arrowhead(vector.position, end_position, vector.color);
        }
    }
    
    fn draw_3d_arrowhead(&self, start: Vec3, end: Vec3, color: Color) {
        let direction = (end - start).normalize();
        let length = 0.0002; // Arrowhead length
        
        // Create perpendicular vectors for arrowhead
        let perpendicular1 = direction.cross(Vec3::new(0.0, 0.0, 1.0)).normalize();
        let perpendicular2 = direction.cross(perpendicular1).normalize();
        
        let head_base = end - direction * length;
        let head_side1 = head_base + perpendicular1 * length * 0.3;
        let head_side2 = head_base + perpendicular2 * length * 0.3;
        let head_side3 = head_base - perpendicular1 * length * 0.3;
        let head_side4 = head_base - perpendicular2 * length * 0.3;
        
        // Draw arrowhead as lines
        draw_line_3d(end, head_side1, color);
        draw_line_3d(end, head_side2, color);
        draw_line_3d(end, head_side3, color);
        draw_line_3d(end, head_side4, color);
    }
    
    fn get_atmospheric_material(&self, opacity: f32) -> Material {
        // Create material with atmospheric data visualization properties
        load_material(
            "atmospheric_shader",
            include_str!("shaders/atmospheric.vert"),
            include_str!("shaders/atmospheric.frag"),
            MaterialParams {
                uniforms: vec![
                    ("opacity".to_string(), UniformType::Float1),
                ],
                ..Default::default()
            },
        ).unwrap()
    }
    
    /// Toggle layer visibility
    pub fn toggle_layer(&mut self, layer_type: LayerType) {
        if self.active_layers.contains(&layer_type) {
            self.active_layers.remove(&layer_type);
        } else {
            self.active_layers.insert(layer_type);
        }
        
        // Update layer visibility
        for layer in &mut self.layers {
            if layer.layer_type == layer_type {
                layer.is_visible = self.active_layers.contains(&layer_type);
            }
        }
    }
    
    /// Set layer opacity
    pub fn set_layer_opacity(&mut self, layer_type: LayerType, opacity: f32) {
        self.opacity_settings.insert(layer_type, opacity.clamp(0.0, 1.0));
        
        for layer in &mut self.layers {
            if layer.layer_type == layer_type {
                layer.opacity = opacity.clamp(0.0, 1.0);
            }
        }
    }
    
    /// Get current layer configuration for UI display
    pub fn get_layer_status(&self) -> Vec<LayerStatus> {
        self.layers
            .iter()
            .map(|layer| LayerStatus {
                layer_type: layer.layer_type,
                is_active: self.active_layers.contains(&layer.layer_type),
                opacity: layer.opacity,
                altitude_m: layer.altitude_m,
                update_frequency: layer.update_frequency,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct LayerStatus {
    pub layer_type: LayerType,
    pub is_active: bool,
    pub opacity: f32,
    pub altitude_m: f64,
    pub update_frequency: UpdateFrequency,
}
```

## 5. Real-Time Performance Optimization

### 5.1 Level-of-Detail System

**Adaptive LOD for Continental Scale**:
```rust
// New module: src/engine/rendering/performance_optimization.rs

/// Level-of-Detail management for spherical atmospheric rendering
pub struct SphericalLODManager {
    pub camera_distance_thresholds: Vec<f64>,
    pub mesh_resolutions: Vec<MeshResolution>,
    pub current_lod_level: usize,
    pub performance_target_fps: f32,
    pub frame_time_history: VecDeque<f32>,
}

impl SphericalLODManager {
    pub fn new(target_fps: f32) -> Self {
        // Define LOD levels based on camera distance
        let distance_thresholds = vec![
            10_000.0,     // < 10km: Highest detail
            50_000.0,     // < 50km: High detail
            200_000.0,    // < 200km: Medium detail  
            1_000_000.0,  // < 1000km: Low detail
            5_000_000.0,  // < 5000km: Very low detail
        ];
        
        let mesh_resolutions = vec![
            MeshResolution { longitude_segments: 512, latitude_segments: 256, altitude_layers: 16 },
            MeshResolution { longitude_segments: 256, latitude_segments: 128, altitude_layers: 12 },
            MeshResolution { longitude_segments: 128, latitude_segments: 64, altitude_layers: 8 },
            MeshResolution { longitude_segments: 64, latitude_segments: 32, altitude_layers: 6 },
            MeshResolution { longitude_segments: 32, latitude_segments: 16, altitude_layers: 4 },
        ];
        
        Self {
            camera_distance_thresholds: distance_thresholds,
            mesh_resolutions,
            current_lod_level: 2, // Start with medium detail
            performance_target_fps: target_fps,
            frame_time_history: VecDeque::with_capacity(60), // Track last 60 frames
        }
    }
    
    /// Update LOD based on camera distance and performance metrics
    pub fn update_lod(&mut self, camera: &SphericalCamera, delta_time: f32) -> bool {
        // Track frame time for performance monitoring
        self.frame_time_history.push_back(delta_time);
        if self.frame_time_history.len() > 60 {
            self.frame_time_history.pop_front();
        }
        
        let camera_altitude = camera.position.altitude;
        let new_lod_level = self.calculate_distance_based_lod(camera_altitude);
        
        // Apply performance-based adjustments
        let performance_adjusted_lod = self.apply_performance_adjustment(new_lod_level);
        
        if performance_adjusted_lod != self.current_lod_level {
            self.current_lod_level = performance_adjusted_lod;
            true // LOD changed, need to regenerate meshes
        } else {
            false
        }
    }
    
    fn calculate_distance_based_lod(&self, camera_altitude: f64) -> usize {
        for (i, &threshold) in self.camera_distance_thresholds.iter().enumerate() {
            if camera_altitude < threshold {
                return i;
            }
        }
        self.mesh_resolutions.len() - 1 // Lowest detail for extreme distances
    }
    
    fn apply_performance_adjustment(&self, base_lod: usize) -> usize {
        if self.frame_time_history.len() < 10 {
            return base_lod; // Not enough data yet
        }
        
        let average_frame_time = self.frame_time_history.iter().sum::<f32>() / 
                                self.frame_time_history.len() as f32;
        let current_fps = 1.0 / average_frame_time;
        
        if current_fps < self.performance_target_fps * 0.8 {
            // Performance is poor, reduce detail
            (base_lod + 1).min(self.mesh_resolutions.len() - 1)
        } else if current_fps > self.performance_target_fps * 1.2 && base_lod > 0 {
            // Performance is good, can increase detail
            base_lod - 1
        } else {
            base_lod
        }
    }
    
    /// Get current mesh resolution for LOD level
    pub fn get_current_resolution(&self) -> MeshResolution {
        self.mesh_resolutions[self.current_lod_level]
    }
    
    /// Get performance statistics for monitoring
    pub fn get_performance_stats(&self) -> PerformanceStats {
        if self.frame_time_history.is_empty() {
            return PerformanceStats::default();
        }
        
        let avg_frame_time = self.frame_time_history.iter().sum::<f32>() / 
                            self.frame_time_history.len() as f32;
        let current_fps = 1.0 / avg_frame_time;
        
        let min_frame_time = self.frame_time_history.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_frame_time = self.frame_time_history.iter().fold(0.0, |a, &b| a.max(b));
        
        PerformanceStats {
            current_fps,
            average_frame_time_ms: avg_frame_time * 1000.0,
            min_fps: 1.0 / max_frame_time,
            max_fps: 1.0 / min_frame_time,
            current_lod_level: self.current_lod_level,
            mesh_vertices: self.estimate_vertex_count(),
        }
    }
    
    fn estimate_vertex_count(&self) -> usize {
        let resolution = &self.mesh_resolutions[self.current_lod_level];
        // Estimate total vertices across all active layers
        (resolution.longitude_segments + 1) * (resolution.latitude_segments + 1) * 3 // 3 typical active layers
    }
}

#[derive(Debug, Default)]
pub struct PerformanceStats {
    pub current_fps: f32,
    pub average_frame_time_ms: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub current_lod_level: usize,
    pub mesh_vertices: usize,
}
```

### 5.2 Frustum Culling and Occlusion

**Spherical Frustum Culling**:
```rust
/// Frustum culling optimized for spherical domains
pub struct SphericalFrustumCuller {
    pub view_frustum: Frustum,
    pub camera_position: Vec3,
    pub planet_radius: f64,
}

impl SphericalFrustumCuller {
    pub fn new(camera: &SphericalCamera) -> Self {
        let view_matrix = camera.view_matrix();
        let projection_matrix = camera.projection_matrix();
        let view_projection = projection_matrix * view_matrix;
        
        let frustum = Frustum::from_matrix(view_projection);
        let camera_cart = camera.position.to_cartesian();
        
        Self {
            view_frustum: frustum,
            camera_position: Vec3::new(camera_cart.0 as f32, camera_cart.1 as f32, camera_cart.2 as f32),
            planet_radius: camera.position.planet_radius,
        }
    }
    
    /// Test if a spherical region is visible in current view
    pub fn is_region_visible(&self, region: SphericalRegion) -> bool {
        // Convert spherical region to bounding sphere
        let region_center = region.center_cartesian();
        let region_radius = region.bounding_radius();
        
        // Test against view frustum
        if !self.view_frustum.contains_sphere(region_center, region_radius) {
            return false;
        }
        
        // Test for horizon culling (if camera is above surface)
        if self.camera_position.length() > self.planet_radius as f32 {
            let horizon_dot = self.calculate_horizon_dot_product(region_center);
            if horizon_dot < 0.0 {
                return false; // Beyond horizon
            }
        }
        
        true
    }
    
    fn calculate_horizon_dot_product(&self, region_center: Vec3) -> f32 {
        let to_region = (region_center - self.camera_position).normalize();
        let to_planet_center = -self.camera_position.normalize();
        to_region.dot(to_planet_center)
    }
    
    /// Batch cull atmospheric mesh segments
    pub fn cull_mesh_segments(&self, segments: &[MeshSegment]) -> Vec<bool> {
        segments.iter()
            .map(|segment| self.is_region_visible(segment.bounding_region))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct SphericalRegion {
    pub center_longitude: f64,
    pub center_latitude: f64,
    pub longitude_extent: f64,
    pub latitude_extent: f64,
    pub altitude: f64,
}

impl SphericalRegion {
    fn center_cartesian(&self) -> Vec3 {
        let coords = SphericalCoordinates {
            longitude: self.center_longitude,
            latitude: self.center_latitude,
            altitude: self.altitude,
            planet_radius: SphericalCoordinates::EARTH_RADIUS_M,
        };
        let (x, y, z) = coords.to_cartesian();
        Vec3::new(x as f32, y as f32, z as f32)
    }
    
    fn bounding_radius(&self) -> f32 {
        // Conservative estimate of bounding sphere radius
        let max_angular_extent = self.longitude_extent.max(self.latitude_extent);
        (max_angular_extent * (SphericalCoordinates::EARTH_RADIUS_M + self.altitude)) as f32
    }
}

#[derive(Debug, Clone)]
pub struct MeshSegment {
    pub bounding_region: SphericalRegion,
    pub vertex_range: Range<usize>,
    pub index_range: Range<usize>,
}
```

### 5.3 Asynchronous Data Streaming

**Background Data Processing**:
```rust
use tokio::sync::mpsc;
use std::sync::Arc;

/// Asynchronous atmospheric data processing for real-time rendering
pub struct AtmosphericDataStreamer {
    pub mesh_update_sender: mpsc::UnboundedSender<MeshUpdateRequest>,
    pub mesh_update_receiver: mpsc::UnboundedReceiver<MeshUpdateResponse>,
    pub processing_tasks: Vec<tokio::task::JoinHandle<()>>,
}

#[derive(Debug)]
pub enum MeshUpdateRequest {
    UpdateSurface {
        heightmap: Arc<HeightMap>,
        resolution: MeshResolution,
        domain: SphericalDomain,
    },
    UpdateAtmospheric {
        layer_type: LayerType,
        atmospheric_data: Arc<AtmosphericPressureLayer>,
        altitude: f64,
        resolution: MeshResolution,
        domain: SphericalDomain,
    },
    UpdateWindVectors {
        wind_layer: Arc<WindLayer>,
        domain: SphericalDomain,
        sample_rate: usize,
    },
}

#[derive(Debug)]
pub enum MeshUpdateResponse {
    SurfaceMeshReady {
        mesh: Mesh,
    },
    AtmosphericMeshReady {
        layer_type: LayerType,
        mesh: Mesh,
    },
    WindVectorsReady {
        vectors: Vec<WindVector3D>,
    },
}

impl AtmosphericDataStreamer {
    pub fn new() -> Self {
        let (mesh_sender, mut mesh_receiver) = mpsc::unbounded_channel();
        let (response_sender, response_receiver) = mpsc::unbounded_channel();
        
        // Spawn background processing task
        let processing_task = tokio::spawn(async move {
            while let Some(request) = mesh_receiver.recv().await {
                let response = Self::process_mesh_request(request).await;
                if response_sender.send(response).is_err() {
                    break; // Channel closed
                }
            }
        });
        
        Self {
            mesh_update_sender: mesh_sender,
            mesh_update_receiver: response_receiver,
            processing_tasks: vec![processing_task],
        }
    }
    
    async fn process_mesh_request(request: MeshUpdateRequest) -> MeshUpdateResponse {
        match request {
            MeshUpdateRequest::UpdateSurface { heightmap, resolution, domain } => {
                // Process surface mesh generation on background thread
                let transform = CoordinateTransform::new(domain, (resolution.longitude_segments, resolution.latitude_segments), WorldScale::new(1000.0, (240, 120), crate::engine::core::scale::DetailLevel::Standard));
                let mesh_generator = SphericalMeshGenerator::new(domain, transform, resolution);
                let mesh = mesh_generator.generate_surface_mesh(&heightmap);
                
                MeshUpdateResponse::SurfaceMeshReady { mesh }
            },
            MeshUpdateRequest::UpdateAtmospheric { layer_type, atmospheric_data, altitude, resolution, domain } => {
                // Process atmospheric mesh generation
                let transform = CoordinateTransform::new(domain, (resolution.longitude_segments, resolution.latitude_segments), WorldScale::new(1000.0, (240, 120), crate::engine::core::scale::DetailLevel::Standard));
                let mesh_generator = SphericalMeshGenerator::new(domain, transform, resolution);
                let mesh = mesh_generator.generate_atmospheric_layer_mesh(&atmospheric_data, altitude);
                
                MeshUpdateResponse::AtmosphericMeshReady { layer_type, mesh }
            },
            MeshUpdateRequest::UpdateWindVectors { wind_layer, domain, sample_rate } => {
                // Process wind vector generation
                let transform = CoordinateTransform::new(domain, (240, 120), WorldScale::new(1000.0, (240, 120), crate::engine::core::scale::DetailLevel::Standard));
                let mesh_generator = SphericalMeshGenerator::new(domain, transform, MeshResolution::adaptive_for_domain(&domain));
                let vectors = mesh_generator.generate_wind_vectors(&wind_layer);
                
                MeshUpdateResponse::WindVectorsReady { vectors }
            },
        }
    }
    
    /// Queue mesh update request (non-blocking)
    pub fn request_mesh_update(&self, request: MeshUpdateRequest) -> Result<(), String> {
        self.mesh_update_sender.send(request)
            .map_err(|_| "Failed to queue mesh update request".to_string())
    }
    
    /// Check for completed mesh updates (non-blocking)
    pub fn poll_mesh_updates(&mut self) -> Vec<MeshUpdateResponse> {
        let mut responses = Vec::new();
        while let Ok(response) = self.mesh_update_receiver.try_recv() {
            responses.push(response);
        }
        responses
    }
}
```

## 6. User Interface Design

### 6.1 3D Navigation Controls

**Intuitive Planetary Navigation**:
```rust
// New module: src/engine/rendering/ui_3d.rs
use ratatui::prelude::*;

/// 3D User Interface for spherical atmospheric simulation
pub struct Spherical3DUI {
    pub camera_mode: CameraMode,
    pub active_layers: HashSet<LayerType>,
    pub layer_opacities: HashMap<LayerType, f32>,
    pub navigation_mode: NavigationMode,
    pub selected_coordinates: Option<SphericalCoordinates>,
    pub performance_monitor: bool,
    pub help_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavigationMode {
    /// Free camera movement
    FreeCam,
    /// Camera orbits around target point
    OrbitTarget,
    /// Camera follows atmospheric features
    FollowWeather,
    /// Fixed camera with rotation only
    FixedPosition,
}

impl Spherical3DUI {
    pub fn new() -> Self {
        let mut active_layers = HashSet::new();
        active_layers.insert(LayerType::Surface);
        
        let mut layer_opacities = HashMap::new();
        layer_opacities.insert(LayerType::Surface, 1.0);
        layer_opacities.insert(LayerType::Pressure, 0.7);
        layer_opacities.insert(LayerType::Wind, 0.9);
        layer_opacities.insert(LayerType::Temperature, 0.8);
        
        Self {
            camera_mode: CameraMode::Surface,
            active_layers,
            layer_opacities,
            navigation_mode: NavigationMode::OrbitTarget,
            selected_coordinates: None,
            performance_monitor: false,
            help_visible: false,
        }
    }
    
    /// Handle keyboard input for 3D navigation
    pub fn handle_keyboard_input(&mut self, camera: &mut SphericalCamera, layer_renderer: &mut AtmosphericLayerRenderer) {
        // Camera mode switching
        if is_key_pressed(KeyCode::F1) {
            camera.view_mode = CameraMode::Orbital;
            self.camera_mode = CameraMode::Orbital;
        }
        if is_key_pressed(KeyCode::F2) {
            camera.view_mode = CameraMode::Surface;
            self.camera_mode = CameraMode::Surface;
        }
        if is_key_pressed(KeyCode::F3) {
            camera.view_mode = CameraMode::Atmospheric;
            self.camera_mode = CameraMode::Atmospheric;
        }
        if is_key_pressed(KeyCode::F4) {
            camera.view_mode = CameraMode::CrossSection;
            self.camera_mode = CameraMode::CrossSection;
        }
        
        // Layer visibility toggles
        if is_key_pressed(KeyCode::Key1) {
            layer_renderer.toggle_layer(LayerType::Surface);
            self.toggle_layer(LayerType::Surface);
        }
        if is_key_pressed(KeyCode::Key2) {
            layer_renderer.toggle_layer(LayerType::Water);
            self.toggle_layer(LayerType::Water);
        }
        if is_key_pressed(KeyCode::Key3) {
            layer_renderer.toggle_layer(LayerType::Pressure);
            self.toggle_layer(LayerType::Pressure);
        }
        if is_key_pressed(KeyCode::Key4) {
            layer_renderer.toggle_layer(LayerType::Wind);
            self.toggle_layer(LayerType::Wind);
        }
        if is_key_pressed(KeyCode::Key5) {
            layer_renderer.toggle_layer(LayerType::Temperature);
            self.toggle_layer(LayerType::Temperature);
        }
        if is_key_pressed(KeyCode::Key6) {
            layer_renderer.toggle_layer(LayerType::WeatherSystems);
            self.toggle_layer(LayerType::WeatherSystems);
        }
        if is_key_pressed(KeyCode::Key7) {
            layer_renderer.toggle_layer(LayerType::Biomes);
            self.toggle_layer(LayerType::Biomes);
        }
        
        // Navigation mode switching
        if is_key_pressed(KeyCode::N) {
            self.cycle_navigation_mode();
        }
        
        // Layer opacity adjustment
        if is_key_down(KeyCode::LeftShift) {
            if is_key_pressed(KeyCode::Key3) { // Shift+3 for pressure opacity
                self.adjust_layer_opacity(LayerType::Pressure, 0.1);
                layer_renderer.set_layer_opacity(LayerType::Pressure, self.layer_opacities[&LayerType::Pressure]);
            }
            if is_key_pressed(KeyCode::Key4) { // Shift+4 for wind opacity
                self.adjust_layer_opacity(LayerType::Wind, 0.1);
                layer_renderer.set_layer_opacity(LayerType::Wind, self.layer_opacities[&LayerType::Wind]);
            }
        }
        if is_key_down(KeyCode::LeftControl) {
            if is_key_pressed(KeyCode::Key3) { // Ctrl+3 for pressure opacity down
                self.adjust_layer_opacity(LayerType::Pressure, -0.1);
                layer_renderer.set_layer_opacity(LayerType::Pressure, self.layer_opacities[&LayerType::Pressure]);
            }
            if is_key_pressed(KeyCode::Key4) { // Ctrl+4 for wind opacity down
                self.adjust_layer_opacity(LayerType::Wind, -0.1);
                layer_renderer.set_layer_opacity(LayerType::Wind, self.layer_opacities[&LayerType::Wind]);
            }
        }
        
        // Toggle UI elements
        if is_key_pressed(KeyCode::F10) {
            self.performance_monitor = !self.performance_monitor;
        }
        if is_key_pressed(KeyCode::F1) {
            self.help_visible = !self.help_visible;
        }
    }
    
    /// Handle mouse input for camera control
    pub fn handle_mouse_input(&mut self, camera: &mut SphericalCamera, domain: &SphericalDomain) {
        let mouse_pos = mouse_position();
        let (_wheel_x, wheel_y) = mouse_wheel();
        
        match self.navigation_mode {
            NavigationMode::FreeCam => {
                // Free camera movement with mouse
                if is_mouse_button_down(MouseButton::Left) {
                    // Mouse drag for camera rotation
                    // TODO: Implement mouse delta tracking
                }
            },
            NavigationMode::OrbitTarget => {
                // Orbital camera around target
                if is_mouse_button_down(MouseButton::Left) {
                    // Orbit around target based on mouse movement
                    // TODO: Implement orbital controls
                }
            },
            NavigationMode::FollowWeather => {
                // Camera follows weather systems
                // TODO: Implement weather feature tracking
            },
            NavigationMode::FixedPosition => {
                // Only rotation, no translation
                // TODO: Implement rotation-only controls
            },
        }
        
        // Zoom with mouse wheel
        if wheel_y != 0.0 {
            let zoom_factor = if wheel_y > 0.0 { 0.9 } else { 1.1 };
            camera.position.altitude *= zoom_factor;
            
            // Clamp zoom limits based on domain size
            let (extent_x, extent_y) = domain.physical_extent_km();
            let max_extent_km = extent_x.max(extent_y);
            camera.position.altitude = camera.position.altitude
                .max(1000.0) // Minimum 1km altitude
                .min(max_extent_km * 1000.0 * 5.0); // Maximum 5x domain extent
        }
        
        // Right-click to select coordinates
        if is_mouse_button_pressed(MouseButton::Right) {
            self.selected_coordinates = self.screen_to_sphere_coordinates(mouse_pos, camera, domain);
        }
    }
    
    fn toggle_layer(&mut self, layer_type: LayerType) {
        if self.active_layers.contains(&layer_type) {
            self.active_layers.remove(&layer_type);
        } else {
            self.active_layers.insert(layer_type);
        }
    }
    
    fn adjust_layer_opacity(&mut self, layer_type: LayerType, delta: f32) {
        if let Some(opacity) = self.layer_opacities.get_mut(&layer_type) {
            *opacity = (*opacity + delta).clamp(0.0, 1.0);
        }
    }
    
    fn cycle_navigation_mode(&mut self) {
        self.navigation_mode = match self.navigation_mode {
            NavigationMode::FreeCam => NavigationMode::OrbitTarget,
            NavigationMode::OrbitTarget => NavigationMode::FollowWeather,
            NavigationMode::FollowWeather => NavigationMode::FixedPosition,
            NavigationMode::FixedPosition => NavigationMode::FreeCam,
        };
    }
    
    fn screen_to_sphere_coordinates(
        &self, 
        screen_pos: (f32, f32), 
        camera: &SphericalCamera, 
        domain: &SphericalDomain
    ) -> Option<SphericalCoordinates> {
        // Ray casting from screen coordinates to sphere surface
        // TODO: Implement proper ray-sphere intersection
        None
    }
    
    /// Render 3D UI overlay information
    pub fn render_ui_overlay(&self, performance_stats: &PerformanceStats, simulation_time: &crate::engine::sim::SimulationTime) {
        // Performance monitor (top-left corner)
        if self.performance_monitor {
            self.render_performance_monitor(performance_stats);
        }
        
        // Layer status (left sidebar)
        self.render_layer_status_panel();
        
        // Camera information (top-right corner)
        self.render_camera_info();
        
        // Simulation time and controls (bottom bar)
        self.render_simulation_controls(simulation_time);
        
        // Coordinate information (bottom-right)
        if let Some(coords) = self.selected_coordinates {
            self.render_coordinate_info(coords);
        }
        
        // Help overlay
        if self.help_visible {
            self.render_help_overlay();
        }
    }
    
    fn render_performance_monitor(&self, stats: &PerformanceStats) {
        let monitor_text = format!(
            "Performance Monitor\nFPS: {:.1} ({:.1}-{:.1})\nFrame: {:.1}ms\nLOD: {} ({} vertices)\nCamera: {:?}\nNavigation: {:?}",
            stats.current_fps,
            stats.min_fps,
            stats.max_fps,
            stats.average_frame_time_ms,
            stats.current_lod_level,
            stats.mesh_vertices,
            self.camera_mode,
            self.navigation_mode,
        );
        
        draw_text(&monitor_text, 10.0, 30.0, 14.0, WHITE);
    }
    
    fn render_layer_status_panel(&self) {
        let mut y_pos = 120.0;
        let x_pos = 10.0;
        
        draw_text("ATMOSPHERIC LAYERS", x_pos, y_pos, 16.0, WHITE);
        y_pos += 25.0;
        
        let layer_info = [
            (LayerType::Surface, "1 - Surface/Terrain", GREEN),
            (LayerType::Water, "2 - Water Bodies", BLUE),
            (LayerType::Pressure, "3 - Pressure Field", RED),
            (LayerType::Wind, "4 - Wind Vectors", YELLOW),
            (LayerType::Temperature, "5 - Temperature", ORANGE),
            (LayerType::WeatherSystems, "6 - Weather Systems", PURPLE),
            (LayerType::Biomes, "7 - Biomes", DARKGREEN),
        ];
        
        for (layer_type, label, color) in layer_info.iter() {
            let is_active = self.active_layers.contains(layer_type);
            let opacity = self.layer_opacities.get(layer_type).copied().unwrap_or(1.0);
            
            let status_color = if is_active { *color } else { GRAY };
            let status_text = if is_active {
                format!("{} ({}%)", label, (opacity * 100.0) as u8)
            } else {
                format!("{} (OFF)", label)
            };
            
            draw_text(&status_text, x_pos, y_pos, 12.0, status_color);
            y_pos += 18.0;
        }
        
        // Navigation instructions
        y_pos += 10.0;
        draw_text("CONTROLS", x_pos, y_pos, 16.0, WHITE);
        y_pos += 20.0;
        
        let controls = [
            "WASD - Camera movement",
            "Mouse - Look around",
            "Scroll - Zoom in/out",
            "N - Navigation mode",
            "F1-F4 - Camera modes",
            "Shift+Number - Increase opacity",
            "Ctrl+Number - Decrease opacity",
            "F10 - Performance monitor",
        ];
        
        for control in controls.iter() {
            draw_text(control, x_pos, y_pos, 10.0, LIGHTGRAY);
            y_pos += 15.0;
        }
    }
    
    fn render_camera_info(&self) {
        let camera_info = format!(
            "Camera Mode: {:?}\nNavigation: {:?}",
            self.camera_mode,
            self.navigation_mode,
        );
        
        draw_text(&camera_info, screen_width() - 200.0, 30.0, 14.0, WHITE);
    }
    
    fn render_simulation_controls(&self, sim_time: &crate::engine::sim::SimulationTime) {
        let time_info = format!(
            "Simulation Time: Day {}, {:02}:{:02} | SPACE: Pause/Play | R: Reset View",
            sim_time.days, sim_time.hours, sim_time.minutes
        );
        
        draw_text(&time_info, screen_width() * 0.5 - 300.0, screen_height() - 20.0, 12.0, LIGHTGRAY);
    }
    
    fn render_coordinate_info(&self, coords: SphericalCoordinates) {
        let coord_info = format!(
            "Selected Location:\nLat: {:.3}° Lon: {:.3}°\nAlt: {:.0}m",
            coords.latitude.to_degrees(),
            coords.longitude.to_degrees(),
            coords.altitude,
        );
        
        draw_text(&coord_info, screen_width() - 200.0, screen_height() - 80.0, 12.0, WHITE);
    }
    
    fn render_help_overlay(&self) {
        let help_text = "3D ATMOSPHERIC SIMULATION HELP\n\nCAMERA MODES:\nF1 - Orbital View (planetary perspective)\nF2 - Surface View (continental focus)\nF3 - Atmospheric View (weather analysis)\nF4 - Cross-Section (vertical structure)\n\nLAYER VISUALIZATION:\n1-7 - Toggle atmospheric layers\nShift+Number - Increase layer opacity\nCtrl+Number - Decrease layer opacity\n\nNAVIGATION:\nWASD - Move camera\nMouse - Look around\nScroll - Zoom in/out\nN - Cycle navigation modes\nRight-click - Select coordinates\n\nPress F1 again to close help";
        
        // Semi-transparent background
        draw_rectangle(screen_width() * 0.25, screen_height() * 0.25, screen_width() * 0.5, screen_height() * 0.5, Color::new(0.0, 0.0, 0.0, 0.8));
        
        // Help text
        draw_text(help_text, screen_width() * 0.25 + 20.0, screen_height() * 0.25 + 30.0, 12.0, WHITE);
    }
}
```

## 7. TDD Sprint Breakdown

### 7.1 Sprint Planning Framework

**User Story Template for 3D Rendering**:
```
As a [atmospheric researcher/game developer], 
I want [specific 3D rendering capability] 
so that [atmospheric physics visualization benefit]

Acceptance Criteria:
- [ ] Functional requirement (what it does)
- [ ] Performance requirement (how fast it runs)
- [ ] Visual quality requirement (how it looks)
- [ ] Integration requirement (how it connects)
- [ ] Physics accuracy requirement (how correct it is)

TDD Tests:
- Unit tests for mathematical transformations
- Integration tests for rendering pipeline
- Performance tests for real-time requirements
- Visual regression tests for rendering consistency

Commit Message: "[type]: [concise description of 3D capability]"
```

### 7.2 Sprint 1: Spherical Coordinate Foundation (15-20 hours)

**Sprint Goal**: Establish mathematical foundation for spherical coordinate systems

**User Story 1.1**: Spherical Coordinate System Implementation
```
As an atmospheric researcher, I want accurate spherical coordinate transformations so that atmospheric data can be properly mapped from rectangular grids to planetary geometry.

Acceptance Criteria:
- [ ] SphericalCoordinates struct with longitude/latitude/altitude
- [ ] Accurate Cartesian to spherical conversion (< 0.01% error)
- [ ] Grid index to spherical coordinate mapping
- [ ] Great circle distance calculations (Haversine formula)
- [ ] All transformations pass dimensional analysis validation

TDD Tests:
- test_cartesian_to_spherical_conversion()
- test_spherical_to_cartesian_conversion()
- test_great_circle_distance_accuracy()
- test_grid_to_spherical_mapping()
- test_coordinate_boundary_conditions()

Commit Message: "feat: implement spherical coordinate system with accurate transformations"
```

**User Story 1.2**: Domain Definition System
```
As a simulation engineer, I want flexible spherical domain definitions so that atmospheric simulations can handle continental to global scale regions.

Acceptance Criteria:
- [ ] SphericalDomain with lat/lon boundaries
- [ ] Continental, hemispheric, and global domain types
- [ ] Physical extent calculations in kilometers
- [ ] Domain containment tests for coordinates
- [ ] Integration with existing WorldScale system

TDD Tests:
- test_continental_domain_creation()
- test_global_domain_coverage()
- test_domain_extent_calculations()
- test_coordinate_containment()
- test_worldscale_integration()

Commit Message: "feat: add spherical domain definition system for multi-scale atmospheric simulation"
```

**User Story 1.3**: Coordinate Transformation Framework
```
As a rendering engineer, I want efficient coordinate transformation utilities so that atmospheric data can be converted between grid indices and spherical coordinates in real-time.

Acceptance Criteria:
- [ ] CoordinateTransform struct with cached parameters
- [ ] Grid to spherical transformation (< 1μs per coordinate)
- [ ] Spherical to grid transformation with bounds checking
- [ ] Coriolis parameter calculation at any latitude
- [ ] Metric factors for numerical derivatives

TDD Tests:
- test_coordinate_transform_performance()
- test_grid_spherical_roundtrip_accuracy()
- test_coriolis_parameter_calculation()
- test_metric_factors_accuracy()
- test_boundary_condition_handling()

Commit Message: "feat: implement coordinate transformation framework with performance optimization"
```

### 7.3 Sprint 2: 3D Camera System (12-16 hours)

**Sprint Goal**: Create intuitive 3D camera system for planetary visualization

**User Story 2.1**: Spherical Camera Implementation
```
As a user, I want smooth 3D camera controls for planetary atmospheric visualization so that I can navigate from surface detail to global overview seamlessly.

Acceptance Criteria:
- [ ] SphericalCamera with position/target in spherical coordinates
- [ ] Orbital, surface, atmospheric, and cross-section view modes
- [ ] Smooth view/projection matrix generation
- [ ] Configurable FOV and clipping planes based on domain scale
- [ ] Camera parameter validation for stability

TDD Tests:
- test_camera_view_matrix_generation()
- test_camera_mode_transitions()
- test_projection_matrix_accuracy()
- test_camera_parameter_validation()
- test_zoom_limits_enforcement()

Commit Message: "feat: implement spherical camera system with multiple view modes"
```

**User Story 2.2**: Camera Input Handling
```
As a user, I want intuitive camera controls (WASD, mouse, scroll) so that I can explore atmospheric phenomena naturally in 3D space.

Acceptance Criteria:
- [ ] WASD keys for camera movement in all modes
- [ ] Mouse look controls for rotation
- [ ] Scroll wheel zoom with appropriate limits
- [ ] Mode-specific input behavior (orbital vs surface)
- [ ] Smooth camera transitions between positions

TDD Tests:
- test_keyboard_camera_movement()
- test_mouse_look_controls()
- test_zoom_boundary_constraints()
- test_camera_mode_input_differences()
- test_smooth_transition_interpolation()

Commit Message: "feat: add intuitive input controls for 3D atmospheric camera navigation"
```

### 7.4 Sprint 3: Spherical Mesh Generation (18-24 hours)

**Sprint Goal**: Generate 3D meshes for atmospheric data visualization on spherical geometry

**User Story 3.1**: Surface Mesh Generation
```
As a visualization system, I want accurate surface meshes representing terrain on spherical geometry so that elevation data appears correctly in 3D planetary view.

Acceptance Criteria:
- [ ] Adaptive mesh resolution based on domain size
- [ ] Surface mesh generation from HeightMap data
- [ ] Proper vertex positioning with elevation mapping
- [ ] Triangle index generation for efficient rendering
- [ ] Color mapping from elevation to visual representation

TDD Tests:
- test_surface_mesh_vertex_accuracy()
- test_triangle_index_generation()
- test_elevation_color_mapping()
- test_adaptive_resolution_scaling()
- test_mesh_topology_validation()

Commit Message: "feat: implement spherical surface mesh generation for terrain visualization"
```

**User Story 3.2**: Atmospheric Layer Meshes
```
As an atmospheric researcher, I want 3D atmospheric layer visualization so that pressure fields, temperature, and other atmospheric data can be viewed as layers above the planetary surface.

Acceptance Criteria:
- [ ] Atmospheric layer mesh generation at specified altitudes
- [ ] Color mapping from atmospheric data to mesh vertices
- [ ] Multiple altitude levels for 3D atmospheric structure
- [ ] Transparency support for layered visualization
- [ ] Integration with existing atmospheric data structures

TDD Tests:
- test_atmospheric_layer_mesh_generation()
- test_altitude_positioning_accuracy()
- test_atmospheric_data_color_mapping()
- test_transparency_rendering_support()
- test_multi_layer_vertex_alignment()

Commit Message: "feat: add atmospheric layer mesh generation for 3D weather visualization"
```

**User Story 3.3**: Wind Vector Field Visualization
```
As a meteorologist, I want 3D wind vector visualization so that atmospheric circulation patterns can be clearly seen in their proper spatial context.

Acceptance Criteria:
- [ ] 3D wind vector generation from WindLayer data
- [ ] Surface wind vector transformation to 3D Cartesian
- [ ] Wind speed color coding and arrow scaling
- [ ] Adaptive sampling rate based on domain size
- [ ] Wind vector clipping for performance

TDD Tests:
- test_wind_vector_3d_transformation()
- test_wind_speed_color_accuracy()
- test_vector_sampling_rate_adaptation()
- test_wind_vector_clipping()
- test_surface_to_cartesian_conversion()

Commit Message: "feat: implement 3D wind vector field visualization for atmospheric circulation"
```

### 7.5 Sprint 4: Multi-Layer Rendering System (20-26 hours)

**Sprint Goal**: Create comprehensive multi-layer atmospheric visualization system

**User Story 4.1**: Layer Management System
```
As a user, I want to control which atmospheric layers are visible so that I can focus on specific atmospheric phenomena while maintaining context.

Acceptance Criteria:
- [ ] AtmosphericLayerRenderer with configurable layers
- [ ] Layer visibility toggles for all atmospheric data types
- [ ] Individual layer opacity controls (0-100%)
- [ ] Layer update frequency management (performance optimization)
- [ ] Layer rendering order with proper transparency

TDD Tests:
- test_layer_visibility_management()
- test_opacity_control_accuracy()
- test_update_frequency_optimization()
- test_transparency_rendering_order()
- test_layer_performance_impact()

Commit Message: "feat: implement comprehensive atmospheric layer management system"
```

**User Story 4.2**: Real-Time Layer Updates
```
As a simulation system, I want efficient layer updates from atmospheric data so that 3D visualization stays synchronized with atmospheric physics in real-time.

Acceptance Criteria:
- [ ] Automatic layer updates when simulation data changes
- [ ] Selective updates based on data change detection
- [ ] Frame rate impact minimization (<5ms per layer update)
- [ ] Asynchronous mesh generation for heavy operations
- [ ] Error handling for malformed atmospheric data

TDD Tests:
- test_automatic_layer_synchronization()
- test_data_change_detection()
- test_update_performance_bounds()
- test_asynchronous_processing()
- test_error_recovery_handling()

Commit Message: "feat: add real-time atmospheric layer updates with performance optimization"
```

### 7.6 Sprint 5: Performance Optimization (16-22 hours)

**Sprint Goal**: Ensure real-time performance for continental-scale atmospheric rendering

**User Story 5.1**: Level-of-Detail System
```
As a performance engineer, I want adaptive level-of-detail for 3D atmospheric rendering so that the system maintains target framerate across different scales and hardware.

Acceptance Criteria:
- [ ] Automatic LOD based on camera distance
- [ ] Performance-based LOD adjustment (maintain 30+ FPS)
- [ ] Configurable mesh resolution levels
- [ ] Frame time monitoring and LOD feedback
- [ ] Smooth transitions between LOD levels

TDD Tests:
- test_distance_based_lod_calculation()
- test_performance_lod_adjustment()
- test_framerate_target_maintenance()
- test_lod_transition_smoothness()
- test_mesh_resolution_scaling()

Commit Message: "feat: implement adaptive level-of-detail system for real-time atmospheric rendering"
```

**User Story 5.2**: Frustum Culling and Occlusion
```
As a rendering system, I want efficient frustum culling for spherical geometry so that only visible atmospheric regions are processed, improving performance.

Acceptance Criteria:
- [ ] View frustum culling for spherical mesh segments
- [ ] Horizon culling for planetary geometry
- [ ] Batch culling operations for multiple mesh segments
- [ ] Occlusion culling for hidden atmospheric layers
- [ ] Performance improvement of 40%+ for large domains

TDD Tests:
- test_frustum_culling_accuracy()
- test_horizon_culling_correctness()
- test_batch_culling_performance()
- test_occlusion_detection()
- test_performance_improvement_measurement()

Commit Message: "feat: add frustum and occlusion culling for spherical atmospheric rendering"
```

### 7.7 Sprint 6: User Interface Integration (14-18 hours)

**Sprint Goal**: Create intuitive 3D user interface for atmospheric simulation control

**User Story 6.1**: 3D Navigation Interface
```
As a user, I want clear visual feedback and controls for 3D atmospheric navigation so that I can efficiently explore atmospheric phenomena.

Acceptance Criteria:
- [ ] Layer status panel with visibility and opacity controls
- [ ] Camera mode indicators and switching controls
- [ ] Navigation mode selection (orbit, free cam, follow weather)
- [ ] Coordinate information display for selected points
- [ ] Help overlay with control explanations

TDD Tests:
- test_layer_status_display_accuracy()
- test_camera_mode_indication()
- test_navigation_mode_switching()
- test_coordinate_selection_accuracy()
- test_help_overlay_completeness()

Commit Message: "feat: implement 3D navigation interface for atmospheric simulation"
```

**User Story 6.2**: Performance Monitoring Integration
```
As a developer, I want real-time performance monitoring in the 3D interface so that I can optimize atmospheric rendering performance.

Acceptance Criteria:
- [ ] Frame rate display (current, min, max)
- [ ] LOD level indicator with vertex count
- [ ] Frame time graph for performance analysis
- [ ] Memory usage tracking for mesh data
- [ ] Toggle-able performance overlay (F10 key)

TDD Tests:
- test_framerate_monitoring_accuracy()
- test_lod_indicator_synchronization()
- test_performance_graph_updates()
- test_memory_tracking_accuracy()
- test_overlay_toggle_functionality()

Commit Message: "feat: add real-time performance monitoring to 3D atmospheric interface"
```

### 7.8 Sprint 7: Integration and System Testing (12-16 hours)

**Sprint Goal**: Integrate 3D spherical rendering with existing atmospheric physics system

**User Story 7.1**: Physics System Integration
```
As an atmospheric simulation, I want seamless integration between 3D spherical rendering and physics-correct atmospheric dynamics so that visualization accurately represents the underlying atmospheric model.

Acceptance Criteria:
- [ ] 3D rendering of existing AtmosphericPressureLayer data
- [ ] Spherical coordinate integration with Coriolis calculations
- [ ] Wind vector visualization matches geostrophic wind calculations
- [ ] Temperature and pressure field accuracy in 3D space
- [ ] Boundary condition visualization for different domain types

TDD Tests:
- test_atmospheric_data_rendering_accuracy()
- test_coriolis_visualization_consistency()
- test_wind_field_physics_accuracy()
- test_temperature_pressure_field_integrity()
- test_boundary_condition_representation()

Commit Message: "feat: integrate 3D spherical rendering with physics-correct atmospheric simulation"
```

**User Story 7.2**: End-to-End System Validation
```
As a quality assurance engineer, I want comprehensive end-to-end testing of the 3D atmospheric rendering system so that it meets all performance and accuracy requirements.

Acceptance Criteria:
- [ ] Continental scale domains (100-2000km) render at >30 FPS
- [ ] Global scale domains (40000km) render at >15 FPS
- [ ] Coordinate transformation accuracy <0.01% error
- [ ] Memory usage <2GB for largest domains
- [ ] Stable operation for >1 hour continuous use

TDD Tests:
- test_continental_performance_requirements()
- test_global_scale_performance()
- test_coordinate_accuracy_validation()
- test_memory_usage_limits()
- test_long_term_stability()

Commit Message: "feat: complete 3D spherical atmospheric rendering system with validation"
```

## 8. Implementation Priority and Dependencies

### 8.1 Critical Path Analysis

**High Priority (Immediate Implementation)**:
1. **Spherical Coordinate System** - Foundation for all 3D rendering
2. **Basic 3D Camera** - Essential for any 3D visualization  
3. **Simple Surface Mesh** - Proof of concept for spherical rendering
4. **Integration with Existing Simulation** - Connect to atmospheric data

**Medium Priority (Phase 2)**:
1. **Multi-Layer Atmospheric Rendering** - Full atmospheric visualization
2. **Performance Optimization** - Real-time requirements
3. **Advanced Camera Modes** - Enhanced user experience
4. **Wind Vector Visualization** - Critical for atmospheric analysis

**Lower Priority (Future Enhancement)**:
1. **Advanced UI Features** - Polish and usability
2. **Specialized Camera Modes** - Cross-section, follow-weather
3. **Advanced Performance Features** - Occlusion culling, streaming
4. **Export and Recording** - Data analysis features

### 8.2 Technical Risk Assessment

**High Risk Areas**:
- **Spherical mesh generation performance** - May need optimization for real-time use
- **Coordinate transformation accuracy** - Critical for atmospheric physics correctness
- **Multi-layer transparency rendering** - Complex shader requirements
- **Memory management** - Large domains may exceed memory limits

**Mitigation Strategies**:
- Start with simple implementations, optimize iteratively
- Extensive testing with reference coordinates (geographic landmarks)
- Use proven graphics libraries (macroquad) for rendering foundation
- Implement streaming/LOD systems early in development

### 8.3 Integration Points with Existing System

**Leverage Existing Architecture**:
- `WorldScale` system for domain size management
- `ScaleAware` trait for parameter adaptation
- Existing atmospheric data structures (`AtmosphericPressureLayer`, `WindLayer`)
- Current dimensional analysis system for physics validation

**Required Modifications**:
- Extend `GraphicsRenderer` to support 3D camera systems
- Add spherical coordinate support to atmospheric calculations
- Modify boundary conditions for spherical geometry
- Update UI system to handle 3D navigation

## Conclusion

This specification provides a complete, engineering-ready implementation plan for 3D spherical atmospheric rendering. The system addresses the fundamental atmospheric physics violations identified by the atmospheric-physicist while building on the strong existing architecture.

**Key Success Metrics**:
- **Physics Accuracy**: Eliminate boundary artifacts through proper spherical geometry
- **Performance**: Maintain >30 FPS for continental domains, >15 FPS for global domains  
- **Usability**: Intuitive 3D navigation comparable to modern 3D applications
- **Integration**: Seamless connection with existing atmospheric physics simulation

**Implementation Path**: The 7-sprint TDD breakdown provides clear milestones with testable acceptance criteria, enabling systematic development with quality gates at each phase.

**Technical Foundation**: Leverages existing scale-aware architecture, dimensional analysis, and atmospheric data structures while introducing proper spherical coordinate handling and 3D visualization capabilities.

This 3D spherical rendering system will transform the atmospheric simulation from a pattern generator into a scientifically accurate, visually compelling planetary atmospheric model suitable for both research and interactive applications.