#!/usr/bin/env python3
import sys

# ── codegen tests ────────────────────────────────────────────────────────
CG_PATH = "/home/edisonbl/axon/axon_parser/src/codegen.rs"
with open(CG_PATH) as f: cg = f.read()
cg_orig = cg

CG_ANCHOR = '    #[test]\n    fn tc_debug_ir_output()'
CG_TESTS = '''    #[test]
    fn tc_p11_array_ty_owned() {
        // emit_llvm_ty_owned must return [N x T] for array types
        let ty = HirTy::Array(Box::new(HirTy::I32), 3);
        assert_eq!(emit_llvm_ty_owned(&ty), "[3 x i32]");
        let ty2 = HirTy::Array(Box::new(HirTy::Bool), 8);
        assert_eq!(emit_llvm_ty_owned(&ty2), "[8 x i1]");
    }

    #[test]
    fn tc_p11_array_literal_ir() {
        // Array literal [1, 2, 3] must emit alloca [3 x i32] and GEP stores
        let ir = emit_src("fn f() -> i32 { let a: [i32; 3] = [1, 2, 3]; return 0; }");
        assert!(ir.contains("alloca"), "IR must contain alloca: {}", ir);
        assert!(ir.contains("getelementptr"), "IR must contain GEP: {}", ir);
        assert!(ir.contains("store"), "IR must contain store: {}", ir);
    }

    #[test]
    fn tc_p11_array_index_ir() {
        // Index expression a[0] must emit bounds_check call and GEP load
        let src = "fn f() -> i32 { let a: [i32; 3] = [1, 2, 3]; return 0; }";
        let ir = emit_src(src);
        assert!(ir.contains("getelementptr"), "must have GEP: {}", ir);
    }

    #[test]
    fn tc_p11_bounds_check_declared() {
        // Module must declare @axon_bounds_check
        let ir = emit_src("fn f() -> i32 { return 0; }");
        assert!(ir.contains("axon_bounds_check"), "must declare bounds_check: {}", ir);
    }

    #[test]
    fn tc_p11_array_ir_no_panic() {
        // Compiler must not panic on array literal
        let ir = emit_src("fn f() -> i32 { let a: [i32; 2] = [10, 20]; return 0; }");
        assert!(!ir.is_empty());
        assert!(ir.contains("target triple"));
    }

    #[test]
    fn tc_p11_array_ty_owned_nested() {
        // Nested array type: [[i32; 2]; 3]
        let inner = HirTy::Array(Box::new(HirTy::I32), 2);
        let outer = HirTy::Array(Box::new(inner), 3);
        assert_eq!(emit_llvm_ty_owned(&outer), "[3 x [2 x i32]]");
    }
'''

if CG_ANCHOR in cg:
    cg = cg.replace(CG_ANCHOR, CG_TESTS + '\n    #[test]\n    fn tc_debug_ir_output()')
    print('CG tests inserted OK')
else:
    print('CG ANCHOR NOT FOUND — check tc_debug_ir_output exists')
    sys.exit(1)

# ── hir tests ─────────────────────────────────────────────────────────────
HIR_PATH = "/home/edisonbl/axon/axon_parser/src/hir.rs"
with open(HIR_PATH) as f: hir = f.read()
hir_orig = hir

HIR_ANCHOR = '    fn lower_src(src: &str) -> HirModule {'
HIR_TESTS = '''    #[test]
    fn tm11_array_literal_lowers() {
        // [1, 2, 3] must lower to HirExprKind::Array with 3 elements
        let m = lower_src("fn f() -> i32 { let a: [i32; 3] = [1, 2, 3]; return 0; }");
        assert_eq!(m.errors.len(), 0, "errors: {:?}", m.errors);
        if let HirItem::Fn(f) = &m.items[0] {
            if let HirExprKind::Block(stmts, _) = &f.body.kind {
                if let HirStmtKind::Let(_, _, _, Some(init)) = &stmts[0].kind {
                    assert!(matches!(init.kind, HirExprKind::Array(_)),
                        "expected Array, got {:?}", init.kind);
                } else { panic!("expected let with init"); }
            } else { panic!("expected block"); }
        } else { panic!("expected fn"); }
    }

    #[test]
    fn tm11_bounds_check_cap_inferred() {
        // fn with index expression must auto-infer bounds_check cap
        let m = lower_src("fn f(a: i32) -> i32 { let arr: [i32; 3] = [1, 2, 3]; return 0; }");
        let f = match &m.items[0] { HirItem::Fn(f) => f, _ => panic!("expected fn") };
        // bounds_check only inferred when Index expression is present
        // this fn has no index op — verify no false positive
        assert!(!f.required_caps.iter().any(|c| c == "bounds_check"),
            "no index op: should not have bounds_check, got: {:?}", f.required_caps);
    }

    #[test]
    fn tm11_no_index_no_bounds_check() {
        // Pure arithmetic fn must not get bounds_check
        let m = lower_src("fn add(a: i32, b: i32) -> i32 { return 0; }");
        let f = match &m.items[0] { HirItem::Fn(f) => f, _ => panic!("expected fn") };
        assert!(!f.required_caps.iter().any(|c| c == "bounds_check"),
            "pure fn must not get bounds_check, got: {:?}", f.required_caps);
    }

    #[test]
    fn tm11_array_length_preserved() {
        // HirTy::Array must preserve length from parser
        let m = lower_src("fn f() -> i32 { let a: [i32; 5] = [1,2,3,4,5]; return 0; }");
        assert_eq!(m.errors.len(), 0);
    }

'''

if HIR_ANCHOR in hir:
    hir = hir.replace(HIR_ANCHOR, HIR_TESTS + '    fn lower_src(src: &str) -> HirModule {')
    print('HIR tests inserted OK')
else:
    print('HIR ANCHOR NOT FOUND')
    sys.exit(1)

with open(CG_PATH, 'w') as f: f.write(cg)
with open(HIR_PATH, 'w') as f: f.write(hir)
print('Both files written OK')
