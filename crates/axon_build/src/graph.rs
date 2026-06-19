// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Dependency graph — DAG resolution and topological sort.
// Cycle detection via DFS with three-color marking.

use axon_std_string::AxString;
use crate::error::{BuildError, BuildResult};
use crate::manifest::AxManifest;

#[derive(Debug)]
pub struct DepGraph {
    pub nodes: Vec<AxString>,
    pub edges: Vec<(usize, usize)>,
}

/// Build a dependency graph from a manifest.
pub fn graph_resolve(manifest: &AxManifest) -> BuildResult<DepGraph> {
    let mut nodes: Vec<AxString> = Vec::new();
    let mut edges: Vec<(usize, usize)> = Vec::new();

    // Root crate is node 0
    nodes.push(manifest.name.clone());

    for dep in &manifest.dependencies {
        let dep_idx = match nodes.iter().position(|n| n == &dep.name) {
            Some(i) => i,
            None => {
                nodes.push(dep.name.clone());
                nodes.len() - 1
            }
        };
        // edge: root (0) depends on dep_idx
        edges.push((0, dep_idx));
    }

    let graph = DepGraph { nodes, edges };
    // Validate — no cycles
    graph_topo_sort(&graph)?;
    Ok(graph)
}

/// Topological sort via Kahn's algorithm.
/// Returns crate names in build order (dependencies first).
pub fn graph_topo_sort(graph: &DepGraph) -> BuildResult<Vec<AxString>> {
    let n = graph.nodes.len();
    let mut in_degree = vec![0usize; n];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

    for &(from, to) in &graph.edges {
        adj[from].push(to);
        in_degree[to] += 1;
    }

    // Start with nodes that have no incoming edges
    let mut queue: Vec<usize> = (0..n)
        .filter(|&i| in_degree[i] == 0)
        .collect();

    let mut order: Vec<AxString> = Vec::new();

    while !queue.is_empty() {
        let node = queue.remove(0);
        order.push(graph.nodes[node].clone());
        for &neighbor in &adj[node] {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                queue.push(neighbor);
            }
        }
    }

    if order.len() != n {
        return Err(BuildError::CycleDetected);
    }

    // Reverse so dependencies come before dependents
    order.reverse();
    Ok(order)
}
