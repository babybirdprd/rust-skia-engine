# Director Engine V1 Release Checklist

> **Target:** Stable, production-ready release of Director Engine
> **Status:** 🟡 In Progress
> **Last Updated:** 2025-12-13

---

## Release Criteria

Before V1 can ship, **ALL** of the following must be true:

| Criterion | Status | Notes |
|-----------|--------|-------|
| All P0 items complete | ⬜ | See checklist below |
| Zero known crash bugs | ⬜ | |
| Visual regression tests pass | ⬜ | |
| CI pipeline green | ⬜ | |
| Documentation complete | ⬜ | API.md, SCRIPTING.md, BUILD_GUIDE.md |
| Examples run successfully | ⬜ | All `.rhai` examples |
| crates.io publish ready | ⬜ | Cargo.toml metadata complete |

---

## Milestone 1: Observability & QA ⬜

> *Focus: Ensuring the engine is debuggable and regressions are impossible.*

| # | Task | Priority | Effort | Status | Issue |
|---|------|----------|--------|--------|-------|
| 001 | [Visual Regression Test Expansion](./issues/001-visual-regression-expansion.md) | **P0** | Large | ⬜ | |
| 002 | [Structured Logging (tracing)](./issues/002-structured-logging.md) | P1 | Medium | ⬜ | |
| 003 | [GitHub Actions CI](./issues/003-ci-integration.md) | **P0** | Large | ⬜ | |
| 004 | [Benchmark Suite](./issues/004-benchmark-suite.md) | P2 | Medium | ⬜ | |

---

## Core Functionality

> *Verified against actual implementation in `director-core`.*

### Elements (Nodes)

| Element | Implemented | API Exposed | Test Coverage | Notes |
|---------|-------------|-------------|---------------|-------|
| BoxNode | ✅ | ✅ | 🟡 Basic | bg, border, shadow, overflow |
| TextNode (SkParagraph) | ✅ | ✅ | ⬜ | Native Skia text, no cosmic-text |
| ImageNode | ✅ | ✅ | ⬜ | Cover, Contain, Fill |
| VideoNode | ✅ | ✅ | ⬜ | Sync + Async backends |
| VectorNode (SVG) | ✅ | ✅ | ⬜ | |
| LottieNode | ✅ | ✅ | ⬜ | 🔶 Expressions sidelined (warnings) |
| EffectNode | ✅ | ✅ | ⬜ | Wraps children with effects |
| CompositionNode | ✅ | ✅ | ⬜ | Nested timelines |

### Layout (Taffy)

| Feature | Implemented | API Exposed | Notes |
|---------|-------------|-------------|-------|
| Flexbox direction | ✅ | ✅ | row, column, reverse |
| justify_content | ✅ | ✅ | center, flex-start, space-between, etc. |
| align_items | ✅ | ✅ | center, stretch, flex-start, etc. |
| flex_grow / flex_shrink | ✅ | ✅ | |
| Percentage sizing | ✅ | ✅ | "100%", "50%" |
| Absolute positioning | ✅ | ✅ | position: "absolute", top/left/right/bottom |
| Z-index ordering | ✅ | ✅ | set_z_index() |
| Margin/Padding | ✅ | ✅ | |
| Grid Layout | 🔶 | ⬜ | Taffy supports, but not parsed in scripting |

### Animation

| Feature | Implemented | API Exposed | Notes |
|---------|-------------|-------------|-------|
| Keyframe tweening | ✅ | ✅ | add_segment() |
| Easing: linear | ✅ | ✅ | |
| Easing: ease_in/out/in_out | ✅ | ✅ | |
| Easing: bounce_out | ✅ | ✅ | |
| Easing: back, elastic | ⬜ | ⬜ | Not in EasingType enum |
| Spring physics | ✅ | ✅ | Baked at 60fps |
| Transform animations | ✅ | ✅ | scale, rotation, translate, skew |
| Path animation | ✅ | ✅ | animate_along_path() |
| **Text Animator (per-glyph)** | ⬜ | 🔶 | **DISABLED** - warns at runtime |
| Shader uniform animation | ✅ | ✅ | Float and Vec |

### Effects & Compositing

| Feature | Implemented | API Exposed | Notes |
|---------|-------------|-------------|-------|
| Gaussian blur | ✅ | ✅ | apply_effect("blur", value) |
| Color matrix (grayscale) | ✅ | ✅ | apply_effect("grayscale") |
| Color matrix (sepia) | ✅ | ✅ | apply_effect("sepia") |
| Color matrix (invert) | ✅ | ✅ | apply_effect("invert") |
| Contrast / Brightness | ✅ | ✅ | apply_effect("contrast", val) |
| Runtime shaders (SkSL) | ✅ | ✅ | apply_effect("shader", #{...}) |
| Drop shadow (Box/Text) | ✅ | ✅ | shadow_color, shadow_blur props |
| Drop shadow (generic) | ✅ | ⬜ | EffectType exists, not in apply_effect |
| Blend modes | ✅ | ✅ | set_blend_mode() - full Skia support |
| Alpha masking | ✅ | ✅ | set_mask() |
| Motion blur config | ✅ | ✅ | configure_motion_blur() |
| Motion blur rendering | 🔶 | — | Config exists, rendering TBD |

### Transitions

| Feature | Implemented | API Exposed | Notes |
|---------|-------------|-------------|-------|
| Fade | ✅ | ✅ | add_transition() |
| Slide left/right | ✅ | ✅ | |
| Wipe left/right | ✅ | ✅ | |
| Circle open | ✅ | ✅ | |
| Ripple edit logic | ✅ | ✅ | Auto-adjusts timeline |

### Audio

| Feature | Implemented | API Exposed | Notes |
|---------|-------------|-------------|-------|
| Audio loading | ✅ | ✅ | add_audio() |
| Multi-track mixing | ✅ | ✅ | |
| Volume automation | ✅ | ✅ | animate_volume() |
| Scene-synced audio | ✅ | ✅ | |

### Export

| Feature | Implemented | API Exposed | Notes |
|---------|-------------|-------------|-------|
| MP4 encoding (FFmpeg) | ✅ | ✅ | render_export() |
| Frame-accurate sync | ✅ | ✅ | |
| Audio muxing | ✅ | ✅ | |

---

## Known Limitations (Document or Fix)

| Item | Priority | Resolution |
|------|----------|------------|
| **Text Animator disabled** | P1 | Document as "Coming Soon" or re-implement with getRectsForRange |
| Grid layout not parsed | P2 | Add grid_template_columns, gap to parse_layout_style |
| Missing easings (Back, Elastic) | P2 | Add to EasingType enum |
| DropShadow not exposed generically | P3 | Add to apply_effect or document |
| Motion blur rendering incomplete | P2 | Implement sub-frame accumulation |
| Color uniforms need Vec\<f32\> | P3 | Document |

---

## Documentation ⬜

| Document | Status | Notes |
|----------|--------|-------|
| [README.md](../README.md) | ✅ | May need CI badge |
| [API.md](./API.md) | 🟡 | Update: text animator limitation |
| [SCRIPTING.md](./SCRIPTING.md) | 🟡 | Update: text animator limitation |
| [BUILD_GUIDE.md](./BUILD_GUIDE.md) | ✅ | |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | ✅ | |
| [ROADMAP.md](./ROADMAP.md) | 🟡 | Typography v2 section needs update |
| CHANGELOG.md | ⬜ | Need to create |
| [CONTRIBUTING.md](./CONTRIBUTING.md) | ✅ | Created |

---

## Examples & Demos ⬜

| Example | Status | Notes |
|---------|--------|-------|
| `test_text_center.rhai` | ⬜ | Verify runs |
| `test_z_index.rhai` | ⬜ | Verify runs |
| `cinematic_showcase.rhai` | ⬜ | Verify runs |
| `debug_showcase.rhai` | ⬜ | Verify runs |
| `ultimate_showcase.rhai` | ⬜ | Verify runs + asset download |
| `vector_example.rhai` | ⬜ | Verify runs |

---

## Publishing Checklist ⬜

| Task | Status | Notes |
|------|--------|-------|
| Cargo.toml version = "1.0.0" | ⬜ | Currently "1.1.1" in README example |
| Cargo.toml description complete | ⬜ | |
| Cargo.toml keywords/categories | ⬜ | |
| Cargo.toml license | ⬜ | |
| Cargo.toml repository URL | ⬜ | |
| `cargo publish --dry-run` passes | ⬜ | |
| GitHub Release created | ⬜ | |
| Release notes written | ⬜ | |

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ⬜ | Not started |
| 🟡 | In progress / Partial |
| ✅ | Complete |
| 🔶 | Blocked / Deferred / Known Issue |
| **P0** | Must have for V1 |
| P1 | Should have for V1 |
| P2 | Nice to have |
| P3 | Post-V1 |

---

## Progress Tracking

### Weekly Check-in Template

```markdown
## Week of YYYY-MM-DD

### Completed
- 

### In Progress
- 

### Blocked
- 

### Next Week
- 
```

---

## Notes

### Sidelined Items (Post-V1)
- Lottie expressions (compiler warnings)
- Text Animator / per-glyph animation (needs getRectsForRange investigation)
- WASM compilation
- Web Playground
- GPU acceleration (Vulkan/Metal)
- 3D transforms

### Key Decision Log
| Date | Decision | Rationale |
|------|----------|-----------|
| 2025-12-13 | Sideline Lottie expressions | Compiler warnings, not critical for V1 |
| 2025-12-13 | Text Animator disabled | SkParagraph migration broke it; needs investigation |
| 2025-12-13 | P0: Visual regression + CI | Foundation for stable releases |
| 2025-12-13 | Switched cosmic-text → SkParagraph | Cosmic-text lacked flexibility for engine needs |
