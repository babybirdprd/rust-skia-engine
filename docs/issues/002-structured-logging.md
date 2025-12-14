# [V1] Structured Logging with Tracing

## Status: ✅ COMPLETE

Structured logging using the `tracing` ecosystem is fully implemented.

## What Was Implemented

- **Zero `eprintln!`/`println!`** in `director-core/src/`
- **`tracing` macros** used throughout (`info!`, `warn!`, `error!`, `debug!`)
- **`#[instrument]`** on key functions for span context
- **CLI subscriber** configured in `director-cli`
- **Test logging** via `tracing_subscriber::fmt::TestWriter`

## Verification

```bash
# Confirm no raw print statements
rg "println!|eprintln!" crates/director-core/src --type rust
# Result: No matches

# Run with debug logging
RUST_LOG=debug cargo run --release -- examples/basics/hello_world.rhai output.mp4
```

## Remaining Improvements (P2)

- [ ] Add `--log-format=json` CLI flag for NDJSON output
- [ ] Add more `#[instrument]` coverage on hot paths
- [ ] Consider `tracing-subscriber` layer for file logging
- [ ] Add span fields for render metrics (frame time, node count)

## Files with Tracing

| File | Usage |
|------|-------|
| `director.rs` | `info!`, `warn!` on lifecycle events |
| `systems/renderer.rs` | `debug!` for frame rendering |
| `systems/layout.rs` | `debug!` for layout computation |
| `node/text.rs` | `warn!` for disabled features |
| `scripting.rs` | `warn!`/`error!` for API issues |

---

*Original issue: Replace ad-hoc logging with tracing - COMPLETE*
