use lottie_core::{LottiePlayer, NodeContent};
use lottie_data::model as data;

// Helper to create a basic layer with a stroke
fn create_stroke_layer(stroke: data::Shape) -> data::Layer {
    // Need a Rect to give context for stroke application
    let rect = data::Shape::Rect(data::RectShape {
        nm: None,
        s: data::Property {
            k: data::Value::Static([100.0, 100.0]),
            ..Default::default()
        },
        p: data::Property::default(),
        r: data::Property::default(),
    });

    data::Layer {
        ty: 4, // Shape Layer
        ind: Some(1),
        parent: None,
        nm: Some("Stroke Layer".to_string()),
        ip: 0.0,
        op: 60.0,
        st: 0.0,
        ks: data::Transform::default(),
        ao: None,
        tm: None,
        ddd: None,
        pe: None,
        masks_properties: None,
        tt: None,
        ef: None,
        sy: None,
        ref_id: None,
        w: None,
        h: None,
        color: None,
        sw: None,
        sh: None,
        shapes: Some(vec![rect, stroke]),
        t: None,
    }
}

// Helper to render and extract stroke dash
fn extract_stroke_dash(layer: data::Layer) -> Option<lottie_core::DashPattern> {
    let model = data::LottieJson {
        v: None,
        ip: 0.0,
        op: 60.0,
        fr: 60.0,
        w: 100,
        h: 100,
        layers: vec![layer],
        assets: vec![],
    };

    let mut player = LottiePlayer::new();
    player.load(model);

    let tree = player.render_tree();
    let root = tree.root;

    if let NodeContent::Group(layers) = root.content {
        let layer_node = &layers[0];
        if let NodeContent::Group(shapes) = &layer_node.content {
            let shape_node = &shapes[0];
            if let NodeContent::Shape(shape) = &shape_node.content {
                if let Some(stroke) = &shape.stroke {
                    return stroke.dash.clone();
                }
            }
        }
    }
    None
}

#[test]
fn test_dash_v_support() {
    let dash_prop = data::DashProperty {
        n: Some("v".to_string()),
        v: data::Property {
            k: data::Value::Static(10.0),
            ..Default::default()
        },
    };

    let stroke = data::Shape::Stroke(data::StrokeShape {
        nm: None,
        c: data::Property::default(),
        w: data::Property::default(),
        o: data::Property::default(),
        lc: 0,
        lj: 0,
        ml: None,
        d: vec![dash_prop],
    });

    let layer = create_stroke_layer(stroke);
    let dash = extract_stroke_dash(layer).expect("Dash should be present");

    // Expect duplication: [10, 10]
    assert_eq!(dash.array.len(), 2);
    assert_eq!(dash.array[0], 10.0);
    assert_eq!(dash.array[1], 10.0);
}

#[test]
fn test_dash_gap_support() {
    let d1 = data::DashProperty {
        n: Some("v".to_string()),
        v: data::Property {
            k: data::Value::Static(10.0),
            ..Default::default()
        },
    };
    let g1 = data::DashProperty {
        n: Some("g".to_string()),
        v: data::Property {
            k: data::Value::Static(5.0),
            ..Default::default()
        },
    };

    let stroke = data::Shape::Stroke(data::StrokeShape {
        nm: None,
        c: data::Property::default(),
        w: data::Property::default(),
        o: data::Property::default(),
        lc: 0,
        lj: 0,
        ml: None,
        d: vec![d1, g1],
    });

    let layer = create_stroke_layer(stroke);
    let dash = extract_stroke_dash(layer).expect("Dash should be present");

    // Expect no duplication: [10, 5]
    assert_eq!(dash.array.len(), 2);
    assert_eq!(dash.array[0], 10.0);
    assert_eq!(dash.array[1], 5.0);
}

#[test]
fn test_offset_normalization_positive_huge() {
    let d1 = data::DashProperty {
        n: Some("v".to_string()),
        v: data::Property {
            k: data::Value::Static(10.0), // Dash 10
            ..Default::default()
        },
    };
    // Implies [10, 10]. Total length 20.

    // Huge offset
    let offset_prop = data::DashProperty {
        n: Some("o".to_string()),
        v: data::Property {
            k: data::Value::Static(2025.0),
            ..Default::default()
        },
    };

    let stroke = data::Shape::Stroke(data::StrokeShape {
        nm: None,
        c: data::Property::default(),
        w: data::Property::default(),
        o: data::Property::default(),
        lc: 0,
        lj: 0,
        ml: None,
        d: vec![d1, offset_prop],
    });

    let layer = create_stroke_layer(stroke);
    let dash = extract_stroke_dash(layer).expect("Dash should be present");

    // Total length = 20.
    // 2025 % 20 = 5.
    assert!((dash.offset - 5.0).abs() < 0.001, "Expected offset 5.0, got {}", dash.offset);
}

#[test]
fn test_offset_normalization_negative() {
    let d1 = data::DashProperty {
        n: Some("v".to_string()),
        v: data::Property {
            k: data::Value::Static(10.0), // Dash 10
            ..Default::default()
        },
    };
    // Implies [10, 10]. Total length 20.

    // Negative offset
    let offset_prop = data::DashProperty {
        n: Some("o".to_string()),
        v: data::Property {
            k: data::Value::Static(-5.0),
            ..Default::default()
        },
    };

    let stroke = data::Shape::Stroke(data::StrokeShape {
        nm: None,
        c: data::Property::default(),
        w: data::Property::default(),
        o: data::Property::default(),
        lc: 0,
        lj: 0,
        ml: None,
        d: vec![d1, offset_prop],
    });

    let layer = create_stroke_layer(stroke);
    let dash = extract_stroke_dash(layer).expect("Dash should be present");

    // Total length = 20.
    // -5 % 20 -> -5.
    // (-5 % 20 + 20) % 20 -> 15.
    assert!((dash.offset - 15.0).abs() < 0.001, "Expected offset 15.0, got {}", dash.offset);
}

#[test]
fn test_gradient_stroke_dash() {
    let d1 = data::DashProperty {
        n: Some("v".to_string()),
        v: data::Property {
            k: data::Value::Static(20.0),
            ..Default::default()
        },
    };
    let g1 = data::DashProperty {
        n: Some("g".to_string()),
        v: data::Property {
            k: data::Value::Static(10.0),
            ..Default::default()
        },
    };
    // Offset
    let o1 = data::DashProperty {
        n: Some("o".to_string()),
        v: data::Property {
            k: data::Value::Static(35.0),
            ..Default::default()
        },
    };

    let stroke = data::Shape::GradientStroke(data::GradientStrokeShape {
        nm: None,
        o: data::Property::default(),
        w: data::Property::default(),
        s: data::Property::default(),
        e: data::Property::default(),
        t: 1,
        g: data::GradientColors::default(),
        lc: 0,
        lj: 0,
        ml: None,
        d: vec![d1, g1, o1],
    });

    let layer = create_stroke_layer(stroke);
    let dash = extract_stroke_dash(layer).expect("Dash should be present");

    // [20, 10]. Total 30. Offset 35 -> 5.
    assert_eq!(dash.array.len(), 2);
    assert_eq!(dash.array[0], 20.0);
    assert_eq!(dash.array[1], 10.0);
    assert!((dash.offset - 5.0).abs() < 0.001);
}
