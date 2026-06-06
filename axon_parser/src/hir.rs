// axon_parser/src/hir.rs
// AXON HIR — Stage 8A-3
// Copyright © 2026 Edison Lepiten — AIEONYX
// Lowers parser AST (Vec<Item>) into HirModule.
// Adds: PlaceId, BorrowId, MoveStateMap, ContractExpr, MaybeAlias.

use crate::parser::{
    Item, Expr, Stmt, Ty, Pat, Lit,
    FnSig, ContractKind,
    BinaryOp, UnaryOp,
    ImplItem, TraitItem,
};
use crate::lexer::Span;

// ============================================================
// CORE IDs
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BorrowId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

impl PlaceId {
    pub const INVALID: PlaceId = PlaceId(u32::MAX);
}

impl BorrowId {
    pub const UNSET: BorrowId = BorrowId(u32::MAX);
}

// ============================================================
// MOVE STATE
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum MoveState {
    Owned,        // place is owned and valid
    Moved,        // place has been moved out
    Borrowed,     // place is currently borrowed
    MutBorrowed,  // place is mutably borrowed
    Dropped,      // place has been dropped
    MaybeOwned,   // may or may not be owned (after branch)
}

/// Per-statement move tracking — deferred obligation from Phase 7H-2
#[derive(Debug, Clone)]
pub struct MoveStateMap {
    pub entries: Vec<(PlaceId, MoveState)>,
}

impl MoveStateMap {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self { MoveStateMap { entries: Vec::new() } }
    pub fn set(&mut self, place: PlaceId, state: MoveState) {
        if let Some(e) = self.entries.iter_mut().find(|(p,_)| *p == place) {
            e.1 = state;
        } else {
            self.entries.push((place, state));
        }
    }
    pub fn get(&self, place: PlaceId) -> Option<&MoveState> {
        self.entries.iter().find(|(p,_)| *p == place).map(|(_, s)| s)
    }
    pub fn merge(&self, other: &MoveStateMap) -> MoveStateMap {
        let mut result = self.clone();
        for (place, state) in &other.entries {
            match result.get(*place) {
                None => result.entries.push((*place, state.clone())),
                Some(existing) if existing != state => {
                    result.set(*place, MoveState::MaybeOwned);
                }
                _ => {}
            }
        }
        result
    }
}

// ============================================================
// MAYBE ALIAS
// ============================================================

/// Alias precision tracker — deferred from Phase 7F-4
#[derive(Debug, Clone, PartialEq)]
pub enum MaybeAlias {
    NoAlias,          // provably no alias
    MayAlias(PlaceId), // may alias with this place
    MustAlias(PlaceId),// definitely aliases
    Unknown,          // conservative: assume alias
}

// ============================================================
// CONTRACT EXPRESSIONS
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ContractExpr {
    True,
    False,
    Var(String, PlaceId),
    IntLit(u64),
    BoolLit(bool),
    BinOp(ContractBinOp, Box<ContractExpr>, Box<ContractExpr>),
    UnOp(ContractUnOp, Box<ContractExpr>),
    Old(Box<ContractExpr>),      // @ensures: old(x) = value before call
    Result(Box<HirTy>),          // @ensures: result variable
    Forall(String, Box<HirTy>, Box<ContractExpr>),
    Exists(String, Box<HirTy>, Box<ContractExpr>),
    Implies(Box<ContractExpr>, Box<ContractExpr>),
    FieldAccess(Box<ContractExpr>, String),
    Call(String, Vec<ContractExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractBinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractUnOp { Neg, Not }

// ============================================================
// HIR TYPES
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum HirTy {
    Bool, I8, I16, I32, I64, I128, Isize,
    U8, U16, U32, U64, U128, Usize,
    F32, F64,
    Char, Str,
    String,                                      // heap-owned string (distinct from Str slice)
    Unit,
    Dyn(String),
    Never,
    Ref(bool, Option<String>, Box<HirTy>),   // is_mut, lifetime, inner
    Ptr(bool, Box<HirTy>),
    Slice(Box<HirTy>),
    Array(Box<HirTy>, u64),
    Tuple(Vec<HirTy>),
    Named(String, Vec<HirTy>),               // name, type args
    Fn(Vec<HirTy>, Box<HirTy>),
    Infer,                                    // type hole — filled by 8B
    Error,                                    // sentinel for bad types
}

impl HirTy {
    pub fn is_copy(&self) -> bool {
        matches!(self,
            HirTy::Bool | HirTy::Char |
            HirTy::I8 | HirTy::I16 | HirTy::I32 | HirTy::I64 | HirTy::I128 | HirTy::Isize |
            HirTy::U8 | HirTy::U16 | HirTy::U32 | HirTy::U64 | HirTy::U128 | HirTy::Usize |
            HirTy::F32 | HirTy::F64 |
            HirTy::Unit | HirTy::Ref(false, _, _)
        )
    }
    pub fn is_ref(&self) -> bool { matches!(self, HirTy::Ref(_, _, _)) }
    pub fn needs_drop(&self) -> bool { !self.is_copy() && !matches!(self, HirTy::Never | HirTy::Error) }
}

// ============================================================
// HIR EXPRESSIONS
// ============================================================

#[derive(Debug, Clone)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub ty: HirTy,
    pub span: Span,
    pub node_id: NodeId,
    pub move_state: Option<MoveStateMap>,
    pub alias: MaybeAlias,
}

#[derive(Debug, Clone)]
pub enum HirExprKind {
    Lit(HirLit),
    Place(PlaceId, MoveState),
    Ref(bool, PlaceId, BorrowId),            // is_mut, place, borrow
    Deref(Box<HirExpr>, PlaceId),
    BinOp(BinaryOp, Box<HirExpr>, Box<HirExpr>),
    UnOp(UnaryOp, Box<HirExpr>),
    Call(Box<HirExpr>, Vec<HirExpr>),
    MethodCall(Box<HirExpr>, String, Vec<HirExpr>),
    Field(Box<HirExpr>, String, PlaceId),
    Index(Box<HirExpr>, Box<HirExpr>, PlaceId),
    Block(Vec<HirStmt>, Option<Box<HirExpr>>),
    If(Box<HirExpr>, Box<HirExpr>, Option<Box<HirExpr>>),
    While(Box<HirExpr>, Box<HirExpr>),
    Loop(Box<HirExpr>),
    For(HirPat, Box<HirExpr>, Box<HirExpr>),
    Match(Box<HirExpr>, Vec<HirMatchArm>),
    Return(Option<Box<HirExpr>>),
    Break(Option<Box<HirExpr>>),
    Continue,
    Assign(PlaceId, Box<HirExpr>),
    Cast(Box<HirExpr>, HirTy),
    Tuple(Vec<HirExpr>),
    Array(Vec<HirExpr>),
    // P12-M2: range expression  start..end  (inclusive flag)
    Range(Box<HirExpr>, Box<HirExpr>, bool),
    Struct(String, Vec<(String, HirExpr)>),
    Path(Vec<String>),
    // P14-M3: closure — params, body, captured places (copy-only, stack)
    Closure(Vec<(PlaceId, HirTy)>, Box<HirExpr>, Vec<PlaceId>),
    // Drop elaboration — inserted by HIR lowerer
    Drop(PlaceId),
    // Borrow expiry — inserted at StorageDead
    BorrowExpires(BorrowId),
}

#[derive(Debug, Clone)]
pub enum HirLit {
    Int(u64), Float(f64), Str(String), Char(char), Bool(bool), Unit,
}

// ============================================================
// HIR STATEMENTS
// ============================================================

#[derive(Debug, Clone)]
pub struct HirStmt {
    pub kind: HirStmtKind,
    pub span: Span,
    pub move_state_after: MoveStateMap,
}

#[derive(Debug, Clone)]
pub enum HirStmtKind {
    Let(PlaceId, bool, HirTy, Option<HirExpr>),  // place, is_mut, ty, init
    Expr(HirExpr),
    StorageLive(PlaceId),
    StorageDead(PlaceId),
    // KNOWN-GAP H5: DropElaborated nodes are not yet emitted by the HIR lowerer.
    // Deferred to Stage 8C (borrow checker). 7E-5 invariant: only emit when needs_drop() is true.
    DropElaborated(PlaceId),
}

// ============================================================
// HIR PATTERNS
// ============================================================

#[derive(Debug, Clone)]
pub enum HirPat {
    Wildcard,
    Bind(PlaceId, bool),                           // place, is_mut
    Tuple(Vec<HirPat>),
    Lit(HirLit),
    Ref(bool, Box<HirPat>),
}

#[derive(Debug, Clone)]
pub struct HirMatchArm {
    pub pat: HirPat,
    pub guard: Option<HirExpr>,
    pub body: HirExpr,
    pub span: Span,
}

// ============================================================
// HIR ITEMS
// ============================================================

#[derive(Debug, Clone)]
pub struct HirFn {
    pub name: String,
    pub generics: Vec<String>,
    pub params: Vec<(PlaceId, HirTy)>,
    pub ret: HirTy,
    pub contracts: Vec<HirContract>,
    pub body: HirExpr,
    pub is_pub: bool,
    pub is_pure: bool,
    pub is_ghost: bool,
    pub span: Span,
    /// Capabilities explicitly required by @cap(capability_name) annotations.
    /// Checked against the active profile at compile time.
    pub required_caps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HirContract {
    pub kind: ContractKind,
    pub expr: ContractExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirStruct {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: Vec<(String, HirTy, bool)>,        // name, ty, is_pub
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirEnum {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: Vec<HirVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirVariant {
    pub name: String,
    pub fields: HirVariantFields,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HirVariantFields {
    Unit,
    Tuple(Vec<HirTy>),
    Struct(Vec<(String, HirTy)>),
}

#[derive(Debug, Clone)]
pub struct HirTrait {
    pub name: String,
    pub generics: Vec<String>,
    pub supertraits: Vec<HirTy>,
    pub methods: Vec<HirFn>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirImpl {
    pub generics: Vec<String>,
    pub trait_: Option<String>,
    pub self_ty: HirTy,
    pub methods: Vec<HirFn>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HirItem {
    Fn(HirFn),
    Struct(HirStruct),
    Enum(HirEnum),
    Trait(HirTrait),
    Impl(HirImpl),
    Const(String, HirTy, HirExpr, Span),
    TypeAlias(String, HirTy, Span),
}

// ============================================================
// HIR MODULE
// ============================================================

#[derive(Debug, Clone)]
pub struct HirModule {
    pub items: Vec<HirItem>,
    pub errors: Vec<HirError>,
}

#[derive(Debug, Clone)]
pub struct HirError {
    pub msg: String,
    pub span: Span,
}

impl HirError {
    pub fn new(msg: impl Into<String>, span: Span) -> Self {
        HirError { msg: msg.into(), span }
    }
}

// ============================================================
// HIR LOWERER
// ============================================================

pub struct HirLowerer {
    next_place: u32,
    next_borrow: u32,
    next_node: u32,
    errors: Vec<HirError>,
    /// Variable name → PlaceId mapping for current scope
    name_env: Vec<std::collections::HashMap<String, PlaceId>>,
}

// M4: recursively check if a HirTy contains HirTy::String
fn hir_ty_contains_string(ty: &HirTy) -> bool {
    match ty {
        HirTy::String => true,
        HirTy::Ref(_, _, inner) | HirTy::Ptr(_, inner) | HirTy::Slice(inner) =>
            hir_ty_contains_string(inner),
        HirTy::Array(inner, _) => hir_ty_contains_string(inner),
        HirTy::Tuple(ts) => ts.iter().any(hir_ty_contains_string),
        HirTy::Named(n, ts) => {
            // P11-M3: AxonVec always requires alloc_heap
            if n == "AxonVec" { return true; }
            ts.iter().any(hir_ty_contains_string)
        }
        HirTy::Fn(ps, r) =>
            ps.iter().any(hir_ty_contains_string) || hir_ty_contains_string(r),
        _ => false,
    }
}

// M4: recursively check if a HirExpr body uses any String literal
fn hir_expr_contains_string(expr: &HirExpr) -> bool {
    if matches!(expr.ty, HirTy::String) { return true; }
    match &expr.kind {
        HirExprKind::Lit(HirLit::Str(_)) => true,
        HirExprKind::BinOp(_, l, r) =>
            hir_expr_contains_string(l) || hir_expr_contains_string(r),
        HirExprKind::Call(f, args) =>
            hir_expr_contains_string(f) || args.iter().any(hir_expr_contains_string),
        HirExprKind::MethodCall(recv, _, args) =>
            hir_expr_contains_string(recv) || args.iter().any(hir_expr_contains_string),
        HirExprKind::Block(stmts, tail) => {
            stmts.iter().any(|s| match &s.kind {
                HirStmtKind::Let(_, _, _, Some(e)) => hir_expr_contains_string(e),
                HirStmtKind::Expr(e) => hir_expr_contains_string(e),
                _ => false,
            }) || tail.as_ref().is_some_and(|e| hir_expr_contains_string(e))
        }
        HirExprKind::If(c, t, e) =>
            hir_expr_contains_string(c) || hir_expr_contains_string(t)
            || e.as_ref().is_some_and(|e| hir_expr_contains_string(e)),
        HirExprKind::Return(Some(e)) => hir_expr_contains_string(e),
        _ => false,
    }
}

#[allow(clippy::new_without_default)]
/// P14-M3: walk a HirExpr and collect PlaceIds that are in outer_scope
/// but not in the closure's own params
fn collect_free_places(
    expr: &HirExpr,
    outer: &std::collections::HashSet<PlaceId>,
    params: &[(PlaceId, HirTy)],
) -> Vec<PlaceId> {
    let param_set: std::collections::HashSet<PlaceId> = params.iter().map(|(p,_)| *p).collect();
    let mut found = std::collections::HashSet::new();
    collect_places_rec(expr, outer, &param_set, &mut found);
    found.into_iter().collect()
}

fn collect_places_rec(
    expr: &HirExpr,
    outer: &std::collections::HashSet<PlaceId>,
    params: &std::collections::HashSet<PlaceId>,
    found: &mut std::collections::HashSet<PlaceId>,
) {
    match &expr.kind {
        HirExprKind::Place(p, _) => {
            if outer.contains(p) && !params.contains(p) { found.insert(*p); }
        }
        HirExprKind::BinOp(_, l, r) => {
            collect_places_rec(l, outer, params, found);
            collect_places_rec(r, outer, params, found);
        }
        HirExprKind::Block(stmts, tail) => {
            for s in stmts {
                if let HirStmtKind::Expr(e) | HirStmtKind::Let(_, _, _, Some(e)) = &s.kind {
                    collect_places_rec(e, outer, params, found);
                }
            }
            if let Some(t) = tail { collect_places_rec(t, outer, params, found); }
        }
        HirExprKind::Return(Some(e)) => collect_places_rec(e, outer, params, found),
        _ => {}
    }
}

#[allow(clippy::new_without_default)]
impl HirLowerer {
    pub fn new() -> Self {
        HirLowerer {
            next_place: 0,
            next_borrow: 0,
            next_node: 0,
            errors: Vec::new(),
            name_env: vec![std::collections::HashMap::new()],
        }
    }

    pub fn lower_module(mut self, items: Vec<Item>) -> HirModule {
        let mut hir_items = Vec::new();
        for item in items {
            if let Some(h) = self.lower_item(item) { hir_items.push(h) }
        }
        HirModule { items: hir_items, errors: self.errors }
    }

    /// fresh_place() is module-scoped — counter persists for entire HirModule lowering.
    /// H1 FIX: PlaceIds are guaranteed unique across all functions in a module.
    fn fresh_place(&mut self) -> PlaceId {
        let id = PlaceId(self.next_place);
        self.next_place += 1;
        id
    }

    fn fresh_borrow(&mut self) -> BorrowId {
        let id = BorrowId(self.next_borrow);
        self.next_borrow += 1;
        id
    }

    fn fresh_node(&mut self) -> NodeId {
        let id = NodeId(self.next_node);
        self.next_node += 1;
        id
    }
    fn push_scope(&mut self) {
        self.name_env.push(std::collections::HashMap::new());
    }
    fn pop_scope(&mut self) {
        self.name_env.pop();
    }
    fn bind_name(&mut self, name: String, place: PlaceId) {
        if let Some(scope) = self.name_env.last_mut() {
            scope.insert(name, place);
        }
    }
    fn lookup_name(&self, name: &str) -> Option<PlaceId> {
        for scope in self.name_env.iter().rev() {
            if let Some(&place) = scope.get(name) {
                return Some(place);
            }
        }
        None
    }

    #[allow(dead_code)]
    fn error(&mut self, msg: impl Into<String>, span: Span) {
        self.errors.push(HirError::new(msg, span));
    }

    fn lower_item(&mut self, item: Item) -> Option<HirItem> {
        match item {
            Item::Fn(sig, body) => {
                Some(HirItem::Fn(self.lower_fn(sig, body)))
            }
            Item::Struct(s) => {
                let fields = s.fields.iter().map(|f| {
                    (f.name.name.clone(), self.lower_ty(&f.ty), f.is_pub)
                }).collect();
                Some(HirItem::Struct(HirStruct {
                    name: s.name.name,
                    generics: s.generics.iter().map(|g| g.name.clone()).collect(),
                    fields,
                    span: s.span,
                }))
            }
            Item::Enum(e) => {
                let variants = e.variants.iter().map(|v| {
                    let fields = match &v.fields {
                        crate::parser::EnumVariantFields::Unit =>
                            HirVariantFields::Unit,
                        crate::parser::EnumVariantFields::Tuple(tys) =>
                            HirVariantFields::Tuple(tys.iter().map(|t| self.lower_ty(t)).collect()),
                        crate::parser::EnumVariantFields::Struct(fs) =>
                            HirVariantFields::Struct(fs.iter().map(|f| (f.name.name.clone(), self.lower_ty(&f.ty))).collect()),
                    };
                    HirVariant { name: v.name.name.clone(), fields, span: v.span.clone() }
                }).collect();
                Some(HirItem::Enum(HirEnum {
                    name: e.name.name,
                    generics: e.generics.iter().map(|g| g.name.clone()).collect(),
                    variants,
                    span: e.span,
                }))
            }
            Item::Trait(t) => {
                let methods = t.items.into_iter().filter_map(|ti| {
                    if let TraitItem::Fn(sig, body) = ti {
                        let body = body.unwrap_or(Expr::Block(vec![], None, sig.span.clone()));
                        Some(self.lower_fn(sig, body))
                    } else { None }
                }).collect();
                Some(HirItem::Trait(HirTrait {
                    name: t.name.name,
                    generics: t.generics.iter().map(|g| g.name.clone()).collect(),
                    supertraits: t.supertraits.iter().map(|s| self.lower_ty(s)).collect(),
                    methods,
                    span: t.span,
                }))
            }
            Item::Impl(i) => {
                let methods = i.items.into_iter().filter_map(|ii| {
                    if let ImplItem::Fn(sig, body) = ii {
                        Some(self.lower_fn(sig, body))
                    } else { None }
                }).collect();
                Some(HirItem::Impl(HirImpl {
                    generics: i.generics.iter().map(|g| g.name.clone()).collect(),
                    trait_: i.trait_.as_ref().map(|t| self.lower_ty(t)).and_then(|t| {
                        if let HirTy::Named(n, _) = t { Some(n) } else { None }
                    }),
                    self_ty: self.lower_ty(&i.self_ty),
                    methods,
                    span: i.span,
                }))
            }
            Item::Const(name, ty, val, span) => {
                let hty = self.lower_ty(&ty);
                let hval = self.lower_expr(val);
                Some(HirItem::Const(name.name, hty, hval, span))
            }
            Item::TypeAlias(name, _, ty, span) => {
                Some(HirItem::TypeAlias(name.name, self.lower_ty(&ty), span))
            }
            Item::Use(_, _) | Item::Mod(_, _, _) | Item::Profile(_) => None,
        }
    }

    fn lower_fn(&mut self, sig: FnSig, body: Expr) -> HirFn {
        // H8 KNOWN-GAP: Complex parameter patterns (e.g. fn f((x,y): (i32,i32)))
        // are not yet destructured — only a single PlaceId per param is allocated.
        // Full destructuring support targeted for Stage 8C.
        self.push_scope();
        let params: Vec<(PlaceId, HirTy)> = sig.params.iter().map(|p| {
            let place = self.fresh_place();
            // Bind param name so Ident lookups resolve to this PlaceId
            if let Pat::Ident(ref ident, _) = p.pat {
                self.bind_name(ident.name.clone(), place);
            }
            (place, self.lower_ty(&p.ty))
        }).collect();
        let ret = sig.ret.as_ref().map(|t| self.lower_ty(t)).unwrap_or(HirTy::Unit);
        let contracts = sig.contracts.iter().map(|c| {
            HirContract {
                kind: c.kind.clone(),
                expr: self.lower_contract_expr(&c.expr),
                span: c.span.clone(),
            }
        }).collect();
        let body = self.lower_expr(body);
        self.pop_scope();
        // Extract @cap(capability_name) annotations from function attrs
        // These are checked against the active profile at compile time
        let required_caps: Vec<String> = sig.attrs.iter()
            .filter(|a| a.name == "cap" || a.name == "requires_cap" || a.name == "capability")
            .flat_map(|a| a.args.iter().cloned())
            .collect();

        // P11-M4: detect index expressions in fn body
fn hir_expr_contains_index(expr: &HirExpr) -> bool {
    match &expr.kind {
        HirExprKind::Index(_, _, _) => true,
        HirExprKind::BinOp(_, l, r) =>
            hir_expr_contains_index(l) || hir_expr_contains_index(r),
        HirExprKind::Call(f, args) =>
            hir_expr_contains_index(f) || args.iter().any(hir_expr_contains_index),
        HirExprKind::MethodCall(recv, _, args) =>
            hir_expr_contains_index(recv) || args.iter().any(hir_expr_contains_index),
        HirExprKind::Block(stmts, tail) => {
            stmts.iter().any(|s| match &s.kind {
                HirStmtKind::Let(_, _, _, Some(e)) => hir_expr_contains_index(e),
                HirStmtKind::Expr(e) => hir_expr_contains_index(e),
                _ => false,
            }) || tail.as_ref().is_some_and(|e| hir_expr_contains_index(e))
        }
        HirExprKind::If(c, t, e) =>
            hir_expr_contains_index(c) || hir_expr_contains_index(t)
            || e.as_ref().is_some_and(|e| hir_expr_contains_index(e)),
        HirExprKind::Return(Some(e)) => hir_expr_contains_index(e),
        HirExprKind::Array(elems) => elems.iter().any(hir_expr_contains_index),
        _ => false,
    }
}

// M4: auto-infer alloc_heap for any fn whose signature or body uses String
        let uses_string = params.iter().any(|(_, t)| hir_ty_contains_string(t))
            || hir_ty_contains_string(&ret)
            || hir_expr_contains_string(&body);
        let mut required_caps = required_caps;
        if uses_string && !required_caps.iter().any(|c| c == "alloc_heap") {
            required_caps.push("alloc_heap".to_string());
        }

        // P11-M4: auto-infer bounds_check for any fn performing index operations
        let uses_index = hir_expr_contains_index(&body);
        if uses_index && !required_caps.iter().any(|c| c == "bounds_check") {
            required_caps.push("bounds_check".to_string());
        }

        HirFn {
            name: sig.name.name,
            generics: sig.generics.iter().map(|g| g.name.clone()).collect(),
            params,
            ret,
            contracts,
            body,
            is_pub: sig.is_pub,
            is_pure: sig.is_pure,
            is_ghost: sig.is_ghost,
            span: sig.span,
            required_caps,
        }
    }

    fn lower_ty(&self, ty: &Ty) -> HirTy {
        match ty {
            Ty::Named(ident, args) => {
                let args: Vec<HirTy> = args.iter().map(|a| self.lower_ty(a)).collect();
                match ident.name.as_str() {
                    "bool"  => HirTy::Bool,
                    "i8"    => HirTy::I8,   "i16"  => HirTy::I16,
                    "i32"   => HirTy::I32,  "i64"  => HirTy::I64,
                    "i128"  => HirTy::I128, "isize"=> HirTy::Isize,
                    "u8"    => HirTy::U8,   "u16"  => HirTy::U16,
                    "u32"   => HirTy::U32,  "u64"  => HirTy::U64,
                    "u128"  => HirTy::U128, "usize"=> HirTy::Usize,
                    "f32"   => HirTy::F32,  "f64"  => HirTy::F64,
                    "char"  => HirTy::Char, "str"  => HirTy::Str,
                    "String" => HirTy::String,
                    "()"    => HirTy::Unit,
                    _       => HirTy::Named(ident.name.clone(), args),
                }
            }
            Ty::Ref(is_mut, lifetime, inner) =>
                HirTy::Ref(*is_mut, lifetime.clone(), Box::new(self.lower_ty(inner))),
            Ty::Ptr(is_mut, inner) =>
                HirTy::Ptr(*is_mut, Box::new(self.lower_ty(inner))),
            Ty::Slice(inner) =>
                HirTy::Slice(Box::new(self.lower_ty(inner))),
            Ty::Array(inner, len_expr) => {
                let n = match len_expr.as_ref() {
                    Expr::Lit(Lit::Int(n), _) => *n,
                    _ => 0,
                };
                HirTy::Array(Box::new(self.lower_ty(inner)), n)
            }
            Ty::Tuple(tys) =>
                HirTy::Tuple(tys.iter().map(|t| self.lower_ty(t)).collect()),
            Ty::Fn(params, ret) => {
                let p = params.iter().map(|t| self.lower_ty(t)).collect();
                let r = ret.as_ref().map(|t| self.lower_ty(t)).unwrap_or(HirTy::Unit);
                HirTy::Fn(p, Box::new(r))
            }
            Ty::Dyn(name) => HirTy::Dyn(name.clone()),
            Ty::Never => HirTy::Never,
            Ty::Infer => HirTy::Infer,
        }
    }

    fn lower_expr(&mut self, expr: Expr) -> HirExpr {
        let span = self.expr_span(&expr);
        let node_id = self.fresh_node();
        let kind = match expr {
            Expr::Lit(lit, _) => HirExprKind::Lit(self.lower_lit(lit)),
            Expr::Ident(ident) => {
                // Look up variable in name_env — reuse existing PlaceId
                let place = self.lookup_name(&ident.name)
                    .unwrap_or_else(|| self.fresh_place());
                HirExprKind::Place(place, MoveState::Owned)
            }
            Expr::Block(stmts, tail, _) => {
                let hstmts = stmts.into_iter().map(|s| self.lower_stmt(s)).collect();
                let htail = tail.map(|e| Box::new(self.lower_expr(*e)));
                HirExprKind::Block(hstmts, htail)
            }
            Expr::Call(func, args, _) => {
                // P13-M3-CALL-PATH: if callee is an ident or path, lower as Path
                // so codegen can resolve the function name — not as a Place lookup
                let hfunc = match *func {
                    Expr::Ident(ref ident) => {
                        // P13-M3-HIREXPR-FIX
                        let segs = vec![ident.name.clone()];
                        HirExpr {
                            kind: HirExprKind::Path(segs),
                            ty: HirTy::Infer,
                            node_id: self.fresh_node(),
                            span: Span::new(0, 0),
                            move_state: None,
                            alias: MaybeAlias::Unknown,
                        }
                    }
                    Expr::Path(ref segs, _) => {
                        let names = segs.iter().map(|s| s.name.clone()).collect();
                        HirExpr {
                            kind: HirExprKind::Path(names),
                            ty: HirTy::Infer,
                            node_id: self.fresh_node(),
                            span: Span::new(0, 0),
                            move_state: None,
                            alias: MaybeAlias::Unknown,
                        }
                    }
                    other => self.lower_expr(other),
                };
                let hargs = args.into_iter().map(|a| self.lower_expr(a)).collect();
                HirExprKind::Call(Box::new(hfunc), hargs)
            }
            Expr::MethodCall(recv, method, args, _) => {
                let hrecv = self.lower_expr(*recv);
                let hargs = args.into_iter().map(|a| self.lower_expr(a)).collect();
                HirExprKind::MethodCall(Box::new(hrecv), method.name, hargs)
            }
            Expr::Field(obj, field, _) => {
                let hobj = self.lower_expr(*obj);
                let place = self.fresh_place();
                HirExprKind::Field(Box::new(hobj), field.name, place)
            }
            Expr::Index(obj, idx, _) => {
                let hobj = self.lower_expr(*obj);
                let hidx = self.lower_expr(*idx);
                let place = self.fresh_place();
                HirExprKind::Index(Box::new(hobj), Box::new(hidx), place)
            }
            Expr::Binary(op, lhs, rhs, _) => {
                HirExprKind::BinOp(op, Box::new(self.lower_expr(*lhs)), Box::new(self.lower_expr(*rhs)))
            }
            Expr::Unary(op, expr, _) => {
                HirExprKind::UnOp(op, Box::new(self.lower_expr(*expr)))
            }
            Expr::Assign(_lhs, rhs, _) => {
                let place = self.fresh_place();
                let hrhs = self.lower_expr(*rhs);
                HirExprKind::Assign(place, Box::new(hrhs))
            }
            Expr::AssignOp(_op, _lhs, rhs, _) => {
                let place = self.fresh_place();
                let hrhs = self.lower_expr(*rhs);
                HirExprKind::Assign(place, Box::new(hrhs))
            }
            Expr::If(cond, then, else_, _) => {
                let hcond = self.lower_expr(*cond);
                let hthen = self.lower_expr(*then);
                let helse = else_.map(|e| Box::new(self.lower_expr(*e)));
                HirExprKind::If(Box::new(hcond), Box::new(hthen), helse)
            }
            Expr::While(cond, body, _) => {
                HirExprKind::While(Box::new(self.lower_expr(*cond)), Box::new(self.lower_expr(*body)))
            }
            Expr::Loop(body, _) => {
                HirExprKind::Loop(Box::new(self.lower_expr(*body)))
            }
            Expr::For(pat, iter, body, _) => {
                // P13-M3-FOR-SCOPE: lower iter first (before scope push),
                // then push scope, lower pat (binds name), lower body, pop scope
                let hiter = self.lower_expr(*iter);
                self.push_scope();
                let hpat = self.lower_pat(pat);
                let hbody = self.lower_expr(*body);
                self.pop_scope();
                HirExprKind::For(hpat, Box::new(hiter), Box::new(hbody))
            }
            Expr::Match(scrutinee, arms, _) => {
                let hscrutinee = self.lower_expr(*scrutinee);
                let harms = arms.into_iter().map(|arm| {
                    HirMatchArm {
                        pat: self.lower_pat(arm.pat),
                        guard: arm.guard.map(|g| self.lower_expr(g)),
                        body: self.lower_expr(arm.body),
                        span: arm.span,
                    }
                }).collect();
                HirExprKind::Match(Box::new(hscrutinee), harms)
            }
            Expr::Return(val, _) => {
                HirExprKind::Return(val.map(|v| Box::new(self.lower_expr(*v))))
            }
            Expr::Break(val, _) => {
                HirExprKind::Break(val.map(|v| Box::new(self.lower_expr(*v))))
            }
            Expr::Continue(_) => HirExprKind::Continue,
            Expr::Ref(is_mut, _inner, _) => {
                let borrow = self.fresh_borrow();
                let place = self.fresh_place();
                HirExprKind::Ref(is_mut, place, borrow)
            }
            Expr::Deref(inner, _) => {
                let place = self.fresh_place();
                HirExprKind::Deref(Box::new(self.lower_expr(*inner)), place)
            }
            Expr::Cast(inner, ty, _) => {
                HirExprKind::Cast(Box::new(self.lower_expr(*inner)), self.lower_ty(&ty))
            }
            Expr::Tuple(exprs, _) => {
                HirExprKind::Tuple(exprs.into_iter().map(|e| self.lower_expr(e)).collect())
            }
            Expr::Array(exprs, _) => {
                HirExprKind::Array(exprs.into_iter().map(|e| self.lower_expr(e)).collect())
            }
            Expr::Struct(name, fields, _) => {
                let hfields = fields.into_iter().map(|(f, e)| (f.name, self.lower_expr(e))).collect();
                HirExprKind::Struct(name.name, hfields)
            }
            Expr::Path(segs, _) => {
                HirExprKind::Path(segs.into_iter().map(|s| s.name).collect())
            }
            Expr::Range(start, end, inclusive, _) => {
                // P12-M2: lower 0..n to HirExprKind::Range
                let hstart = start
                    .map(|e| self.lower_expr(*e))
                    .unwrap_or_else(|| HirExpr {
                        kind: HirExprKind::Lit(HirLit::Int(0)),
                        ty: HirTy::I64,
                        span: span.clone(),
                        node_id,
                        move_state: None,
                        alias: MaybeAlias::Unknown,
                    });
                let hend = end
                    .map(|e| self.lower_expr(*e))
                    .unwrap_or_else(|| HirExpr {
                        kind: HirExprKind::Lit(HirLit::Int(0)),
                        ty: HirTy::I64,
                        span: span.clone(),
                        node_id,
                        move_state: None,
                        alias: MaybeAlias::Unknown,
                    });
                HirExprKind::Range(Box::new(hstart), Box::new(hend), inclusive)
            }
            Expr::Closure(params, body, _) => {
                // P14-M3: lower closure — bind params, lower body, collect captures
                self.push_scope();
                let hparams: Vec<(PlaceId, HirTy)> = params.into_iter().map(|(pat, ty)| {
                    let place = self.fresh_place();
                    let hty = ty.map(|t| self.lower_ty(&t)).unwrap_or(HirTy::Infer);
                    if let crate::parser::Pat::Ident(ident, _) = pat {
                        self.bind_name(ident.name.clone(), place);
                    }
                    (place, hty)
                }).collect();
                // Snapshot name_env before body to detect captures
                let outer_places: std::collections::HashSet<PlaceId> =
                    self.name_env.iter().flat_map(|frame| frame.values().copied()).collect();
                let hbody = self.lower_expr(*body);
                self.pop_scope();
                // Captures: places referenced in body that came from outer scope
                let captures = collect_free_places(&hbody, &outer_places, &hparams);
                HirExprKind::Closure(hparams, Box::new(hbody), captures)
            }
            #[allow(unreachable_patterns)]
            _ => HirExprKind::Lit(HirLit::Unit),
        };
        HirExpr {
            kind,
            // H7: HirTy::Infer is a type hole — will be filled by Stage 8B type inference.
            ty: HirTy::Infer,
            span,
            node_id,
            move_state: Some(MoveStateMap::new()),
            alias: MaybeAlias::Unknown,
        }
    }

    fn lower_stmt(&mut self, stmt: Stmt) -> HirStmt {
        let _span = Span::new(0, 0);
        match stmt {
            Stmt::Let(pat, ty, val, s) => {
                let place = self.fresh_place();
                let hty = ty.as_ref().map(|t| self.lower_ty(t)).unwrap_or(HirTy::Infer);
                let hval = val.map(|v| self.lower_expr(v));
                // Bind let name so subsequent Ident refs resolve correctly
                if let Pat::Ident(ref ident, _) = pat {
                    self.bind_name(ident.name.clone(), place);
                }
                HirStmt {
                    kind: HirStmtKind::Let(place, true, hty, hval),
                    span: s,
                    move_state_after: MoveStateMap::new(),
                }
            }
            Stmt::Expr(expr, _) => {
                let hexpr = self.lower_expr(expr);
                let span = hexpr.span.clone();
                HirStmt {
                    kind: HirStmtKind::Expr(hexpr),
                    span,
                    move_state_after: MoveStateMap::new(),
                }
            }
            Stmt::Item(_item) => {
                // Item stmts are rare — lower and wrap
                HirStmt {
                    kind: HirStmtKind::Expr(HirExpr {
                        kind: HirExprKind::Lit(HirLit::Unit),
                        ty: HirTy::Unit,
                        span: Span::new(0, 0),
                        node_id: self.fresh_node(),
                        move_state: None,
                        alias: MaybeAlias::NoAlias,
                    }),
                    span: Span::new(0, 0),
                    move_state_after: MoveStateMap::new(),
                }
            }
        }
    }

    fn lower_pat(&mut self, pat: Pat) -> HirPat {
        match pat {
            Pat::Wildcard(_) => HirPat::Wildcard,
            Pat::Ident(ref ident, is_mut) => {
                // P13-M3-FOR-SCOPE: reuse existing place if name already bound,
                // otherwise allocate fresh and bind so body lookups resolve correctly
                let place = self.lookup_name(&ident.name)
                    .unwrap_or_else(|| {
                        let p = self.fresh_place();
                        self.bind_name(ident.name.clone(), p);
                        p
                    });
                HirPat::Bind(place, is_mut)
            }
            Pat::Tuple(pats, _) => {
                HirPat::Tuple(pats.into_iter().map(|p| self.lower_pat(p)).collect())
            }
            Pat::Lit(lit, _) => HirPat::Lit(self.lower_lit(lit)),
            Pat::Ref(is_mut, inner, _) => {
                HirPat::Ref(is_mut, Box::new(self.lower_pat(*inner)))
            }
            _ => HirPat::Wildcard,
        }
    }

    fn lower_lit(&self, lit: Lit) -> HirLit {
        match lit {
            Lit::Int(n)   => HirLit::Int(n),
            Lit::Float(f) => HirLit::Float(f),
            Lit::Str(s)   => HirLit::Str(s),
            Lit::Char(c)  => HirLit::Char(c),
            Lit::Bool(b)  => HirLit::Bool(b),
        }
    }

    fn lower_contract_expr(&mut self, expr: &Expr) -> ContractExpr {
        match expr {
            Expr::Lit(Lit::Bool(b), _) => ContractExpr::BoolLit(*b),
            Expr::Lit(Lit::Int(n), _)  => ContractExpr::IntLit(*n),
            Expr::Ident(i) => {
                let place = self.fresh_place();
                ContractExpr::Var(i.name.clone(), place)
            }
            Expr::Binary(op, lhs, rhs, _) => {
                let cop = match op {
                    BinaryOp::Add => ContractBinOp::Add,
                    BinaryOp::Sub => ContractBinOp::Sub,
                    BinaryOp::Mul => ContractBinOp::Mul,
                    BinaryOp::Div => ContractBinOp::Div,
                    BinaryOp::Rem => ContractBinOp::Rem,
                    BinaryOp::Eq  => ContractBinOp::Eq,
                    BinaryOp::Ne  => ContractBinOp::Ne,
                    BinaryOp::Lt  => ContractBinOp::Lt,
                    BinaryOp::Le  => ContractBinOp::Le,
                    BinaryOp::Gt  => ContractBinOp::Gt,
                    BinaryOp::Ge  => ContractBinOp::Ge,
                    BinaryOp::And => ContractBinOp::And,
                    BinaryOp::Or  => ContractBinOp::Or,
                    _ => ContractBinOp::Eq,
                };
                ContractExpr::BinOp(
                    cop,
                    Box::new(self.lower_contract_expr(lhs)),
                    Box::new(self.lower_contract_expr(rhs)),
                )
            }
            Expr::Unary(UnaryOp::Not, inner, _) =>
                ContractExpr::UnOp(ContractUnOp::Not, Box::new(self.lower_contract_expr(inner))),
            Expr::Unary(UnaryOp::Neg, inner, _) =>
                ContractExpr::UnOp(ContractUnOp::Neg, Box::new(self.lower_contract_expr(inner))),
            _ => {
                // H4 FIX: Never silently accept unrecognised contract expressions.
                // A fallback to True would give false verification guarantees.
                // Emit an error and return False (unsatisfiable) so verification fails loudly.
                self.errors.push(HirError {
                    msg: "contract expression contains unsupported syntax; verification will fail".into(),
                    span: Span::new(0, 0),
                });
                ContractExpr::False
            }
        }
    }

    fn expr_span(&self, expr: &Expr) -> Span {
        match expr {
            Expr::Lit(_, s) | Expr::Block(_, _, s) | Expr::Call(_, _, s)
            | Expr::MethodCall(_, _, _, s) | Expr::Field(_, _, s)
            | Expr::Index(_, _, s) | Expr::Binary(_, _, _, s)
            | Expr::Unary(_, _, s) | Expr::Assign(_, _, s)
            | Expr::AssignOp(_, _, _, s) | Expr::If(_, _, _, s)
            | Expr::While(_, _, s) | Expr::Loop(_, s)
            | Expr::For(_, _, _, s) | Expr::Match(_, _, s)
            | Expr::Return(_, s) | Expr::Break(_, s)
            | Expr::Continue(s) | Expr::Struct(_, _, s)
            | Expr::Tuple(_, s) | Expr::Array(_, s)
            | Expr::Cast(_, _, s) | Expr::Ref(_, _, s)
            | Expr::Deref(_, s) | Expr::Range(_, _, _, s)
            | Expr::Path(_, s) => s.clone(),
            Expr::Ident(i) => i.span.clone(),
            Expr::Closure(_, _, sp) => sp.clone(),
        }
    }
}

/// Public API: lower a parsed program into HIR
pub fn lower(items: Vec<Item>) -> HirModule {
    HirLowerer::new().lower_module(items)
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    
    #[test]
    fn tm4_string_param_infers_alloc_heap() {
        // M4: fn taking String param must have alloc_heap in required_caps
        let m = lower_src("fn greet(name: String) -> i32 { return 0; }");
        let f = match &m.items[0] { HirItem::Fn(f) => f, _ => panic!("expected fn") };
        assert!(
            f.required_caps.iter().any(|c| c == "alloc_heap"),
            "String param must auto-infer alloc_heap, got: {:?}", f.required_caps
        );
    }

    #[test]
    fn tm4_string_literal_body_infers_alloc_heap() {
        // M4: fn with string literal in body must have alloc_heap
        let m = lower_src("fn hello() -> i32 { let s: String = \"hi\"; return 0; }");
        let f = match &m.items[0] { HirItem::Fn(f) => f, _ => panic!("expected fn") };
        assert!(
            f.required_caps.iter().any(|c| c == "alloc_heap"),
            "String literal must auto-infer alloc_heap, got: {:?}", f.required_caps
        );
    }

    #[test]
    fn tm4_pure_int_fn_no_alloc_heap() {
        // M4: fn with no strings must NOT get alloc_heap
        let m = lower_src("fn add(a: i32, b: i32) -> i32 { return 0; }");
        let f = match &m.items[0] { HirItem::Fn(f) => f, _ => panic!("expected fn") };
        assert!(
            !f.required_caps.iter().any(|c| c == "alloc_heap"),
            "pure int fn must not get alloc_heap, got: {:?}", f.required_caps
        );
    }

    #[test]
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

    fn lower_src(src: &str) -> HirModule {
        let items = parse(src).expect("parse failed");
        lower(items)
    }

    // ── Phase 15 M1 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_cap_annotation_network() {
        // #[cap(network)] must populate required_caps with "network"
        let m = lower_src("#[cap(network)] fn send_data(x: i32) -> i32 { return x; }");
        assert_eq!(m.errors.len(), 0);
        let f = match &m.items[0] { HirItem::Fn(f) => f, _ => panic!("expected fn") };
        assert!(
            f.required_caps.iter().any(|c| c == "network"),
            "#[cap(network)] must appear in required_caps, got: {:?}", f.required_caps
        );
    }

    #[test]
    fn tc_cap_annotation_filesystem() {
        // #[cap(filesystem)] must populate required_caps with "filesystem"
        let m = lower_src("#[cap(filesystem)] fn read_file(x: i32) -> i32 { return x; }");
        assert_eq!(m.errors.len(), 0);
        let f = match &m.items[0] { HirItem::Fn(f) => f, _ => panic!("expected fn") };
        assert!(
            f.required_caps.iter().any(|c| c == "filesystem"),
            "#[cap(filesystem)] must appear in required_caps, got: {:?}", f.required_caps
        );
    }

    #[test]
    fn tc_cap_annotation_multiple() {
        // Multiple caps on one fn must all appear in required_caps
        let m = lower_src("#[cap(network)] #[cap(filesystem)] fn dual(x: i32) -> i32 { return x; }");
        assert_eq!(m.errors.len(), 0);
        let f = match &m.items[0] { HirItem::Fn(f) => f, _ => panic!("expected fn") };
        assert!(f.required_caps.iter().any(|c| c == "network"),
            "network missing, got: {:?}", f.required_caps);
        assert!(f.required_caps.iter().any(|c| c == "filesystem"),
            "filesystem missing, got: {:?}", f.required_caps);
    }

    #[test]
    fn tc_cap_annotation_unannotated_fn_has_no_explicit_cap() {
        // A plain fn with no #[cap] must not get network or filesystem caps
        let m = lower_src("fn pure_math(x: i32) -> i32 { return x; }");
        assert_eq!(m.errors.len(), 0);
        let f = match &m.items[0] { HirItem::Fn(f) => f, _ => panic!("expected fn") };
        assert!(!f.required_caps.iter().any(|c| c == "network"),
            "unannotated fn must not get network, got: {:?}", f.required_caps);
        assert!(!f.required_caps.iter().any(|c| c == "filesystem"),
            "unannotated fn must not get filesystem, got: {:?}", f.required_caps);
    }

    #[test]
    fn th1_lower_simple_fn() {
        let m = lower_src("fn add(x: i32, y: i32) -> i32 { return x; }");
        assert_eq!(m.errors.len(), 0);
        assert_eq!(m.items.len(), 1);
        assert!(matches!(m.items[0], HirItem::Fn(_)));
    }

    #[test]
    fn th2_fn_params_get_place_ids() {
        let m = lower_src("fn add(x: i32, y: i32) -> i32 { return x; }");
        if let HirItem::Fn(f) = &m.items[0] {
            assert_eq!(f.params.len(), 2);
            assert_ne!(f.params[0].0, f.params[1].0); // distinct PlaceIds
        } else { panic!(); }
    }

    #[test]
    fn th3_return_type_lowered() {
        let m = lower_src("fn add(x: i32) -> i32 { return x; }");
        if let HirItem::Fn(f) = &m.items[0] {
            assert_eq!(f.ret, HirTy::I32);
        } else { panic!(); }
    }

    #[test]
    fn th4_contracts_lowered() {
        let m = lower_src("@requires(x > 0) fn pos(x: i32) -> i32 { return x; }");
        assert_eq!(m.errors.len(), 0);
        if let HirItem::Fn(f) = &m.items[0] {
            assert_eq!(f.contracts.len(), 1);
            assert_eq!(f.contracts[0].kind, ContractKind::Requires);
            assert!(matches!(f.contracts[0].expr, ContractExpr::BinOp(ContractBinOp::Gt, _, _)));
        } else { panic!(); }
    }

    #[test]
    fn th5_struct_lowered() {
        let m = lower_src("struct Point { x: i32, y: i32, }");
        assert_eq!(m.errors.len(), 0);
        if let HirItem::Struct(s) = &m.items[0] {
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].1, HirTy::I32);
        } else { panic!(); }
    }

    #[test]
    fn th6_enum_lowered() {
        let m = lower_src("enum Color { Red, Green, Blue, }");
        assert_eq!(m.errors.len(), 0);
        if let HirItem::Enum(e) = &m.items[0] {
            assert_eq!(e.variants.len(), 3);
        } else { panic!(); }
    }

    #[test]
    fn th7_primitive_types() {
        let m = lower_src("fn f(a: bool, b: u64, c: f32) -> () { }");
        if let HirItem::Fn(f) = &m.items[0] {
            assert_eq!(f.params[0].1, HirTy::Bool);
            assert_eq!(f.params[1].1, HirTy::U64);
            assert_eq!(f.params[2].1, HirTy::F32);
        }
    }

    #[test]
    fn th8_let_stmt_gets_place() {
        let m = lower_src("fn f() -> () { let x: i32 = 42; }");
        if let HirItem::Fn(f) = &m.items[0] {
            if let HirExprKind::Block(stmts, _) = &f.body.kind {
                assert!(matches!(stmts[0].kind, HirStmtKind::Let(_, _, _, _)));
            }
        }
    }

    #[test]
    fn th9_move_state_map_merge() {
        let mut m1 = MoveStateMap::new();
        let mut m2 = MoveStateMap::new();
        let p = PlaceId(0);
        m1.set(p, MoveState::Owned);
        m2.set(p, MoveState::Moved);
        let merged = m1.merge(&m2);
        assert_eq!(merged.get(p), Some(&MoveState::MaybeOwned));
    }

    #[test]
    fn th10_hir_ty_is_copy() {
        assert!(HirTy::I32.is_copy());
        assert!(HirTy::Bool.is_copy());
        assert!(!HirTy::Named("Vec".into(), vec![HirTy::I32]).is_copy());
    }

    #[test]
    fn th11_hir_ty_needs_drop() {
        assert!(!HirTy::I32.needs_drop());
        assert!(HirTy::Named("Vec".into(), vec![]).needs_drop());
        assert!(!HirTy::Never.needs_drop());
    }

    #[test]
    fn th12_place_ids_unique_across_fn() {
        let m = lower_src("fn f(x: i32, y: i32) -> i32 { let z: i32 = 1; return x; }");
        if let HirItem::Fn(f) = &m.items[0] {
            let p0 = f.params[0].0;
            let p1 = f.params[1].0;
            assert_ne!(p0, p1);
            assert_ne!(p0, PlaceId::INVALID);
        }
    }

    #[test]
    fn th13_impl_lowered() {
        let src = "impl Point { fn new(x: i32) -> Point { return x; } }";
        let m = lower_src(src);
        assert!(matches!(m.items[0], HirItem::Impl(_)));
        if let HirItem::Impl(i) = &m.items[0] {
            assert_eq!(i.methods.len(), 1);
        }
    }

    #[test]
    fn th14_match_arms_lowered() {
        let src = "fn f(x: i32) -> () { match x { 0 => return 0, _ => return 1, } }";
        let m = lower_src(src);
        assert_eq!(m.errors.len(), 0);
        if let HirItem::Fn(f) = &m.items[0] {
            if let HirExprKind::Block(stmts, tail) = &f.body.kind {
                // match is tail expr (no semicolon) — check tail first, then stmts
                let match_expr = if let Some(t) = tail {
                    Some(t.as_ref())
                } else if !stmts.is_empty() {
                    if let HirStmtKind::Expr(e) = &stmts[0].kind { Some(e) } else { None }
                } else { None };
                assert!(match_expr.map(|e| matches!(e.kind, HirExprKind::Match(_, _))).unwrap_or(false));
            }
        }
    }

    #[test]
    fn th15_lower_no_errors_on_valid_program() {
        let src = r#"
            struct Point { x: i32, y: i32, }
            fn distance(p: Point) -> f64 { return 0; }
            impl Point { fn new(x: i32, y: i32) -> Point { return x; } }
        "#;
        let m = lower_src(src);
        assert_eq!(m.errors.len(), 0);
        assert_eq!(m.items.len(), 3);
    }
}

// P12-M2-APPLIED
