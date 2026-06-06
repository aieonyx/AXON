// axon_parser/src/driver.rs
// AXON Phase 19 — Multi-File Compilation Driver
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// Compiles multiple .axon source files into a single unified HirModule.
// Each file is parsed independently then merged — no cross-file name
// resolution at this stage (that is Phase 19 M2 use_map territory).
//
// API:
//   compile_sources(sources: &[&str]) -> HirModule
//   merge_modules(a: HirModule, b: HirModule) -> HirModule

use crate::hir::{HirModule, HirError};
use crate::parser::parse;
use std::collections::HashMap;

// ============================================================
// MERGE
// ============================================================

/// Merge two HirModules into one — items concatenated, use_maps unioned,
/// errors concatenated. Later modules win on use_map key conflicts.
/// Merge two HIR modules.
/// use_map key conflicts are resolved by last-writer-wins (b overrides a).
/// This is intentional for incremental compilation; conflict errors
/// are deferred to a later analysis phase.
///
/// NOTE: PlaceIds from separate lower() calls are NOT remapped here.
/// For correct multi-file codegen, use compile_sources() which merges
/// source text before lowering, giving a single shared PlaceId space.
pub fn merge_modules(mut a: HirModule, b: HirModule) -> HirModule {
    a.items.extend(b.items);
    a.errors.extend(b.errors);
    for (k, v) in b.use_map {
        a.use_map.insert(k, v);
    }
    a
}

/// Empty HirModule — identity element for merge_modules.
pub fn empty_module() -> HirModule {
    HirModule {
        items: Vec::new(),
        errors: Vec::new(),
        use_map: HashMap::new(),
    }
}

// ============================================================
// COMPILE
// ============================================================

/// Parse and lower multiple source strings into a single HirModule.
/// Sources are concatenated before lowering to guarantee a single shared
/// PlaceId space — this prevents PlaceId collisions in merged IR.
/// Parse errors in any source are collected; successfully parsed sources
/// are still included.
pub fn compile_sources(sources: &[&str]) -> HirModule {
    // Collect parse errors separately, lower valid sources together
    let mut errors: Vec<HirError> = Vec::new();
    let mut all_items = Vec::new();
    let mut all_use_paths: Vec<Vec<String>> = Vec::new();

    for src in sources {
        match parse(src) {
            Ok(items) => {
                // Collect use paths before consuming items
                for item in &items {
                    if let crate::parser::Item::Use(path, _) = item {
                        all_use_paths.push(path.clone());
                    }
                }
                all_items.extend(items);
            }
            Err(e) => {
                errors.push(HirError {
                    msg: format!("parse error: {}", e.msg),
                    span: e.span,
                });
            }
        }
    }

    // Lower all items together — single HirLowerer, no PlaceId collision
    let mut module = crate::hir::lower(all_items);
    module.errors.extend(errors);
    module
}

/// KNOWN GAP: cap enforcement (check_transitive) is not called by the driver.
/// Capability violations are only caught if check_transitive is explicitly
/// invoked after compile_sources/compile_files. Wiring into the driver
/// is deferred to the compiler CLI integration pass.
/// Parse and lower source files by reading from disk.
/// Returns merged HirModule; parse/IO errors are collected into module.errors.
pub fn compile_files(paths: &[&str]) -> HirModule {
    let mut result = empty_module();
    for path in paths {
        match std::fs::read_to_string(path) {
            Ok(src) => {
                match parse(&src) {
                    Ok(items) => {
                        let module = crate::hir::lower(items);
                        result = merge_modules(result, module);
                    }
                    Err(e) => {
                        result.errors.push(HirError {
                            msg: format!("parse error in {}: {}", path, e.msg),
                            span: e.span,
                        });
                    }
                }
            }
            Err(e) => {
                result.errors.push(HirError {
                    msg: format!("IO error reading {}: {}", path, e),
                    span: crate::lexer::Span::new(0, 0),
                });
            }
        }
    }
    result
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::HirItem;

    // ── Phase 19 M4 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_p19_integration() {
        // Full program: mod block + use declaration + multi-file merge.
        // File 1: defines mod math with fn add
        // File 2: uses math::add
        // Merged module must have both the Module item and the use_map entry.
        let src1 = r#"
            mod math {
                fn add(x: i32, y: i32) -> i32 { return x; }
            }
        "#;
        let src2 = "use math::add;";

        let m = compile_sources(&[src1, src2]);
        assert_eq!(m.errors.len(), 0, "errors: {:?}", m.errors);

        // mod math present
        assert!(m.items.iter().any(|i| matches!(i, crate::hir::HirItem::Module(n, _) if n == "math")),
            "must contain mod math, items: {:?}", m.items.len());

        // use math::add resolved
        assert!(m.use_map.contains_key("add"),
            "use_map must contain add, got: {:?}", m.use_map);
        assert_eq!(m.use_map.get("add").map(|s| s.as_str()), Some("math::add"),
            "add must resolve to math::add");
    }

    // ── Phase 19 M3 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_multi_file_merges_items() {
        // Two source strings — items from both must appear in merged module
        let src1 = "fn add(x: i32, y: i32) -> i32 { return x; }";
        let src2 = "fn sub(x: i32, y: i32) -> i32 { return x; }";
        let m = compile_sources(&[src1, src2]);
        assert_eq!(m.errors.len(), 0, "errors: {:?}", m.errors);
        assert_eq!(m.items.len(), 2, "merged module must have 2 items");
        let names: Vec<&str> = m.items.iter().filter_map(|i| {
            if let HirItem::Fn(f) = i { Some(f.name.as_str()) } else { None }
        }).collect();
        assert!(names.contains(&"add"), "must contain add");
        assert!(names.contains(&"sub"), "must contain sub");
    }

    #[test]
    fn tc_multi_file_use_map_merged() {
        // use declarations from both sources appear in merged use_map
        let src1 = "use foo::bar;";
        let src2 = "use baz::qux;";
        let m = compile_sources(&[src1, src2]);
        assert!(m.use_map.contains_key("bar"),
            "use_map must contain bar, got: {:?}", m.use_map);
        assert!(m.use_map.contains_key("qux"),
            "use_map must contain qux, got: {:?}", m.use_map);
    }

    #[test]
    fn tc_multi_file_parse_error_collected() {
        // A source with a syntax error — error collected, other source still merged
        let src1 = "fn good(x: i32) -> i32 { return x; }";
        let src2 = "fn bad( { "; // syntax error
        let m = compile_sources(&[src1, src2]);
        // good fn still present
        assert!(m.items.iter().any(|i| matches!(i, HirItem::Fn(f) if f.name == "good")),
            "good fn must be present despite other source error");
        // error collected
        assert!(!m.errors.is_empty(), "parse error must be collected");
    }

    #[test]
    fn tc_empty_module_is_identity() {
        // merge_modules(empty, m) == m items-wise
        let src = "fn f(x: i32) -> i32 { return x; }";
        let m = compile_sources(&[src]);
        let merged = merge_modules(empty_module(), compile_sources(&[src]));
        assert_eq!(merged.items.len(), m.items.len());
    }

    #[test]
    fn tc_p19_driver_compile_files() {
        // compile_files reads from disk — write temp files and verify merge
        use crate::hir::HirItem;
        let dir = "/tmp/axon_p19_test";
        let _ = std::fs::create_dir(dir);
        let f1 = format!("{}/a.axon", dir);
        let f2 = format!("{}/b.axon", dir);
        std::fs::write(&f1, "fn fa(x: i32) -> i32 { return x; }").unwrap();
        std::fs::write(&f2, "fn fb(x: i32) -> i32 { return x; }").unwrap();
        let module = compile_files(&[f1.as_str(), f2.as_str()]);
        assert!(module.errors.is_empty(), "errors: {:?}", module.errors);
        let names: Vec<String> = module.items.iter().filter_map(|i| {
            if let HirItem::Fn(f) = i { Some(f.name.clone()) } else { None }
        }).collect();
        assert!(names.contains(&"fa".to_string()), "must contain fa, got: {:?}", names);
        assert!(names.contains(&"fb".to_string()), "must contain fb, got: {:?}", names);
    }

    #[test]
    fn tc_single_source_compile() {
        // compile_sources with one source works correctly
        let m = compile_sources(&["fn id(x: i32) -> i32 { return x; }"]);
        assert_eq!(m.errors.len(), 0, "errors: {:?}", m.errors);
        assert_eq!(m.items.len(), 1);
    }
}
