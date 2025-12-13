use crate::animation::Animated;
use crate::element::Element;
use crate::errors::RenderError;
use crate::node::{parse_easing, calculate_object_fit_rect};
use crate::types::ObjectFit;
use crate::video_wrapper::{VideoLoader, RenderMode, VideoResponse};
use skia_safe::{AlphaType, Canvas, ClipOp, Color4f, ColorType, Data, Image, Paint, Rect};
use std::any::Any;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use taffy::style::Style;
use tempfile::NamedTempFile;
use tracing::error;

/// Specifies the data source for a video node.
pub enum VideoSource {
    /// Load from a local file path.
    Path(PathBuf),
    /// Load from raw bytes (in-memory).
    Bytes(Vec<u8>),
}

// --- Video Node ---
/// A node that plays a video file.
///
/// Handles async decoding and frame buffering.
#[derive(Debug)]
pub struct VideoNode {
    pub opacity: Animated<f32>,
    pub style: Style,
    pub object_fit: ObjectFit,
    current_frame: Mutex<Option<(f64, Image)>>,

    loader: Option<VideoLoader>,
    render_mode: RenderMode,

    // Keep temp file alive
    #[allow(dead_code)]
    temp_file: Option<Arc<NamedTempFile>>,
    // Also keep path for cloning if it was a file path
    path: PathBuf,
}

impl Clone for VideoNode {
    fn clone(&self) -> Self {
        let loader = if self.loader.is_some() {
            // Create new loader pointing to same file.
            VideoLoader::new(self.path.clone(), self.render_mode).ok()
        } else {
            None
        };

        Self {
            opacity: self.opacity.clone(),
            style: self.style.clone(),
            object_fit: self.object_fit,
            current_frame: Mutex::new(None),
            loader,
            render_mode: self.render_mode,
            temp_file: self.temp_file.clone(),
            path: self.path.clone(),
        }
    }
}

impl VideoNode {
    pub fn new(source: VideoSource, mode: RenderMode) -> Self {
        let (path, temp_file) = match source {
            VideoSource::Path(p) => (p, None),
            VideoSource::Bytes(data) => {
                let mut temp = NamedTempFile::new().expect("Failed to create temp file");
                temp.write_all(&data).expect("Failed to write video data");
                let p = temp.path().to_owned();
                (p, Some(Arc::new(temp)))
            }
        };

        let loader = match VideoLoader::new(path.clone(), mode) {
            Ok(l) => Some(l),
            Err(e) => {
                error!("Failed to create VideoLoader for {:?}: {}", path, e);
                None
            }
        };

        Self {
            opacity: Animated::new(1.0),
            style: Style::DEFAULT,
            object_fit: ObjectFit::Cover,
            current_frame: Mutex::new(None),
            loader,
            render_mode: mode,
            temp_file,
            path,
        }
    }
}

impl Element for VideoNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn layout_style(&self) -> Style {
        self.style.clone()
    }

    fn set_layout_style(&mut self, style: Style) {
        self.style = style;
    }

    fn update(&mut self, time: f64) -> bool {
        self.opacity.update(time);

        if let Some(loader) = &mut self.loader {
            match loader {
                VideoLoader::Threaded(decoder) => {
                    decoder.send_request(time);

                    if let Some(resp) = decoder.get_response() {
                        match resp {
                            VideoResponse::Frame(t, data, w, h) => {
                                let data = Data::new_copy(&data);
                                let info = skia_safe::ImageInfo::new(
                                    (w as i32, h as i32),
                                    ColorType::RGBA8888,
                                    AlphaType::Unpremul,
                                    None,
                                );

                                if let Some(img) = skia_safe::images::raster_from_data(
                                    &info,
                                    data,
                                    (w * 4) as usize,
                                ) {
                                    *self.current_frame.lock().unwrap() = Some((t, img));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                VideoLoader::Sync(decoder) => match decoder.get_frame_at(time) {
                    Ok((t, data, w, h)) => {
                        let data = Data::new_copy(&data);
                        let info = skia_safe::ImageInfo::new(
                            (w as i32, h as i32),
                            ColorType::RGBA8888,
                            AlphaType::Unpremul,
                            None,
                        );

                        if let Some(img) =
                            skia_safe::images::raster_from_data(&info, data, (w * 4) as usize)
                        {
                            *self.current_frame.lock().unwrap() = Some((t, img));
                        }
                    }
                    Err(e) => {
                        error!("Sync Video Error: {}", e);
                    }
                },
            }
        }
        true
    }

    fn render(
        &self,
        canvas: &Canvas,
        rect: Rect,
        parent_opacity: f32,
        draw_children: &mut dyn FnMut(&Canvas),
    ) -> Result<(), RenderError> {
        let op = self.opacity.current_value * parent_opacity;

        let current = self.current_frame.lock().unwrap();
        if let Some((_, img)) = current.as_ref() {
            let paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, op), None);

            let draw_rect = calculate_object_fit_rect(
                img.width() as f32,
                img.height() as f32,
                rect,
                self.object_fit,
            );

            canvas.save();
            if self.object_fit == ObjectFit::Cover {
                canvas.clip_rect(rect, ClipOp::Intersect, true);
            }
            canvas.draw_image_rect(img, None, draw_rect, &paint);
            canvas.restore();
        } else {
            let mut p = Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None);
            p.set_alpha_f(op);
            canvas.draw_rect(rect, &p);
        }
        draw_children(canvas);
        Ok(())
    }

    fn animate_property(
        &mut self,
        property: &str,
        start: f32,
        target: f32,
        duration: f64,
        easing: &str,
    ) {
        let ease_fn = parse_easing(easing);
        if property == "opacity" {
            self.opacity.add_segment(start, target, duration, ease_fn);
        }
    }

    fn animate_property_spring(
        &mut self,
        property: &str,
        start: Option<f32>,
        target: f32,
        config: crate::animation::SpringConfig,
    ) {
        if property == "opacity" {
            if let Some(s) = start {
                self.opacity.add_spring_with_start(s, target, config);
            } else {
                self.opacity.add_spring(target, config);
            }
        }
    }
}
