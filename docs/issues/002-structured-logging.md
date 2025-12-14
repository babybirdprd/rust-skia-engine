# [V1] Implement Structured Logging with Tracing

## Task Summary
Replace all ad-hoc logging (`eprintln!`, `println!`) with the `tracing` ecosystem to enable structured, filterable logs for debugging and production observability.

## Context
The `AGENTS.md` already specifies that we should use the `tracing` ecosystem, but adoption may be incomplete. For V1, all logging should be:
- Structured (key-value pairs, not just strings)
- Filterable by level and module
- Instrumentable for performance tracking
- Compatible with JSON output for log aggregation

This is **P1** because good logging is essential for debugging issues in production and for users to diagnose problems.

## Scope

### In Scope
- Audit all crates for `eprintln!`/`println!` usage
- Replace with appropriate `tracing` macros
- Add `#[instrument]` to key functions
- Configure `director-cli` subscriber with JSON output option
- Add structured context to critical code paths

### Out of Scope
- Distributed tracing (OpenTelemetry integration)
- Log shipping/aggregation infrastructure
- Metrics collection

## Implementation Plan

### Phase 1: Audit
1. Search for all `eprintln!` and `println!` calls
2. Categorize by type:
   - Debug output → `tracing::debug!`
   - Progress info → `tracing::info!`
   - Warnings → `tracing::warn!`
   - Errors → `tracing::error!`
3. Identify hot paths that should NOT log (performance-critical loops)

### Phase 2: Core Instrumentation
Add `#[instrument]` to critical functions:

| Function | Crate | Span Fields |
|----------|-------|-------------|
| `Director::update` | director-core | `time`, `scene_count` |
| `LayoutEngine::compute_layout` | director-core | `node_count` |
| `render_frame` | director-core | `frame_number` |
| `AssetLoader::load_bytes` | director-core | `path` |
| `render_export` | director-core | `output_path`, `total_frames` |

### Phase 3: Structured Events
Add structured logging at key points:

```rust
// Before
eprintln!("Loading font: {}", path);

// After
tracing::info!(
    path = %path,
    "Loading font"
);
```

```rust
// Before
eprintln!("Render took {}ms", elapsed);

// After
tracing::debug!(
    elapsed_ms = elapsed.as_millis(),
    frame = frame_number,
    "Frame rendered"
);
```

### Phase 4: CLI Configuration
Update `director-cli` to support:
- `--log-level` flag (trace, debug, info, warn, error)
- `--log-format` flag (pretty, json)
- Environment variable fallback (`RUST_LOG`)

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/director-core/src/director.rs` | MODIFY | Add spans |
| `crates/director-core/src/systems/renderer.rs` | MODIFY | Add render logging |
| `crates/director-core/src/systems/layout.rs` | MODIFY | Add layout logging |
| `crates/director-core/src/systems/asset_manager.rs` | MODIFY | Add asset loading logs |
| `crates/director-cli/src/main.rs` | MODIFY | Configure subscriber |
| `crates/director-core/Cargo.toml` | MODIFY | Ensure `tracing` dep |

## Acceptance Criteria

- [ ] Zero `eprintln!` or `println!` calls in `director-core` (except tests)
- [ ] `#[instrument]` on all public API entry points
- [ ] Structured fields on all log events
- [ ] `director-cli --log-format=json` produces valid NDJSON
- [ ] Log levels are appropriate (no `info!` spam in hot paths)
- [ ] Tests use `tracing-test` or `tracing_subscriber::fmt::TestWriter`

## Verification Plan

### Automated Tests
```bash
# Verify no println/eprintln in core
rg "println!|eprintln!" crates/director-core/src --type rust

# Run with tracing
RUST_LOG=debug cargo run --release -- examples/test_text_center.rhai output.mp4
```

### Manual Verification
1. Run CLI with `--log-format=json` and pipe to `jq` to verify valid JSON
2. Run with `RUST_LOG=trace` and verify spans are properly nested
3. Check that render loop doesn't spam logs (debug level only)

## Dependencies

**Blocked By:**
- None

**Blocks:**
- [003-ci-integration.md](./003-ci-integration.md) - CI may want to capture logs

## Estimated Effort
- [x] 🟡 Medium (2-8 hours)

## Priority
- [x] P1: Should have for V1

## Notes

### Performance Considerations
- `tracing` macros with `debug!` and `trace!` are compiled out in release builds when not enabled
- Use `skip` field attribute for large data structures
- Avoid logging in per-pixel or per-glyph loops

### Example Patterns
```rust
// Span for a function
#[tracing::instrument(skip(self), fields(scene_index))]
pub fn render_scene(&mut self, index: usize) {
    tracing::Span::current().record("scene_index", index);
    // ...
}

// Event with context
tracing::info!(
    width = self.width,
    height = self.height,
    fps = self.fps,
    "Director initialized"
);
```
