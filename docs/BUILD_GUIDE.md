# Director Engine Build Guide

This guide explains how to set up the development environment for `director-engine`, with a focus on its system-level dependencies: **Skia** (via `skia-safe`) and **FFmpeg** (via `video-rs`).

## 1. System Dependencies

Since `cargo build` cannot install system libraries, you must install them manually before compiling.

### Linux (Ubuntu/Debian)

**Skia Requirements (LLVM/Clang)**
`skia-safe` builds its bindings using `bindgen`, which requires Clang.
```bash
sudo apt install clang libclang-dev llvm-dev
```

**FFmpeg Requirements**
`video-rs` links against FFmpeg libraries.
```bash
sudo apt install libavutil-dev libavformat-dev libavcodec-dev libswscale-dev libavfilter-dev libavdevice-dev
```

**Audio Requirements**
```bash
sudo apt install libasound2-dev
```

### MacOS

**Homebrew** is the recommended package manager.

```bash
# Clang is usually included in Xcode Command Line Tools
xcode-select --install

# Install FFmpeg
brew install ffmpeg
```

### Windows

**Skia**
Install **LLVM** from the official website or `winget install LLVM`. Ensure `libclang` is in your `PATH`.

**FFmpeg**
1.  Download the **Shared** build of FFmpeg (containing `.dll` and `.lib` files) from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) or [BtbN](https://github.com/BtbN/FFmpeg-Builds/releases).
2.  Extract it to a folder (e.g., `C:\ffmpeg`).
3.  Set the `FFMPEG_DIR` environment variable to that folder.
4.  Add `C:\ffmpeg\bin` to your system `PATH`.

## 2. Feature Flags

The engine exposes several feature flags in `crates/director-core/Cargo.toml` that alter the build process.

### `mock_video`
*   **Default**: Disabled.
*   **Purpose**: Enables building the engine **without FFmpeg installed**.
*   **Usage**: Useful for CI/CD pipelines, documentation builds (`docs.rs`), or working on layout/rendering logic where video encoding is not needed.
*   **Effect**: Video nodes will fail to load or render placeholders, and `render_export` will panic or no-op.

```bash
cargo build --no-default-features --features mock_video
```

### `vulkan`
*   **Default**: Disabled.
*   **Purpose**: Enables Vulkan backend for Skia.
*   **Usage**: Required if you intend to use hardware-accelerated rendering contexts (e.g., passing a `DirectContext` to the renderer).
*   **Requirements**: Vulkan SDK installed on the host machine.

```bash
cargo build --features vulkan
```

## 3. Troubleshooting

**"binding.rs not found" / Clang errors**
*   Ensure `LIBCLANG_PATH` is set if your distribution puts it in a non-standard location.
*   On Windows, ensure you are using the MSVC toolchain, not GNU.

**"cannot find -lavcodec" / Linker errors**
*   Verify `PKG_CONFIG_PATH` includes the directory containing `libavcodec.pc` (usually `/usr/lib/pkgconfig` or `/usr/local/lib/pkgconfig`).
*   On Windows, check `FFMPEG_DIR` matches the root of the extracted FFmpeg archive (containing `include` and `lib` folders).
