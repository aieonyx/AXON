// ============================================================
// AXON Parser — error.rs
// Structured parse error types with human-readable diagnostics
// ============================================================

use axon_lexer::{Span, TokenKind};

/// A structured parse error.
/// All errors carry a source span for precise error reporting.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Found an unexpected token.
    UnexpectedToken {
        expected : String,
        found    : TokenKind,
        span     : Span,
        hint     : Option<String>,
    },
    /// Reached end of file unexpectedly.
    UnexpectedEof {
        expected : String,
        span     : Span,
    },
    /// Indentation error.
    InvalidIndentation {
        span   : Span,
        detail : String,
    },
    /// Mixed tabs and spaces.
    MixedIndentation {
        span : Span,
    },
    /// Chained comparison — forbidden in AXON.
    ChainedComparison {
        span : Span,
    },
    /// Invalid use of a raw-tainted value outside a raw: block.
    RawTaintedEscape {
        span    : Span,
        var     : String,
    },
    /// A decorator argument is malformed.
    InvalidDecoratorArg {
        span   : Span,
        detail : String,
    },
    /// A mem_mode keyword in invalid position.
    InvalidMemMode {
        found  : String,
        span   : Span,
    },
    /// Custom error with a free-form message.
    Custom {
        message : String,
        span    : Span,
        hint    : Option<String>,
    },
}

impl ParseError {
    /// The source span of this error.
    pub fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken    { span, .. } => *span,
            ParseError::UnexpectedEof      { span, .. } => *span,
            ParseError::InvalidIndentation { span, .. } => *span,
            ParseError::MixedIndentation   { span, .. } => *span,
            ParseError::ChainedComparison  { span, .. } => *span,
            ParseError::RawTaintedEscape   { span, .. } => *span,
            ParseError::InvalidDecoratorArg{ span, .. } => *span,
            ParseError::InvalidMemMode     { span, .. } => *span,
            ParseError::Custom             { span, .. } => *span,
        }
    }

    /// Format this error as a human-readable diagnostic.
    /// Follows the AXON error format:
    ///   error[EXXX]: message
    ///     --> file:line:col
    ///      |
    ///   NN | source line
    ///      | ^ hint
    pub fn display(&self, source: &str) -> String {
        match self {
            ParseError::UnexpectedToken { expected, found, span, hint } => {
                let mut msg = format!(
                    "error[E001]: expected {}, found {}\n  --> {}:{}\n",
                    expected,
                    found.display_name(),
                    span.line,
                    span.col,
                );
                if let Some(h) = hint {
                    msg.push_str(&format!("   = hint: {}\n", h));
                }
                msg
            },
            ParseError::ChainedComparison { span } => {
                format!(
                    "error[E002]: comparison operators cannot be chained\n  --> {}:{}\n   = hint: use 'and' to combine comparisons: a < b and b < c\n",
                    span.line, span.col,
                )
            },
            ParseError::UnexpectedEof { expected, span } => {
                format!(
                    "error[E003]: unexpected end of file, expected {}\n  --> {}:{}\n",
                    expected, span.line, span.col,
                )
            },
            ParseError::MixedIndentation { span } => {
                format!(
                    "error[E004]: mixed tabs and spaces in indentation\n  --> {}:{}\n   = hint: use only spaces or only tabs — never both in the same file\n",
                    span.line, span.col,
                )
            },
            ParseError::Custom { message, span, hint } => {
                let mut msg = format!(
                    "error[E099]: {}\n  --> {}:{}\n",
                    message, span.line, span.col,
                );
                if let Some(h) = hint {
                    msg.push_str(&format!("   = hint: {}\n", h));
                }
                msg
            },
            _ => format!("error: {:?}", self),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display(""))
    }
}

// ── v0.3.1 Error Codes ───────────────────────────────────────
// E206: Deferred expression references a moved value
//       defer file.close() but file was moved before scope exit
// E207: defer is not permitted inside raw: blocks
//       Use explicit cleanup in raw zones
// E208: defer cannot capture a let@ binding whose lifetime
//       ends before the deferred expression executes
// E209: Capability escapes with block via closure capture
//       A closure inside a with block captures the binding
//       and is returned or stored outside the block
// E411: Program intent violation
//       A function violates the module's @program_intent declaration
//       without @ai.allow override
// W411: Program intent warning (--no-ai mode)
//       Static analysis detected a likely program_intent violation
//       Full verification requires AI pass
