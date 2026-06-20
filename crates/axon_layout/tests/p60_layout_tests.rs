// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P60 QA -- axon_layout PRISM sovereign layout engine tests
// Pass bar: 24/24
// P3 Doctrine: complements axon_gpu P58, axon_media P59, HANIEL CANVAS
use axon_layout::{
    Point, Rect, Size, EdgeSizes, BoxModel,
    TextStyle, measure_text, break_lines, text_fits_in,
    LayoutNode, LayoutStyle, Direction, Align,
    compute_layout, find_node, LayoutError,
};

// ── Point tests ───────────────────────────────────────────────────────────────
#[test]
fn test_point_new() {
    let p = Point::new(3.0, 4.0);
    assert_eq!(p.x, 3.0);
    assert_eq!(p.y, 4.0);
}
#[test]
fn test_point_offset() {
    let p = Point::new(1.0, 2.0).offset(3.0, 4.0);
    assert_eq!(p.x, 4.0);
    assert_eq!(p.y, 6.0);
}
#[test]
fn test_point_distance() {
    let a = Point::new(0.0, 0.0);
    let b = Point::new(3.0, 4.0);
    assert!((a.distance_to(&b) - 5.0).abs() < 1e-5);
}

// ── Size tests ────────────────────────────────────────────────────────────────
#[test]
fn test_size_valid() {
    let s = Size::new(10.0, 20.0).unwrap();
    assert_eq!(s.area(), 200.0);
}
#[test]
fn test_size_negative_fails() {
    assert!(Size::new(-1.0, 10.0).is_err());
    assert!(Size::new(10.0, -1.0).is_err());
}
#[test]
fn test_size_is_empty() {
    assert!(Size::zero().is_empty());
    assert!(!Size::new(1.0, 1.0).unwrap().is_empty());
}

// ── Rect tests ────────────────────────────────────────────────────────────────
#[test]
fn test_rect_contains() {
    let r = Rect::new(0.0, 0.0, 100.0, 100.0).unwrap();
    assert!(r.contains(&Point::new(50.0, 50.0)));
    assert!(!r.contains(&Point::new(150.0, 50.0)));
}
#[test]
fn test_rect_intersects() {
    let a = Rect::new(0.0, 0.0, 10.0, 10.0).unwrap();
    let b = Rect::new(5.0, 5.0, 10.0, 10.0).unwrap();
    let c = Rect::new(20.0, 20.0, 10.0, 10.0).unwrap();
    assert!(a.intersects(&b));
    assert!(!a.intersects(&c));
}
#[test]
fn test_rect_intersection() {
    let a = Rect::new(0.0, 0.0, 10.0, 10.0).unwrap();
    let b = Rect::new(5.0, 5.0, 10.0, 10.0).unwrap();
    let i = a.intersection(&b).unwrap();
    assert!((i.size.width  - 5.0).abs() < 1e-5);
    assert!((i.size.height - 5.0).abs() < 1e-5);
}
#[test]
fn test_rect_center() {
    let r = Rect::new(0.0, 0.0, 10.0, 10.0).unwrap();
    let c = r.center();
    assert_eq!(c.x, 5.0);
    assert_eq!(c.y, 5.0);
}
#[test]
fn test_rect_translate() {
    let r = Rect::new(0.0, 0.0, 10.0, 10.0).unwrap();
    let t = r.translate(5.0, 3.0);
    assert_eq!(t.origin.x, 5.0);
    assert_eq!(t.origin.y, 3.0);
}

// ── BoxModel tests ────────────────────────────────────────────────────────────
#[test]
fn test_box_model_padding_box() {
    let m   = BoxModel::new(
        EdgeSizes::zero(),
        EdgeSizes::zero(),
        EdgeSizes::uniform(10.0),
        Size::new(100.0, 50.0).unwrap(),
    );
    let pb = m.padding_box_size();
    assert_eq!(pb.width,  120.0);
    assert_eq!(pb.height, 70.0);
}
#[test]
fn test_box_model_border_box() {
    let m = BoxModel::new(
        EdgeSizes::zero(),
        EdgeSizes::uniform(2.0),
        EdgeSizes::uniform(10.0),
        Size::new(100.0, 50.0).unwrap(),
    );
    let bb = m.border_box_size();
    assert_eq!(bb.width,  124.0);
    assert_eq!(bb.height, 74.0);
}
#[test]
fn test_box_model_content_rect() {
    let m = BoxModel::default_with_content(100.0, 50.0).unwrap();
    let r = m.content_rect(10.0, 20.0).unwrap();
    assert_eq!(r.origin.x, 10.0);
    assert_eq!(r.origin.y, 20.0);
}

// ── Text tests ────────────────────────────────────────────────────────────────
#[test]
fn test_measure_text_empty() {
    let m = measure_text("", &TextStyle::default(), 200.0).unwrap();
    assert_eq!(m.lines, 0);
    assert_eq!(m.chars, 0);
}
#[test]
fn test_measure_text_single_line() {
    let m = measure_text("hello", &TextStyle::default(), 200.0).unwrap();
    assert_eq!(m.lines, 1);
    assert_eq!(m.chars, 5);
}
#[test]
fn test_break_lines_wraps() {
    let style = TextStyle::new(10.0);
    let text  = "one two three four five six seven eight nine ten";
    let lines = break_lines(text, &style, 50.0);
    assert!(lines > 1);
}
#[test]
fn test_text_fits_in() {
    let style = TextStyle::default();
    let size  = Size::new(200.0, 100.0).unwrap();
    assert!(text_fits_in("hello", &style, &size));
}
#[test]
fn test_text_too_long_error() {
    let long = "x".repeat(70_000);
    assert!(measure_text(&long, &TextStyle::default(), 200.0).is_err());
}

// ── Layout tree tests ─────────────────────────────────────────────────────────
#[test]
fn test_layout_single_node() {
    let model = BoxModel::default_with_content(100.0, 50.0).unwrap();
    let node  = LayoutNode::new("root", model, LayoutStyle::column());
    let size  = Size::new(800.0, 600.0).unwrap();
    let computed = compute_layout(&node, Point::zero(), size).unwrap();
    assert_eq!(computed.id, "root");
    assert_eq!(computed.rect.size.width,  100.0);
    assert_eq!(computed.rect.size.height, 50.0);
}
#[test]
fn test_layout_column_children() {
    let root_model = BoxModel::default_with_content(200.0, 200.0).unwrap();
    let mut root   = LayoutNode::new("root", root_model, LayoutStyle::column().with_gap(10.0));
    let c1 = LayoutNode::new("c1", BoxModel::default_with_content(100.0, 30.0).unwrap(), LayoutStyle::column());
    let c2 = LayoutNode::new("c2", BoxModel::default_with_content(100.0, 30.0).unwrap(), LayoutStyle::column());
    root.add_child(c1);
    root.add_child(c2);
    let size = Size::new(800.0, 600.0).unwrap();
    let computed = compute_layout(&root, Point::zero(), size).unwrap();
    assert_eq!(computed.children.len(), 2);
    let y1 = computed.children[0].rect.origin.y;
    let y2 = computed.children[1].rect.origin.y;
    assert!(y2 > y1);
}
#[test]
fn test_layout_find_node() {
    let root_model = BoxModel::default_with_content(200.0, 200.0).unwrap();
    let mut root   = LayoutNode::new("root", root_model, LayoutStyle::column());
    root.add_child(LayoutNode::new("child", BoxModel::default_with_content(50.0, 50.0).unwrap(), LayoutStyle::column()));
    let size     = Size::new(800.0, 600.0).unwrap();
    let computed = compute_layout(&root, Point::zero(), size).unwrap();
    assert!(find_node(&computed, "child").is_some());
    assert!(find_node(&computed, "missing").is_none());
}
