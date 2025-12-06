#[cfg(test)]
mod tests {
    use glam::Vec2;
    use lottie_core::animatable::Animator;
    use lottie_data::model::{Keyframe, Property, Value};

    #[test]
    fn test_spatial_bezier_interpolation() {
        // P0 = (0, 0)
        // P3 = (100, 100)
        // Tangent Out (from P0) = (50, 0) -> P1 = (50, 0)
        // Tangent In (to P3) = (0, -50)   -> P2 = (100, 50)

        // At t = 0.5:
        // (1-t) = 0.5
        // (1-t)^3 = 0.125
        // 3(1-t)^2*t = 3 * 0.25 * 0.5 = 0.375
        // 3(1-t)*t^2 = 3 * 0.5 * 0.25 = 0.375
        // t^3 = 0.125

        // x = 0.125*0 + 0.375*50 + 0.375*100 + 0.125*100
        // x = 0 + 18.75 + 37.5 + 12.5 = 68.75

        // y = 0.125*0 + 0.375*0 + 0.375*50 + 0.125*100
        // y = 0 + 0 + 18.75 + 12.5 = 31.25

        let kf1 = Keyframe {
            t: 0.0,
            s: Some([0.0, 0.0]),
            e: Some([100.0, 100.0]),
            i: None, // Time easing
            o: None, // Time easing
            to: Some(vec![50.0, 0.0]), // Spatial tangent out
            ti: None,
            h: None,
        };

        let kf2 = Keyframe {
            t: 10.0,
            s: Some([100.0, 100.0]),
            e: None,
            i: None,
            o: None,
            to: None,
            ti: Some(vec![0.0, -50.0]), // Spatial tangent in
            h: None,
        };

        let prop = Property {
            a: 1,
            k: Value::Animated(vec![kf1, kf2]),
            ix: None,
            x: None,
        };

        // Frame 5.0 is exactly 50% between 0.0 and 10.0
        #[cfg(feature = "expressions")]
        let evaluator = None;
        #[cfg(not(feature = "expressions"))]
        let evaluator = None;

        let result = Animator::resolve(&prop, 5.0, |v| Vec2::from_slice(v), Vec2::ZERO, evaluator, 60.0);

        // Tolerance due to floating point
        assert!((result.x - 68.75).abs() < 0.001, "X should be ~68.75, got {}", result.x);
        assert!((result.y - 31.25).abs() < 0.001, "Y should be ~31.25, got {}", result.y);
    }
}
