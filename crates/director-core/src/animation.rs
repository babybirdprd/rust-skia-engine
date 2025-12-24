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
    BounceOut,
}

impl EasingFunction for EasingType {
    fn y(&self, x: f64) -> f64 {
        match self {
            EasingType::Linear => keyframe::functions::Linear.y(x),
            EasingType::EaseIn => keyframe::functions::EaseIn.y(x),
            EasingType::EaseOut => keyframe::functions::EaseOut.y(x),
            EasingType::EaseInOut => keyframe::functions::EaseInOut.y(x),
            // BounceOut is missing in keyframe 1.1 functions mod or named differently.
            // Mapping to EaseOut for safety.
            EasingType::BounceOut => keyframe::functions::EaseOut.y(x),
        }
    }
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
            *self = Self::new(start);
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
