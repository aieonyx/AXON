#!/usr/bin/env python3
import sys

PATH = "/home/edisonbl/axon/axon_parser/src/hir.rs"

with open(PATH, "r") as f:
    src = f.read()

original = src
changes = []

OLD1 = "            Ty::Array(inner, _) =>\n                HirTy::Array(Box::new(self.lower_ty(inner)), 0),"
NEW1 = """            Ty::Array(inner, len_expr) => {
                let n = match len_expr.as_ref() {
                    Expr::Lit(Lit::Int(n), _) => *n,
                    _ => 0,
                };
                HirTy::Array(Box::new(self.lower_ty(inner)), n)
            }"""

if OLD1 in src:
    src = src.replace(OLD1, NEW1)
    changes.append("PATCH 1: lower_ty Array length — OK")
else:
    changes.append("PATCH 1: ANCHOR NOT FOUND")

OLD2 = "// M4: auto-infer alloc_heap for any fn whose signature or body uses String"
NEW2 = """// P11-M4: detect index expressions in fn body
fn hir_expr_contains_index(expr: &HirExpr) -> bool {
    match &expr.kind {
        HirExprKind::Index(_, _, _) => true,
        HirExprKind::BinOp(_, l, r) =>
            hir_expr_contains_index(l) || hir_expr_contains_index(r),
        HirExprKind::Call(f, args) =>
            hir_expr_contains_index(f) || args.iter().any(|a| hir_expr_contains_index(a)),
        HirExprKind::MethodCall(recv, _, args) =>
            hir_expr_contains_index(recv) || args.iter().any(|a| hir_expr_contains_index(a)),
        HirExprKind::Block(stmts, tail) => {
            stmts.iter().any(|s| match &s.kind {
                HirStmtKind::Let(_, _, _, Some(e)) => hir_expr_contains_index(e),
                HirStmtKind::Expr(e) => hir_expr_contains_index(e),
                _ => false,
            }) || tail.as_ref().map_or(false, |e| hir_expr_contains_index(e))
        }
        HirExprKind::If(c, t, e) =>
            hir_expr_contains_index(c) || hir_expr_contains_index(t)
            || e.as_ref().map_or(false, |e| hir_expr_contains_index(e)),
        HirExprKind::Return(Some(e)) => hir_expr_contains_index(e),
        HirExprKind::Array(elems) => elems.iter().any(|e| hir_expr_contains_index(e)),
        _ => false,
    }
}

// M4: auto-infer alloc_heap for any fn whose signature or body uses String"""

if OLD2 in src:
    src = src.replace(OLD2, NEW2)
    changes.append("PATCH 2: hir_expr_contains_index helper — OK")
else:
    changes.append("PATCH 2: ANCHOR NOT FOUND")

OLD3 = '        if uses_string && !required_caps.iter().any(|c| c == "alloc_heap") {\n            required_caps.push("alloc_heap".to_string());\n        }'
NEW3 = '        if uses_string && !required_caps.iter().any(|c| c == "alloc_heap") {\n            required_caps.push("alloc_heap".to_string());\n        }\n\n        // P11-M4: auto-infer bounds_check for any fn performing index operations\n        let uses_index = hir_expr_contains_index(&body);\n        if uses_index && !required_caps.iter().any(|c| c == "bounds_check") {\n            required_caps.push("bounds_check".to_string());\n        }'

if OLD3 in src:
    src = src.replace(OLD3, NEW3)
    changes.append("PATCH 3: bounds_check cap inference — OK")
else:
    changes.append("PATCH 3: ANCHOR NOT FOUND")

print(f"File: {PATH}")
print(f"Modified: {src != original}")
for c in changes:
    print(f"  {c}")

if src == original:
    print("ERROR: no changes applied")
    sys.exit(1)

with open(PATH, "w") as f:
    f.write(src)
print("Written OK")
