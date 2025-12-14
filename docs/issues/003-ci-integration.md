# [V1] Setup GitHub Actions CI Pipeline

## Task Summary
Implement a GitHub Actions CI pipeline that runs on every push/PR to ensure code quality, run tests, and catch regressions before merge.

## Context
The project currently has **no CI/CD** automation. For V1, we need:
- Automated builds to catch compile errors
- Test execution (unit, integration, visual regression)
- Clippy linting for code quality
- Rustfmt checking for consistent formatting

This is **P0** because without CI, regressions can silently merge to main.

## Scope

### In Scope
- GitHub Actions workflow for CI (`test.yml`)
- Build matrix (Linux, Windows, macOS)
- Rust toolchain management (stable)
- Test execution with caching
- Clippy and rustfmt checks
- Visual regression test execution
- Artifact upload for failed visual tests

### Out of Scope
- Release automation / publishing (separate issue)
- Cross-compilation (mobile platforms)
- Performance benchmarking in CI (separate issue)
- Deployment to any hosting

## Implementation Plan

### Phase 1: Basic CI Workflow
Create `.github/workflows/test.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --all-targets

  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets -- -D warnings

  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install FFmpeg (Ubuntu)
        if: matrix.os == 'ubuntu-latest'
        run: sudo apt-get update && sudo apt-get install -y ffmpeg libavutil-dev libavformat-dev libavcodec-dev libswscale-dev
      - name: Install FFmpeg (macOS)
        if: matrix.os == 'macos-latest'
        run: brew install ffmpeg
      - name: Install FFmpeg (Windows)
        if: matrix.os == 'windows-latest'
        run: choco install ffmpeg
      - run: cargo test --release
```

### Phase 2: Visual Regression in CI
Add visual regression test support:

```yaml
  visual-regression:
    name: Visual Regression
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install Dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y ffmpeg libavutil-dev libavformat-dev libavcodec-dev libswscale-dev
      - name: Run Visual Tests
        run: cargo test --release -p director-core visual
      - name: Upload Failure Artifacts
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: visual-regression-failures
          path: crates/director-core/target/visual_regression_failures/
          retention-days: 7
```

### Phase 3: Caching Optimization
Optimize build times with proper caching:
- Cache Cargo registry and target directory
- Use `rust-cache` action for intelligent caching
- Cache Skia bindings (largest build artifact)

### Phase 4: Branch Protection
Document recommended branch protection rules:
- Require status checks to pass before merge
- Require PR reviews
- Require linear history

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `.github/workflows/test.yml` | CREATE | Main CI workflow |
| `.github/workflows/release.yml` | CREATE | (Future) Release workflow |
| `docs/CONTRIBUTING.md` | MODIFY | Document CI requirements |
| `README.md` | MODIFY | Add CI badge |

## Acceptance Criteria

- [ ] CI runs on every push to main and on PRs
- [ ] Build passes on Linux, Windows, and macOS
- [ ] `cargo fmt --check` enforced
- [ ] `cargo clippy` with `-D warnings` enforced
- [ ] All tests run in CI
- [ ] Visual regression test failures upload artifacts
- [ ] Build cache reduces subsequent build times by 50%+
- [ ] CI badge displayed in README

## Verification Plan

### Automated Tests
```bash
# Local equivalent of CI checks
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --release
```

### Manual Verification
1. Create a PR with intentional formatting issue → verify CI fails
2. Create a PR with clippy warning → verify CI fails
3. Create a PR with failing test → verify CI fails and artifacts uploaded
4. Verify build times are reasonable (< 15 min with cache)

## Dependencies

**Blocked By:**
- [001-visual-regression-expansion.md](./001-visual-regression-expansion.md) - Need tests to run in CI

**Blocks:**
- [004-benchmark-suite.md](./004-benchmark-suite.md) - Benchmarks may run in CI

## Estimated Effort
- [x] 🟠 Large (1-3 days)

## Priority
- [x] P0: Must have for V1

## Notes

### Skia Build Considerations
- Skia bindings take ~5-10 minutes to build from scratch
- The `rust-cache` action should cache the compiled bindings
- Consider `sccache` for additional cross-job caching

### FFmpeg Considerations
- FFmpeg system dependencies vary by platform
- Windows may need pre-built binaries (chocolatey)
- Consider `mock_video` feature flag for tests that don't need real encoding

### GitHub Actions Costs
- Free tier: 2,000 minutes/month for private repos
- macOS runners use 10x minutes
- Consider running macOS only on main branch / releases

### Example Badge
```markdown
[![CI](https://github.com/YOUR_ORG/rust-skia-engine/actions/workflows/test.yml/badge.svg)](https://github.com/YOUR_ORG/rust-skia-engine/actions/workflows/test.yml)
```
