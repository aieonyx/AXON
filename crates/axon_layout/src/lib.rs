// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_layout -- PRISM sovereign layout engine.
// P60.0: box model, coordinate primitives, text flow, flex layout.
// P60.1: font-aware text, grid, GPU-accelerated rendering.
pub mod box_model;
pub mod error;
pub mod layout;
pub mod rect;
pub mod text;
pub use box_model::{BoxModel, EdgeSizes};
pub use error::{LayoutError, LayoutResult};
pub use layout::{LayoutNode, LayoutStyle, ComputedLayout, Direction, Align, compute_layout, find_node};
pub use rect::{Point, Rect, Size};
pub use text::{TextStyle, TextMetrics, measure_text, break_lines, text_fits_in};
