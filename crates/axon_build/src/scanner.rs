// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Source file scanner — discovers all .ax files in a source directory.
// Uses axon_std_io for path operations; std::fs for directory traversal
// (full sovereign traversal deferred to P49 when axon_std::fs exists).

use axon_std_string::AxString;
use crate::error::{BuildError, BuildResult};
use crate::hasher::hash_bytes;

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: AxString,
    pub hash: u64,
}

/// Scan a directory recursively for .ax source files.
pub fn scan_sources(src_dir: &str) -> BuildResult<Vec<SourceFile>> {
    let mut files: Vec<SourceFile> = Vec::new();
    scan_dir(src_dir, &mut files)?;
    // Sort for deterministic order
    files.sort_by(|a, b| a.path.as_str().cmp(b.path.as_str()));
    Ok(files)
}

fn scan_dir(dir: &str, files: &mut Vec<SourceFile>) -> BuildResult<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        BuildError::SourceScanError(AxString::ax_from_str(&format!("{}: {}", dir, e)))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            BuildError::SourceScanError(AxString::ax_from_str(&e.to_string()))
        })?;
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();

        if path.is_dir() {
            scan_dir(&path_str, files)?;
        } else if path.extension().map(|e| e == "ax").unwrap_or(false) {
            let content = std::fs::read(&path).map_err(|e| {
                BuildError::SourceScanError(AxString::ax_from_str(&e.to_string()))
            })?;
            let hash = hash_bytes(&content);
            files.push(SourceFile {
                path: AxString::ax_from_str(&path_str),
                hash,
            });
        }
    }
    Ok(())
}
