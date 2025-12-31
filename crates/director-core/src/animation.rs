//! # Animation Module
//!
//! Keyframe and spring-based animation system.
//!
//! ## Responsibilities
//! - **Animated<T>**: Generic animated value with keyframe segments.
//! - **Easing**: All easing functions (linear, ease_in_out, elastic, bounce, etc.).
//! - **Spring Physics**: Critically-damped spring for smooth animations.
//!
//! ## Key Types
//! - `Animated<T>`: Holds keyframes and evaluates at a given time.
//! - `EasingType`: Enum of all supported easing functions.
//! - `SpringConfig`: Configuration for spring animations.

use keyframe::{AnimationSequence, CanTween, EasingFunction, Keyframe};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported easing functions for animations.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    // Bounce easings - ball bouncing effect
    BounceOut,
    BounceIn,
    BounceInOut,
    // Elastic easings - spring-like overshoot oscillation
    ElasticOut,
    ElasticIn,
    ElasticInOut,
    // Back easings - overshoots then settles
    BackOut,
    BackIn,
    BackInOut,
}

impl EasingFunction for EasingType {
    fn y(&self, x: f64) -> f64 {
        match self {
            EasingType::Linear => keyframe::functions::Linear.y(x),
            EasingType::EaseIn => keyframe::functions::EaseIn.y(x),
            EasingType::EaseOut => keyframe::functions::EaseOut.y(x),
            EasingType::EaseInOut => keyframe::functions::EaseInOut.y(x),
            // Bounce easings
            EasingType::BounceOut => bounce_out(x),
            EasingType::BounceIn => 1.0 - bounce_out(1.0 - x),
            EasingType::BounceInOut => {
                if x < 0.5 {
                    (1.0 - bounce_out(1.0 - 2.0 * x)) / 2.0
                } else {
                    (1.0 + bounce_out(2.0 * x - 1.0)) / 2.0
                }
            }
            // Elastic easings
            EasingType::ElasticOut => elastic_out(x),
            EasingType::ElasticIn => 1.0 - elastic_out(1.0 - x),
            EasingType::ElasticInOut => {
                if x < 0.5 {
                    (1.0 - elastic_out(1.0 - 2.0 * x)) / 2.0
                } else {
                    (1.0 + elastic_out(2.0 * x - 1.0)) / 2.0
                }
            }
            // Back easings
            EasingType::BackOut => back_out(x),
            EasingType::BackIn => 1.0 - back_out(1.0 - x),
            EasingType::BackInOut => {
                if x < 0.5 {
                    (1.0 - back_out(1.0 - 2.0 * x)) / 2.0
                } else {
                    (1.0 + back_out(2.0 * x - 1.0)) / 2.0
                }
            }
        }
    }
}

/// Bounce out easing - piecewise parabolic segments simulating ball bouncing
fn bounce_out(x: f64) -> f64 {
    const N1: f64 = 7.5625;
    const D1: f64 = 2.75;

    if x < 1.0 / D1 {
        N1 * x * x
    } else if x < 2.0 / D1 {
        let x = x - 1.5 / D1;
        N1 * x * x + 0.75
    } else if x < 2.5 / D1 {
        let x = x - 2.25 / D1;
        N1 * x * x + 0.9375
    } else {
        let x = x - 2.625 / D1;
        N1 * x * x + 0.984375
    }
}

/// Elastic out easing - decaying sinusoidal oscillation
fn elastic_out(x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    if x == 1.0 {
        return 1.0;
    }
    let c4 = (2.0 * std::f64::consts::PI) / 3.0;
    2.0_f64.powf(-10.0 * x) * ((x * 10.0 - 0.75) * c4).sin() + 1.0
}

/// Back out easing - overshoots then settles
fn back_out(x: f64) -> f64 {
    const C1: f64 = 1.70158;
    const C3: f64 = C1 + 1.0;
    let t = x - 1.0;
    1.0 + C3 * t * t * t + C1 * t * t
}

impl EasingType {
    /// Evaluates the easing curve at a specific point `x` (0.0 to 1.0).
    pub fn eval(&self, x: f32) -> f32 {
        self.y(x as f64) as f32
    }
}

/// Configuration for physics-based spring animations.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SpringConfig {
    /// Tension of the spring (controls speed).
    pub stiffness: f32,
    /// Friction (controls oscillation decay).
    pub damping: f32,
    /// Mass of the object (controls inertia).
    pub mass: f32,
    /// Initial velocity of the animation.
    pub velocity: f32,
}

impl Default for SpringConfig {
    fn default() -> Self {
        // "Wobbly" default for visibility
        Self {
            stiffness: 100.0,
            damping: 10.0,
            mass: 1.0,
            velocity: 0.0,
        }
    }
}

/// A wrapper for `Vec<f32>` that implements `CanTween`, allowing interpolation of vector uniforms.
#[derive(Clone, Debug, Default)]
pub struct TweenableVector(pub Vec<f32>);

impl CanTween for TweenableVector {
    fn ease(from: Self, to: Self, time: impl keyframe::num_traits::Float) -> Self {
        let t = time.to_f64().unwrap();
        let len = from.0.len().min(to.0.len());
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let start = from.0[i] as f64;
            let end = to.0[i] as f64;
            let val = start + (end - start) * t;
            result.push(val as f32);
        }

        // If lengths differ, maybe we should fill with 0 or the target value?
        // But for shader uniforms, length usually matches.
        // We'll just take the min length.

        TweenableVector(result)
    }
}

/// A generic animated value that tracks keyframes and current state.
#[derive(Clone)]
pub struct Animated<T>
where
    T: Clone + keyframe::CanTween + Default,
{
    /// Raw storage of keyframes (value, absolute_time, easing).
    pub raw_keyframes: Vec<(T, f64, EasingType)>,
    /// The underlying keyframe sequence used for interpolation.
    pub sequence: AnimationSequence<T>,
    /// The current calculated value for the last updated time.
    pub current_value: T,
}

impl<T> Animated<T>
where
    T: Clone + keyframe::CanTween + Default,
{
    /// Creates a new animated value with an initial state and no motion.
    pub fn new(initial: T) -> Self {
        let raw = vec![(initial.clone(), 0.0, EasingType::Linear)];
        let kf = Keyframe::new(initial.clone(), 0.0, EasingType::Linear);

        Self {
            sequence: AnimationSequence::from(vec![kf]),
            raw_keyframes: raw,
            current_value: initial,
        }
    }

    /// Appends a new keyframe to the end of the current sequence.
    ///
    /// # Arguments
    /// * `target` - The value to reach.
    /// * `duration` - Time in seconds to reach the target from the previous keyframe.
    /// * `easing` - The easing curve to use.
    pub fn add_keyframe(&mut self, target: T, duration: f64, easing: EasingType) {
        let current_end_time = self.sequence.duration();
        let new_time = current_end_time + duration;

        self.raw_keyframes.push((target.clone(), new_time, easing));

        // Rebuild sequence
        let frames: Vec<Keyframe<T>> = self
            .raw_keyframes
            .iter()
            .map(|(val, time, ease_type)| Keyframe::new(val.clone(), *time, *ease_type))
            .collect();

        self.sequence = AnimationSequence::from(frames);
    }

    /// Returns the total duration of the animation sequence in seconds.
    pub fn duration(&self) -> f64 {
        self.sequence.duration()
    }

    /// Adds a discrete animation segment (jump to start, then animate to target).
    ///
    /// Useful for stringing together unrelated movements (e.g. "move from A to B" then later "move from C to D").
    pub fn add_segment(&mut self, start: T, target: T, duration: f64, easing: EasingType) {
        if self.sequence.duration() == 0.0 {
            // If no animation exists yet, treat start as the initial value
            // We use the requested easing for the first keyframe too
            self.raw_keyframes = vec![(start.clone(), 0.0, easing)];
            let kf = Keyframe::new(start.clone(), 0.0, easing);
            self.sequence = AnimationSequence::from(vec![kf]);
            self.current_value = start;
        } else {
            // If animation exists, we jump to 'start' immediately at the current end time
            self.add_keyframe(start, 0.0, EasingType::Linear);
        }
        self.add_keyframe(target, duration, easing);
    }

    /// Updates `current_value` based on the provided absolute time.
    pub fn update(&mut self, time: f64) {
        self.sequence.advance_to(time);
        self.current_value = self.sequence.now();
    }
}

impl Animated<f32> {
    /// Adds a spring animation from the current value to the target.
    ///
    /// This "bakes" the physics simulation into a series of linear keyframes at 60fps.
    pub fn add_spring(&mut self, target: f32, config: SpringConfig) {
        let start = if let Some(last) = self.raw_keyframes.last() {
            last.0
        } else {
            self.current_value
        };

        self.add_spring_with_start(start, target, config);
    }

    /// Adds a spring animation starting from an explicit value.
    pub fn add_spring_with_start(&mut self, start: f32, target: f32, config: SpringConfig) {
        // If start is different from last keyframe, we insert a jump
        if let Some(last) = self.raw_keyframes.last() {
            if (last.0 - start).abs() > 0.0001 {
                self.add_keyframe(start, 0.0, EasingType::Linear);
            }
        } else {
            // Should verify if this ever happens as ::new sets a keyframe
            *self = Self::new(start);
        }

        let frames = solve_spring(start, target, config);

        let mut previous_time = 0.0;
        for (value, time) in frames {
            let dt = time - previous_time;
            // dt is f64, time is f64
            self.add_keyframe(value, dt, EasingType::Linear);
            previous_time = time;
        }
    }
}

/// Simulates a spring physics system and returns a list of (value, time) tuples.
fn solve_spring(start: f32, end: f32, config: SpringConfig) -> Vec<(f32, f64)> {
    let mut frames = Vec::new();
    let mut t = 0.0;
    let dt: f32 = 1.0 / 60.0; // Bake resolution

    let mut current = start;
    let mut velocity = config.velocity;

    // Safety break
    let max_duration = 10.0;

    // Epsilon for settling
    let position_epsilon = 0.1; // 0.1 pixel/unit
    let velocity_epsilon = 0.1;

    loop {
        let force = -config.stiffness * (current - end);
        let damping = -config.damping * velocity;
        let acceleration = (force + damping) / config.mass;

        velocity += acceleration * dt;
        current += velocity * dt;
        t += dt as f64; // Keep time as f64

        frames.push((current, t));

        if t > max_duration as f64 {
            break;
        }

        let is_settled =
            (current - end).abs() < position_epsilon && velocity.abs() < velocity_epsilon;
        if is_settled {
            // Add one final frame exactly at target to ensure we land
            frames.push((end, t + dt as f64));
            break;
        }
    }
    frames
}

impl<T> fmt::Debug for Animated<T>
where
    T: Clone + keyframe::CanTween + Default + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Animated")
            .field("current_value", &self.current_value)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_endpoints() {
        // All easings should be 0 at x=0 and 1 at x=1
        let easings = [
            EasingType::Linear,
            EasingType::EaseIn,
            EasingType::EaseOut,
            EasingType::EaseInOut,
            EasingType::BounceOut,
            EasingType::BounceIn,
            EasingType::BounceInOut,
            EasingType::ElasticOut,
            EasingType::ElasticIn,
            EasingType::ElasticInOut,
            EasingType::BackOut,
            EasingType::BackIn,
            EasingType::BackInOut,
        ];

        for easing in easings {
            let y0 = easing.y(0.0);
            let y1 = easing.y(1.0);
            assert!(
                (y0 - 0.0).abs() < 0.001,
                "{:?} at x=0 should be 0, got {}",
                easing,
                y0
            );
            assert!(
                (y1 - 1.0).abs() < 0.001,
                "{:?} at x=1 should be 1, got {}",
                easing,
                y1
            );
        }
    }

    #[test]
    fn test_bounce_out_has_bounces() {
        // BounceOut should have characteristic parabolic shape
        // Values should increase overall but have the bouncing pattern
        let y_mid = bounce_out(0.5);
        assert!(
            y_mid > 0.7 && y_mid < 1.0,
            "BounceOut at 0.5 should be high, got {}",
            y_mid
        );
    }

    #[test]
    fn test_elastic_out_overshoots() {
        // ElasticOut should overshoot past 1.0 at some point during the animation
        // The overshoot happens early due to the decaying oscillation (2^(-10x) * sin(...))
        // At x=0.05: 2^(-0.5) ≈ 0.707, combined with sin can exceed 1
        let mut found_overshoot = false;
        for i in 1..20 {
            let x = i as f64 / 100.0; // Check at x = 0.01, 0.02, ... 0.19
            let y = elastic_out(x);
            if y > 1.0 {
                found_overshoot = true;
                break;
            }
        }
        assert!(
            found_overshoot,
            "ElasticOut should overshoot past 1.0 at some point"
        );
    }

    #[test]
    fn test_back_out_overshoots() {
        // BackOut should momentarily exceed 1.0
        let y_mid = back_out(0.8);
        assert!(
            y_mid > 1.0,
            "BackOut at 0.8 should overshoot, got {}",
            y_mid
        );
    }

    #[test]
    fn test_easing_monotonic_behavior() {
        // Linear should be strictly monotonic
        for i in 0..10 {
            let x1 = i as f64 / 10.0;
            let x2 = (i + 1) as f64 / 10.0;
            assert!(
                EasingType::Linear.y(x2) > EasingType::Linear.y(x1),
                "Linear should be monotonic"
            );
        }
    }
}
