# Rhai Standard Library Specification

## Overview

This document defines the **Standard Library** (DSL) for Director. These functions are exposed to the Rhai scripting environment via the `director-sdk` crate.
AI Agents should output scripts using *these* high-level functions, rather than raw math, to ensure consistency and reduce hallucination.

## 1\. The "Prelude" (Animation Shortcuts)

*Implemented in `assets/prelude.rhai` or Rust native wrappers.*

### Visibility & Transitions

  * `node.fade_in(duration: float, ease: string)`
      * **Logic:** Animates opacity 0.0 -\> 1.0.
  * `node.fade_out(duration: float, ease: string)`
      * **Logic:** Animates opacity 1.0 -\> 0.0.
  * `node.slide_in(from: string, duration: float)`
      * **Args:** `from` enum: "left", "right", "top", "bottom".
      * **Logic:** Animates `translate_x` or `translate_y` from offset to 0.
  * `node.pop_in(scale_overshoot: float)`
      * **Logic:** Scales 0.0 -\> `scale_overshoot` -\> 1.0 (Spring).

### Attention Grabbers

  * `node.shake(intensity: float, duration: float)`
      * **Logic:** Adds Perlin noise to rotation/position for `duration`.
  * `node.pulse(scale: float, bpm: float)`
      * **Logic:** loops scale animation matching a beat.
  * `node.glitch(intensity: float)`
      * **Logic:** Randomly jumps `skew_x` and `chromatic_aberration` uniform.

## 2\. The "Semantic" Wrappers (AI Integrations)

*Implemented in Rust (`director-sdk`) calling the `AiWorker`.*

### Vision (SAM 3)

  * `video.track_object(label: string) -> TrackHandle`
      * **Usage:** `let car = video.track_object("red sports car");`
      * **Behavior:** Runs SAM 3 inference. Returns a handle to the tracking data.
  * `node.pin_to(track: TrackHandle, anchor: string)`
      * **Usage:** `text.pin_to(car, "center_top");`
      * **Behavior:** Updates Taffy layout constraints every frame to match the tracked object's bounding box.
  * `video.extract_object(label: string) -> Node`
      * **Usage:** `let person = video.extract_object("speaker");`
      * **Behavior:** Returns a new `VideoNode` that shares the source but has a dynamic SAM 3 Alpha Mask applied.

### Audio (Parakeet/Supertonic)

  * `scene.generate_voiceover(text: string, voice: string) -> AudioTrack`
      * **Behavior:** Calls Supertonic. Adds audio to timeline. Returns track ID.
  * `video.auto_subtitle(style: Map)`
      * **Usage:** `video.auto_subtitle(#{ font: "Inter", color: "#FFF", highlight: "#F00" })`
      * **Behavior:** Calls Parakeet. Generates `TextNode`s for every word. Applies "Karaoke" animation (opacity/color shift) based on timestamps.

## 3\. Layout Intelligence

  * `container.distribute_children(direction: string, gap: float)`
      * **Logic:** Sets Flexbox properties to evenly space children.
  * `node.fit_to_screen(margin: float)`
      * **Logic:** Sets width/height to 100% minus margin.