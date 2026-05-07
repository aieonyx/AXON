// ============================================================
// AXON Lexer — indent.rs
// Indent Tracker — P2-04 implementation target
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
//
// P2-04 implementation target:
//   - Inject INDENT tokens when indentation increases after ':'
//   - Inject DEDENT tokens when indentation decreases
//   - Suppress NEWLINE/INDENT/DEDENT inside ( ) [ ] { }
//   - Hard fail on mixed tabs and spaces (E004)
//   - Flush remaining DEDENTs at EOF
// ============================================================

use crate::token::{Token, TokenKind};

/// Inject INDENT and DEDENT tokens into the raw token stream.
/// Called after the lexer produces the raw token stream.
/// Returns the augmented stream ready for the parser.
pub fn inject_indentation(tokens: Vec<Token>) -> Vec<Token> {
    // P2-04: implement full indent tracking
    // For now: pass tokens through unchanged
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_is_identity() {
        let tokens = vec![];
        let result = inject_indentation(tokens);
        assert!(result.is_empty());
    }

    // P2-04 tests will be added here.
    // See Compiler Pipeline Contracts v1.0 Section 5 for
    // complete test checklist: I1 through I8.
}
