# Development Standards

ABOUTME: Project-specific workflow requirements and quality gates for the planetary simulation
ABOUTME: Updated after atmospheric physics redesign - captures proven development practices

## Quality Gate Requirements

### Pre-Commit Checklist (MANDATORY)
Every commit MUST complete this checklist:

```bash
# 1. Code Quality
cargo fmt                # Code formatting
cargo clippy            # Linting and best practices  
cargo check            # Type checking and compilation

# 2. Testing  
cargo test              # Unit and integration tests
# Run project-specific test suite if available

# 3. Physics Validation (for physics systems)
cargo run --bin debug_atmospheric_validation  # System-specific diagnostics
# Verify conservation laws and physical realism

# 4. Code Review
# Request code-reviewer agent approval for ALL code changes
# NEVER proceed without explicit approval

# 5. Documentation
# Update relevant docs/ files for significant changes
# Ensure ABOUTME headers present in all new files
```

### Code-Reviewer Approval Protocol (MANDATORY)
- **ALL code changes require code-reviewer approval BEFORE committing**
- **Maximum increment**: Single logical change (15-30 minutes of work)
- **STOP after each logical increment** and request review
- **NEVER proceed without explicit code-reviewer approval**

## Testing Standards

### Physics System Testing Requirements
Based on atmospheric physics validation success:

#### 1. Mathematical Validation Testing
```rust
#[test]
fn test_conservation_laws() {
    // Validate mass/momentum/energy conservation
    // Use quantitative metrics (e.g., boundary flux < 1e5 kg/s)
    assert!(boundary_flux.abs() < 1e5, "Mass conservation violation");
}

#[test]  
fn test_physical_realism() {
    // Validate realistic parameter ranges
    // Wind speeds, pressure gradients, etc. within physical bounds
    assert!(wind_speed < MAX_REALISTIC_WIND, "Unphysical wind speed");
}
```

#### 2. Scale-Aware Testing
```rust
#[test]
fn test_scale_independence() {
    // Same physics quality across scale range (1km - 40,000km)
    for domain_size in [1.0, 100.0, 1000.0, 40000.0] {
        let scale = WorldScale::new(domain_size, (50, 50), DetailLevel::Standard);
        let system = PhysicsSystem::new_for_scale(&scale);
        
        validate_physics_quality(&system, &scale);
    }
}
```

#### 3. Diagnostic Framework Integration
```rust
#[test]
fn test_diagnostic_validation() {
    // Use comprehensive diagnostic frameworks
    let validation = validate_geostrophic_balance(&system, &pressure, &wind);
    
    assert!(validation.is_geostrophic_balanced);
    assert!(validation.pressure_wind_correlation > 0.8);
    assert!(validation.average_balance_residual < 5.0);
}
```

### Test Coverage Requirements
- **Unit tests**: All public functions and critical internal functions
- **Integration tests**: System interactions and physics coupling
- **Validation tests**: Conservation laws and physical realism
- **Scale tests**: Behavior across full domain size range (1km-40,000km)

## Documentation Standards

### Required Documentation Files
All significant changes must update relevant documentation:

#### 1. ABOUTME Headers (MANDATORY)
```rust
// ABOUTME: Brief description of file purpose in 1 line
// ABOUTME: Key technical details or architectural role in 1 line
```

#### 2. Physics System Documentation
For new physics systems, create comprehensive documentation:
- `docs/education/deep-dive-[system]-physics.md` - Mathematical foundations and implementation
- `docs/analysis/[system]-validation-results.md` - Validation metrics and improvements  
- `docs/specifications/phase-[n]-[system]-implementation.md` - Implementation approach and results

#### 3. Session Continuity Documentation
Keep updated for session handoffs:
- `docs/project-status.md` - Current status and next steps
- `docs/roadmap.md` - Milestone progress and completion metrics
- `docs/architecture/decisions.md` - Key design choices and rationale

## Development Workflow

### Mathematical Validation First (New Standard)
For complex physics systems:

#### 1. SageMath Analysis Phase
```bash
# Create mathematical validation file
touch atmospheric_physics_validation.sage

# Develop mathematical model
sage atmospheric_physics_validation.sage

# Derive safety parameters and realistic bounds
# Document conservation laws and stability requirements
```

#### 2. Implementation Phase  
```rust
// Use safety parameters from mathematical validation
const F_THRESHOLD: f64 = 1e-6; // From SageMath analysis

// Implement with diagnostic validation
pub fn implement_physics_system() {
    // Implementation with real-time validation
    validate_conservation_laws();
    validate_physical_realism();
}
```

#### 3. Validation Phase
```bash
# Run comprehensive diagnostic framework
cargo run --bin validate_[system]_physics

# Verify quantitative improvements
# Compare against baseline metrics
# Document results in validation files
```

### TDD Workflow for Physics Systems
Based on atmospheric physics success:

1. **Write failing test** that validates desired physics behavior
2. **Run test** to confirm failure with specific metrics
3. **Implement** minimum code to pass physics validation
4. **Request code-reviewer approval**
5. **Refactor** if needed while maintaining physics quality
6. **Document** patterns and lessons learned

### Commit Discipline
- **Atomic commits**: Single logical change with clear scope
- **Descriptive messages**: Explain the physics or functional change
- **Attribution**: Include `Co-Authored-By: Claude <noreply@anthropic.com>`
- **Physics metrics**: Include quantitative improvements when applicable

Example commit message:
```
feat: implement geostrophic wind calculation with F_THRESHOLD safety

- Apply proper geostrophic balance v = -(1/ρf) × ∇P equation
- Use F_THRESHOLD = 1e-6 s⁻¹ for numerical stability  
- Achieve 63% wind speed reduction (50 → 18.6 m/s average)
- Perfect pressure-wind coupling (0.990 correlation)

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

## Performance Standards

### Physics System Performance
- **Diagnostic overhead**: <10% impact on simulation performance
- **Scale performance**: No degradation when scaling domain size 10x
- **Memory efficiency**: Use PhysicsGrid for 2-3x improvement on vector operations
- **SIMD ready**: Structure data for future SIMD optimization

### Testing Performance  
- **Unit tests**: <1 second per test
- **Integration tests**: <10 seconds per comprehensive physics validation
- **Scale tests**: <30 seconds for full scale range validation
- **CI/CD ready**: All tests complete in <5 minutes total

## Security and Safety Standards

### Numerical Safety
- **Division by zero protection**: Use safety thresholds (e.g., F_THRESHOLD)
- **Overflow prevention**: Realistic bounds on all physical quantities
- **Stability validation**: Verify numerical stability across parameter ranges

### Memory Safety
- **Bounds checking**: All array accesses validated
- **No unsafe code**: Use safe Rust patterns exclusively
- **Resource management**: Proper RAII and ownership patterns

## Code Style Standards

### Rust-Specific Guidelines
```rust
// Prefer descriptive names over clever abbreviations
let geostrophic_balance_residual = ...; // Good
let gbr = ...;                         // Bad

// Use meaningful error types
pub enum PhysicsError {
    ConservationViolation { system: String, magnitude: f64 },
    NumericalInstability { parameter: String, value: f64 },
}

// Document physics equations in comments
// Geostrophic balance: f × v = -(1/ρ)∇P
// Therefore: u = ∇P_y/(ρf), v = -∇P_x/(ρf)
let geostrophic_u = pressure_gradient.y / (rho * f);
```

### Physics Implementation Patterns
- **Conservation laws first**: Implement conservation before optimization
- **Diagnostic validation**: Include validation calls in implementation
- **Scale awareness**: Use WorldScale parameter, never hardcode thresholds
- **Physical realism**: Validate against real atmospheric/oceanic data ranges

## Experimental Development Standards

### Experiment-Friendly Practices

#### It's OK To:
- Leave TODO comments for future exploration
- Commit code that's not fully polished (with proper review)
- Try multiple approaches to the same problem
- Have unused code while experimenting
- Use `dbg!()` and `println!()` for debugging (just clean up eventually)

#### Context Window Management
- **Large debug outputs**: Use logging or file output instead of console spam
- **Analysis scripts**: Write detailed data to files, summaries to stdout
- **Agent debugging**: Preserve context tokens for reasoning, not raw data dumps

#### Must Avoid:
- Breaking the build (others can't experiment if it doesn't compile)
- Committing without code-reviewer approval
- Large commits mixing multiple experiments
- Deleting experiments without documenting what was learned

## Success Metrics

### Development Quality Indicators
- **Physics correctness**: Conservation laws validated quantitatively  
- **Code review quality**: A+ ratings from code-reviewer agent
- **Mathematical foundation**: SageMath validation prevents implementation bugs
- **Scale robustness**: Same quality across 4 orders of magnitude in domain size
- **Performance**: Optimized data structures provide measurable improvements

### Project Health Metrics
- **Commit discipline**: Clean, atomic commits with proper attribution
- **Documentation coverage**: All major systems have deep-dive documentation
- **Test coverage**: Comprehensive physics validation frameworks
- **Architecture clarity**: Clear separation of concerns, extensible design

---

**Standards Status**: Validated through atmospheric physics redesign success  
**Quality Achievement**: 99.6% momentum improvement, A+ code quality rating  
**Updated**: August 11, 2025 - Reflecting proven development practices and experimental flexibility