// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P47 QA — axon_build test suite
// Pass bar: 10/10 before P48 begins.

use axon_build::{
    graph_resolve, graph_topo_sort, parse_content, plan_build, scan_sources,
    Backend, BuildCache, BuildError, DepGraph,
};
use axon_std_string::AxString;
use tempfile::tempdir;

// T1: valid manifest parses correctly
#[test]
fn test_manifest_parse() {
    let raw = r#"
[crate]
name = "axon_lex"
version = "0.49.0"

[build]
src = "src/"
entry = "src/lib.ax"
target = "target/axon/"
backend = "posix"

[dependencies.axon_std_io]
path = "../axon_std/io"
"#;
    let m = parse_content(raw).unwrap();
    assert_eq!(m.name.as_str(), "axon_lex");
    assert_eq!(m.version.as_str(), "0.49.0");
    assert_eq!(m.src_dir.as_str(), "src/");
    assert_eq!(m.backend, Backend::Posix);
    assert_eq!(m.dependencies.len(), 1);
    assert_eq!(m.dependencies[0].name.as_str(), "axon_std_io");
}

// T2: missing manifest file returns ManifestNotFound
#[test]
fn test_manifest_missing() {
    let result = axon_build::manifest_parse("/tmp/axon_p47_no_such.axbuild");
    assert!(matches!(result, Err(BuildError::ManifestNotFound)));
}

// T3: malformed manifest returns ManifestParseError
#[test]
fn test_manifest_invalid() {
    let raw = r#"
[build]
src = "src/"
"#;
    // Missing [crate] name and version
    let result = parse_content(raw);
    assert!(matches!(result, Err(BuildError::ManifestParseError(_))));
}

// T4: scan discovers .ax files
#[test]
fn test_scan_sources() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("lib.ax"), b"fn main() {}").unwrap();
    std::fs::write(src.join("lexer.ax"), b"fn lex() {}").unwrap();
    std::fs::write(src.join("readme.md"), b"# ignore me").unwrap();

    let files = scan_sources(src.to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.path.as_str().ends_with("lib.ax")));
    assert!(files.iter().any(|f| f.path.as_str().ends_with("lexer.ax")));
}

// T5: empty dir returns empty vec
#[test]
fn test_scan_empty_dir() {
    let dir = tempdir().unwrap();
    let files = scan_sources(dir.path().to_str().unwrap()).unwrap();
    assert!(files.is_empty());
}

// T6: two-crate dependency resolves correctly
#[test]
fn test_graph_resolve() {
    let raw = r#"
[crate]
name = "axon_lex"
version = "0.49.0"

[build]
src = "src/"
entry = "src/lib.ax"
target = "target/axon/"

[dependencies.axon_std_string]
path = "../axon_std/string"
"#;
    let m = parse_content(raw).unwrap();
    let graph = graph_resolve(&m).unwrap();
    assert_eq!(graph.nodes.len(), 2);
    assert!(graph.nodes.iter().any(|n| n.as_str() == "axon_lex"));
    assert!(graph.nodes.iter().any(|n| n.as_str() == "axon_std_string"));
}

// T7: topological order — deps before dependents
#[test]
fn test_graph_topo_sort() {
    let raw = r#"
[crate]
name = "axon_lex"
version = "0.49.0"

[build]
src = "src/"
entry = "src/lib.ax"
target = "target/axon/"

[dependencies.axon_std_string]
path = "../axon_std/string"
"#;
    let m = parse_content(raw).unwrap();
    let graph = graph_resolve(&m).unwrap();
    let order = graph_topo_sort(&graph).unwrap();
    // axon_std_string must appear before axon_lex
    let pos_dep = order.iter().position(|n| n.as_str() == "axon_std_string").unwrap();
    let pos_root = order.iter().position(|n| n.as_str() == "axon_lex").unwrap();
    assert!(pos_dep < pos_root, "dependency must precede dependent");
}

// T8: cycle detection
#[test]
fn test_graph_cycle() {
    // Manually construct a cyclic graph: A -> B -> A
    let graph = DepGraph {
        nodes: vec![
            AxString::ax_from_str("crate_a"),
            AxString::ax_from_str("crate_b"),
        ],
        edges: vec![(0, 1), (1, 0)],
    };
    let result = graph_topo_sort(&graph);
    assert!(matches!(result, Err(BuildError::CycleDetected)));
}

// T9: cache detects changed file, skips unchanged
#[test]
fn test_cache_check() {
    use axon_build::SourceFile;

    let file = SourceFile {
        path: AxString::ax_from_str("src/lib.ax"),
        hash: 12345678,
    };

    let mut cache = BuildCache::new();

    // Not in cache yet — reports changed
    assert!(cache.check(&file), "new file must be reported as changed");

    // Update cache
    cache.update(&file);

    // Same hash — not changed
    assert!(!cache.check(&file), "unchanged file must not be reported as changed");

    // Different hash — changed
    let modified = SourceFile {
        path: AxString::ax_from_str("src/lib.ax"),
        hash: 99999999,
    };
    assert!(cache.check(&modified), "modified file must be reported as changed");
}

// T10: build plan marks changed units, skips clean
#[test]
fn test_build_plan() {
    use axon_build::SourceFile;

    let raw = r#"
[crate]
name = "axon_lex"
version = "0.49.0"

[build]
src = "src/"
entry = "src/lib.ax"
target = "target/axon/"
"#;
    let manifest = parse_content(raw).unwrap();
    let graph = graph_resolve(&manifest).unwrap();

    let sources = vec![SourceFile {
        path: AxString::ax_from_str("src/lib.ax"),
        hash: 42,
    }];

    // Empty cache — everything changed
    let empty_cache = BuildCache::new();
    let plan = plan_build(&manifest, &graph, &empty_cache, &sources).unwrap();
    assert!(plan.units.iter().any(|u| u.changed), "fresh build must have changed units");

    // Warm cache — nothing changed
    let mut warm_cache = BuildCache::new();
    for f in &sources { warm_cache.update(f); }
    let plan2 = plan_build(&manifest, &graph, &warm_cache, &sources).unwrap();
    assert!(!plan2.units.iter().any(|u| u.changed), "warm build must have no changed units");
}
