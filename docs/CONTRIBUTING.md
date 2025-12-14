# Contributing to Director Engine

Thank you for your interest in contributing to Director Engine!

## Development Setup

### Prerequisites
- Rust (stable toolchain)
- FFmpeg development libraries
- LLVM/Clang (for Skia bindings)

See [BUILD_GUIDE.md](./BUILD_GUIDE.md) for detailed platform-specific setup.

### Building
```bash
# Build the CLI
cargo build -p director-cli

# Build in release mode
cargo build --release -p director-cli
```

### Running Tests
```bash
# Run all tests
cargo test --release

# Run specific crate tests
cargo test --release -p director-core
```

---

## Visual Regression Testing

We use golden master snapshot testing to catch visual regressions.

### Running Visual Tests
```bash
cargo test --release -p director-core visual
```

### Updating Snapshots
When you intentionally change rendering behavior:
```bash
UPDATE_SNAPSHOTS=1 cargo test --release -p director-core visual
```

### Failure Artifacts
Failed tests generate diff images in:
```
crates/director-core/target/visual_regression_failures/
```

---

## Code Style

### Formatting
All code must pass `rustfmt`:
```bash
cargo fmt --all
cargo fmt --all -- --check  # Verify only
```

### Linting
All code must pass `clippy` with no warnings:
```bash
cargo clippy --all-targets -- -D warnings
```

### Logging
Use the `tracing` ecosystem for all logging:
```rust
// Good
tracing::info!(path = %file_path, "Loading asset");
tracing::debug!(elapsed_ms = elapsed.as_millis(), "Frame rendered");

// Bad
println!("Loading asset: {}", file_path);
eprintln!("Error: {}", e);
```

See [AGENTS.md](../AGENTS.md) for detailed guidelines.

---

## Pull Request Process

1. **Create an issue first** (use templates in `.github/ISSUE_TEMPLATE/`)
2. **Fork and branch** from `main`
3. **Write tests** for new functionality
4. **Update documentation** if changing public APIs
5. **Run CI locally** before pushing:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test --release
   ```
6. **Open PR** with clear description linking to issue

### PR Title Format
- `feat: Add new feature`
- `fix: Resolve bug in X`
- `docs: Update API documentation`
- `refactor: Improve X structure`
- `test: Add tests for Y`
- `chore: Update dependencies`

---

## Issue Guidelines

### Bug Reports
- Provide minimal reproduction script (Rhai)
- Include error output and environment info
- Check for existing issues first

### Feature Requests
- Explain the use case and motivation
- Consider backward compatibility
- Reference relevant roadmap milestones

---

## Architecture Overview

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the frame execution loop and system design.

Key concepts:
- **Director**: Manages timeline and scenes
- **SceneGraph**: Flat arena storage of nodes
- **Element trait**: Visual node interface
- **Taffy**: Layout computation
- **Skia**: Rasterization

---

## Questions?

- Check existing [documentation](./README.md)
- Open a [Discussion](https://github.com/YOUR_ORG/rust-skia-engine/discussions) (when enabled)
- File an issue with the `question` label
