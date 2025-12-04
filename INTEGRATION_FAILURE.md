# Integration Failure Report: rust-lottie-skia

## Overview
The integration of `rust-lottie-skia` (v0.1.0) into `director-engine` is functional for basic Lottie files but fails to parse complex or specific Lottie JSON structures found in common examples.

## Tested Files

### 1. Heart Eyes Burst
*   **Source:** `https://assets1.lottiefiles.com/packages/lf20_u4j3xm6r.json`
*   **Result:** Failure
*   **Error:** `Failed to load lottie: data did not match any variant of untagged enum Value`
*   **Analysis:** The `lottie-data` crate uses `serde` with strict schema validation. The error suggests that an animated property (likely `opacity`, `position`, or `scale`) uses a data format (e.g., specific keyframe structure or static value type) that the crate's data model does not support.

### 2. Mobilo/A
*   **Source:** `https://raw.githubusercontent.com/xvrh/lottie-flutter/master/example/assets/Mobilo/A.json`
*   **Result:** Failure
*   **Error:** `Failed to load lottie: data did not match any variant of untagged enum Value`
*   **Analysis:** Similar `serde` validation error. This file contains 3D position data (`[x, y, z]`) and other advanced properties. The `Value` enum in `lottie-data` likely fails to match the specific variant used in this file.

## Working Example
A verified Lottie JSON (`tests/fixtures/lottie_simple.json`) containing a basic Shape Layer (Solid Red Square) parses and renders correctly. We successfully animated this node using the engine's `animate` API, proving the `LottieNode` implementation is correct but limited by the underlying parser's compatibility.

## Recommendation
The `rust-lottie-skia` crate requires updates to its `lottie-data` models to support a wider range of Lottie JSON features (specifically relaxed parsing for Property Values). Until then, only simple Lottie files are guaranteed to work.
