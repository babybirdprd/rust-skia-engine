# Development Guide

This guide explains how to set up the development environment and contribute to Director Engine.

## Prerequisites

| Tool | Purpose |
|------|---------|
| Rust (stable) | Compiler |
| LLVM/Clang | Skia bindings (via bindgen) |
| FFmpeg | Video encoding/decoding |

## Platform Setup

### Linux (Ubuntu/Debian)
```bash
# Skia requirements
sudo apt install clang libclang-dev llvm-dev

# FFmpeg requirements
sudo apt install libavutil-dev libavformat-dev libavcodec-dev libswscale-dev libavfilter-dev libavdevice-dev

# Audio requirements
sudo apt install libasound2-dev
```

### macOS
```bash
xcode-select --install
brew install ffmpeg
```

### Windows
1. Install **LLVM** from [llvm.org](https://releases.llvm.org/) or `winget install LLVM`
2. Download FFmpeg from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) (Shared build)
3. Set `FFMPEG_DIR` environment variable to the extracted folder
4. Add `<ffmpeg>/bin` to your system `PATH`

---

## Building

```bash
# Build CLI
cargo build -p director-cli

# Build in release mode
cargo build --release
```

### Feature Flags

| Flag | Purpose |
|------|---------|
| `mock_video` | Build without FFmpeg (for CI or docs.rs) |
| `vulkan` | Enable Vulkan backend for Skia |

```bash
cargo build --no-default-features --features mock_video
```

---

## Testing

```bash
# All tests
cargo test --release

# Specific crate
cargo test --release -p director-core

# Visual regression tests
cargo test --release -p director-core visual

# Update snapshots
$env:UPDATE_SNAPSHOTS="1"; cargo test --release -p director-core visual
```

Failed visual tests output diffs to:
```
crates/director-core/target/visual_regression_failures/
```

---

## Code Style

```bash
# Format
cargo fmt --all

# Lint
cargo clippy --all-targets -- -D warnings
```

### Logging

Use the `tracing` ecosystem:
```rust
tracing::info!(path = %file_path, "Loading asset");
tracing::debug!(elapsed_ms, "Frame rendered");
```

---

## Pull Request Process

1. Create an issue first
2. Fork and branch from `main`
3. Write tests for new functionality
4. Update documentation if changing public APIs
5. Run CI locally:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test --release
   ```
6. Open PR with clear description linking to issue

### PR Title Format
- `feat: Add new feature`
- `fix: Resolve bug in X`
- `docs: Update documentation`
- `refactor: Improve X structure`
- `test: Add tests for Y`

---

## Troubleshooting

**"binding.rs not found" / Clang errors**
- Ensure `LIBCLANG_PATH` is set correctly
- On Windows, use MSVC toolchain (not GNU)

**"cannot find -lavcodec" / Linker errors**
- Verify `PKG_CONFIG_PATH` includes `libavcodec.pc`
- On Windows, check `FFMPEG_DIR` points to FFmpeg root

---

*See [AGENTS.md](../../AGENTS.md) for comprehensive AI agent guidelines.*
