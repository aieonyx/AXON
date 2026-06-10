// axon_parser/src/mono.rs
// AXON Phase 17 — Generics Monomorphization
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// Stamps concrete HirFn copies from generic templates by substituting
// HirTy::Param("T") with a caller-supplied concrete type.
//
// API:
//   stamp(module, fn_name, &[("T", HirTy::I32)]) → concrete HirFn named fn_i32
//   MonoTable::collect(module) → table of all generic fns, keyed by name
//   MonoTable::instantiate(fn_name, type_args) → stamped HirFn

use std::collections::HashMap;
use crate::hir::{HirFn, HirModule, HirItem, HirTy, HirExpr, HirExprKind,
                  HirStmt, HirStmtKind, HirPat, HirMatchArm};

// ============================================================
// TYPE SUBSTITUTION
// ============================================================

/// Substitute all HirTy::Param occurrences according to the provided map.
pub fn subst_ty(ty: HirTy, map: &HashMap<String, HirTy>) -> HirTy {
    match ty {
        HirTy::Param(ref name) => {
            map.get(name).cloned().unwrap_or(ty)
        }
        HirTy::Ref(m, lt, inner) =>
            HirTy::Ref(m, lt, Box::new(subst_ty(*inner, map))),
        HirTy::Ptr(m, inner) =>
            HirTy::Ptr(m, Box::new(subst_ty(*inner, map))),
        HirTy::Slice(inner) =>
            HirTy::Slice(Box::new(subst_ty(*inner, map))),
        HirTy::Array(inner, n) =>
            HirTy::Array(Box::new(subst_ty(*inner, map)), n),
        HirTy::Tuple(tys) =>
            HirTy::Tuple(tys.into_iter().map(|t| subst_ty(t, map)).collect()),
        HirTy::Named(name, args) =>
            HirTy::Named(name, args.into_iter().map(|t| subst_ty(t, map)).collect()),
        HirTy::Fn(ps, r) =>
            HirTy::Fn(
                ps.into_iter().map(|t| subst_ty(t, map)).collect(),
                Box::new(subst_ty(*r, map)),
            ),
        other => other,
    }
}

/// Substitute types throughout a HirExpr tree.
pub fn subst_expr(expr: HirExpr, map: &HashMap<String, HirTy>) -> HirExpr {
    let ty = subst_ty(expr.ty, map);
    let kind = subst_expr_kind(expr.kind, map);
    HirExpr { kind, ty, ..expr }
}

fn subst_expr_kind(kind: HirExprKind, map: &HashMap<String, HirTy>) -> HirExprKind {
    match kind {
        HirExprKind::BinOp(op, l, r) =>
            HirExprKind::BinOp(op, Box::new(subst_expr(*l, map)), Box::new(subst_expr(*r, map))),
        HirExprKind::UnOp(op, e) =>
            HirExprKind::UnOp(op, Box::new(subst_expr(*e, map))),
        HirExprKind::Call(f, args) =>
            HirExprKind::Call(Box::new(subst_expr(*f, map)),
                              args.into_iter().map(|a| subst_expr(a, map)).collect()),
        HirExprKind::MethodCall(recv, name, args) =>
            HirExprKind::MethodCall(Box::new(subst_expr(*recv, map)), name,
                                    args.into_iter().map(|a| subst_expr(a, map)).collect()),
        HirExprKind::Block(stmts, tail) =>
            HirExprKind::Block(
                stmts.into_iter().map(|s| subst_stmt(s, map)).collect(),
                tail.map(|e| Box::new(subst_expr(*e, map))),
            ),
        HirExprKind::If(c, t, e) =>
            HirExprKind::If(Box::new(subst_expr(*c, map)),
                            Box::new(subst_expr(*t, map)),
                            e.map(|e| Box::new(subst_expr(*e, map)))),
        HirExprKind::Return(v) =>
            HirExprKind::Return(v.map(|e| Box::new(subst_expr(*e, map)))),
        HirExprKind::Try(e) =>
            HirExprKind::Try(Box::new(subst_expr(*e, map))),
        HirExprKind::Cast(e, ty) =>
            HirExprKind::Cast(Box::new(subst_expr(*e, map)), subst_ty(ty, map)),
        HirExprKind::Match(scrut, arms) =>
            HirExprKind::Match(Box::new(subst_expr(*scrut, map)),
                arms.into_iter().map(|arm| HirMatchArm {
                    pat: subst_pat(arm.pat, map),
                    guard: arm.guard.map(|g| subst_expr(g, map)),
                    body: subst_expr(arm.body, map),
                    span: arm.span,
                }).collect()),
        HirExprKind::Assign(p, e) =>
            HirExprKind::Assign(p, Box::new(subst_expr(*e, map))),
        HirExprKind::While(c, b) =>
            HirExprKind::While(Box::new(subst_expr(*c, map)), Box::new(subst_expr(*b, map))),
        HirExprKind::Loop(b) =>
            HirExprKind::Loop(Box::new(subst_expr(*b, map))),
        HirExprKind::For(pat, iter, body) =>
            HirExprKind::For(subst_pat(pat, map), Box::new(subst_expr(*iter, map)),
                             Box::new(subst_expr(*body, map))),
        HirExprKind::Tuple(es) =>
            HirExprKind::Tuple(es.into_iter().map(|e| subst_expr(e, map)).collect()),
        HirExprKind::Array(es) =>
            HirExprKind::Array(es.into_iter().map(|e| subst_expr(e, map)).collect()),
        HirExprKind::Struct(name, fields) =>
            HirExprKind::Struct(name, fields.into_iter()
                .map(|(f, e)| (f, subst_expr(e, map))).collect()),
        HirExprKind::Field(e, f, p) =>
            HirExprKind::Field(Box::new(subst_expr(*e, map)), f, p),
        HirExprKind::Index(e, i, p) =>
            HirExprKind::Index(Box::new(subst_expr(*e, map)), Box::new(subst_expr(*i, map)), p),
        HirExprKind::Ref(m, p, b) => HirExprKind::Ref(m, p, b),
        HirExprKind::Deref(e, p) =>
            HirExprKind::Deref(Box::new(subst_expr(*e, map)), p),
        HirExprKind::Range(s, e, inc) =>
            HirExprKind::Range(Box::new(subst_expr(*s, map)), Box::new(subst_expr(*e, map)), inc),
        HirExprKind::Closure(params, body, caps) =>
            HirExprKind::Closure(
                params.into_iter().map(|(p, ty)| (p, subst_ty(ty, map))).collect(),
                Box::new(subst_expr(*body, map)),
                caps,
            ),
        other => other,
    }
}

fn subst_stmt(stmt: HirStmt, map: &HashMap<String, HirTy>) -> HirStmt {
    let kind = match stmt.kind {
        HirStmtKind::Let(p, m, ty, val) =>
            HirStmtKind::Let(p, m, subst_ty(ty, map), val.map(|e| subst_expr(e, map))),
        HirStmtKind::Expr(e) => HirStmtKind::Expr(subst_expr(e, map)),
        other => other,
    };
    HirStmt { kind, ..stmt }
}

fn subst_pat(pat: HirPat, _map: &HashMap<String, HirTy>) -> HirPat {
    // Pats don't carry types yet — pass through
    pat
}

// ============================================================
// MONO TABLE
// ============================================================

/// Registry of generic functions available for instantiation.
pub struct MonoTable {
    pub generics: HashMap<String, HirFn>,
}

impl MonoTable {
    /// Collect all generic functions from the module.
    pub fn collect(module: &HirModule) -> MonoTable {
        let mut generics = HashMap::new();
        for item in &module.items {
            if let HirItem::Fn(f) = item {
                if !f.generics.is_empty() {
                    generics.insert(f.name.clone(), f.clone());
                }
            }
        }
        MonoTable { generics }
    }

    /// Stamp a concrete copy of a generic function with the given type substitutions.
    /// Returns None if the function is not in the table or has wrong arity.
    pub fn instantiate(&self, fn_name: &str, type_args: &[(&str, HirTy)]) -> Option<HirFn> {
        let template = self.generics.get(fn_name)?;
        let map: HashMap<String, HirTy> = type_args.iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        // Mangled name: fn_name_T1_T2 e.g. id_i32
        let suffix: Vec<String> = template.generics.iter()
            .map(|g| {
                map.get(g).map(llvm_ty_name).unwrap_or_else(|| g.clone())
            })
            .collect();
        let mangled_name = format!("{}_{}", fn_name, suffix.join("_"));
        let new_params = template.params.iter()
            .map(|(p, ty)| (*p, subst_ty(ty.clone(), &map)))
            .collect();
        let new_ret = subst_ty(template.ret.clone(), &map);
        let new_body = subst_expr(template.body.clone(), &map);
        Some(HirFn {
            name: mangled_name,
            generics: vec![], // concrete — no remaining type params
            params: new_params,
            ret: new_ret,
            body: new_body,
            contracts: template.contracts.clone(),
            is_pub: template.is_pub,
            is_pure: template.is_pure,
            is_ghost: template.is_ghost,
            span: template.span.clone(),
            required_caps: template.required_caps.clone(),
            // P27-M1: propagate notification handler through monomorphization
            is_notification_handler: template.is_notification_handler,
            // P25-M1: propagate panic handler through monomorphization
            is_panic_handler: template.is_panic_handler,
            // P24-M1: propagate linker control attrs through monomorphization
            no_mangle: template.no_mangle,
            link_section: template.link_section.clone(),
            stack_size: template.stack_size,
        })
    }
}

/// Short LLVM-style name for a type, used in mangled function names.
fn llvm_ty_name(ty: &HirTy) -> String {
    match ty {
        HirTy::I8  => "i8".into(),  HirTy::I16 => "i16".into(),
        HirTy::I32 => "i32".into(), HirTy::I64 => "i64".into(),
        HirTy::U8  => "u8".into(),  HirTy::U16 => "u16".into(),
        HirTy::U32 => "u32".into(), HirTy::U64 => "u64".into(),
        HirTy::F32 => "f32".into(), HirTy::F64 => "f64".into(),
        HirTy::Bool => "bool".into(),
        HirTy::Named(n, _) => n.clone(),
        _ => "unknown".into(),
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::{lower, HirTy};
    use crate::parser::parse;

    fn make_table(src: &str) -> MonoTable {
        let items = parse(src).expect("parse failed");
        let module = lower(items);
        MonoTable::collect(&module)
    }

    // ── Phase 17 M2 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_mono_table_collects_generic_fn() {
        let table = make_table("fn id<T>(x: T) -> T { return x; }");
        assert!(table.generics.contains_key("id"),
            "MonoTable must contain generic fn id");
    }

    #[test]
    fn tc_mono_table_ignores_non_generic() {
        let table = make_table("fn add(x: i32, y: i32) -> i32 { return x; }");
        assert!(!table.generics.contains_key("add"),
            "MonoTable must not contain non-generic fn");
    }

    #[test]
    fn tc_mono_stamp_i32() {
        let table = make_table("fn id<T>(x: T) -> T { return x; }");
        let f = table.instantiate("id", &[("T", HirTy::I32)])
            .expect("instantiate must succeed");
        assert_eq!(f.name, "id_i32", "mangled name must be id_i32");
        assert!(f.generics.is_empty(), "concrete fn must have no generics");
        assert_eq!(f.params[0].1, HirTy::I32,
            "param must be substituted to i32");
        assert_eq!(f.ret, HirTy::I32,
            "ret must be substituted to i32");
    }

    #[test]
    fn tc_mono_stamp_bool() {
        let table = make_table("fn id<T>(x: T) -> T { return x; }");
        let f = table.instantiate("id", &[("T", HirTy::Bool)])
            .expect("instantiate must succeed");
        assert_eq!(f.name, "id_bool");
        assert_eq!(f.params[0].1, HirTy::Bool);
    }

    #[test]
    fn tc_mono_stamp_unknown_fn_returns_none() {
        let table = make_table("fn id<T>(x: T) -> T { return x; }");
        assert!(table.instantiate("nonexistent", &[("T", HirTy::I32)]).is_none());
    }

    #[test]
    fn tc_mono_two_type_params() {
        let table = make_table("fn swap<A, B>(a: A, b: B) -> A { return a; }");
        let f = table.instantiate("swap", &[("A", HirTy::I32), ("B", HirTy::Bool)])
            .expect("instantiate must succeed");
        assert_eq!(f.name, "swap_i32_bool");
        assert_eq!(f.params[0].1, HirTy::I32);
        assert_eq!(f.params[1].1, HirTy::Bool);
        assert_eq!(f.ret, HirTy::I32);
    }
}
