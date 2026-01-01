//! # Theme API
//!
//! Design system tokens exposed to Rhai scripts.
//!
//! ## Responsibilities
//! - **Spacing**: `theme.space("md")` for spacing values
//! - **Safe Areas**: `theme.safe_area("tiktok")` for platform-specific safe zones
//! - **Border Radius**: `theme.radius("lg")` for border radius tokens
//! - **Border Width**: `theme.border("thin")` for border width tokens
//! - **Z-Index**: `theme.z("overlay")` for z-index values

use crate::tokens::DesignSystem;
use rhai::{Map, Module};
use std::sync::Arc;

/// Creates a Rhai module for accessing design system tokens.
pub fn create_theme_api(system: DesignSystem) -> Module {
    let mut module = Module::new();
    let sys = Arc::new(system);

    // 1. Spacing: theme.space("md")
    let s = sys.clone();
    module.set_native_fn("space", move |key: &str| {
        Ok(s.spacing
            .get(key)
            .copied()
            .map(|v| v as f64)
            .unwrap_or(16.0))
    });

    // 2. Safe Area: theme.safe_area("tiktok") -> Map
    let s = sys.clone();
    module.set_native_fn("safe_area", move |platform: &str| {
        let zone = s
            .safe_areas
            .get(platform)
            .or_else(|| s.safe_areas.get("desktop"))
            .unwrap();
        let mut map = Map::new();
        map.insert("top".into(), (zone.top as f64).into());
        map.insert("bottom".into(), (zone.bottom as f64).into());
        map.insert("left".into(), (zone.left as f64).into());
        map.insert("right".into(), (zone.right as f64).into());
        Ok(map)
    });

    // 3. Border Radius: theme.radius("lg")
    let s = sys.clone();
    module.set_native_fn("radius", move |key: &str| {
        Ok(s.border_radius
            .get(key)
            .copied()
            .map(|v| v as f64)
            .unwrap_or(0.0))
    });

    // 4. Border Width: theme.border("thin")
    let s = sys.clone();
    module.set_native_fn("border", move |key: &str| {
        Ok(s.border_width
            .get(key)
            .copied()
            .map(|v| v as f64)
            .unwrap_or(0.0))
    });

    // 5. Z-Index: theme.z("overlay")
    let s = sys.clone();
    module.set_native_fn("z", move |key: &str| {
        Ok(s.z_index.get(key).copied().map(|v| v as i64).unwrap_or(1))
    });

    module
}
