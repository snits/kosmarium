# Kosmarium

> Planetary physics simulation platform with real-time temporal scaling and educational visualization

Kosmarium generates realistic terrain using advanced procedural algorithms and provides interactive visualization of planetary systems. Built for educational exploration of simulation concepts and environmental physics.

## Features

- **Diamond-Square terrain generation** - Creates realistic heightmaps with configurable parameters
- **Multiple visualization modes** - ASCII, interactive TUI, multi-viewport, and graphics interfaces
- **Advanced temporal scaling** - Demo, realistic, and research modes with custom scaling factors
- **Comprehensive physics systems** - Atmospheric, hydrological, geological, and ecological modeling
- **Scientific workflow support** - Built-in presets, YAML configuration, and data layer analysis
- **Interactive exploration** - Navigate generated worlds with keyboard controls (WASD/arrows)
- **Educational focus** - Detailed explanations of algorithms and mathematical foundations

## Quick Start

```bash
# Interactive TUI mode (default) - navigate with WASD/arrow keys
cargo run

# Multi-viewport TUI - simultaneous monitoring of multiple data layers
cargo run -- --multi-viewport

# ASCII mode - static terrain visualization
cargo run -- --ascii

# Animated ASCII frames - time-lapse visualization with multiple layers
cargo run -- --ascii-frames --layers elevation,water,temperature,pressure

# Graphics mode - 2D macroquad rendering
cargo run -- --graphics

# Weather simulation demo
cargo run --bin weather-demo

# Scientific workflow presets
cargo run -- --ascii-frames --preset climate-analysis
cargo run -- --ascii-frames --preset storm-tracking

# Temporal scaling examples
cargo run -- --temporal-mode realistic --temporal-stats
cargo run -- --study-phenomenon ecosystem --scaling-factor 0.1
cargo run -- --scale-atmospheric --scale-biological
```

## Visualization Modes

**TUI Mode (Default)**: Interactive terminal interface with real-time navigation
- Use WASD or arrow keys to pan around the generated world
- Press Q or Esc to quit
- Requires terminal support

**Multi-Viewport TUI**: Advanced interface for simultaneous data layer monitoring
- 2-4 viewport layout for comparing different data layers (temperature, pressure, wind, water)
- Scientific workflow optimized for atmospheric research
- WASD navigation with viewport switching capabilities
- Use `--multi-viewport` flag

**ASCII Mode**: Static colored ASCII output
- Best for headless environments or scripting
- Single-frame terrain visualization 
- Use `--ascii` flag

**ASCII Frames Mode**: Animated ASCII time-lapse
- Real-time simulation updates with ASCII visualization
- Multiple data layers: `elevation`, `water`, `temperature`, `pressure`, `wind`, `flow`, `biomes`, `sediment`
- Scientific workflow presets: `climate-analysis`, `storm-tracking`, `change-detection`
- Configurable frame buffer size, zoom levels (`continental`, `regional`, `local`)
- Time-stepped animation for observing system evolution
- Use `--ascii-frames` flag with `--layers` and `--preset` options

**Graphics Mode**: 2D hardware-accelerated rendering via macroquad
- Enhanced 2D visualization with smooth graphics
- Real-time rendering with improved visual clarity
- Use `--graphics` flag

## Temporal Scaling

Kosmarium supports advanced temporal scaling for scientific research:

**Temporal Modes:**
- `--temporal-mode demo` - Fast changes for observation (default)
- `--temporal-mode realistic` - Scientific rates (2.5 kg/m²/year ecological accuracy)  
- `--temporal-mode research` - Custom scaling factors for hypothesis testing

**Research Focus:**
- `--study-phenomenon ecosystem` - Auto-configure for ecological studies
- `--study-phenomenon climate` - Auto-configure for climate research

**Custom Scaling:**
- `--scaling-factor 0.5` - Slow down processes (0.001 to 1000.0 range)
- `--scale-biological` - Scale biological processes (ecosystem growth, vegetation)
- `--scale-geological` - Scale geological processes (erosion, sediment transport)
- `--scale-atmospheric` - Scale atmospheric processes (precipitation, evaporation)

**Analysis Tools:**
- `--temporal-stats` - Show temporal scaling performance statistics
- `--temporal-help` - Educational help about temporal scaling concepts
- `--temporal-config config.yaml` - Load temporal configuration from file

## Example Output

The terrain renderer displays elevation as colored ASCII characters:
- `.` (blue) - Water/ocean (elevation < 0.2)
- `~` (cyan) - Coastline (0.2-0.4) 
- `-` (green) - Plains (0.4-0.6)
- `^` (yellow) - Hills (0.6-0.8)
- `@` (red) - Mountains (> 0.8)

## Development Commands

```bash
cargo check    # Quick syntax checking
cargo build    # Build the project
cargo test     # Run tests  
cargo clippy   # Linting and code quality
cargo fmt      # Format code
```

## Architecture

Kosmarium follows a modular architecture with trait-based extensibility:

- **Terrain Generation** - Pluggable algorithms via `TerrainGenerator` trait
- **Physics Systems** - Atmospheric pressure, water flow, climate modeling
- **Visualization** - Multiple rendering backends (ASCII, TUI, 2D graphics via macroquad)
- **Temporal Scaling** - Real-time to geological timescale simulation

## Contributing

Development setup and detailed architecture documentation:
- **Development Guide**: See [CLAUDE.md](CLAUDE.md) for development workflow
- **Architecture**: [docs/01-architecture/](docs/01-architecture/) for design decisions
- **Project Status**: [docs/00-project/](docs/00-project/) for current implementation state

## Documentation

- [CLAUDE.md](CLAUDE.md) - Development setup and architecture overview
- [ASCII Framebuffer User Guide](docs/04-analysis/educational/ascii-framebuffer-user-guide.md) - Comprehensive guide for `--ascii-frames` visualization
- [docs/](docs/) - Deep-dive documentation on algorithms and systems
- [Cargo.toml](Cargo.toml) - Build configuration and dependencies

## License

MIT License - see [LICENSE](LICENSE) file for details.