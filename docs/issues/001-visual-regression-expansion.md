# [V1] Expand Visual Regression Test Suite

## Task Summary
Expand the visual regression test harness to cover all core rendering features, ensuring regressions are caught before V1 release.

## Context
The engine currently has a basic visual regression infrastructure in `crates/director-core/tests/visual_regression.rs` with a single test (`test_visual_basic_box`). For a V1 release, we need comprehensive coverage of:
- All node types (Box, Text, Image, Video, Vector, Lottie)
- Animation states (start, middle, end of tweens)
- Layout configurations (Flexbox centering, alignment, sizing)
- Visual effects (blur, shadows, blend modes, shaders)
- Compositing (masking, opacity, z-index)

This is **P0** because visual regressions directly impact users and are the most visible form of bug.

## Scope

### In Scope
- Adding snapshot tests for each core Element type
- Testing animation interpolation at key frames
- Testing Flexbox layout permutations
- Testing effect rendering (blur, shadows)
- Documenting the snapshot update workflow

### Out of Scope
- Lottie rendering tests (sidelined per roadmap)
- Audio testing (no visual component)
- Performance benchmarking (separate issue)

## Implementation Plan

### Phase 1: Infrastructure Improvements
1. Refactor `assert_frame_match` into a shared test utility module
2. Add configurable tolerance thresholds per test (some effects like blur may have platform variance)
3. Create snapshot naming convention: `{element}_{feature}_{state}.png`
4. Add helper macros for common test patterns

### Phase 2: Core Element Tests
Create tests for each node type with default and styled variants:

| Test Name | Element | Description |
|-----------|---------|-------------|
| `box_default` | BoxNode | Default styling |
| `box_styled` | BoxNode | Border, radius, gradient |
| `box_shadow` | BoxNode | Drop shadow rendering |
| `text_basic` | TextNode | Simple text rendering |
| `text_styled` | TextNode | Bold, italic, color |
| `text_multiline` | TextNode | Wrapped text layout |
| `image_cover` | ImageNode | ObjectFit::Cover |
| `image_contain` | ImageNode | ObjectFit::Contain |
| `vector_svg` | VectorNode | SVG rendering |

### Phase 3: Layout Tests
Test Flexbox layout permutations:

| Test Name | Description |
|-----------|-------------|
| `layout_center_center` | justify-content: center, align-items: center |
| `layout_space_between` | justify-content: space-between |
| `layout_column` | flex-direction: column |
| `layout_wrap` | flex-wrap: wrap |
| `layout_absolute` | position: absolute with offsets |
| `layout_z_index` | z-index ordering |

### Phase 4: Animation Tests
Capture frames at animation keypoints:

| Test Name | Description |
|-----------|-------------|
| `anim_opacity_start` | Opacity animation at t=0 |
| `anim_opacity_mid` | Opacity animation at t=0.5 |
| `anim_opacity_end` | Opacity animation at t=1.0 |
| `anim_scale` | Scale transform animation |
| `anim_translate` | Position animation |
| `anim_spring` | Spring physics animation |

### Phase 5: Effects Tests
Test visual effects:

| Test Name | Description |
|-----------|-------------|
| `effect_blur` | Gaussian blur filter |
| `effect_blend_multiply` | Blend mode: Multiply |
| `effect_blend_screen` | Blend mode: Screen |
| `effect_mask_alpha` | Alpha masking |

### Phase 6: Documentation
- Update `docs/CONTRIBUTING.md` with visual regression workflow
- Document `UPDATE_SNAPSHOTS=1` usage
- Add CI instructions for snapshot management

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/director-core/tests/visual_regression.rs` | MODIFY | Refactor and add helper macros |
| `crates/director-core/tests/visual/mod.rs` | CREATE | Shared test utilities |
| `crates/director-core/tests/visual/elements.rs` | CREATE | Element rendering tests |
| `crates/director-core/tests/visual/layout.rs` | CREATE | Layout tests |
| `crates/director-core/tests/visual/animation.rs` | CREATE | Animation state tests |
| `crates/director-core/tests/visual/effects.rs` | CREATE | Effect rendering tests |
| `crates/director-core/tests/snapshots/*.png` | CREATE | Golden master images |
| `docs/CONTRIBUTING.md` | MODIFY | Add visual regression docs |

## Acceptance Criteria

- [ ] At least 20 visual regression tests covering core functionality
- [ ] All tests pass on the current codebase
- [ ] Snapshot update workflow is documented
- [ ] Test helper macros reduce boilerplate by 50%+
- [ ] Failure artifacts (diff images) are generated in `target/visual_regression_failures/`
- [ ] 0.1% pixel difference tolerance is configurable per-test

## Verification Plan

### Automated Tests
```bash
# Run all visual regression tests
cargo test --release -p director-core visual

# Update snapshots (when intentionally changing rendering)
UPDATE_SNAPSHOTS=1 cargo test --release -p director-core visual

# Run specific test
cargo test --release -p director-core test_visual_box_styled
```

### Manual Verification
1. Intentionally break a test (change color) and verify diff image is generated
2. Review all generated snapshots for visual correctness
3. Run on both Windows and Linux to check cross-platform consistency

## Dependencies

**Blocked By:**
- None (can start immediately)

**Blocks:**
- [003-ci-integration.md](./003-ci-integration.md) - CI needs these tests to run

## Estimated Effort
- [x] 🟠 Large (1-3 days)

## Priority
- [x] P0: Must have for V1

## Notes

### Open Questions
1. Should we commit PNG snapshots to git or use a separate artifact store?
   - **Recommendation**: Commit to git for simplicity, use Git LFS if they exceed 10MB total
2. How to handle cross-platform rendering differences (font hinting, anti-aliasing)?
   - **Recommendation**: Use higher tolerance (0.5%) for text tests, or render text to paths

### Design Decisions
- Using PNG format for lossless comparison (not JPEG)
- Diff images highlight mismatches in magenta for easy spotting
- Test resolution is kept small (200x200 - 400x400) for fast comparison
