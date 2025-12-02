use crate::element::Element;
use skia_safe::{Canvas, Image, Rect, Paint, Data, ColorType, AlphaType, ColorSpace};
use taffy::style::Style;
use std::sync::{Arc, Mutex};
use std::any::Any;
use usvg::{Tree, Options};
use tiny_skia::{Pixmap, Transform};

#[derive(Debug)]
pub struct VectorNode {
    tree: Arc<Tree>,
    cache: Mutex<Option<(u32, u32, Image)>>,
    pub style: Style,
    pub opacity: crate::animation::Animated<f32>,
}

impl Clone for VectorNode {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            cache: Mutex::new(None), // Don't share cache across clones (or maybe we should? No, size might differ)
            style: self.style.clone(),
            opacity: self.opacity.clone(),
        }
    }
}

impl VectorNode {
    pub fn new(data: &[u8]) -> Self {
        let opt = Options::default();
        // usvg 0.44: Tree::from_data
        let tree = Tree::from_data(data, &opt).expect("Failed to parse SVG");
        Self {
            tree: Arc::new(tree),
            cache: Mutex::new(None),
            style: Style::DEFAULT,
            opacity: crate::animation::Animated::new(1.0),
        }
    }
}

impl Element for VectorNode {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }

    fn layout_style(&self) -> Style {
        self.style.clone()
    }

    fn set_layout_style(&mut self, style: Style) {
        self.style = style;
    }

    fn update(&mut self, time: f64) -> bool {
        self.opacity.update(time);
        true
    }

    fn render(&self, canvas: &Canvas, rect: Rect, parent_opacity: f32, draw_children: &mut dyn FnMut(&Canvas)) {
        let width = rect.width().ceil() as u32;
        let height = rect.height().ceil() as u32;

        if width == 0 || height == 0 {
            draw_children(canvas);
            return;
        }

        let mut cache_guard = self.cache.lock().unwrap();

        // Check if cache is valid
        let needs_update = if let Some((w, h, _)) = *cache_guard {
             w != width || h != height
        } else {
             true
        };

        if needs_update {
            // Rasterize
            let tree_size = self.tree.size();

            // Aspect Ratio: Contain
            let sx = width as f32 / tree_size.width();
            let sy = height as f32 / tree_size.height();
            let scale = sx.min(sy);

            let tx = (width as f32 - tree_size.width() * scale) / 2.0;
            let ty = (height as f32 - tree_size.height() * scale) / 2.0;
            let transform = Transform::from_scale(scale, scale).post_translate(tx, ty);

            if let Some(mut pixmap) = Pixmap::new(width, height) {
                resvg::render(&self.tree, transform, &mut pixmap.as_mut());

                // Convert to Skia Image
                let data = Data::new_copy(pixmap.data());
                let image_info = skia_safe::ImageInfo::new(
                    (width as i32, height as i32),
                    ColorType::RGBA8888,
                    AlphaType::Premul,
                    Some(ColorSpace::new_srgb()),
                );

                if let Some(img) = skia_safe::images::raster_from_data(&image_info, data, (width * 4) as usize) {
                    *cache_guard = Some((width, height, img));
                }
            }
        }

        // Draw
        let final_opacity = self.opacity.current_value * parent_opacity;
        let mut paint = Paint::default();
        paint.set_alpha_f(final_opacity);

        if let Some((_, _, img)) = cache_guard.as_ref() {
            canvas.draw_image_rect(img, None, rect, &paint);
        }

        draw_children(canvas);
    }

    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str) {
        if property == "opacity" {
            let ease = match easing {
                "linear" => crate::animation::EasingType::Linear,
                "ease_in" => crate::animation::EasingType::EaseIn,
                "ease_out" => crate::animation::EasingType::EaseOut,
                "ease_in_out" => crate::animation::EasingType::EaseInOut,
                _ => crate::animation::EasingType::Linear,
            };
            self.opacity.add_segment(start, target, duration, ease);
        }
    }
}
