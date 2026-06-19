// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use crate::error::IoResult;

pub fn stdin_read_line() -> IoResult<String> { crate::backend::active::stdin_read_line() }
pub fn stdout_write(buf: &[u8]) -> IoResult<()> { crate::backend::active::stdout_write(buf) }
pub fn stderr_write(buf: &[u8]) -> IoResult<()> { crate::backend::active::stderr_write(buf) }
pub fn stdout_flush() -> IoResult<()> { crate::backend::active::stdout_flush() }
