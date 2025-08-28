// ABOUTME: Scientific workspace configuration system for YAML-based team collaboration
// ABOUTME: Enables persistent analysis workflows, shareable templates, and version-controlled research setups

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::core::temporal_scaling::TemporalScalingConfig;

/// Complete scientific workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Configuration metadata
    pub metadata: WorkspaceMetadata,
    /// Global simulation defaults
    pub defaults: SimulationDefaults,
    /// ASCII framebuffer layout configuration
    pub layout: FramebufferLayout,
}

/// Workspace metadata for collaboration and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    /// Human-readable workspace name
    pub name: String,
    /// Author/creator of the configuration
    pub author: String,
    /// Research team or organization
    pub team: Option<String>,
    /// Creation timestamp (ISO 8601)
    pub created: String,
    /// Last modified timestamp (ISO 8601)
    pub modified: Option<String>,
    /// Description of the analysis purpose
    pub description: Option<String>,
    /// Version for configuration evolution
    pub version: String,
}

/// Global simulation configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationDefaults {
    /// Random seed for reproducible simulations
    pub seed: Option<u64>,
    /// Physical scale in kilometers
    pub scale_km: f64,
    /// Terrain roughness parameter
    pub roughness: f32,
    /// Detail persistence across scales
    pub persistence: f32,
    /// Map dimensions (width, height)
    pub dimensions: (usize, usize),
    /// Update interval in simulation ticks
    pub interval: usize,
    /// Temporal scaling configuration for realistic vs demo modes
    pub temporal_scaling: TemporalScalingConfig,
}

/// ASCII framebuffer layout and visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FramebufferLayout {
    /// Frame buffer size for temporal analysis
    pub buffer_size: usize,
    /// Visualization layers to display
    pub layers: Vec<String>,
    /// Scale zoom level (continental, regional, local)
    pub zoom: String,
    /// Frame dimensions (width, height) - 0 means auto-size
    pub frame_size: (usize, usize),
    /// Show timestamps in output
    pub show_timestamps: bool,
    /// Highlight changes between frames
    pub highlight_changes: bool,
    /// Subsample rate for large maps
    pub subsample_rate: usize,
    /// Custom layer-specific settings
    pub layer_settings: Option<HashMap<String, LayerSettings>>,
}

/// Per-layer visualization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerSettings {
    /// Layer-specific zoom override
    pub zoom_override: Option<String>,
    /// Custom color scheme
    pub color_scheme: Option<String>,
    /// Value range for normalization
    pub value_range: Option<(f64, f64)>,
    /// Display symbols override
    pub symbols: Option<Vec<char>>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            metadata: WorkspaceMetadata {
                name: "Default Scientific Workspace".to_string(),
                author: "Unknown".to_string(),
                team: None,
                created: chrono::Utc::now().to_rfc3339(),
                modified: None,
                description: Some("Default configuration for scientific analysis".to_string()),
                version: "1.0".to_string(),
            },
            defaults: SimulationDefaults {
                seed: None,
                scale_km: 200.0,
                roughness: 0.7,
                persistence: 0.6,
                dimensions: (240, 120),
                interval: 10,
                temporal_scaling: TemporalScalingConfig::default(),
            },
            layout: FramebufferLayout {
                buffer_size: 5,
                layers: vec![
                    "elevation".to_string(),
                    "water".to_string(),
                    "biomes".to_string(),
                ],
                zoom: "continental".to_string(),
                frame_size: (0, 0), // Auto-size
                show_timestamps: true,
                highlight_changes: false,
                subsample_rate: 1,
                layer_settings: None,
            },
        }
    }
}

impl WorkspaceConfig {
    /// Create a workspace config from a workflow preset
    pub fn from_preset(preset_name: &str, author: &str) -> Self {
        let mut config = Self::default();
        config.metadata.author = author.to_string();
        config.metadata.name = format!(
            "{} Workspace",
            preset_name.replace('-', " ").to_title_case()
        );

        match preset_name {
            "climate-analysis" => {
                config.layout.layers = vec![
                    "temperature".to_string(),
                    "biomes".to_string(),
                    "elevation".to_string(),
                ];
                config.layout.zoom = "continental".to_string();
                config.metadata.description = Some(
                    "Climate scientists: temperature-biome relationships across scales".to_string(),
                );
            }
            "storm-tracking" => {
                config.layout.layers = vec![
                    "pressure".to_string(),
                    "wind".to_string(),
                    "temperature".to_string(),
                ];
                config.layout.zoom = "regional".to_string();
                config.metadata.description = Some(
                    "Atmospheric physicists: pressure systems and circulation patterns".to_string(),
                );
            }
            "change-detection" => {
                config.layout.layers = vec![
                    "pressure".to_string(),
                    "temperature".to_string(),
                    "water".to_string(),
                    "changes".to_string(),
                ];
                config.layout.zoom = "continental".to_string();
                config.layout.buffer_size = 10;
                config.metadata.description =
                    Some("Research teams: temporal analysis and system evolution".to_string());
            }
            "regional-deep-dive" => {
                config.layout.layers = vec![
                    "elevation".to_string(),
                    "water".to_string(),
                    "temperature".to_string(),
                    "pressure".to_string(),
                    "biomes".to_string(),
                    "wind".to_string(),
                ];
                config.layout.zoom = "local".to_string();
                config.metadata.description =
                    Some("Detailed regional analysis: all layers at high resolution".to_string());
            }
            _ => {
                config.metadata.description =
                    Some("Custom scientific workspace configuration".to_string());
            }
        }

        config
    }

    /// Load workspace configuration from YAML file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: WorkspaceConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Save workspace configuration to YAML file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    /// Update the modified timestamp
    pub fn mark_modified(&mut self) {
        self.metadata.modified = Some(chrono::Utc::now().to_rfc3339());
    }
}

// Helper trait for string formatting
trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
