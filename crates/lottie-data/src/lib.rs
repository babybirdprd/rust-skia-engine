// lottie-data: Serde structs for Lottie JSON format
pub mod model;

#[cfg(test)]
mod tests {
    use super::model::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_minimal() {
        let data = json!({
            "v": "5.5.2",
            "ip": 0,
            "op": 60,
            "fr": 60,
            "w": 500,
            "h": 500,
            "layers": []
        });
        let lottie: LottieJson = serde_json::from_value(data).unwrap();
        assert_eq!(lottie.w, 500);
    }

    #[test]
    fn test_deserialize_shape_layer() {
        let data = json!({
            "v": "5.5.2",
            "ip": 0, "op": 60, "fr": 60, "w": 100, "h": 100,
            "layers": [
                {
                    "ty": 4, // Shape Layer
                    "ind": 1,
                    "nm": "MyShape",
                    "ip": 0, "op": 60, "st": 0,
                    "ks": {}, // Transform
                    "shapes": [
                        {
                            "ty": "rc",
                            "nm": "Rect",
                            "s": { "a": 0, "k": [100, 100] },
                            "p": { "a": 0, "k": [50, 50] },
                            "r": { "a": 0, "k": 0 }
                        },
                        {
                            "ty": "fl",
                            "c": { "a": 0, "k": [1, 0, 0, 1] },
                            "o": { "a": 0, "k": 100 }
                        }
                    ]
                }
            ]
        });
        let lottie: LottieJson = serde_json::from_value(data).unwrap();
        let layer = &lottie.layers[0];
        assert_eq!(layer.ty, 4);
        if let Some(shapes) = &layer.shapes {
            assert_eq!(shapes.len(), 2);
            if let Shape::Rect(rect) = &shapes[0] {
                assert_eq!(rect.nm.as_deref(), Some("Rect"));
            } else {
                panic!("Expected Rect, got {:?}", shapes[0]);
            }
        } else {
            panic!("Expected shapes");
        }
    }
}
