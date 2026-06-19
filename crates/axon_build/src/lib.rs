// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_build — Sovereign build system.
// Internal orchestration layer. Not ARPi-exposed.
// External exposure via ARPi boundary defined at AWP layer.

pub mod error;
pub mod graph;
pub mod hasher;
pub mod manifest;
pub mod runner;
pub mod scanner;

pub use error::{BuildError, BuildResult};
pub use graph::{graph_resolve, graph_topo_sort, DepGraph};
pub use hasher::{hash_bytes, BuildCache};
pub use manifest::{manifest_parse, parse_content, AxDep, AxManifest, AxProfile, Backend};
pub use runner::{plan_build, run_build, BuildPlan, BuildUnit};
pub use scanner::{scan_sources, SourceFile};
