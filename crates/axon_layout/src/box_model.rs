// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Box model -- margin, border, padding, content.
// Clean-room: studied W3C CSS Box Model Level 3 spec only. No code copied.
use crate::rect::{Rect, Size};
use crate::error::LayoutResult;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeSizes {
    pub top:    f32,
    pub right:  f32,
    pub bottom: f32,
    pub left:   f32,
}

impl EdgeSizes {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        EdgeSizes { top, right, bottom, left }
    }
    pub fn zero() -> Self { EdgeSizes { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 } }
    pub fn uniform(v: f32) -> Self { EdgeSizes { top: v, right: v, bottom: v, left: v } }
    pub fn horizontal(&self) -> f32 { self.left + self.right }
    pub fn vertical(&self) -> f32   { self.top + self.bottom }
}

#[derive(Debug, Clone)]
pub struct BoxModel {
    pub margin:  EdgeSizes,
    pub border:  EdgeSizes,
    pub padding: EdgeSizes,
    pub content: Size,
}

impl BoxModel {
    pub fn new(margin: EdgeSizes, border: EdgeSizes, padding: EdgeSizes, content: Size) -> Self {
        BoxModel { margin, border, padding, content }
    }

    pub fn default_with_content(width: f32, height: f32) -> LayoutResult<Self> {
        Ok(BoxModel {
            margin:  EdgeSizes::zero(),
            border:  EdgeSizes::zero(),
            padding: EdgeSizes::zero(),
            content: Size::new(width, height)?,
        })
    }

    pub fn padding_box_size(&self) -> Size {
        Size {
            width:  self.content.width  + self.padding.horizontal(),
            height: self.content.height + self.padding.vertical(),
        }
    }

    pub fn border_box_size(&self) -> Size {
        let pb = self.padding_box_size();
        Size {
            width:  pb.width  + self.border.horizontal(),
            height: pb.height + self.border.vertical(),
        }
    }

    pub fn margin_box_size(&self) -> Size {
        let bb = self.border_box_size();
        Size {
            width:  bb.width  + self.margin.horizontal(),
            height: bb.height + self.margin.vertical(),
        }
    }

    pub fn content_rect(&self, origin_x: f32, origin_y: f32) -> LayoutResult<Rect> {
        let x = origin_x + self.margin.left + self.border.left + self.padding.left;
        let y = origin_y + self.margin.top  + self.border.top  + self.padding.top;
        Rect::new(x, y, self.content.width, self.content.height)
    }

    pub fn border_rect(&self, origin_x: f32, origin_y: f32) -> LayoutResult<Rect> {
        let x = origin_x + self.margin.left;
        let y = origin_y + self.margin.top;
        let bb = self.border_box_size();
        Rect::new(x, y, bb.width, bb.height)
    }
}
