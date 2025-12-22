use director_core::node::BoxNode;
use director_core::director::TimelineItem;
use taffy::style::{Dimension, AlignItems, JustifyContent, FlexDirection};
use director_core::types::Color;
use director_core::animation::Animated;

// Matrix Test: Flex Layout Alignments
crate::visual_test_matrix!(
    name: layout_alignments,
    suite: "layout",
    variations: [
        (align_start, AlignItems::FlexStart),
        (align_center, AlignItems::Center),
        (align_end, AlignItems::FlexEnd),
    ],
    setup: |d: &mut Director, align: AlignItems| {
        // Root Container
        let mut root = BoxNode::new();
        root.style.size.width = Dimension::percent(1.0);
        root.style.size.height = Dimension::percent(1.0);
        root.style.display = taffy::style::Display::Flex;
        root.style.flex_direction = FlexDirection::Column;
        root.style.align_items = Some(align); // Apply variation
        root.style.justify_content = Some(JustifyContent::Center);
        root.bg_color = Some(Animated::new(Color::new(0.9, 0.9, 0.9, 1.0)));

        let root_id = d.scene.add_node(Box::new(root));

        d.timeline.push(TimelineItem {
            scene_root: root_id,
            start_time: 0.0,
            duration: 5.0,
            z_index: 0,
            audio_tracks: vec![],
        });

        // Child Items of different sizes
        for i in 0..3 {
            let mut item = BoxNode::new();
            let size = 50.0 + (i as f32 * 20.0);
            item.style.size.width = Dimension::length(size);
            item.style.size.height = Dimension::length(50.0);
            item.bg_color = Some(Animated::new(Color::new(0.0, 0.0, 0.0, 1.0)));
            // Margin to see separation
            item.style.margin.bottom = taffy::style::LengthPercentageAuto::length(10.0);

            let id = d.scene.add_node(Box::new(item));
            d.scene.add_child(root_id, id);
        }
    }
);
