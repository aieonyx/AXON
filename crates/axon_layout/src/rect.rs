// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Coordinate primitives -- Point, Size, Rect.
// Clean-room: coordinate geometry from first principles.
use crate::error::{LayoutError, LayoutResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self { Point { x, y } }
    pub fn zero() -> Self { Point { x: 0.0, y: 0.0 } }
    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Point { x: self.x + dx, y: self.y + dy }
    }
    pub fn distance_to(&self, other: &Point) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width:  f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> LayoutResult<Self> {
        if width < 0.0 || height < 0.0 {
            return Err(LayoutError::InvalidSize { width, height });
        }
        Ok(Size { width, height })
    }
    pub fn zero() -> Self { Size { width: 0.0, height: 0.0 } }
    pub fn area(&self) -> f32 { self.width * self.height }
    pub fn is_empty(&self) -> bool { self.width == 0.0 || self.height == 0.0 }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size:   Size,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> LayoutResult<Self> {
        Ok(Rect {
            origin: Point::new(x, y),
            size:   Size::new(width, height)?,
        })
    }
    pub fn zero() -> Self {
        Rect { origin: Point::zero(), size: Size::zero() }
    }
    pub fn from_points(tl: Point, br: Point) -> LayoutResult<Self> {
        if br.x < tl.x || br.y < tl.y { return Err(LayoutError::InvalidRect); }
        Self::new(tl.x, tl.y, br.x - tl.x, br.y - tl.y)
    }
    pub fn min_x(&self) -> f32 { self.origin.x }
    pub fn min_y(&self) -> f32 { self.origin.y }
    pub fn max_x(&self) -> f32 { self.origin.x + self.size.width }
    pub fn max_y(&self) -> f32 { self.origin.y + self.size.height }
    pub fn center(&self) -> Point {
        Point::new(
            self.origin.x + self.size.width  / 2.0,
            self.origin.y + self.size.height / 2.0,
        )
    }
    pub fn contains(&self, p: &Point) -> bool {
        p.x >= self.min_x() && p.x <= self.max_x() &&
        p.y >= self.min_y() && p.y <= self.max_y()
    }
    pub fn intersects(&self, other: &Rect) -> bool {
        self.min_x() < other.max_x() && self.max_x() > other.min_x() &&
        self.min_y() < other.max_y() && self.max_y() > other.min_y()
    }
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x1 = self.min_x().max(other.min_x());
        let y1 = self.min_y().max(other.min_y());
        let x2 = self.max_x().min(other.max_x());
        let y2 = self.max_y().min(other.max_y());
        if x2 > x1 && y2 > y1 {
            Rect::new(x1, y1, x2-x1, y2-y1).ok()
        } else { None }
    }
    pub fn inset(&self, dx: f32, dy: f32) -> LayoutResult<Rect> {
        Rect::new(
            self.origin.x + dx,
            self.origin.y + dy,
            self.size.width  - dx * 2.0,
            self.size.height - dy * 2.0,
        )
    }
    pub fn translate(&self, dx: f32, dy: f32) -> Rect {
        Rect { origin: self.origin.offset(dx, dy), size: self.size }
    }
}
