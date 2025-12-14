# [V1] Implement Benchmark Suite

## Task Summary
Create a benchmark suite to track render performance and catch performance regressions before they reach users.

## Context
Video rendering is performance-critical. A single frame taking 100ms instead of 10ms means 10x slower export times. We need to:
- Establish baseline performance metrics
- Catch regressions before merge
- Track improvements over time

This is **P2** for V1 because correctness is more important than performance, but we should have *some* performance tracking.

## Scope

### In Scope
- Criterion.rs benchmark harness setup
- Core benchmarks (frame render, layout computation)
- Baseline establishment
- Local benchmark comparison workflow

### Out of Scope
- CI benchmark automation (complex, can be post-V1)
- GPU benchmarks (hardware-dependent)
- Profiling infrastructure (flamegraphs, etc.)

## Implementation Plan

### Phase 1: Setup Criterion.rs
Add benchmark infrastructure:

```toml
# crates/director-core/Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "render_benchmarks"
harness = false
```

### Phase 2: Core Benchmarks

| Benchmark | Description | Target |
|-----------|-------------|--------|
| `render_empty_frame` | Single empty scene render | < 1ms |
| `render_text_simple` | Single text node | < 5ms |
| `render_box_100` | 100 box nodes | < 20ms |
| `layout_100_nodes` | Taffy layout computation | < 5ms |
| `animation_update` | Animation system tick | < 1ms |

### Phase 3: Benchmark Implementation

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use director_core::{Director, DefaultAssetLoader, video_wrapper::RenderMode};
use std::sync::Arc;

fn bench_render_empty(c: &mut Criterion) {
    let mut director = Director::new(
        1920, 1080, 30,
        Arc::new(DefaultAssetLoader),
        RenderMode::Preview,
        None
    );
    // Setup scene...
    
    c.bench_function("render_empty_frame", |b| {
        b.iter(|| {
            director.update(black_box(0.0));
            // Render to surface
        })
    });
}

criterion_group!(benches, bench_render_empty);
criterion_main!(benches);
```

### Phase 4: Documentation
- Document how to run benchmarks locally
- Establish baseline numbers for reference
- Add to CONTRIBUTING.md

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/director-core/Cargo.toml` | MODIFY | Add criterion dev-dep |
| `crates/director-core/benches/render_benchmarks.rs` | CREATE | Core benchmarks |
| `docs/CONTRIBUTING.md` | MODIFY | Document benchmark workflow |

## Acceptance Criteria

- [ ] Criterion.rs benchmarks compile and run
- [ ] At least 5 core benchmarks covering render, layout, animation
- [ ] HTML reports generated in `target/criterion/`
- [ ] Baseline numbers documented
- [ ] Local comparison workflow documented

## Verification Plan

### Automated Tests
```bash
# Run all benchmarks
cargo bench -p director-core

# Run specific benchmark
cargo bench -p director-core -- render_empty

# Compare against baseline
cargo bench -p director-core -- --baseline main
```

### Manual Verification
1. Run benchmarks on clean main branch to establish baseline
2. Make an intentional performance regression (add sleep)
3. Verify benchmark detects the regression

## Dependencies

**Blocked By:**
- None

**Blocks:**
- None (nice-to-have for V1)

## Estimated Effort
- [x] 🟡 Medium (2-8 hours)

## Priority
- [x] P2: Nice to have for V1

## Notes

### Benchmark Stability
- Use `black_box` to prevent compiler optimizations
- Run benchmarks on a quiet machine (no other heavy processes)
- Use `--warm-up-time 3` for more stable results

### Future CI Integration
Post-V1, consider:
- Storing benchmark results in a database
- Charting performance over time
- Automated regression detection in PRs
- GitHub Actions `benchmark` workflow with artifact comparison

### Hardware Variance
Benchmark numbers will vary by machine. Document:
- Reference hardware specs
- Expected ranges rather than exact numbers
- Percentage change thresholds for concern
