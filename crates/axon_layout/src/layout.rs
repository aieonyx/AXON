// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Layout tree -- sovereign flex-style positioning.
// Clean-room: studied W3C CSS Flexbox spec concepts only. No code copied.
// P60.0: vertical stack and horizontal row layouts.
// P60.1: full flex wrap, grid, absolute positioning.
use crate::rect::{Point, Rect, Size};
use crate::box_model::BoxModel;
use crate::error::{LayoutError, LayoutResult};

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Row,
    Column,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone)]
pub struct LayoutStyle {
    pub direction: Direction,
    pub align:     Align,
    pub gap:       f32,
}

impl LayoutStyle {
    pub fn column() -> Self {
        LayoutStyle { direction: Direction::Column, align: Align::Start, gap: 0.0 }
    }
    pub fn row() -> Self {
        LayoutStyle { direction: Direction::Row, align: Align::Start, gap: 0.0 }
    }
    pub fn with_gap(mut self, gap: f32) -> Self { self.gap = gap; self }
    pub fn with_align(mut self, align: Align) -> Self { self.align = align; self }
}

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id:       String,
    pub model:    BoxModel,
    pub style:    LayoutStyle,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn new(id: &str, model: BoxModel, style: LayoutStyle) -> Self {
        LayoutNode { id: id.to_string(), model, style, children: vec![] }
    }

    pub fn add_child(&mut self, child: LayoutNode) {
        self.children.push(child);
    }

    pub fn child_count(&self) -> usize { self.children.len() }
}

#[derive(Debug, Clone)]
pub struct ComputedLayout {
    pub id:   String,
    pub rect: Rect,
    pub children: Vec<ComputedLayout>,
}

pub fn compute_layout(node: &LayoutNode, origin: Point, available: Size) -> LayoutResult<ComputedLayout> {
    let border_rect = node.model.border_rect(origin.x, origin.y)?;
    let content_rect = node.model.content_rect(origin.x, origin.y)?;

    if node.children.is_empty() {
        return Ok(ComputedLayout {
            id:       node.id.clone(),
            rect:     border_rect,
            children: vec![],
        });
    }

    let mut children_computed = vec![];
    let mut cursor = Point::new(content_rect.origin.x, content_rect.origin.y);

    let child_available = Size {
        width:  available.width,
        height: available.height,
    };

    for child in &node.children {
        let child_layout = compute_layout(child, cursor, child_available)?;
        let child_size = child_layout.rect.size;

        match node.style.direction {
            Direction::Column => {
                cursor.y += child_size.height + node.style.gap;
            }
            Direction::Row => {
                cursor.x += child_size.width + node.style.gap;
            }
        }
        children_computed.push(child_layout);
    }

    Ok(ComputedLayout {
        id:       node.id.clone(),
        rect:     border_rect,
        children: children_computed,
    })
}

pub fn find_node<'a>(layout: &'a ComputedLayout, id: &str) -> Option<&'a ComputedLayout> {
    if layout.id == id { return Some(layout); }
    for child in &layout.children {
        if let Some(found) = find_node(child, id) {
            return Some(found);
        }
    }
    None
}
