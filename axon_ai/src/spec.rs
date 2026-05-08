// ============================================================
// axon_ai — spec.rs
// Formal Specification DSL
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// This is the language of AXON's formal verification layer.
// The LLM proposes specs in this language.
// The deterministic verifier enforces them.
//
// Syntax in AXON source:
//   @ai.intent("always returns non-negative")  ← NL description
//   @ensures(result >= 0)                       ← formal constraint
//   @requires(x != null)                        ← precondition
//   @effect(readonly)                           ← side effect declaration
//   fn abs(x : Int) -> Int:
//       ...
// ============================================================

use serde::{Deserialize, Serialize};

// ── Formal Specification ──────────────────────────────────────

/// A complete formal specification for a function or module.
/// Proposed by AI, verified by the deterministic constraint checker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormalSpec {
    /// The original natural language intent string
    pub intent_nl    : String,
    /// Postconditions — what must hold when the function returns
    pub ensures      : Vec<Constraint>,
    /// Preconditions — what the caller must guarantee
    pub requires     : Vec<Constraint>,
    /// Side effect declarations
    pub effects      : Vec<Effect>,
    /// Confidence of AI translation (0.0–1.0, advisory only)
    pub ai_confidence: f64,
    /// Whether the developer has reviewed and approved this spec
    pub approved     : bool,
}

impl FormalSpec {
    pub fn new(intent_nl: impl Into<String>) -> Self {
        FormalSpec {
            intent_nl    : intent_nl.into(),
            ensures      : Vec::new(),
            requires     : Vec::new(),
            effects      : Vec::new(),
            ai_confidence: 0.0,
            approved     : false,
        }
    }

    pub fn with_ensures(mut self, c: Constraint) -> Self {
        self.ensures.push(c); self
    }

    pub fn with_requires(mut self, c: Constraint) -> Self {
        self.requires.push(c); self
    }

    pub fn with_effect(mut self, e: Effect) -> Self {
        self.effects.push(e); self
    }

    pub fn approved(mut self) -> Self {
        self.approved = true; self
    }

    pub fn is_verifiable(&self) -> bool {
        !self.ensures.is_empty() || !self.effects.is_empty()
    }
}

// ── Constraint types ──────────────────────────────────────────

/// A formal constraint that can be checked by the verifier.
/// These are the properties the LLM extracts from NL intent strings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value")]
pub enum Constraint {
    // ── Numeric bounds ────────────────────────────────────────
    /// result >= 0
    ResultNonNegative,
    /// result > 0
    ResultPositive,
    /// result >= n
    ResultAtLeast(i64),
    /// result <= n
    ResultAtMost(i64),
    /// result == n
    ResultEquals(i64),
    /// result != n
    ResultNotEquals(i64),

    // ── Relational ────────────────────────────────────────────
    /// result > input (input named by parameter index)
    ResultGreaterThanParam(usize),
    /// result == input
    ResultEqualsParam(usize),

    // ── Nullability ───────────────────────────────────────────
    /// result is never null/None
    ResultNonNull,
    /// parameter at index is not null
    ParamNonNull(usize),

    // ── Reachability ──────────────────────────────────────────
    /// The statement at this label is always reached
    AlwaysReaches(String),
    /// The statement at this label is never reached
    NeverReaches(String),

    // ── Effect constraints ────────────────────────────────────
    /// Function does not allocate heap memory
    NoHeapAllocation,
    /// Function does not perform I/O
    NoIO,
    /// Function does not modify its arguments
    PureInputs,

    // ── Custom (human-written formal expression) ──────────────
    /// A formal expression that the verifier evaluates
    /// Used when the LLM cannot classify the constraint
    Custom(String),
}

impl Constraint {
    /// Human-readable description of this constraint
    pub fn description(&self) -> String {
        match self {
            Constraint::ResultNonNegative      => "result >= 0".into(),
            Constraint::ResultPositive         => "result > 0".into(),
            Constraint::ResultAtLeast(n)       => format!("result >= {}", n),
            Constraint::ResultAtMost(n)        => format!("result <= {}", n),
            Constraint::ResultEquals(n)        => format!("result == {}", n),
            Constraint::ResultNotEquals(n)     => format!("result != {}", n),
            Constraint::ResultGreaterThanParam(i) => format!("result > param[{}]", i),
            Constraint::ResultEqualsParam(i)   => format!("result == param[{}]", i),
            Constraint::ResultNonNull          => "result != null".into(),
            Constraint::ParamNonNull(i)        => format!("param[{}] != null", i),
            Constraint::AlwaysReaches(lbl)     => format!("always reaches '{}'", lbl),
            Constraint::NeverReaches(lbl)      => format!("never reaches '{}'", lbl),
            Constraint::NoHeapAllocation       => "no heap allocation".into(),
            Constraint::NoIO                   => "no I/O operations".into(),
            Constraint::PureInputs             => "inputs not modified".into(),
            Constraint::Custom(expr)           => expr.clone(),
        }
    }
}

// ── Effect types ──────────────────────────────────────────────

/// Declared side effects for a function or module.
/// These are checked by the effect system, not the LLM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Effect {
    /// Function is pure — no observable side effects
    Pure,
    /// Function reads from IPC/network but does not write system state
    ReadOnly,
    /// Function writes to audit log (required for security-critical modules)
    WritesAuditLog,
    /// Function communicates on specific channel (named)
    UsesChannel(String),
    /// Function may allocate
    MayAllocate,
    /// No memory allocation permitted
    NoAllocate,
    /// Custom effect declaration
    Custom(String),
}

impl Effect {
    pub fn description(&self) -> String {
        match self {
            Effect::Pure               => "pure function (no side effects)".into(),
            Effect::ReadOnly           => "read-only (no state writes)".into(),
            Effect::WritesAuditLog     => "writes to audit log".into(),
            Effect::UsesChannel(name)  => format!("uses channel '{}'", name),
            Effect::MayAllocate        => "may allocate heap memory".into(),
            Effect::NoAllocate         => "no heap allocation".into(),
            Effect::Custom(s)          => s.clone(),
        }
    }
}

// ── Module-level intent ───────────────────────────────────────

/// Module-level formal specification from @program_intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleIntent {
    /// The full @program_intent description text
    pub description  : String,
    /// What the module is allowed to access
    pub allowed      : Vec<String>,
    /// What the module is forbidden from doing
    pub forbidden    : Vec<String>,
    /// What the module always does (liveness properties)
    pub always       : Vec<String>,
    /// Effects this module declares
    pub effects      : Vec<Effect>,
}

impl ModuleIntent {
    pub fn from_description(description: impl Into<String>) -> Self {
        ModuleIntent {
            description : description.into(),
            allowed     : Vec::new(),
            forbidden   : Vec::new(),
            always      : Vec::new(),
            effects     : Vec::new(),
        }
    }
}
