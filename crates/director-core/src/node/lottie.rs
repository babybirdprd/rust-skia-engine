use crate::element::Element;
use skia_safe::{Canvas, Rect, Image, Paint, SamplingOptions, surfaces, ImageInfo, ColorType, AlphaType, Color, FontMgr, Data};
use taffy::style::Style;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::any::Any;
use lottie_core::{LottiePlayer, LottieAsset};
use lottie_data::model::LottieJson;
use lottie_skia::{SkiaRenderer, LottieContext};
use crate::animation::{Animated, EasingType};
use crate::systems::assets::AssetManager;
use crate::AssetLoader;
use crate::RenderError;

/// Manages external assets (images, fonts) required by a Lottie animation.
pub struct LottieAssetManager {
    /// Map of asset IDs (from JSON) to pre-loaded Skia Images.
    pub images: HashMap<String, Image>,
    /// Shared loader for resolving font files.
    pub asset_loader: Arc<dyn AssetLoader>,
}

impl std::fmt::Debug for LottieAssetManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LottieAssetManager")
         .field("images_count", &self.images.len())
         .finish()
    }
}

impl LottieContext for LottieAssetManager {
    fn load_typeface(&self, family: &str, style: &str) -> Option<skia_safe::Typeface> {
        // 1. Check shared typeface cache first (not implemented in this PR but good practice)

        // 2. Try loading via AssetLoader (it handles paths/fallbacks now)
        let candidates = vec![
            format!("{}-{}.ttf", family, style),
            format!("{}.ttf", family),
        ];

        for path in candidates {
            if let Some(bytes) = self.load_bytes(&path) {
                 let data = Data::new_copy(&bytes);
                 return FontMgr::new().new_from_data(&data, 0);
            }
        }
        None
    }

    fn load_image(&self, id: &str) -> Option<Image> {
        self.images.get(id).cloned()
    }

    fn load_bytes(&self, path: &str) -> Option<Vec<u8>> {
        self.asset_loader.load_bytes(path).ok()
    }
}

/// A node that renders Lottie animations (JSON-based vector animations).
pub struct LottieNode {
    asset: Arc<LottieAsset>,
    player: Mutex<LottiePlayer>,
    pub style: Style,
    pub opacity: Animated<f32>,
    /// Current frame number (can be animated).
    pub frame: Animated<f32>,
    /// Playback speed multiplier (default 1.0).
    pub speed: f32,
    /// Whether to loop the animation automatically.
    pub loop_anim: bool,
    pub asset_manager: Arc<LottieAssetManager>,
    // Cache: (frame, width, height, image)
    cache: Mutex<Option<(f32, u32, u32, Image)>>,
}

impl std::fmt::Debug for LottieNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cache_state = if self.cache.lock().unwrap().is_some() { "Some" } else { "None" };
        f.debug_struct("LottieNode")
         .field("style", &self.style)
         .field("opacity", &self.opacity)
         .field("frame", &self.frame)
         .field("speed", &self.speed)
         .field("loop_anim", &self.loop_anim)
         .field("asset_manager", &self.asset_manager)
         .field("cache", &cache_state)
         .finish()
    }
}

impl Clone for LottieNode {
    fn clone(&self) -> Self {
        let mut player = LottiePlayer::new();
        player.load(self.asset.clone());

        Self {
            asset: self.asset.clone(),
            player: Mutex::new(player),
            style: self.style.clone(),
            opacity: self.opacity.clone(),
            frame: self.frame.clone(),
            speed: self.speed,
            loop_anim: self.loop_anim,
            asset_manager: self.asset_manager.clone(),
            cache: Mutex::new(None),
        }
    }
}

impl LottieNode {
    /// Creates a new LottieNode from raw JSON bytes.
    pub fn new(data: &[u8], assets: HashMap<String, Image>, asset_manager: &AssetManager) -> anyhow::Result<Self> {
        let json_str = std::str::from_utf8(data)?;
        let model: LottieJson = serde_json::from_str(json_str)?;

        let asset = Arc::new(LottieAsset::from_model(model));
        let mut player = LottiePlayer::new();
        player.load(asset.clone());

        Ok(Self {
            asset,
            player: Mutex::new(player),
            style: Style::DEFAULT,
            opacity: Animated::new(1.0),
            frame: Animated::new(0.0),
            speed: 1.0,
            loop_anim: false,
            asset_manager: Arc::new(LottieAssetManager { images: assets, asset_loader: asset_manager.loader.clone() }),
            cache: Mutex::new(None),
        })
    }
}

impl Element for LottieNode {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }

    fn layout_style(&self) -> Style { self.style.clone() }
    fn set_layout_style(&mut self, style: Style) { self.style = style; }

    fn update(&mut self, time: f64) -> bool {
        self.opacity.update(time);
        self.frame.update(time);

        let mut player = self.player.lock().unwrap();

        // If frame property has keyframes (more than the initial one), use it.
        if self.frame.raw_keyframes.len() > 1 {
             player.current_frame = self.frame.current_value;
        } else {
             let fps = self.asset.frame_rate;
             let start_frame = self.asset.model.ip;
             let end_frame = self.asset.model.op;

             // Check if duration is valid
             let total_frames = end_frame - start_frame;

             let current_raw = time * fps as f64 * self.speed as f64;

             if self.loop_anim && total_frames > 0.0 {
                 let looped = (current_raw % total_frames as f64) + start_frame as f64;
                 player.current_frame = looped as f32;
             } else {
                 let frame = start_frame as f64 + current_raw;
                 // Clamp to end
                 if frame > end_frame as f64 {
                     player.current_frame = end_frame;
                 } else {
                     player.current_frame = frame as f32;
                 }
             }
        }

        true
    }

    fn render(&self, canvas: &Canvas, rect: Rect, parent_opacity: f32, _draw_children: &mut dyn FnMut(&Canvas)) -> Result<(), RenderError> {
        let mut player = self.player.lock().unwrap();
        let current_frame = player.current_frame;
        let final_opacity = self.opacity.current_value * parent_opacity;

        // Skip rendering if fully transparent
        if final_opacity <= 0.0 {
            return Ok(());
        }

        // Determine integer dimensions for the cache
        let w = rect.width().ceil() as i32;
        let h = rect.height().ceil() as i32;

        if w <= 0 || h <= 0 {
            return Ok(());
        }

        let w_u32 = w as u32;
        let h_u32 = h as u32;

        let mut cache = self.cache.lock().unwrap();

        // Check cache hit
        let hit = if let Some((cached_frame, cached_w, cached_h, _)) = cache.as_ref() {
             (current_frame - cached_frame).abs() < 0.01 && *cached_w == w_u32 && *cached_h == h_u32
        } else {
            false
        };

        let image = if hit {
             cache.as_ref().unwrap().3.clone()
        } else {
            // Miss: Render to surface
            let image_info = ImageInfo::new(
                (w, h),
                ColorType::RGBA8888,
                AlphaType::Premul,
                None
            );

            // Try make_surface from canvas (GPU friendly)
            // Note: canvas.make_surface is not available in safe bindings directly or has different name.
            // Using raster fallback for now.
            let mut surface = surfaces::raster(&image_info, None, None).ok_or(RenderError::SurfaceFailure)?;

            // Clear surface
            surface.canvas().clear(Color::TRANSPARENT);

            // Render Lottie to surface
            let tree = player.render_tree();
            let draw_rect = Rect::from_wh(w as f32, h as f32);

            // Draw with alpha 1.0
            SkiaRenderer::draw(surface.canvas(), &tree, draw_rect, 1.0, &*self.asset_manager);

            let img = surface.image_snapshot();

            // Update cache
            *cache = Some((current_frame, w_u32, h_u32, img.clone()));
            img
        };

        // Draw cached image to main canvas
        let mut paint = Paint::default();
        paint.set_alpha_f(final_opacity);

        let sampling = SamplingOptions::default();

        canvas.draw_image_rect_with_sampling_options(
            &image,
            None,
            rect,
            sampling,
            &paint,
        );
        Ok(())
    }

    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str) {
        let ease = match easing {
            "linear" => EasingType::Linear,
            "ease_in" => EasingType::EaseIn,
            "ease_out" => EasingType::EaseOut,
            "ease_in_out" => EasingType::EaseInOut,
             _ => EasingType::Linear,
        };

        if property == "opacity" {
            self.opacity.add_segment(start, target, duration, ease);
        } else if property == "frame" {
            self.frame.add_segment(start, target, duration, ease);
        }
    }
}
