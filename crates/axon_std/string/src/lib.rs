// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_std::string — Sovereign string, Unicode, SSO, formatting.
// Internal substrate layer. Not ARPi-exposed.
// External exposure via ARPi boundary is defined at the AWP layer.

pub mod axchar;
pub mod axstr;
pub mod axstring;
pub mod fmt;

pub use axchar::AxChar;
pub use axstr::AxStr;
pub use axstring::AxString;
pub use fmt::{ax_format, ax_format_into, AxFormat};
