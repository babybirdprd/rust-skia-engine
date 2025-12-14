//! Layout Engine Tests
//!
//! Tests for Taffy-based layout computation and scene graph structure.

use director_core::{
    director::TimelineItem, node::BoxNode, scene::SceneGraph, types::NodeId,
    video_wrapper::RenderMode, DefaultAssetLoader, Director,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use taffy::style::Dimension;

/// Dump scene tree to string for snapshot testing.
fn dump_scene_tree(director: &Director) -> String {
    let mut output = String::new();
    if let Some(item) = director.timeline.first() {
        recursive_dump(&director.scene, item.scene_root, 0, &mut output);
    }
    output
}

fn recursive_dump(graph: &SceneGraph, node_id: NodeId, depth: usize, output: &mut String) {
    if let Some(node) = graph.get_node(node_id) {
        let indent = "  ".repeat(depth);
        let rect = format!(
            "Rect(x:{:.1}, y:{:.1}, w:{:.1}, h:{:.1})",
            node.layout_rect.left,
            node.layout_rect.top,
            node.layout_rect.width(),
            node.layout_rect.height()
        );
        output.push_str(&format!(
            "{}Node[{}] z:{} {}\n",
            indent, node_id, node.z_index, rect
        ));
        for &child_id in &node.children {
            recursive_dump(graph, child_id, depth + 1, output);
        }
    }
}

/// Test layout structure stability via snapshot.
///
/// This test builds a complex scene and compares the computed layout
/// against a stored snapshot to detect regressions.
#[test]
fn layout_structure_snapshot() {
    let director = Director::new(
        1000,
        1000,
        30,
        Arc::new(DefaultAssetLoader),
        RenderMode::Preview,
        None,
    );
    let director_arc = Arc::new(Mutex::new(director));

    // Build scene
    {
        let mut d = director_arc.lock().unwrap();

        // Root container (1000x1000, centered column)
        let mut root = BoxNode::new();
        root.style.size = taffy::geometry::Size {
            width: Dimension::length(1000.0),
            height: Dimension::length(1000.0),
        };
        root.style.display = taffy::style::Display::Flex;
        root.style.flex_direction = taffy::style::FlexDirection::Column;
        root.style.justify_content = Some(taffy::style::JustifyContent::Center);
        root.style.align_items = Some(taffy::style::AlignItems::Center);
        let root_id = d.scene.add_node(Box::new(root));

        d.timeline.push(TimelineItem {
            scene_root: root_id,
            start_time: 0.0,
            duration: 10.0,
            z_index: 0,
            audio_tracks: vec![],
        });

        // Row with space-between children
        let mut row = BoxNode::new();
        row.style.size.width = Dimension::percent(1.0);
        row.style.size.height = Dimension::length(200.0);
        row.style.flex_direction = taffy::style::FlexDirection::Row;
        row.style.justify_content = Some(taffy::style::JustifyContent::SpaceBetween);
        let row_id = d.scene.add_node(Box::new(row));
        d.scene.add_child(root_id, row_id);

        // Two children in the row
        for _ in 0..2 {
            let mut child = BoxNode::new();
            child.style.size.width = Dimension::length(50.0);
            child.style.size.height = Dimension::length(50.0);
            let child_id = d.scene.add_node(Box::new(child));
            d.scene.add_child(row_id, child_id);
        }

        // Absolute positioned element
        let mut abs = BoxNode::new();
        abs.style.position = taffy::style::Position::Absolute;
        abs.style.size.width = Dimension::length(100.0);
        abs.style.size.height = Dimension::length(100.0);
        abs.style.inset.bottom = taffy::style::LengthPercentageAuto::length(10.0);
        abs.style.inset.right = taffy::style::LengthPercentageAuto::length(10.0);
        let abs_id = d.scene.add_node(Box::new(abs));
        d.scene.add_child(root_id, abs_id);

        // Z-index test element
        let mut z_elem = BoxNode::new();
        z_elem.style.size.width = Dimension::length(100.0);
        z_elem.style.size.height = Dimension::length(100.0);
        let z_id = d.scene.add_node(Box::new(z_elem));
        if let Some(node) = d.scene.get_node_mut(z_id) {
            node.z_index = 10;
        }
        d.scene.add_child(root_id, z_id);
    }

    // Compute layout
    {
        let mut d = director_arc.lock().unwrap();
        let mut layout_engine = director_core::systems::layout::LayoutEngine::new();
        d.update(0.0);
        let width = d.width;
        let height = d.height;
        layout_engine.compute_layout(&mut d.scene, width, height, 0.0);
        d.run_post_layout(0.0);
    }

    // Generate dump
    let d = director_arc.lock().unwrap();
    let actual = dump_scene_tree(&d);

    // Snapshot handling
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let snapshot_path = PathBuf::from(&manifest_dir).join("tests/snapshots/layout_structure.txt");

    if env::var("UPDATE_SNAPSHOTS").is_ok() {
        if let Some(parent) = snapshot_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&snapshot_path, &actual).expect("Failed to write snapshot");
        println!("Updated snapshot: {:?}", snapshot_path);
        return;
    }

    if !snapshot_path.exists() {
        panic!(
            "Snapshot not found: {:?}. Run with UPDATE_SNAPSHOTS=1 to generate.",
            snapshot_path
        );
    }

    let expected = fs::read_to_string(&snapshot_path).expect("Failed to read snapshot");
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Layout structure mismatch!\nActual:\n{}\nExpected:\n{}",
        actual,
        expected
    );
}
