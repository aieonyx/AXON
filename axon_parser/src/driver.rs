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

use crate::hir::{HirModule, HirItem, HirError};
use crate::parser::parse;
use std::collections::HashMap;

// ============================================================
// MERGE
// ============================================================

/// Merge two HirModules into one — items concatenated, use_maps unioned,
/// errors concatenated. Later modules win on use_map key conflicts.
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
/// Sources are merged in order — later sources take precedence on conflicts.
pub fn compile_sources(sources: &[&str]) -> HirModule {
    let mut result = empty_module();
    for src in sources {
        match parse(src) {
            Ok(items) => {
                let module = crate::hir::lower(items);
                result = merge_modules(result, module);
            }
            Err(e) => {
                result.errors.push(HirError {
                    msg: format!("parse error: {}", e.msg),
                    span: e.span,
                });
            }
        }
    }
    result
}

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
    fn tc_single_source_compile() {
        // compile_sources with one source works correctly
        let m = compile_sources(&["fn id(x: i32) -> i32 { return x; }"]);
        assert_eq!(m.errors.len(), 0, "errors: {:?}", m.errors);
        assert_eq!(m.items.len(), 1);
    }
}
