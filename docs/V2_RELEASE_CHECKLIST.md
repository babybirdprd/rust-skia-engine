# Director Engine V2 Release Checklist

> **Target:** Feature-rich release with advanced typography, developer experience, and performance
> **Status:** 📋 Planned
> **Depends On:** V1 Release

---

## V2 Focus Areas

V2 builds on the stable V1 foundation with three major themes:

1. **Typography v2** — Re-implement text features lost in SkParagraph migration
2. **Developer Experience** — Tauri desktop app with the engine at its core
3. **Performance & Hardware** — Scale to 4K and high framerates

---

## Milestone 2: Typography v2

> *Focus: Re-implementing "flashy" text features on top of the stable SkParagraph foundation.*

| Feature | Priority | Effort | Status | Notes |
|---------|----------|--------|--------|-------|
| Text Animator (per-glyph) | **P0** | Large | ⬜ | Investigate `SkParagraph::getRectsForRange` |
| Wave/Typewriter effects | P0 | Medium | ⬜ | Depends on text animator |
| Text Shadows | P1 | Medium | ⬜ | Re-implement with Skia ImageFilters on Paragraph layer |
| Stroke + Fill simultaneous | P1 | Medium | ⬜ | Currently mutually exclusive |
| Letter spacing | P2 | Small | ⬜ | Expose `TextStyle` property |
| Text decorations | P2 | Small | ⬜ | Underline, strikethrough |
| Text gradients | P2 | Medium | ⬜ | fill_gradient for spans |

### Implementation Notes

**Text Animator Strategy:**
```
1. Use SkParagraph::getRectsForRange() to get bounding boxes per character
2. Render paragraph to offscreen surface
3. Extract and transform individual glyph regions
4. Composite with per-glyph animations (opacity, position, scale)
```

---

## Milestone 3: Developer Experience (Tauri App)

> *Focus: A native desktop app with Director Engine as the core rendering engine.*

| Feature | Priority | Effort | Status | Notes |
|---------|----------|--------|--------|-------|
| Tauri App Shell | **P0** | Large | ⬜ | Desktop app with Rust backend |
| Live Preview Window | **P0** | Large | ⬜ | Real-time render preview |
| Hot Reloading | P0 | Medium | ⬜ | Watch `.rhai` files, instant preview update |
| Code Editor Integration | P1 | Medium | ⬜ | Monaco/CodeMirror with Rhai syntax |
| Timeline UI | P1 | Large | ⬜ | Visual timeline for scenes/transitions |
| Asset Browser | P2 | Medium | ⬜ | Import and manage fonts/images/videos |
| Export Dialog | P1 | Small | ⬜ | Resolution, format, quality settings |
| Error Panel | P1 | Medium | ⬜ | Rhai errors with line numbers and suggestions |
| Project Management | P2 | Medium | ⬜ | Save/load projects, recent files |

### Tauri App Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri App Window                      │
├──────────────────────┬──────────────────────────────────┤
│   Frontend (Web)     │      Preview Panel               │
│  ┌────────────────┐  │  ┌────────────────────────────┐  │
│  │ Code Editor    │  │  │                            │  │
│  │ (Monaco/CM)    │  │  │   Live Skia Canvas         │  │
│  ├────────────────┤  │  │   (Director Engine)        │  │
│  │ Timeline UI    │  │  │                            │  │
│  ├────────────────┤  │  └────────────────────────────┘  │
│  │ Asset Browser  │  │  ┌────────────────────────────┐  │
│  ├────────────────┤  │  │ Error Panel / Console      │  │
│  │ Export Dialog  │  │  └────────────────────────────┘  │
│  └────────────────┘  │                                  │
└──────────────────────┴──────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│                  Rust Backend (Tauri)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │director-core│  │   Watcher   │  │  FFmpeg Export  │  │
│  │  (Engine)   │  │(.rhai files)│  │                 │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## Milestone 4: Performance & Hardware

> *Focus: Scaling to 4K and high framerates.*

| Feature | Priority | Effort | Status | Notes |
|---------|----------|--------|--------|-------|
| GPU Acceleration (Vulkan) | P1 | Large | ⬜ | Skia Vulkan backend |
| GPU Acceleration (Metal) | P1 | Large | ⬜ | Skia Metal backend for macOS |
| Hardware Encoding (NVENC) | P1 | Medium | ⬜ | Enable in video-rs |
| Hardware Encoding (VideoToolbox) | P1 | Medium | ⬜ | macOS hardware encoder |
| Motion Blur Rendering | P0 | Medium | ⬜ | Sub-frame accumulation (config exists) |
| Parallel Frame Rendering | P2 | Large | ⬜ | Render frame N and N+1 on separate threads |
| Render Caching | P2 | Medium | ⬜ | Cache static layers between frames |

---

## Milestone 5: Advanced Features

> *Focus: Expanding creative possibilities.*

| Feature | Priority | Effort | Status | Notes |
|---------|----------|--------|--------|-------|
| Audio FFT / Reactive | P1 | Medium | ⬜ | Expose frequency data to Rhai |
| Complex Shapes (PathOps) | P2 | Medium | ⬜ | Union, Difference, Xor |
| 3D Transforms | P2 | Large | ⬜ | Skia M44 matrix support |
| Particle Systems | P3 | Large | ⬜ | |
| Expression Engine | P3 | Large | ⬜ | After Effects-style expressions |

---

## API Gaps to Close

> *Carried over from V1 if not addressed.*

| Gap | Priority | Status | Notes |
|-----|----------|--------|-------|
| Grid layout parsing | P1 | ⬜ | Add grid_template_columns, gap |
| Missing easings (Back, Elastic) | P1 | ⬜ | Add to EasingType |
| DropShadow generic effect | P2 | ⬜ | Expose via apply_effect |
| Color shader uniforms | P3 | ⬜ | Native Color type support |

---

## Documentation

| Document | Status | Notes |
|----------|--------|-------|
| TAURI_APP.md | ⬜ | Desktop app user guide |
| PERFORMANCE.md | ⬜ | GPU setup, hardware encoding |
| TEXT_EFFECTS.md | ⬜ | Typography v2 feature guide |
| PROJECT_FORMAT.md | ⬜ | Project file specification |

---

## Compatibility

| Item | Status | Notes |
|------|--------|-------|
| V1 API backward compatibility | ⬜ | Ensure V1 scripts still work |
| Migration guide (if breaking) | ⬜ | Document any breaking changes |
| Deprecation warnings | ⬜ | For APIs that will change |

---

## Success Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| App install size | < 100MB | Including runtime |
| App startup time | < 2s | Cold start |
| 4K frame render time | < 50ms | On GPU |
| Hot reload latency | < 100ms | Script change to preview |

---

## Dependencies

| Dependency | Version | Notes |
|------------|---------|-------|
| tauri | 2.x | Desktop app framework |
| skia-safe | Latest | May need feature flags for GPU |
| video-rs | TBD | Hardware encoding support |
| notify | Latest | File watching for hot reload |

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ⬜ | Not started |
| 🟡 | In progress |
| ✅ | Complete |
| 🔶 | Blocked / Deferred |
| **P0** | Must have for V2 |
| P1 | Should have |
| P2 | Nice to have |
| P3 | Future / V3+ |

---

## Notes

### Rough Timeline (Post-V1)
- **Typography v2**: First priority after V1 stabilizes
- **Tauri App**: Can start parallel with Typography
- **GPU/Hardware**: After app is functional to enable real-world performance testing

### Risk Items
- SkParagraph `getRectsForRange` may have limitations for complex scripts
- GPU backend may require Skia feature flags not in skia-safe
- Tauri + Skia rendering integration (may need custom window)

### Tauri Considerations
- Skia canvas may need to render to a texture and blit to webview
- Alternative: Use custom Tauri window with raw Skia surface
- Consider using `tauri-egui` or similar for native rendering panel
