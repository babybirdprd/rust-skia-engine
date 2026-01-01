//! # Export Module
//!
//! Video export and encoding functionality.
//!
//! ## Responsibilities
//! - **Video Encoding**: FFmpeg integration via video-rs.
//! - **Motion Blur**: Multi-sample frame accumulation.
//! - **Audio Mixing**: Synchronizes audio with video frames.

pub mod video;

pub use video::render_export;
