//! # Tokens Module
//!
//! Design system tokens exposed to scripting.
//!
//! ## Responsibilities
//! - **Spacing**: Preset spacing values (`xs`, `md`, `xl`, etc.).
//! - **Safe Areas**: Platform-specific UI-safe zones.
//! - **Border Tokens**: Radius and width presets.
//!
//! ## Key Types
//! - `DesignSystem`: Collection of all design tokens.
//! - `SafeZone`: Platform padding presets.

use crate::types::SafeZone;
use std::collections::HashMap;

/// A collection of standard design tokens (spacing, radii, safe areas) exposed to the scripting API.
#[derive(Clone, Debug)]
pub struct DesignSystem {
    /// Spacing tokens (px). Keys: `xs` (8), `md` (16), `xl` (32), etc.
    pub spacing: HashMap<String, f32>,
    /// Safe area presets for different platforms. Keys: `mobile`, `tiktok`, `desktop`, etc.
    pub safe_areas: HashMap<String, SafeZone>,
    /// Border radius presets (px). Keys: `sm` (4), `md` (8), `full` (9999).
    pub border_radius: HashMap<String, f32>,
    /// Border width presets (px). Keys: `thin` (1), `thick` (4).
    pub border_width: HashMap<String, f32>,
    /// Z-index presets. Keys: `background` (0), `overlay` (50), `modal` (100).
    pub z_index: HashMap<String, i32>,
    /// Internal layout width presets (currently unused).
    pub layout_widths: HashMap<String, f32>,
}

impl DesignSystem {
    /// Initializes the default design system with standard values.
    pub fn new() -> Self {
        let mut spacing = HashMap::new();
        spacing.insert("none".to_string(), 0.0);
        spacing.insert("xxs".to_string(), 4.0);
        spacing.insert("xs".to_string(), 8.0);
        spacing.insert("sm".to_string(), 12.0);
        spacing.insert("md".to_string(), 16.0);
        spacing.insert("lg".to_string(), 24.0);
        spacing.insert("xl".to_string(), 32.0);
        spacing.insert("2xl".to_string(), 48.0);
        spacing.insert("3xl".to_string(), 64.0);
        spacing.insert("4xl".to_string(), 80.0);
        spacing.insert("5xl".to_string(), 120.0);
        spacing.insert("6xl".to_string(), 160.0);
        spacing.insert("7xl".to_string(), 200.0);

        let mut border_radius = HashMap::new();
        border_radius.insert("none".to_string(), 0.0);
        border_radius.insert("xs".to_string(), 2.0);
        border_radius.insert("sm".to_string(), 4.0);
        border_radius.insert("md".to_string(), 8.0);
        border_radius.insert("lg".to_string(), 12.0);
        border_radius.insert("xl".to_string(), 16.0);
        border_radius.insert("2xl".to_string(), 24.0);
        border_radius.insert("3xl".to_string(), 32.0);
        border_radius.insert("full".to_string(), 9999.0);

        let mut border_width = HashMap::new();
        border_width.insert("none".to_string(), 0.0);
        border_width.insert("thin".to_string(), 1.0);
        border_width.insert("base".to_string(), 2.0);
        border_width.insert("thick".to_string(), 4.0);
        border_width.insert("heavy".to_string(), 8.0);
        border_width.insert("ultra".to_string(), 12.0);

        let mut z_index = HashMap::new();
        z_index.insert("underground".to_string(), -10);
        z_index.insert("background".to_string(), 0);
        z_index.insert("base".to_string(), 1);
        z_index.insert("content".to_string(), 10);
        z_index.insert("elevated".to_string(), 20);
        z_index.insert("overlay".to_string(), 50);
        z_index.insert("dropdown".to_string(), 75);
        z_index.insert("modal".to_string(), 100);
        z_index.insert("toast".to_string(), 200);
        z_index.insert("tooltip".to_string(), 300);
        z_index.insert("debug".to_string(), 9999);

        let mut safe_areas = HashMap::new();

        // Desktop / Default
        safe_areas.insert(
            "desktop".to_string(),
            SafeZone {
                top: 64.0,
                bottom: 64.0,
                left: 96.0,
                right: 96.0,
                aspect_ratio: "16:9".into(),
            },
        );

        // Mobile (Stories, Shorts, Reels)
        safe_areas.insert(
            "mobile".to_string(),
            SafeZone {
                top: 96.0,
                bottom: 144.0,
                left: 64.0,
                right: 64.0,
                aspect_ratio: "9:16".into(),
            },
        );

        // YouTube Shorts (Specific Overlays)
        safe_areas.insert(
            "youtube_shorts".to_string(),
            SafeZone {
                top: 120.0,
                bottom: 200.0,
                left: 48.0,
                right: 48.0,
                aspect_ratio: "9:16".into(),
            },
        );

        // TikTok (Heavy UI)
        safe_areas.insert(
            "tiktok".to_string(),
            SafeZone {
                top: 100.0,
                bottom: 180.0,
                left: 24.0,
                right: 80.0,
                aspect_ratio: "9:16".into(),
            },
        );

        // Instagram Reels
        safe_areas.insert(
            "instagram_reel".to_string(),
            SafeZone {
                top: 120.0,
                bottom: 160.0,
                left: 32.0,
                right: 32.0,
                aspect_ratio: "9:16".into(),
            },
        );

        // Instagram Story
        safe_areas.insert(
            "instagram_story".to_string(),
            SafeZone {
                top: 100.0,
                bottom: 120.0,
                left: 24.0,
                right: 24.0,
                aspect_ratio: "9:16".into(),
            },
        );

        // LinkedIn (Aggressive Cropping)
        safe_areas.insert(
            "linkedin".to_string(),
            SafeZone {
                top: 40.0,
                bottom: 40.0,
                left: 24.0,
                right: 24.0,
                aspect_ratio: "16:9".into(),
            },
        );

        // Twitter
        safe_areas.insert(
            "twitter".to_string(),
            SafeZone {
                top: 32.0,
                bottom: 32.0,
                left: 32.0,
                right: 32.0,
                aspect_ratio: "16:9".into(),
            },
        );

        Self {
            spacing,
            safe_areas,
            border_radius,
            border_width,
            z_index,
            layout_widths: HashMap::new(),
        }
    }
}
