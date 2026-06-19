// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Build runner — plan generation and build unit execution.
// At P47, run_build is stub-complete: planning is live,
// actual compiler invocation wired at P49 (axon_lex exists).

use axon_std_string::AxString;
use crate::error::BuildResult;
use crate::graph::DepGraph;
use crate::hasher::BuildCache;
use crate::manifest::AxManifest;
use crate::scanner::SourceFile;

#[derive(Debug)]
pub struct BuildUnit {
    pub crate_name: AxString,
    pub sources: Vec<SourceFile>,
    pub changed: bool,
}

#[derive(Debug)]
pub struct BuildPlan {
    pub units: Vec<BuildUnit>,
}

/// Generate a build plan from manifest, graph, and cache.
pub fn plan_build(
    _manifest: &AxManifest,
    graph: &DepGraph,
    cache: &BuildCache,
    sources: &[SourceFile],
) -> BuildResult<BuildPlan> {
    let topo = crate::graph::graph_topo_sort(graph)?;
    let mut units: Vec<BuildUnit> = Vec::new();

    for crate_name in topo {
        let changed = sources.iter().any(|f| cache.check(f));
        units.push(BuildUnit {
            crate_name,
            sources: sources.to_vec(),
            changed,
        });
    }

    Ok(BuildPlan { units })
}

/// Execute a build plan.
/// At P47: logs what would be built — compiler invocation wired at P49.
pub fn run_build(plan: &BuildPlan, _manifest: &AxManifest) -> BuildResult<()> {
    for unit in &plan.units {
        if unit.changed {
            let _ = axon_std_io::stdout_write(
                format!("[axon_build] would compile: {}
", unit.crate_name.as_str()).as_bytes()
            );
        } else {
            let _ = axon_std_io::stdout_write(
                format!("[axon_build] up to date: {}
", unit.crate_name.as_str()).as_bytes()
            );
        }
    }
    Ok(())
}
