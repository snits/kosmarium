# Testing Standards for Sim Prototype

## Testing Philosophy

All code in this project follows comprehensive testing practices with three levels of testing:
- **Unit Tests**: Test individual functions and methods in isolation
- **Integration Tests**: Test component interactions and system behavior  
- **End-to-End Tests**: Test complete user workflows and system integration

## Current Test Infrastructure

### Test Organization

Tests are properly integrated with Rust's testing framework using standard conventions:

- **Unit Tests**: Located in `#[cfg(test)]` modules within source files
- **Integration Tests**: Would be in `tests/` directory (none currently needed)
- **Binary Tests**: Each binary in `src/bin/` can have its own test module

### Test Execution

All tests run via `cargo test` and are automatically discovered:

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test worldgen::tests

# Run tests with output
cargo test -- --nocapture
```

### Current Test Coverage

**Total Tests**: 142 comprehensive unit tests covering:

- **Agents** (7 tests): Pathfinding, performance, spatial queries, lifecycle management
- **Atmosphere** (12 tests): Wind generation, atmospheric parameters, climate systems
- **Atmospheric Moisture** (7 tests): Evaporation cycles, transport, precipitation
- **Biome** (8 tests): Classification, distribution, environmental generation
- **Cache System** (4 tests): LRU eviction, expiration, terrain change detection
- **Climate** (11 tests): Temperature/pressure generation, seasonal variation
- **Convergence** (7 tests): Detection algorithms, metrics, order estimation
- **Dimensional** (9 tests): Unit conversions, CFL validation, climate analysis
- **Drainage** (10 tests): Flow direction/accumulation, water conservation
- **Geological Evolution** (2 tests): Basic terrain evolution functionality
- **Heightmap** (6 tests): Creation, indexing, format conversions
- **Optimized Systems** (7 tests): Performance optimizations, caching strategies
- **Scale** (2 tests): World scale calculations and parameters
- **Simulation** (34 tests): Water systems, mass conservation, scaling integration
- **Spatial Partitioning** (5 tests): Performance tracking, convergence, water flow
- **WorldGen** (5 tests): Terrain generation, NaN safety, edge cases

## Testing Standards

### Test Naming Conventions

- Test function names should describe the scenario being tested: `test_water_conservation_with_no_flow`
- Test modules should be named `tests` and use `#[cfg(test)]`
- Use descriptive assertions with clear error messages

### Test Quality Requirements

1. **Independence**: Each test must be completely independent and repeatable
2. **Real Functionality**: Tests must exercise actual code paths, never mock the code being tested
3. **Clear Purpose**: Each test validates specific business logic or system behavior
4. **Comprehensive Coverage**: All public APIs and critical edge cases must be tested
5. **Performance Validation**: Critical algorithms include performance characteristic tests

### Test Categories

#### Unit Tests (Required)
- Test individual functions with various inputs
- Validate error conditions and edge cases
- Ensure mathematical correctness
- Verify defensive programming (NaN/infinity handling)

#### Integration Tests (Required) 
- Test component interactions
- Validate system state transitions
- Test cross-module functionality
- Ensure proper scaling behavior

#### Performance Tests (When Applicable)
- Algorithm complexity validation
- Scaling behavior verification
- Resource usage tracking
- Regression detection

### Safety and Robustness Testing

Given this is a simulation engine with mathematical operations:

1. **NaN/Infinity Safety**: All mathematical operations must be tested for edge cases
2. **Bounds Checking**: Array access and mathematical ranges must be validated
3. **Scale Invariance**: Systems must work correctly across different map sizes
4. **Conservation Laws**: Physical simulations must preserve mass/energy where applicable

### Anti-Patterns to Avoid

- **Never** test mocked behavior instead of real functionality
- **Never** ignore test output or system logs
- **Never** write tests that only validate implementation details
- **Never** create tests that pass regardless of code correctness

## Performance and Scattered File Policy

### Performance Analysis Scripts

Files like `performance_test.rs` and `geological_performance_test.rs` are **analysis tools**, not tests:
- Purpose: Algorithm complexity analysis and optimization guidance
- Location: Root directory or `src/bin/` for executable analysis
- Execution: Run directly as binaries, not through `cargo test`

### Debug Scripts

Files like `test_water_bug.rs` are **debugging utilities**, not tests:
- Purpose: Reproduce and analyze specific issues
- Location: Root directory for temporary investigation
- Lifecycle: Should be removed after issue resolution or converted to proper tests

### Integration with Cargo Test

All actual tests MUST be discoverable via `cargo test`. Any file with real test functionality should:
1. Use `#[test]` attribute on test functions
2. Be located in proper test module (`#[cfg(test)]`)
3. Follow Rust testing conventions

## Quality Gates

Before any commit:
1. All tests must pass: `cargo test`
2. No warnings in test code: `cargo test 2>&1 | grep -i warning`
3. Tests must validate actual functionality, not mocked behavior
4. New functionality requires corresponding test coverage

## Continuous Testing

- Tests run automatically in CI/CD (when implemented)
- All pull requests must include tests for new functionality
- Test failures block merges
- Performance regression detection for critical algorithms

This testing infrastructure ensures code reliability and gives complete confidence in system behavior changes.