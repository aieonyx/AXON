// ============================================================
// AXON Parser — ast.rs
// Complete AST node definitions — v0.3
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
//
// Changelog v0.2.1 → v0.3:
//   + ActorDecl         — actor declarations with handle blocks
//   + OpaqueTypeDecl    — opaque type T = U
//   + UsesList          — uses [effect, ...] on fn/task
//   + EphemeralStmt     — let@ linear binding
//   + ForeachYieldStmt  — foreach + yield stream
//   + IntentBlockStmt   — intent { secure | auditable } block
//   + PipeExpr          — expr |> fn() as Contract
//   + MorphExpr         — x ~> method()
//   + TemporalExpr      — @now @lifetime @epoch
//   + CapPinExpr        — expr!method() expr?method()
//   + RefinementType    — Int { _ > 0 }
//   + OpaqueType        — opaque type reference
//   + ProvenanceType    — Tainted<T> Clean<T>
//   + TimedCapType      — cap<T>[@lifetime]
//   + Extended Decorator — pre: post: invariant: fields
//   + ReplyStmt         — reply expr (inside actor handle)
// ============================================================

use axon_lexer::Span;

// ══════════════════════════════════════════════════════════════
// PRIMITIVES
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    pub name : String,
    pub span : Span,
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Ident { name: name.into(), span }
    }
}

// ══════════════════════════════════════════════════════════════
// PROGRAM ROOT
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Program {
    pub span           : Span,
    pub program_intent : Option<ProgramIntent>,  // [v0.3.1]
    pub module         : Option<ModuleDecl>,
    pub imports        : Vec<ImportDecl>,
    pub items          : Vec<TopLevelItem>,
}

#[derive(Debug, Clone)]
pub struct ModuleDecl {
    pub span : Span,
    pub path : Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub span  : Span,
    pub path  : Vec<Ident>,
    pub alias : Option<Ident>,
}

#[derive(Debug, Clone)]
pub enum TopLevelItem {
    Fn(FnDecl),
    Task(TaskDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    TypeAlias(TypeAlias),
    Const(ConstDecl),
    Impl(ImplBlock),
    Trait(TraitDecl),
    Actor(ActorDecl),          // [v0.3]
    OpaqueType(OpaqueTypeDecl), // [v0.3]
}

// ══════════════════════════════════════════════════════════════
// DECLARATIONS
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub span        : Span,
    pub decorators  : Vec<Decorator>,
    pub name        : Ident,
    pub generics    : Vec<GenericParam>,
    pub params      : Vec<Param>,
    pub uses        : Option<UsesList>,   // [v0.3]
    pub ret_type    : Option<Type>,
    pub body        : Block,
}

#[derive(Debug, Clone)]
pub struct TaskDecl {
    pub span        : Span,
    pub decorators  : Vec<Decorator>,
    pub name        : Ident,
    pub generics    : Vec<GenericParam>,
    pub params      : Vec<Param>,
    pub uses        : Option<UsesList>,   // [v0.3]
    pub ret_type    : Option<Type>,
    pub body        : Block,
}

/// Effect/capability declarations on fn and task — v0.3
#[derive(Debug, Clone)]
pub struct UsesList {
    pub span    : Span,
    pub effects : Vec<EffectName>,
}

#[derive(Debug, Clone)]
pub struct EffectName {
    pub span  : Span,
    pub parts : Vec<Ident>,   // e.g. ["Disk", "Read"] for Disk.Read
}

#[derive(Debug, Clone)]
pub struct Param {
    pub span     : Span,
    pub mem_mode : Option<MemMode>,
    pub name     : Ident,
    pub ty       : Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemMode { Own, Borrow, MutBorrow, Share }

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub span     : Span,
    pub name     : Ident,
    pub generics : Vec<GenericParam>,
    pub fields   : Vec<FieldDecl>,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub span : Span,
    pub name : Ident,
    pub ty   : Type,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub span     : Span,
    pub name     : Ident,
    pub generics : Vec<GenericParam>,
    pub variants : Vec<VariantDecl>,
}

#[derive(Debug, Clone)]
pub struct VariantDecl {
    pub span   : Span,
    pub name   : Ident,
    pub fields : Vec<FieldDecl>,
}

#[derive(Debug, Clone)]
pub struct TraitDecl {
    pub span     : Span,
    pub name     : Ident,
    pub generics : Vec<GenericParam>,
    pub items    : Vec<TraitItem>,
}

#[derive(Debug, Clone)]
pub enum TraitItem {
    Signature(FnSignature),
    Default(FnDecl),
}

#[derive(Debug, Clone)]
pub struct FnSignature {
    pub span     : Span,
    pub name     : Ident,
    pub params   : Vec<Param>,
    pub ret_type : Option<Type>,
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub span       : Span,
    pub trait_name : Option<Ident>,
    pub ty         : Type,
    pub methods    : Vec<FnDecl>,
}

#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub span     : Span,
    pub name     : Ident,
    pub generics : Vec<GenericParam>,
    pub ty       : Type,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub span  : Span,
    pub name  : Ident,
    pub ty    : Type,
    pub value : Expr,
}

/// Actor declaration — v0.3
#[derive(Debug, Clone)]
pub struct ActorDecl {
    pub span  : Span,
    pub name  : Ident,
    pub items : Vec<ActorItem>,
}

#[derive(Debug, Clone)]
pub enum ActorItem {
    Handle(HandleBlock),
    Method(FnDecl),
}

#[derive(Debug, Clone)]
pub struct HandleBlock {
    pub span     : Span,
    pub msg_name : Ident,
    pub msg_type : Type,
    pub body     : Block,
}

/// Opaque type declaration — v0.3
#[derive(Debug, Clone)]
pub struct OpaqueTypeDecl {
    pub span : Span,
    pub name : Ident,
    pub ty   : Type,
}

// ══════════════════════════════════════════════════════════════
// DECORATORS — Extended v0.3
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Decorator {
    pub span : Span,
    pub name : Vec<Ident>,          // ["ai", "verify"]
    pub args : Vec<DecoratorArg>,
}

#[derive(Debug, Clone)]
pub struct DecoratorArg {
    pub span  : Span,
    pub label : Option<Ident>,      // named arg: pre: "..." post: "..." invariant: "..."
    pub value : Expr,
}

// ══════════════════════════════════════════════════════════════
// BLOCK AND STATEMENTS
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Block {
    pub span  : Span,
    pub stmts : Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Mut(MutStmt),
    Ephemeral(EphemeralStmt),      // [v0.3] let@
    Assign(AssignStmt),
    Return(ReturnStmt),
    Expr(ExprStmt),
    If(IfStmt),
    For(ForStmt),
    ForeachYield(ForeachYieldStmt), // [v0.3] foreach + yield
    While(WhileStmt),
    Match(MatchStmt),
    IntentBlock(IntentBlockStmt),   // [v0.3] intent { mode }
    Break(Span),
    Continue(Span),
    Pass(Span),
    Reply(ReplyStmt),               // [v0.3] inside actor handle
    Raw(RawBlock),
    Spawn(SpawnStmt),
    Defer(DeferStmt),        // [v0.3.1] defer cleanup
    With(WithStmt),          // [v0.3.1] capability scope
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub span : Span,
    pub name : Ident,
    pub ty   : Option<Type>,
    pub init : Expr,
}

#[derive(Debug, Clone)]
pub struct MutStmt {
    pub span : Span,
    pub name : Ident,
    pub ty   : Option<Type>,
    pub init : Expr,
}

/// Ephemeral binding — v0.3 — let@ x = expr
/// Value must be consumed exactly once.
#[derive(Debug, Clone)]
pub struct EphemeralStmt {
    pub span : Span,
    pub name : Ident,
    pub ty   : Option<Type>,
    pub init : Expr,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub span   : Span,
    pub target : AssignTarget,
    pub op     : AssignOp,
    pub value  : Expr,
}

#[derive(Debug, Clone)]
pub enum AssignTarget {
    Ident(Ident),
    Field(Box<Expr>, Ident),
    Index(Box<Expr>, Box<Expr>),
    Deref(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,
    Add, Sub, Mul, Div, Mod,
    BitAnd, BitOr, BitXor,
    Shl, Shr,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub span  : Span,
    pub value : Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ReplyStmt {  // [v0.3]
    pub span  : Span,
    pub value : Expr,
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub span : Span,
    pub expr : Expr,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub span       : Span,
    pub condition  : Expr,
    pub then_block : Block,
    pub else_ifs   : Vec<(Expr, Block)>,
    pub else_block : Option<Block>,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub span     : Span,
    pub binding  : Ident,
    pub iterable : Expr,
    pub body     : Block,
}

/// foreach + yield stream — v0.3
#[derive(Debug, Clone)]
pub struct ForeachYieldStmt {
    pub span     : Span,
    pub binding  : Ident,
    pub iterable : Expr,
    pub body     : Vec<ForeachItem>,
}

#[derive(Debug, Clone)]
pub enum ForeachItem {
    Stmt(Stmt),
    Yield(Expr, Span),   // yield expr
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub span      : Span,
    pub condition : Expr,
    pub body      : Block,
}

#[derive(Debug, Clone)]
pub struct MatchStmt {
    pub span    : Span,
    pub subject : Expr,
    pub arms    : Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub span    : Span,
    pub pattern : Pattern,
    pub guard   : Option<Expr>,
    pub body    : Expr,
}

/// Intent block — v0.3 — intent { secure | auditable }
#[derive(Debug, Clone)]
pub struct IntentBlockStmt {
    pub span  : Span,
    pub modes : Vec<IntentMode>,
    pub body  : Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntentMode {
    Secure,
    Performant,
    Auditable,
    Verifiable,
    MinimalRuntime,
}

#[derive(Debug, Clone)]
pub struct RawBlock {
    pub span  : Span,
    pub stmts : Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct SpawnStmt {
    pub span   : Span,
    pub name   : Option<Ident>,
    pub target : SpawnTarget,
}

#[derive(Debug, Clone)]
pub enum SpawnTarget {
    FnCall(CallExpr),
    MethodCall(MethodCallExpr),
    Closure(ClosureExpr),
    Block(Block),
}

// ══════════════════════════════════════════════════════════════
// PATTERNS
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Enum(EnumPattern),
    Wildcard(Span),
    Binding(Ident),
    Tuple(Vec<Pattern>, Span),
    Range(RangePattern),
    Nested(Box<Pattern>),
    Or(Vec<Pattern>, Span),
    Rest(Span),
}

#[derive(Debug, Clone)]
pub struct EnumPattern {
    pub span   : Span,
    pub name   : Ident,
    pub fields : Vec<PatternField>,
}

#[derive(Debug, Clone)]
pub enum PatternField {
    Named(Ident, Pattern),
    Positional(Pattern),
    Rest(Span),
}

#[derive(Debug, Clone)]
pub struct RangePattern {
    pub span      : Span,
    pub start     : Literal,
    pub end       : Literal,
    pub inclusive : bool,
}

// ══════════════════════════════════════════════════════════════
// EXPRESSIONS — v0.3 Extended
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Literal),
    Ident(Ident),
    BinOp(Box<BinOpExpr>),
    UnaryOp(Box<UnaryOpExpr>),
    Call(Box<CallExpr>),
    MethodCall(Box<MethodCallExpr>),
    FieldAccess(Box<FieldAccessExpr>),
    Index(Box<IndexExpr>),
    Propagate(Box<Expr>, Span),
    Await(Box<Expr>, Span),
    AwaitTuple(Vec<Expr>, Span),
    List(ListExpr),
    ListComp(Box<ListCompExpr>),
    Tuple(TupleExpr),
    Range(Box<RangeExpr>),
    Struct(Box<StructInitExpr>),
    IfExpr(Box<IfExprInline>),
    Match(Box<MatchExpr>),
    Closure(Box<ClosureExpr>),
    Spawn(Box<SpawnExpr>),
    Ref(Box<Expr>, bool, Span),
    Deref(Box<Expr>, Span),
    Cast(Box<Expr>, Type, Span),
    StrInterp(StrInterpExpr),
    OptionDefault(Box<Expr>, Box<Expr>, Span),

    // ── v0.3 ─────────────────────────────────────────────────
    Pipe(Box<PipeExpr>),           // expr |> fn() as Contract
    Morph(Box<MorphExpr>),         // x ~> method()
    Temporal(TemporalExpr),        // @now @lifetime @epoch
    CapPin(Box<CapPinExpr>),       // expr!method() / expr?method()
}

/// Pipe expression — v0.3
#[derive(Debug, Clone)]
pub struct PipeExpr {
    pub span   : Span,
    pub head   : Expr,
    pub stages : Vec<PipeStage>,
}

#[derive(Debug, Clone)]
pub struct PipeStage {
    pub span     : Span,
    pub call     : PipeCall,
    pub contract : Option<Ident>,  // 'as Contract'
}

#[derive(Debug, Clone)]
pub enum PipeCall {
    FnCall(CallExpr),
    MethodCall(MethodCallExpr),
    Closure(ClosureExpr),
}

/// Morphing type expression — v0.3
/// x ~> method() permanently changes x's type
#[derive(Debug, Clone)]
pub struct MorphExpr {
    pub span   : Span,
    pub target : Ident,
    pub method : MethodCallExpr,
}

/// Temporal expression — v0.3
#[derive(Debug, Clone)]
pub enum TemporalExpr {
    Now(Span),
    Lifetime(Span),
    Epoch(Span),
    NowOffset { span: Span, op: TemporalOp, duration: Box<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemporalOp { Add, Sub }

/// Capability pin expression — v0.3
/// expr!method() — requires capability, may fail
/// expr?method() — compiler proves infallible
#[derive(Debug, Clone)]
pub struct CapPinExpr {
    pub span      : Span,
    pub receiver  : Expr,
    pub method    : Ident,
    pub args      : Vec<Arg>,
    pub infallible: bool,   // false = !, true = ?
}

// ── Existing expression nodes ─────────────────────────────────

#[derive(Debug, Clone)]
pub struct BinOpExpr {
    pub span : Span,
    pub op   : BinOp,
    pub lhs  : Expr,
    pub rhs  : Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    BitAnd, BitOr, BitXor, Shl, Shr,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
    Range, RangeInclusive,
}

impl BinOp {
    pub fn binding_power(&self) -> (u8, u8) {
        match self {
            BinOp::Or                           => (10, 11),
            BinOp::And                          => (20, 21),
            BinOp::Eq | BinOp::NotEq |
            BinOp::Lt | BinOp::Gt |
            BinOp::LtEq | BinOp::GtEq          => (30, 31),
            BinOp::BitOr                        => (32, 33),
            BinOp::BitXor                       => (34, 35),
            BinOp::BitAnd                       => (36, 37),
            BinOp::Shl | BinOp::Shr             => (38, 39),
            BinOp::Add | BinOp::Sub             => (40, 41),
            BinOp::Mul | BinOp::Div | BinOp::Mod => (50, 51),
            BinOp::Range | BinOp::RangeInclusive => (5, 6),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnaryOpExpr {
    pub span : Span,
    pub op   : UnaryOp,
    pub expr : Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp { Neg, Not, Ref, MutRef, Deref }

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub span     : Span,
    pub callee   : Ident,
    pub generics : Vec<Type>,
    pub args     : Vec<Arg>,
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub span     : Span,
    pub receiver : Box<Expr>,
    pub method   : Ident,
    pub generics : Vec<Type>,
    pub args     : Vec<Arg>,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub span  : Span,
    pub label : Option<Ident>,
    pub value : Expr,
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub span   : Span,
    pub object : Box<Expr>,
    pub field  : Ident,
}

#[derive(Debug, Clone)]
pub struct IndexExpr {
    pub span   : Span,
    pub object : Box<Expr>,
    pub index  : Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ListExpr {
    pub span     : Span,
    pub elements : Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ListCompExpr {
    pub span     : Span,
    pub expr     : Box<Expr>,
    pub binding  : Ident,
    pub iterable : Box<Expr>,
    pub filter   : Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct TupleExpr {
    pub span     : Span,
    pub elements : Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct RangeExpr {
    pub span      : Span,
    pub start     : Box<Expr>,
    pub end       : Box<Expr>,
    pub inclusive : bool,
}

#[derive(Debug, Clone)]
pub struct StructInitExpr {
    pub span   : Span,
    pub name   : Ident,
    pub fields : Vec<StructFieldInit>,
}

#[derive(Debug, Clone)]
pub struct StructFieldInit {
    pub span  : Span,
    pub name  : Ident,
    pub value : Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct IfExprInline {
    pub span      : Span,
    pub condition : Box<Expr>,
    pub then_expr : Box<Expr>,
    pub else_expr : Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub span    : Span,
    pub subject : Box<Expr>,
    pub arms    : Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct ClosureExpr {
    pub span     : Span,
    pub params   : Vec<ClosureParam>,
    pub ret_type : Option<Type>,
    pub body     : ClosureBody,
}

#[derive(Debug, Clone)]
pub struct ClosureParam {
    pub span : Span,
    pub name : Ident,
    pub ty   : Option<Type>,
}

#[derive(Debug, Clone)]
pub enum ClosureBody {
    Expr(Box<Expr>),
    Block(Block),
}

#[derive(Debug, Clone)]
pub struct SpawnExpr {
    pub span   : Span,
    pub target : SpawnTarget,
}

#[derive(Debug, Clone)]
pub struct StrInterpExpr {
    pub span  : Span,
    pub parts : Vec<StrInterpPart>,
}

#[derive(Debug, Clone)]
pub enum StrInterpPart {
    Literal(String),
    Expr(Box<Expr>),
}

// ══════════════════════════════════════════════════════════════
// LITERALS
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64, Span),
    Float(f64, Span),
    Str(String, Span),
    Bool(bool, Span),
    Bytes(Vec<u8>, Span),
    None(Span),
}

impl Literal {
    pub fn span(&self) -> Span {
        match self {
            Literal::Int(_, s)   => *s,
            Literal::Float(_, s) => *s,
            Literal::Str(_, s)   => *s,
            Literal::Bool(_, s)  => *s,
            Literal::Bytes(_, s) => *s,
            Literal::None(s)     => *s,
        }
    }
}

// ══════════════════════════════════════════════════════════════
// TYPES — v0.3 Extended
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum Type {
    Primitive(PrimitiveType, Span),
    Named(Vec<Ident>),
    Generic(Ident, Vec<Type>, Span),
    Option(Box<Type>, Span),
    Result(Box<Type>, Box<Type>, Span),
    List(Box<Type>, Span),
    Tuple(Vec<Type>, Span),
    Fn(Vec<Type>, Box<Type>, Span),
    Cap(Box<Type>, Span),
    Unit(Span),
    Ref(Box<Type>, Span),
    MutRef(Box<Type>, Span),
    PtrConst(Box<Type>, Span),
    PtrMut(Box<Type>, Span),
    Array(Box<Type>, u64, Span),
    Infer(Span),

    // ── v0.3 ─────────────────────────────────────────────────
    Refinement(Box<Type>, RefinementPred, Span),  // Int { _ > 0 }
    Provenance(ProvenanceKind, Box<Type>, Span),  // Tainted<T> Clean<T>
    TimedCap(Box<Type>, TemporalExpr, Span),       // cap<T>[@lifetime]
    Opaque(Ident, Span),                           // opaque type reference
}

/// Refinement type predicate — v0.3
#[derive(Debug, Clone)]
pub enum RefinementPred {
    /// Simple comparison: _ > 0, _ <= 120
    Compare(RefinementOp, Literal),
    /// Range: _ >= 0 and _ <= 120
    Range(RefinementOp, Literal, RefinementOp, Literal),
    /// Named predicate function: sql_sanitized(_)
    Named(Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RefinementOp { Gt, Lt, GtEq, LtEq, Eq, NotEq }

/// Provenance type kind — v0.3
#[derive(Debug, Clone, PartialEq)]
pub enum ProvenanceKind {
    Tainted,
    Clean,
    Network,
    FileSystem,
    UserInput,
    Trusted,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    Int, Int32, Int64, Int8,
    UInt, UInt32, UInt64, UInt8,
    Float, Float32,
    Bool, Char, Str, Bytes,
}

// ── Generic parameters ────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub span   : Span,
    pub name   : Ident,
    pub bounds : Vec<Ident>,   // trait bounds: T: Display + Clone
}

// ══════════════════════════════════════════════════════════════
// v0.3.1 ADDITIONS
// ══════════════════════════════════════════════════════════════

// Add to Stmt enum — append these variants conceptually:
// Defer(DeferStmt)
// With(WithStmt)
// These are defined below. The Stmt enum will be updated
// when the full parser implementation begins (P2-06).

/// defer statement — v0.3.1
///
/// Schedules a cleanup expression to run when the current scope exits.
/// Guaranteed to run on every exit path: normal return, early return,
/// and ? error propagation.
///
/// GRAMMAR (v0.3.1 — restricted from bare expr to call_expr):
///   defer_stmt ::= 'defer' call_expr NEWLINE
///   call_expr  ::= fn_call | method_call_expr
///
/// Block expressions (if, match, closures) are NOT permitted.
/// This eliminates the ambiguity: defer if a: b is now a parse error.
///
/// SEMANTIC RULINGS (from DeepSeek adversarial review, April 2026):
///
/// 1. Capture: by REFERENCE — expression evaluated at scope exit,
///    not at the defer declaration point.
///    Example: for x in items: defer log(x)
///    logs x at each iteration's exit with the current value of x.
///
/// 2. Moved-after-declaration: if a variable referenced in defer
///    is moved before scope exit — compile error E206.
///    Example: defer file.close() then consume(file) → E206
///
/// 3. raw: blocks: defer is NOT valid inside raw: blocks.
///    Raw zones manage cleanup explicitly. E207 if attempted.
///
/// 4. Effects: deferred expressions are part of the function body
///    for effect checking. Their effects are included in the
///    function's inferred effect signature.
///
/// 5. let@ interaction: defer CANNOT capture a let@ binding
///    whose lifetime ends before the defer executes.
///    The compiler detects this and rejects with E208.
///
/// 6. Multiple defers run LIFO: last declared = first executed.
///    defer inside with: user defers run BEFORE with releases.
#[derive(Debug, Clone)]
pub struct DeferStmt {
    pub span : Span,
    pub expr : Expr,   // must be fn_call or method_call_expr — not block expr
}

/// with statement — v0.3.1
///
/// Creates an explicit capability scope. The bound capability lives
/// exactly as long as the block and cannot escape it.
///
/// GRAMMAR:
///   with_stmt ::= 'with' expr 'as' IDENT ':' body
///
/// SEMANTIC RULINGS (from DeepSeek adversarial review, April 2026):
///
/// 1. Failable expressions: expr must type-check to a non-Result type.
///    If the capability expression is failable (uses ! pin), the
///    programmer must apply ? BEFORE with binds it:
///      with file!open(path)? as f:   ← correct
///      with file!open(path) as f:    ← compile error — Result<File,E>
///    The ? propagates the error from the enclosing function.
///
/// 2. Escape analysis covers CLOSURES: a closure that captures the
///    with binding cannot be returned or stored outside the block.
///    The compiler's escape analysis must track closure captures.
///    Returning such a closure is E209: capability escapes with block.
///
/// 3. Release mechanism: with implies automatic defer of release.
///    The binding's type must implement a Release or Drop trait.
///    The implicit release is scheduled at block entry (outermost defer).
///    User-written defers inside the block run BEFORE release.
///
/// 4. Nesting: inner with capabilities released before outer ones.
///    This matches LIFO defer ordering.
///
/// 5. let@ passed to with: safe if the let@ lifetime contains
///    the with block's lifetime. The with binding takes ownership.
///
/// 6. match arm: valid — with_stmt is a statement,
///    permissible inside any block including match arm bodies.
///
/// Example:
///   with file!open("config.axon")? as f:
///       let data = f.read_all()?
///   # f released here automatically
#[derive(Debug, Clone)]
pub struct WithStmt {
    pub span    : Span,
    pub expr    : Expr,     // must type-check to non-Result
    pub binding : Ident,    // 'as name' — cannot escape block
    pub body    : Block,
}

/// Program-level intent declaration — v0.3.1
///
/// Declares the mission and behavioral boundaries of an entire module.
/// The AI compiler pass (Stage 5) verifies that every function,
/// capability declaration, and effect in the module is consistent
/// with the stated mission.
///
/// SYNTAX (v0.3.1 — fixed from #[program_intent] which conflicted
/// with AXON's # comment character):
///
///   @program_intent
///   """
///   This module monitors security threats across Aegis layers.
///   ONLY reads threat signals, classifies them, and emits alerts.
///   does NOT modify system state or access user data.
///   ALWAYS logs every classification to the audit trail.
///   requires sel4.Syscall capability.
///   """
///
/// SEMANTIC RULINGS (from DeepSeek adversarial review, April 2026):
///
/// 1. Constraint extraction — four types parsed from description:
///    ONLY/only         → restricts module to stated operations
///    does NOT/never    → prohibits stated operations  
///    ALWAYS/must always → invariants that must hold
///    requires/needs    → capability requirements
///
/// 2. Violation produces E411. Override with @ai.allow("reason")
///    logged permanently in .axon-audit.
///
/// 3. In --no-ai mode: produces W411 warning for static violations
///    detectable without AI (explicit forbidden capability use).
///
/// 4. Submodule inheritance: NOT inherited. Each module declares
///    its own intent independently.
///
/// 5. defer expressions: treated as part of function body for
///    program_intent verification. Deferred code is checked.
///
/// 6. actor modules: verification applies per-module, covering
///    all handle blocks. Each handler is checked against module intent.
///
/// 7. Triple-quote escaping: use \"\"\" inside content to include
///    a literal triple-quote in the description.
#[derive(Debug, Clone)]
pub struct ProgramIntent {
    pub span        : Span,
    pub description : String,                 // natural language mission statement
    pub constraints : Vec<IntentConstraint>,  // parsed constraints (AI Stage 5)
}

/// A specific constraint extracted from the program intent description.
/// The AI compiler pass parses these from the natural language description
/// and verifies them against the module's actual behavior.
#[derive(Debug, Clone)]
pub struct IntentConstraint {
    pub span        : Span,
    pub kind        : IntentConstraintKind,
    pub description : String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntentConstraintKind {
    /// "ONLY does X" — restricts module to stated operations
    OnlyDoes,
    /// "does NOT do X" — explicitly prohibits stated operations
    DoesNot,
    /// "ALWAYS does X" — invariant that must always hold
    AlwaysDoes,
    /// "requires X" — capability or precondition requirement
    Requires,
}

// Update Program root to include ProgramIntent
// The Program struct gains an optional program_intent field:
//
//   pub struct Program {
//       pub span           : Span,
//       pub program_intent : Option<ProgramIntent>,   // [v0.3.1]
//       pub module         : Option<ModuleDecl>,
//       pub imports        : Vec<ImportDecl>,
//       pub items          : Vec<TopLevelItem>,
//   }
//
// This is noted here as a pending update to the Program struct above.
// The parser implementation (P2-07) will apply this change.
