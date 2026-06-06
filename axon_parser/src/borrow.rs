// axon_parser/src/borrow.rs
// AXON Phase 18 — Sovereign Borrow Checker
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// Enforces memory safety without lifetime annotation burden.
// Rule 1: No use-after-move — reading a Moved place is a violation.
// Rule 2: No double mutable borrow — second &mut on same place is a violation.
// Escape: @unsafe_axon functions bypass the checker (violations logged, not fatal).
//
// Doctrine: safety without servitude. The checker serves the sovereign individual,
// not the compiler's convenience. Lifetime inference, not annotation burden.

use std::collections::HashMap;
use crate::hir::{
    HirModule, HirItem, HirFn, HirExpr, HirExprKind, HirStmt, HirStmtKind,
    PlaceId, MoveState, BorrowId,
};
use crate::lexer::Span;

// ============================================================
// ERROR TYPES
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum BorrowErrorKind {
    UseAfterMove,
    DoubleMutBorrow,
    UseWhileMutBorrowed,
}

#[derive(Debug, Clone)]
pub struct BorrowError {
    pub kind: BorrowErrorKind,
    pub place: PlaceId,
    pub span: Span,
    pub msg: String,
    /// If true, caller had @unsafe_axon — error is logged but not fatal.
    pub suppressed: bool,
}

impl BorrowError {
    pub fn use_after_move(place: PlaceId, span: Span) -> Self {
        BorrowError {
            kind: BorrowErrorKind::UseAfterMove,
            place,
            span,
            msg: format!("use of moved value: place {:?} has been moved", place),
            suppressed: false,
        }
    }

    pub fn double_mut_borrow(place: PlaceId, span: Span) -> Self {
        BorrowError {
            kind: BorrowErrorKind::DoubleMutBorrow,
            place,
            span,
            msg: format!("cannot borrow place {:?} as mutable more than once", place),
            suppressed: false,
        }
    }

    pub fn use_while_mut_borrowed(place: PlaceId, span: Span) -> Self {
        BorrowError {
            kind: BorrowErrorKind::UseWhileMutBorrowed,
            place,
            span,
            msg: format!("cannot use place {:?} while mutably borrowed", place),
            suppressed: false,
        }
    }
}

impl std::fmt::Display for BorrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tag = if self.suppressed { "[suppressed by @unsafe_axon] " } else { "" };
        write!(f, "BorrowError: {}{}", tag, self.msg)
    }
}

// ============================================================
// BORROW STATE
// ============================================================

/// Per-place borrow tracking during checker walk.
#[derive(Debug, Clone, Default)]
struct PlaceState {
    move_state: Option<MoveState>,
    /// Active mutable borrow ids on this place
    mut_borrows: Vec<BorrowId>,
    /// Active shared borrow count
    shared_borrows: u32,
}

// ============================================================
// BORROW CHECKER
// ============================================================

pub struct BorrowChecker {
    pub errors: Vec<BorrowError>,
    /// Place state during walk
    state: HashMap<PlaceId, PlaceState>,
    /// Whether current function has @unsafe_axon
    in_unsafe_axon: bool,
}

impl Default for BorrowChecker {
    fn default() -> Self { Self::new() }
}

impl BorrowChecker {
    pub fn new() -> Self {
        BorrowChecker {
            errors: Vec::new(),
            state: HashMap::new(),
            in_unsafe_axon: false,
        }
    }

    pub fn check_module(&mut self, module: &HirModule) {
        for item in &module.items {
            match item {
                HirItem::Fn(f) => self.check_fn(f),
                HirItem::Impl(imp) => {
                    for method in &imp.methods {
                        self.check_fn(method);
                    }
                }
                _ => {}
            }
        }
    }

    fn check_fn(&mut self, f: &HirFn) {
        self.state.clear();
        self.in_unsafe_axon = f.required_caps.iter().any(|c| c == "unsafe_axon");
        // Initialise params as Owned
        for (place, _ty) in &f.params {
            self.state.entry(*place).or_default().move_state = Some(MoveState::Owned);
        }
        self.check_expr(&f.body);
    }

    fn check_expr(&mut self, expr: &HirExpr) {
        let span = expr.span.clone();
        match &expr.kind {
            HirExprKind::Place(place, _move_state) => {
                // Rule 1: use-after-move
                // Extract check results before mutably borrowing self
                let is_moved = self.state.get(place)
                    .map(|ps| matches!(ps.move_state, Some(MoveState::Moved)))
                    .unwrap_or(false);
                let is_mut_borrowed = self.state.get(place)
                    .map(|ps| !ps.mut_borrows.is_empty())
                    .unwrap_or(false);
                if is_moved {
                    self.emit_error(BorrowError::use_after_move(*place, span.clone()));
                }
                if is_mut_borrowed {
                    self.emit_error(BorrowError::use_while_mut_borrowed(*place, span));
                }
            }
            HirExprKind::Ref(is_mut, place, borrow_id) => {
                let ps = self.state.entry(*place).or_default();
                if *is_mut {
                    // Rule 2: double mutable borrow
                    if !ps.mut_borrows.is_empty() {
                        self.emit_error(BorrowError::double_mut_borrow(*place, span));
                    } else {
                        ps.mut_borrows.push(*borrow_id);
                        ps.move_state = Some(MoveState::MutBorrowed);
                    }
                } else {
                    ps.shared_borrows += 1;
                    ps.move_state = Some(MoveState::Borrowed);
                }
            }
            HirExprKind::Assign(place, rhs) => {
                self.check_expr(rhs);
                // Assignment to place resets it to Owned
                self.state.entry(*place).or_default().move_state = Some(MoveState::Owned);
            }
            HirExprKind::Block(stmts, tail) => {
                for stmt in stmts { self.check_stmt(stmt); }
                if let Some(t) = tail { self.check_expr(t); }
            }
            HirExprKind::Call(func, args) => {
                self.check_expr(func);
                for arg in args { self.check_expr(arg); }
            }
            HirExprKind::MethodCall(recv, _, args) => {
                self.check_expr(recv);
                for arg in args { self.check_expr(arg); }
            }
            HirExprKind::BinOp(_, l, r) => {
                self.check_expr(l); self.check_expr(r);
            }
            HirExprKind::UnOp(_, e) => self.check_expr(e),
            HirExprKind::If(cond, then, else_) => {
                self.check_expr(cond);
                self.check_expr(then);
                if let Some(e) = else_ { self.check_expr(e); }
            }
            HirExprKind::While(cond, body) => {
                self.check_expr(cond); self.check_expr(body);
            }
            HirExprKind::Loop(body) => self.check_expr(body),
            HirExprKind::For(_, iter, body) => {
                self.check_expr(iter); self.check_expr(body);
            }
            HirExprKind::Return(v) => {
                if let Some(e) = v { self.check_expr(e); }
            }
            HirExprKind::Try(e) => self.check_expr(e),
            HirExprKind::Cast(e, _) => self.check_expr(e),
            HirExprKind::Field(e, _, _) => self.check_expr(e),
            HirExprKind::Index(e, i, _) => {
                self.check_expr(e); self.check_expr(i);
            }
            HirExprKind::Tuple(es) | HirExprKind::Array(es) => {
                for e in es { self.check_expr(e); }
            }
            HirExprKind::Struct(_, fields) => {
                for (_, e) in fields { self.check_expr(e); }
            }
            HirExprKind::Deref(e, _) => self.check_expr(e),
            HirExprKind::Range(s, e, _) => {
                self.check_expr(s); self.check_expr(e);
            }
            HirExprKind::Match(scrut, arms) => {
                self.check_expr(scrut);
                for arm in arms {
                    if let Some(g) = &arm.guard { self.check_expr(g); }
                    self.check_expr(&arm.body);
                }
            }
            HirExprKind::Closure(_, body, _) => self.check_expr(body),
            HirExprKind::Drop(_) | HirExprKind::BorrowExpires(_) => {}
            // Leaves — no sub-expressions
            HirExprKind::Lit(_) | HirExprKind::Path(_) | HirExprKind::Continue
            | HirExprKind::Break(_) => {}
        }
    }

    fn check_stmt(&mut self, stmt: &HirStmt) {
        match &stmt.kind {
            HirStmtKind::Let(place, _, _, val) => {
                if let Some(e) = val { self.check_expr(e); }
                // New binding starts as Owned
                self.state.entry(*place).or_default().move_state = Some(MoveState::Owned);
            }
            HirStmtKind::Expr(e) => self.check_expr(e),
            HirStmtKind::StorageLive(_) | HirStmtKind::StorageDead(_) => {}
            HirStmtKind::DropElaborated(_) => {}
            // Other stmt kinds handled implicitly
        }
    }

    fn emit_error(&mut self, mut err: BorrowError) {
        if self.in_unsafe_axon {
            err.suppressed = true;
        }
        self.errors.push(err);
    }
}

/// Convenience entry point.
pub fn check(module: &HirModule) -> Vec<BorrowError> {
    let mut checker = BorrowChecker::new();
    checker.check_module(module);
    checker.errors
}

/// Fatal enforcement — call after check() from CLI.
pub fn enforce(errors: &[BorrowError]) {
    let fatal: Vec<&BorrowError> = errors.iter().filter(|e| !e.suppressed).collect();
    if !fatal.is_empty() {
        eprintln!("AXON borrow checker: {} violation(s):", fatal.len());
        for e in &fatal {
            eprintln!("  {}", e);
        }
        eprintln!("Compilation aborted.");
        std::process::exit(1);
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::lower;
    use crate::parser::parse;

    fn check_src(src: &str) -> Vec<BorrowError> {
        let items = parse(src).expect("parse failed");
        let module = lower(items);
        check(&module)
    }

    // ── Phase 18 M4 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_p18_integration() {
        // Full program: clean fn has zero borrow errors.
        // check() entry point works end-to-end on a HirModule.
        let src = r#"
            fn add(x: i32, y: i32) -> i32 { return x; }
            fn sub(x: i32, y: i32) -> i32 { return y; }
            fn mul(x: i32, y: i32) -> i32 { return x; }
        "#;
        let errs = check_src(src);
        assert!(errs.is_empty(),
            "clean module must have zero borrow errors, got: {:?}", errs);
    }

    #[test]
    fn tc_p18_unsafe_axon_suppresses_all() {
        // @unsafe_axon fn: injected violations are all suppressed
        use crate::hir::{HirExpr, HirExprKind, HirTy, MoveState, MaybeAlias, PlaceId};
        use crate::lexer::Span;

        let place = PlaceId(55);
        let mut checker = BorrowChecker::new();
        checker.in_unsafe_axon = true;
        checker.state.entry(place).or_default().move_state = Some(MoveState::Moved);

        checker.check_expr(&HirExpr {
            kind: HirExprKind::Place(place, MoveState::Owned),
            ty: HirTy::I32,
            span: Span::new(0, 1),
            node_id: crate::hir::NodeId(0),
            move_state: None,
            alias: MaybeAlias::Unknown,
        });

        // enforce() must NOT exit — only fatal (non-suppressed) errors trigger exit
        let fatal_count = checker.errors.iter().filter(|e| !e.suppressed).count();
        assert_eq!(fatal_count, 0,
            "unsafe_axon must suppress all violations, fatal count: {}", fatal_count);
        // But the error is still logged
        assert!(!checker.errors.is_empty(),
            "violations must still be logged under unsafe_axon");
    }

    // ── Phase 18 M3 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_double_mut_borrow() {
        // Second &mut on same place must emit DoubleMutBorrow
        use crate::hir::{HirExpr, HirExprKind, HirTy, MoveState, MaybeAlias, PlaceId, BorrowId};
        use crate::lexer::Span;

        let place = PlaceId(7);
        let span = Span::new(0, 1);
        let mut checker = BorrowChecker::new();
        checker.state.entry(place).or_default().move_state = Some(MoveState::Owned);

        // First &mut — should succeed (no error)
        let ref1 = HirExpr {
            kind: HirExprKind::Ref(true, place, BorrowId(1)),
            ty: HirTy::I32,
            span: span.clone(),
            node_id: crate::hir::NodeId(0),
            move_state: None,
            alias: MaybeAlias::Unknown,
        };
        checker.check_expr(&ref1);
        assert!(checker.errors.is_empty(),
            "first &mut must not error, got: {:?}", checker.errors);

        // Second &mut — must emit DoubleMutBorrow
        let ref2 = HirExpr {
            kind: HirExprKind::Ref(true, place, BorrowId(2)),
            ty: HirTy::I32,
            span: span.clone(),
            node_id: crate::hir::NodeId(0),
            move_state: None,
            alias: MaybeAlias::Unknown,
        };
        checker.check_expr(&ref2);
        assert!(
            checker.errors.iter().any(|e| e.kind == BorrowErrorKind::DoubleMutBorrow),
            "second &mut must emit DoubleMutBorrow, got: {:?}", checker.errors
        );
    }

    #[test]
    fn tc_shared_borrow_no_error() {
        // Two shared borrows on same place must not error
        use crate::hir::{HirExpr, HirExprKind, HirTy, MoveState, MaybeAlias, PlaceId, BorrowId};
        use crate::lexer::Span;

        let place = PlaceId(8);
        let span = Span::new(0, 1);
        let mut checker = BorrowChecker::new();
        checker.state.entry(place).or_default().move_state = Some(MoveState::Owned);

        for bid in [1u32, 2] {
            let expr = HirExpr {
                kind: HirExprKind::Ref(false, place, BorrowId(bid)),
                ty: HirTy::I32,
                span: span.clone(),
                node_id: crate::hir::NodeId(0),
                move_state: None,
                alias: MaybeAlias::Unknown,
            };
            checker.check_expr(&expr);
        }
        assert!(checker.errors.is_empty(),
            "shared borrows must not error, got: {:?}", checker.errors);
    }

    #[test]
    fn tc_use_while_mut_borrowed() {
        // Reading a place while it is mutably borrowed must emit UseWhileMutBorrowed
        use crate::hir::{HirExpr, HirExprKind, HirTy, MoveState, MaybeAlias, PlaceId, BorrowId};
        use crate::lexer::Span;

        let place = PlaceId(9);
        let span = Span::new(0, 1);
        let mut checker = BorrowChecker::new();

        // Mark place as already mut borrowed
        let ps = checker.state.entry(place).or_default();
        ps.move_state = Some(MoveState::MutBorrowed);
        ps.mut_borrows.push(BorrowId(1));

        // Try to read it
        let expr = HirExpr {
            kind: HirExprKind::Place(place, MoveState::Owned),
            ty: HirTy::I32,
            span: span.clone(),
            node_id: crate::hir::NodeId(0),
            move_state: None,
            alias: MaybeAlias::Unknown,
        };
        checker.check_expr(&expr);
        assert!(
            checker.errors.iter().any(|e| e.kind == BorrowErrorKind::UseWhileMutBorrowed),
            "must detect UseWhileMutBorrowed, got: {:?}", checker.errors
        );
    }

    // ── Phase 18 M2 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_use_after_move() {
        // Directly inject a Moved place into checker state and verify detection.
        use crate::hir::{HirExpr, HirExprKind, HirTy, HirLit, MoveState};
        use crate::hir::MaybeAlias;
        use crate::lexer::Span;
        use crate::hir::PlaceId;

        let place = PlaceId(42);
        let span = Span::new(0, 1);

        let mut checker = BorrowChecker::new();
        // Manually mark place as Moved
        checker.state.entry(place).or_default().move_state = Some(MoveState::Moved);

        // Construct a Place expr referencing the moved place
        let expr = HirExpr {
            kind: HirExprKind::Place(place, MoveState::Owned),
            ty: HirTy::I32,
            span: span.clone(),
            node_id: crate::hir::NodeId(0),
            move_state: None,
            alias: MaybeAlias::Unknown,
        };
        checker.check_expr(&expr);

        assert!(
            checker.errors.iter().any(|e| e.kind == BorrowErrorKind::UseAfterMove),
            "must detect UseAfterMove, got: {:?}", checker.errors
        );
    }

    #[test]
    fn tc_no_error_on_owned_place() {
        // Owned place must not trigger any error
        use crate::hir::{HirExpr, HirExprKind, HirTy, MoveState, MaybeAlias, PlaceId};
        use crate::lexer::Span;

        let place = PlaceId(1);
        let mut checker = BorrowChecker::new();
        checker.state.entry(place).or_default().move_state = Some(MoveState::Owned);

        let expr = HirExpr {
            kind: HirExprKind::Place(place, MoveState::Owned),
            ty: HirTy::I32,
            span: Span::new(0, 1),
            node_id: crate::hir::NodeId(0),
            move_state: None,
            alias: MaybeAlias::Unknown,
        };
        checker.check_expr(&expr);
        assert!(checker.errors.is_empty(),
            "owned place must have no errors, got: {:?}", checker.errors);
    }

    #[test]
    fn tc_use_after_move_suppressed_by_unsafe_axon() {
        // @unsafe_axon suppresses use-after-move — error is logged but suppressed=true
        use crate::hir::{HirExpr, HirExprKind, HirTy, MoveState, MaybeAlias, PlaceId};
        use crate::lexer::Span;

        let place = PlaceId(99);
        let mut checker = BorrowChecker::new();
        checker.in_unsafe_axon = true;
        checker.state.entry(place).or_default().move_state = Some(MoveState::Moved);

        let expr = HirExpr {
            kind: HirExprKind::Place(place, MoveState::Owned),
            ty: HirTy::I32,
            span: Span::new(0, 1),
            node_id: crate::hir::NodeId(0),
            move_state: None,
            alias: MaybeAlias::Unknown,
        };
        checker.check_expr(&expr);

        assert!(checker.errors.iter().all(|e| e.suppressed),
            "unsafe_axon must suppress all errors, got: {:?}", checker.errors);
        assert!(!checker.errors.is_empty(),
            "error must still be logged even when suppressed");
    }

    // ── Phase 18 M1 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_borrow_checker_init() {
        // Clean program — zero borrow errors
        let errs = check_src("fn add(x: i32, y: i32) -> i32 { return x; }");
        assert!(errs.is_empty(), "clean fn must have zero borrow errors, got: {:?}", errs);
    }

    #[test]
    fn tc_borrow_checker_multi_fn() {
        // Multiple clean fns — zero errors
        let src = r#"
            fn add(x: i32, y: i32) -> i32 { return x; }
            fn sub(x: i32, y: i32) -> i32 { return x; }
        "#;
        let errs = check_src(src);
        assert!(errs.is_empty(), "clean module must have zero borrow errors, got: {:?}", errs);
    }

    #[test]
    fn tc_borrow_error_display() {
        let place = PlaceId(0);
        let span = crate::lexer::Span::new(0, 1);
        let err = BorrowError::use_after_move(place, span);
        let s = format!("{}", err);
        assert!(s.contains("BorrowError"), "display must contain BorrowError");
        assert!(s.contains("moved"), "display must mention moved");
    }
}
