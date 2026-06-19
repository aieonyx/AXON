// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Incremental build cache — hash-based change detection.
// Uses FNV-64 as a lightweight hash at P47.
// Full Blake3 sovereign hash arrives at P57 (axon_crypto).

use axon_std_string::AxString;
use crate::error::{BuildError, BuildResult};
use crate::scanner::SourceFile;

/// FNV-64 hash — fast, no-dep, sufficient for build cache at P47.
pub fn hash_bytes(data: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME:  u64 = 1099511628211;
    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// In-memory build cache mapping file path → last known hash.
#[derive(Debug, Default)]
pub struct BuildCache {
    entries: Vec<(AxString, u64)>,
}

impl BuildCache {
    pub fn new() -> Self {
        BuildCache { entries: Vec::new() }
    }

    /// Returns true if the file has changed since last cache entry.
    pub fn check(&self, file: &SourceFile) -> bool {
        match self.entries.iter().find(|(p, _)| p == &file.path) {
            Some((_, cached_hash)) => *cached_hash != file.hash,
            None => true, // not in cache = new file = changed
        }
    }

    /// Update cache entry for a file.
    pub fn update(&mut self, file: &SourceFile) {
        match self.entries.iter_mut().find(|(p, _)| p == &file.path) {
            Some((_, h)) => *h = file.hash,
            None => self.entries.push((file.path.clone(), file.hash)),
        }
    }

    /// Persist cache to disk as a simple key=value file.
    pub fn save(&self, target_dir: &str) -> BuildResult<()> {
        let cache_path = format!("{}/build.cache", target_dir);
        std::fs::create_dir_all(target_dir).map_err(|e| {
            BuildError::CacheError(AxString::ax_from_str(&e.to_string()))
        })?;
        let mut content = String::new();
        for (path, hash) in &self.entries {
            content.push_str(&format!("{}={}
", path.as_str(), hash));
        }
        std::fs::write(&cache_path, content).map_err(|e| {
            BuildError::CacheError(AxString::ax_from_str(&e.to_string()))
        })
    }

    /// Load cache from disk.
    pub fn load(target_dir: &str) -> BuildResult<Self> {
        let cache_path = format!("{}/build.cache", target_dir);
        let mut cache = BuildCache::new();
        let content = match std::fs::read_to_string(&cache_path) {
            Ok(c) => c,
            Err(_) => return Ok(cache), // no cache yet = fresh build
        };
        for line in content.lines() {
            if let Some(eq) = line.rfind('=') {
                let path = &line[..eq];
                let hash_str = &line[eq + 1..];
                if let Ok(hash) = hash_str.parse::<u64>() {
                    cache.entries.push((
                        AxString::ax_from_str(path),
                        hash,
                    ));
                }
            }
        }
        Ok(cache)
    }
}
