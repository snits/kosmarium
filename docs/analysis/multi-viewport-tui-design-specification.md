# Multi-Viewport TUI Design Specification

## ABOUTME: Comprehensive UX design for multi-viewport atmospheric simulation monitoring interface
## ABOUTME: Terminal-based interface for simultaneous monitoring of multiple scientific data layers

## Executive Summary

This specification defines a multi-viewport TUI interface that extends the existing ASCII framebuffer system to support simultaneous monitoring of 2-4 atmospheric data layers. The design maintains the scientific workflow efficiency of the current system while adding intuitive viewport management and navigation capabilities.

**Key Design Principles:**
- Preserve existing scientific workflow patterns and presets
- Maintain terminal-based approach for broad compatibility
- Provide clear visual hierarchy and active viewport indication
- Support efficient WASD navigation within active viewport
- Enable independent zoom/scale control per viewport
- Educational-friendly for classroom demonstrations

## Current System Analysis

### Existing Capabilities
- **8 Scientific Data Layers**: elevation, water, temperature, pressure, wind, flow, biomes, sediment
- **Real-time Monitoring**: Continental-scale (240x120 @ 200km typical) with configurable update intervals
- **Multiple Zoom Levels**: continental, regional, local with scale-aware rendering
- **Scientific Workflow Presets**: climate-analysis, storm-tracking, change-detection, regional-deep-dive
- **Terminal Compatibility**: UTF-8 with crossterm/ratatui for cross-platform support

### Current Navigation (Single Viewport)
- WASD: Basic movement with single-cell steps
- WASD (Shift): Fast movement with multi-cell steps
- +/-: Zoom in/out with level constraints
- Space: Pause/resume simulation
- Number keys: Preset switching
- 'r': Toggle real-time mode

## Multi-Viewport Interface Design

### 1. Viewport Layout Patterns

#### Grid Layout (2x2 - Recommended Primary)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ┌─ ELEVATION ───────────────┐  ┌─ TEMPERATURE ─────────────┐                │
│ │ @@##^^.,~.        [CONT] │  │ ░░▒▒▓▓+#.      [REGI] *  │  Status Panel  │
│ │ ##^^.,~..                │  │ ▒▒▓▓+#..               │                │
│ │ ^^.,~....                │  │ ▓▓+#....               │  Viewport: 1/4  │
│ │ .,~......                │  │ +#......               │  Scale: CONT    │
│ │ ~........                │  │ #.......               │  Position:      │
│ │ .........                │  │ ........               │  (125, 67)      │
│ └──────────────────────────┘  └────────────────────────┘                │
│ ┌─ PRESSURE ────────────────┐  ┌─ WIND ─────────────────┐  Update: 5s     │
│ │ #+0.--.+# [REGI]         │  │ →↗↑←.↓↘   [LOCAL]     │  Tick: #4521    │
│ │ +0.--.+##                │  │ ↗↑←.↓↘→                │                │
│ │ 0.--.+###                │  │ ↑←.↓↘→↗                │  Presets:       │
│ │ --.+#####                │  │ ←.↓↘→↗↑                │  [F1] Climate   │
│ │ -.+######                │  │ .↓↘→↗↑←                │  [F2] Storm     │
│ │ .+#######                │  │ ↓↘→↗↑←.                │  [F3] Change    │
│ └──────────────────────────┘  └────────────────────────┘  [F4] Deep      │
└─────────────────────────────────────────────────────────────────────────────┘
[TAB] Next Viewport  [1-4] Select  [WASD] Navigate Active  [+/-] Zoom  [Q] Quit
```

#### Split-Screen Layout (2x1 - Alternative)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ┌─ ELEVATION ─────────────────────────────────┐ ┌─ TEMPERATURE ──────────────┐ │
│ │ @@##^^.,~...              [CONTINENTAL] *  │ │ ░░▒▒▓▓+#.    [REGIONAL]   │ │
│ │ ##^^.,~....                              │ │ ▒▒▓▓+#..                 │ │
│ │ ^^.,~......                              │ │ ▓▓+#....                 │ │
│ │ .,~........                              │ │ +#......                 │ │
│ │ ~..........                              │ │ #.......                 │ │
│ │ ...........                              │ │ ........                 │ │
│ │ ...........                              │ │ ........                 │ │
│ │ ...........                              │ │ ........                 │ │
│ │ ...........                              │ │ ........                 │ │
│ │ ...........                              │ │ ........                 │ │
│ │ ...........                              │ │ ........                 │ │
│ └─────────────────────────────────────────────┘ └─────────────────────────────┘ │
│ ┌─ PRESSURE ──────────────────────────────────┐ ┌─ Status & Controls ──────────┐ │
│ │ #+0.--.+#                 [REGIONAL]      │ │ Active: Elevation (1/4)      │ │
│ │ +0.--.+##                               │ │ Position: (125, 67)           │ │
│ │ 0.--.+###                               │ │ Scale: Continental            │ │
│ │ --.+#####                               │ │ Update: Every 5s              │ │
│ │ -.+######                               │ │ Simulation: Running #4521     │ │
│ │ .+#######                               │ │                              │ │
│ │ +########                               │ │ Navigation:                   │ │
│ │ #########                               │ │ [WASD] Move active viewport   │ │
│ │ #########                               │ │ [TAB] Cycle viewports         │ │
│ │ #########                               │ │ [1-4] Select viewport         │ │
│ │ #########                               │ │ [+/-] Zoom active viewport    │ │
│ └─────────────────────────────────────────────┘ └─────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Triple Layout (3-Panel - Educational Focus)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ┌─ TEMPERATURE ──────────────────┐ ┌─ PRESSURE ───────────────────────────────┐ │
│ │ ░░▒▒▓▓+#.        [REGI] *     │ │ #+0.--.+#              [REGIONAL]       │ │
│ │ ▒▒▓▓+#..                    │ │ +0.--.+##                             │ │
│ │ ▓▓+#....                    │ │ 0.--.+###                             │ │
│ │ +#......                    │ │ --.+#####                             │ │
│ │ #.......                    │ │ -.+######                             │ │
│ │ ........                    │ │ .+#######                             │ │
│ │ ........                    │ │ +########                             │ │
│ │ ........                    │ │ #########                             │ │
│ └────────────────────────────────┘ └───────────────────────────────────────────┘ │
│ ┌─ WIND ─────────────────────────────────────────────────────────────────────┐ │
│ │ →↗↑←.↓↘→                                    [REGIONAL]                 │ │
│ │ ↗↑←.↓↘→↗                                                               │ │
│ │ ↑←.↓↘→↗↑                                                               │ │
│ │ ←.↓↘→↗↑←                                                               │ │
│ │ .↓↘→↗↑←.                                                               │ │
│ │ ↓↘→↗↑←.↓                                                               │ │
│ │ ↘→↗↑←.↓↘                                                               │ │
│ │ →↗↑←.↓↘→                                                               │ │
│ └─────────────────────────────────────────────────────────────────────────────┘ │
│ Active: Temperature (1/3)  |  Controls: [TAB] Next  [WASD] Move  [+/-] Zoom    │ │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. Active Viewport Visual Indicators

#### Primary Indicator: Border Style and Asterisk
- **Active Viewport**: Double border with asterisk `*` in header
- **Inactive Viewports**: Single border without asterisk
- **Scale Indicator**: [CONT], [REGI], [LOCA] in header right side

#### Secondary Indicators: Color and Status
- **Active Border Color**: Bright white/yellow (if color terminal)
- **Inactive Border Color**: Dim gray
- **Status Panel**: Always shows active viewport details

### 3. Keybinding Specification

#### Core Navigation (Maintains Existing Patterns)
```
Movement (Active Viewport Only):
  w, ↑          Move north (1 cell)
  s, ↓          Move south (1 cell) 
  a, ←          Move west (1 cell)
  d, →          Move east (1 cell)
  W, Shift+↑    Fast north (5 cells)
  S, Shift+↓    Fast south (5 cells)
  A, Shift+←    Fast west (5 cells)
  D, Shift+→    Fast east (5 cells)

Zoom (Active Viewport Only):
  +, =          Zoom in (continental→regional→local)
  -             Zoom out (local→regional→continental)
```

#### Multi-Viewport Management
```
Viewport Selection:
  Tab           Cycle to next viewport (1→2→3→4→1)
  Shift+Tab     Cycle to previous viewport (4→3→2→1)
  1-4           Direct viewport selection
  `             Return to viewport 1 (primary)

Layout Control:
  L             Cycle layout mode (2x2 → 2x1 → 3-panel → 2x2)
  Shift+L       Toggle status panel visibility
```

#### Layer and Preset Management (Global)
```
Layer Control:
  F1-F8         Quick layer selection for active viewport
  Shift+F1-F8   Toggle layer in active viewport

Preset System (Compatible with Existing):
  Ctrl+1        Apply climate-analysis preset to all viewports
  Ctrl+2        Apply storm-tracking preset to all viewports  
  Ctrl+3        Apply change-detection preset to all viewports
  Ctrl+4        Apply regional-deep-dive preset to all viewports

Custom Presets:
  Ctrl+S        Save current multi-viewport configuration
  Ctrl+O        Load saved multi-viewport configuration
```

#### System Control
```
Simulation Control:
  Space         Pause/resume simulation (global)
  r             Toggle real-time mode (global)
  R             Reset all viewport positions to origin

Information and Help:
  i             Toggle information overlay
  h, ?          Show help overlay
  q, Esc        Quit interface
```

### 4. User Workflow Examples

#### Scientific Research Workflow
**Storm System Analysis:**
1. Load storm-tracking preset: `Ctrl+2`
2. Configure viewports:
   - Viewport 1: Pressure (continental scale)
   - Viewport 2: Wind (regional scale)
   - Viewport 3: Temperature (regional scale)
   - Viewport 4: Water (local scale focused on storm center)
3. Navigate to storm system using viewport 1: `1` `wasd`
4. Switch to detailed wind analysis: `Tab` or `2`
5. Zoom to storm core: `+++`
6. Monitor storm evolution while tracking across viewports

**Climate-Biome Relationship Study:**
1. Load climate-analysis preset: `Ctrl+1`
2. Configure comparative analysis:
   - Viewport 1: Temperature (continental)
   - Viewport 2: Biomes (same region, continental)
   - Viewport 3: Elevation (for context)
3. Navigate to interesting climate boundary: `wasd`
4. Switch all viewports to same position for comparison
5. Zoom in synchronously for detailed analysis

#### Educational Demonstration Workflow
**Basic Atmospheric Circulation:**
1. Start with 3-panel layout: `L` until 3-panel mode
2. Configure for teaching:
   - Top-left: Temperature
   - Top-right: Pressure  
   - Bottom: Wind (full width for clear arrows)
3. Navigate to clear thermal circulation example
4. Demonstrate cause-effect: "See temperature → pressure → wind"
5. Use different scales to show multi-scale effects

**Water Cycle Demonstration:**
1. Use 2x2 layout with:
   - Elevation (context)
   - Temperature (evaporation driver)
   - Water (surface water)
   - Flow (water movement)
2. Find area with interesting water cycle dynamics
3. Show interconnections between layers
4. Use pause/resume to control teaching pace

### 5. Implementation Complexity Assessment

#### Low Complexity (Existing System Extension)
- **Viewport Management**: Extend existing Viewport struct to support multiple instances
- **Layer Rendering**: Current ASCII framebuffer system already supports multiple layers
- **Basic Keybinding**: Crossterm event handling system in place

#### Medium Complexity (New Architecture Components)  
- **Layout System**: New component to manage viewport positioning and sizing
- **Active State Management**: Track active viewport and update visual indicators
- **Synchronized Navigation**: Coordinate movement across viewports with different scales

#### High Complexity (Advanced Features)
- **Configuration Save/Load**: YAML-based viewport configuration persistence
- **Dynamic Layout Switching**: Runtime layout reconfiguration without data loss
- **Advanced Preset System**: Multi-viewport preset definitions and management

#### Recommended Implementation Phases

**Phase 1: Core Multi-Viewport (2-3 weeks)**
- Basic 2x2 layout with fixed positioning
- Tab-based viewport switching
- Independent WASD navigation per viewport
- Active viewport visual indication

**Phase 2: Enhanced Navigation (1-2 weeks)**
- Multiple layout modes (2x2, 2x1, 3-panel)
- Direct viewport selection (1-4 keys)
- Zoom control per viewport
- Status panel with active viewport details

**Phase 3: Scientific Workflow Integration (1-2 weeks)**  
- Multi-viewport preset system
- Configuration save/load
- Enhanced layer management
- Educational-focused features

### 6. Rendering-Engineer Involvement Recommendation

**Recommended: Yes, but limited scope**

The rendering-engineer should be involved for:
- **Layout System Architecture**: Design clean separation between viewport management and rendering
- **Performance Optimization**: Ensure efficient rendering of multiple simultaneous frames
- **Integration Strategy**: Proper integration with existing ASCII framebuffer system

**Not Required for:**
- Basic keybinding and event handling (standard crossterm patterns)
- Visual design decisions (UX domain)
- Scientific workflow requirements (domain expert knowledge)

### 7. Configuration Integration

#### Extend Existing FramebufferConfig
```rust
pub struct MultiViewportConfig {
    /// Individual viewport configurations
    pub viewports: Vec<ViewportConfig>,
    /// Active layout mode
    pub layout_mode: LayoutMode,
    /// Show status panel
    pub show_status_panel: bool,
    /// Synchronized navigation mode
    pub sync_navigation: bool,
}

pub struct ViewportConfig {
    /// Layer configuration (reuse existing)
    pub framebuffer_config: FramebufferConfig,
    /// Viewport position and scale
    pub viewport: Viewport,
    /// Zoom level
    pub zoom_level: ZoomLevel,
}
```

#### YAML Configuration Format
```yaml
metadata:
  name: "Storm Analysis Multi-Viewport"
  author: "Dr. Smith"
  description: "4-viewport storm tracking configuration"

layout:
  mode: "grid_2x2"  # "grid_2x2", "split_2x1", "triple_panel"
  show_status_panel: true
  sync_navigation: false

viewports:
  - name: "Pressure Overview"
    layers: ["pressure"]
    position: [0, 0]
    zoom: "continental"
    panel_size: [40, 20]
  
  - name: "Wind Detail"  
    layers: ["wind"]
    position: [100, 50]
    zoom: "regional"
    panel_size: [40, 20]

  - name: "Temperature Context"
    layers: ["temperature"] 
    position: [0, 0]
    zoom: "continental"
    panel_size: [40, 20]

  - name: "Storm Core"
    layers: ["pressure", "wind"]
    position: [120, 60]
    zoom: "local" 
    panel_size: [40, 20]
```

## Conclusion

This multi-viewport TUI design preserves the scientific rigor and efficiency of the existing system while adding powerful new capabilities for simultaneous multi-layer monitoring. The design prioritizes:

1. **Intuitive Navigation**: Familiar WASD patterns with clear active viewport indication
2. **Scientific Workflow Compatibility**: Existing presets and layer systems integrate seamlessly  
3. **Educational Accessibility**: Clear visual hierarchy and configurable layouts for teaching
4. **Implementation Feasibility**: Incremental enhancement of existing architecture

The recommended grid 2x2 layout provides the optimal balance of information density and usability for most scientific applications, while the alternative layouts serve specialized use cases in education and research.

**Next Steps:**
1. Review design with domain experts for scientific workflow validation
2. Create technical architecture specification with rendering-engineer
3. Implement Phase 1 core functionality with iterative user testing
4. Extend to full multi-viewport ecosystem based on user feedback

This design maintains the terminal-based approach that makes the system broadly accessible while dramatically enhancing its capability for complex atmospheric system analysis and educational demonstration.