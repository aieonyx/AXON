// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use axon_std_string::AxString;

#[derive(Debug)]
pub enum BuildError {
    ManifestNotFound,
    ManifestParseError(AxString),
    CycleDetected,
    SourceScanError(AxString),
    CacheError(AxString),
    RunnerError(AxString),
}

impl core::fmt::Display for BuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BuildError::ManifestNotFound          => write!(f, "build: manifest not found"),
            BuildError::ManifestParseError(m)     => write!(f, "build: manifest parse error: {}", m.as_str()),
            BuildError::CycleDetected             => write!(f, "build: cycle detected in dependency graph"),
            BuildError::SourceScanError(m)        => write!(f, "build: source scan error: {}", m.as_str()),
            BuildError::CacheError(m)             => write!(f, "build: cache error: {}", m.as_str()),
            BuildError::RunnerError(m)            => write!(f, "build: runner error: {}", m.as_str()),
        }
    }
}

pub type BuildResult<T> = Result<T, BuildError>;
