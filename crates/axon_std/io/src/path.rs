// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

pub fn path_exists(path: &str) -> bool { crate::backend::active::path_exists(path) }
pub fn path_is_file(path: &str) -> bool { crate::backend::active::path_is_file(path) }
pub fn path_is_dir(path: &str) -> bool { crate::backend::active::path_is_dir(path) }
