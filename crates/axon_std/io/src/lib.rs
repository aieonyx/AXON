// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_std::io — Sovereign I/O substrate.
// Internal layer. Not ARPi-exposed.
// External exposure via ARPi boundary is defined at the AWP layer.

pub mod backend;
pub mod error;
pub mod file;
pub mod path;
pub mod stdio;

pub use error::{IoError, IoResult};
pub use file::{close, create, flush, open, read_to_end, read_to_string, write_all};
pub use path::{path_exists, path_is_dir, path_is_file};
pub use stdio::{stderr_write, stdin_read_line, stdout_flush, stdout_write};
