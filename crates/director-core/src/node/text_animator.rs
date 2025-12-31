//! Text animator system for per-glyph animations.
//!
//! Enables kinetic typography effects like karaoke reveals, wave animations,
//! and staggered letter entrances.

use crate::animation::{Animated, EasingType};

/// Properties that can be animated on individual glyphs.
#[derive(Clone, Debug, PartialEq)]
pub enum TextAnimProperty {
    /// Glyph opacity (0.0 to 1.0)
    Opacity,
    /// Horizontal offset in pixels
    OffsetX,
    /// Vertical offset in pixels  
    OffsetY,
    /// Uniform scale factor (1.0 = normal)
    Scale,
    /// Rotation in degrees
    Rotation,
}

impl TextAnimProperty {
    /// Returns the default "from" value for this property type.
    pub fn default_start(&self) -> f32 {
        match self {
            TextAnimProperty::Opacity => 0.0,
            TextAnimProperty::OffsetX => 0.0,
            TextAnimProperty::OffsetY => 0.0,
            TextAnimProperty::Scale => 1.0,
            TextAnimProperty::Rotation => 0.0,
        }
    }

    /// Returns the default "to" value (resting state) for this property type.
    pub fn default_end(&self) -> f32 {
        match self {
            TextAnimProperty::Opacity => 1.0,
            TextAnimProperty::OffsetX => 0.0,
            TextAnimProperty::OffsetY => 0.0,
            TextAnimProperty::Scale => 1.0,
            TextAnimProperty::Rotation => 0.0,
        }
    }
}

/// Computed animation values for a single glyph at a specific time.
#[derive(Clone, Debug, Default)]
pub struct GlyphAnimState {
    pub opacity: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale: f32,
    pub rotation: f32,
}

impl GlyphAnimState {
    /// Returns a default state with no transformations applied.
    pub fn identity() -> Self {
        Self {
            opacity: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
            rotation: 0.0,
        }
    }
}

/// Animates a range of glyphs over time with optional stagger.
#[derive(Clone, Debug)]
pub struct TextAnimator {
    /// Starting character index (inclusive)
    pub start_idx: usize,
    /// Ending character index (exclusive)
    pub end_idx: usize,
    /// The property being animated
    pub property: TextAnimProperty,
    /// The animation curve
    pub animation: Animated<f32>,
    /// Delay between each glyph's animation start (in seconds)
    /// 0.0 means all glyphs animate together
    pub stagger: f32,
    /// Total duration of the base animation (before stagger)
    pub duration: f64,
}

impl TextAnimator {
    /// Creates a new text animator for the specified glyph range.
    pub fn new(
        start_idx: usize,
        end_idx: usize,
        property: TextAnimProperty,
        start_val: f32,
        end_val: f32,
        duration: f64,
        easing: EasingType,
        stagger: f32,
    ) -> Self {
        let mut animation = Animated::new(start_val);
        animation.add_keyframe(end_val, duration, easing);

        Self {
            start_idx,
            end_idx,
            property,
            animation,
            stagger,
            duration,
        }
    }

    /// Returns the number of glyphs this animator affects.
    pub fn glyph_count(&self) -> usize {
        self.end_idx.saturating_sub(self.start_idx)
    }

    /// Returns the total duration including stagger.
    pub fn total_duration(&self) -> f64 {
        let stagger_delay = self.stagger as f64 * (self.glyph_count().saturating_sub(1)) as f64;
        self.duration + stagger_delay
    }

    /// Gets the animated value for a specific glyph at the given time.
    ///
    /// # Arguments
    /// * `glyph_idx` - The glyph index within the animator's range (0-based relative to start_idx)
    /// * `time` - Current animation time in seconds
    pub fn value_for_glyph(&self, glyph_idx: usize, time: f64) -> f32 {
        // Calculate the staggered start time for this glyph
        let glyph_start = glyph_idx as f64 * self.stagger as f64;
        let local_time = (time - glyph_start).max(0.0);

        // Clamp to animation duration
        let clamped_time = local_time.min(self.duration);

        // Clone animation and evaluate at local time
        let mut anim = self.animation.clone();
        anim.update(clamped_time);
        anim.current_value
    }
}

/// Parses a property name string into a TextAnimProperty.
pub fn parse_text_anim_property(name: &str) -> Option<TextAnimProperty> {
    match name.to_lowercase().as_str() {
        "opacity" | "alpha" => Some(TextAnimProperty::Opacity),
        "offset_x" | "x" | "translatex" => Some(TextAnimProperty::OffsetX),
        "offset_y" | "y" | "translatey" => Some(TextAnimProperty::OffsetY),
        "scale" => Some(TextAnimProperty::Scale),
        "rotation" | "rotate" => Some(TextAnimProperty::Rotation),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stagger_timing() {
        let animator = TextAnimator::new(
            0,
            5,
            TextAnimProperty::Opacity,
            0.0,
            1.0,
            1.0,
            EasingType::Linear,
            0.1, // 0.1s stagger
        );

        // First glyph should start immediately
        assert!((animator.value_for_glyph(0, 0.0) - 0.0).abs() < 0.01);
        assert!((animator.value_for_glyph(0, 0.5) - 0.5).abs() < 0.01);
        assert!((animator.value_for_glyph(0, 1.0) - 1.0).abs() < 0.01);

        // Second glyph starts at 0.1s
        assert!((animator.value_for_glyph(1, 0.0) - 0.0).abs() < 0.01); // Not started
        assert!((animator.value_for_glyph(1, 0.1) - 0.0).abs() < 0.01); // Just started
        assert!((animator.value_for_glyph(1, 0.6) - 0.5).abs() < 0.01); // Halfway
    }

    #[test]
    fn test_total_duration() {
        let animator = TextAnimator::new(
            0,
            5,
            TextAnimProperty::Opacity,
            0.0,
            1.0,
            1.0,
            EasingType::Linear,
            0.1,
        );

        // 1.0s base + 0.4s stagger (4 gaps between 5 glyphs)
        assert!((animator.total_duration() - 1.4).abs() < 0.01);
    }
}
