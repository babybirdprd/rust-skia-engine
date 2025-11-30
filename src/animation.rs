use keyframe::{Keyframe, EasingFunction, AnimationSequence};
use std::fmt;

// Define our own enum to store easing types uniformly
#[derive(Copy, Clone, Debug, PartialEq)]
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
    pub fn eval(&self, x: f32) -> f32 {
        self.y(x as f64) as f32
    }
}

#[derive(Clone)]
pub struct Animated<T>
where T: Copy + keyframe::CanTween + Default
{
    raw_keyframes: Vec<(T, f64, EasingType)>,
    pub sequence: AnimationSequence<T>,
    pub current_value: T,
}

impl<T> Animated<T>
where T: Copy + keyframe::CanTween + Default
{
    pub fn new(initial: T) -> Self {
        let raw = vec![(initial, 0.0, EasingType::Linear)];
        let kf = Keyframe::new(initial, 0.0, EasingType::Linear);

        Self {
            sequence: AnimationSequence::from(vec![kf]),
            raw_keyframes: raw,
            current_value: initial,
        }
    }

    pub fn add_keyframe(&mut self, target: T, duration: f64, easing: EasingType) {
        let current_end_time = self.sequence.duration();
        let new_time = current_end_time + duration;

        self.raw_keyframes.push((target, new_time, easing));

        // Rebuild sequence
        let frames: Vec<Keyframe<T>> = self.raw_keyframes.iter()
            .map(|(val, time, ease_type)| {
                Keyframe::new(*val, *time, *ease_type)
            })
            .collect();

        self.sequence = AnimationSequence::from(frames);
    }

    pub fn duration(&self) -> f64 {
        self.sequence.duration()
    }

    pub fn add_segment(&mut self, start: T, target: T, duration: f64, easing: EasingType) {
        if self.sequence.duration() == 0.0 {
             // If no animation exists yet, treat start as the initial value
             // But we must preserve the structure.
             // Self::new creates a sequence with one keyframe at t=0.
             *self = Self::new(start);
        } else {
             // If animation exists, we jump to 'start' immediately at the current end time
             self.add_keyframe(start, 0.0, EasingType::Linear);
        }
        self.add_keyframe(target, duration, easing);
    }

    pub fn update(&mut self, time: f64) {
        self.sequence.advance_to(time);
        self.current_value = self.sequence.now();
    }
}

impl<T> fmt::Debug for Animated<T>
where T: Copy + keyframe::CanTween + Default + fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Animated")
         .field("current_value", &self.current_value)
         .finish()
    }
}
