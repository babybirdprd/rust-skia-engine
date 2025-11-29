use skia_safe::{Canvas, Rect};
use taffy::style::Style;

pub trait Element: std::fmt::Debug {
    // 1. Layout Phase: Return CSS-like style for Taffy
    fn layout_style(&self) -> Style;

    // 2. Update Phase: Advance animations to 'time'
    // Returns true if the element needs a redraw
    fn update(&mut self, time: f64) -> bool;

    // 3. Render Phase: Draw to Skia canvas
    // 'layout_rect' is provided by Taffy calculations
    // Changed to &Canvas because skia-safe 0.70+ uses interior mutability/shared ref for drawing methods
    fn render(&self, canvas: &Canvas, layout_rect: Rect, opacity: f32);

    // 4. Animation Interface
    fn animate_property(&mut self, property: &str, target: f32, duration: f64, easing: &str);
}
