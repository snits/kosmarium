# Development Standards

ABOUTME: Essential quality gates and workflow requirements for agentic development
ABOUTME: Minimal standards to keep code working and support productive experimentation

## Quality Gates (Must Pass Before Commit)

### Basic Requirements
```bash
cargo check         # Must compile without errors
cargo test          # All tests must pass (when tests exist)
cargo fmt           # Code must be formatted
```

### Optional But Recommended
```bash
cargo clippy        # Fix obvious issues, but warnings OK for experiments
```

## Agentic Workflow Requirements

### Code Review Process
- **code-reviewer approval required** before any commit
- **Atomic commits**: One logical change per commit  
- **Build requirement**: Each commit must compile successfully
- **Co-authorship**: Include `Co-developed-by: Claude claude-sonnet-4` trailer

### Commit Message Format
```
module: brief description (50 chars max)

Optional longer explanation if needed.

Co-developed-by: Claude claude-sonnet-4
```

## Testing Approach

### Test When It Makes Sense
- **Unit tests**: Add for complex logic that's likely to break
- **Integration tests**: When experimenting with end-to-end flows
- **No rigid coverage requirements**: Focus on testing what matters for experiments

### Test Organization
- Tests in same file as code using `#[cfg(test)]`
- Use descriptive test names that explain what's being tested
- Prefer real data over mocks for experiments

## Code Organization

### Module Structure
- Keep the established worldgen/sim/render separation
- Add new modules as experiments grow
- Don't over-architect - simple is better for experiments

### Dependencies
- Add dependencies freely for experimentation
- Document interesting ones in architecture-decisions.md
- Use latest stable versions unless there's a specific reason not to

## Development Environment

### Required Tools
```bash
cargo --version     # Any recent stable version
rustc --version     # Rust 2024 edition support
```

### Useful for Debugging
```bash
cargo run           # Quick iteration
cargo run --release # Performance testing
cargo flamegraph    # When you need to profile (install with: cargo install flamegraph)
```

## Session Management

### Update Documentation After Sessions
- **session-handoff.md**: What you were working on, what's broken, what to try next
- **experiment-roadmap.md**: New ideas discovered, experiments completed
- **architecture-decisions.md**: When you try something interesting and want to remember why

### Context for Future Sessions
- Commit frequently during experiments (even incomplete work)  
- Document interesting failures and dead ends
- Note performance characteristics of different approaches

## Experiment-Friendly Practices

### It's OK To:
- Leave TODO comments for future exploration
- Commit code that's not fully polished
- Try multiple approaches to the same problem
- Have unused code while experimenting
- Use `dbg!()` and `println!()` for debugging (just clean up eventually)

### Avoid:
- Breaking the build (others can't experiment if it doesn't compile)
- Committing without code-reviewer approval
- Large commits mixing multiple experiments
- Deleting experiments without documenting what was learned

*Remember: This is for learning and experimentation - prioritize being able to iterate quickly over perfect code quality*