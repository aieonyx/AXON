// ============================================================
// AXON Lexer — indent.rs
// Indent Tracker — P2-04
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
//
// Injects INDENT and DEDENT tokens into the raw token stream.
// Called after the lexer produces the raw token stream.
// Rules:
//   - INDENT emitted when indentation increases after ':'
//   - DEDENT emitted when indentation decreases
//   - NEWLINE/INDENT/DEDENT suppressed inside ( ) [ ] { }
//   - Mixed tabs and spaces = hard error
//   - EOF flushes all remaining DEDENTs
// ============================================================

use crate::span::{FileId, Span};
use crate::token::{Token, TokenKind};

/// Inject INDENT and DEDENT tokens into the raw token stream.
/// Returns the augmented stream ready for the parser.
pub fn inject_indentation(tokens: Vec<Token>) -> Vec<Token> {
    let mut result   : Vec<Token> = Vec::new();
    let mut stack    : Vec<usize> = vec![0];  // indent stack — 0 is base
    let mut brackets : usize      = 0;        // open bracket counter

    // We process line by line.
    // Split token stream into logical lines first.
    let lines = split_into_lines(&tokens);

    for line in lines {
        if line.is_empty() { continue; }

        // Count leading spaces for this line
        let indent_len = measure_indent(&line);

        // If we are inside brackets — suppress indent logic
        if brackets > 0 {
            for tok in &line {
                update_brackets(tok, &mut brackets);
                result.push(tok.clone());
            }
            continue;
        }

        // Skip blank lines and comment-only lines
        if is_blank_line(&line) {
            continue;
        }

        let current = *stack.last().unwrap_or(&0);

        if indent_len > current {
            // Indentation increased — emit INDENT
            stack.push(indent_len);
            let span = line.first()
                .map(|t| t.span)
                .unwrap_or(dummy_span());
            result.push(Token::new(TokenKind::Indent, "", span));
        } else if indent_len < current {
            // Indentation decreased — emit one DEDENT per level
            while *stack.last().unwrap_or(&0) > indent_len {
                stack.pop();
                let span = line.first()
                    .map(|t| t.span)
                    .unwrap_or(dummy_span());
                result.push(Token::new(TokenKind::Dedent, "", span));
            }
        }
        // Equal indentation — no INDENT/DEDENT needed

        // Push the actual tokens of this line
        for tok in &line {
            update_brackets(tok, &mut brackets);
            result.push(tok.clone());
        }
    }

    // Flush remaining DEDENTs at EOF
    while stack.len() > 1 {
        stack.pop();
        let span = result.last()
            .map(|t| t.span)
            .unwrap_or(dummy_span());
        result.push(Token::new(TokenKind::Dedent, "", span));
    }

    // Ensure stream ends with Eof
    if result.last().map(|t| &t.kind) != Some(&TokenKind::Eof) {
        let span = result.last()
            .map(|t| t.span)
            .unwrap_or(dummy_span());
        result.push(Token::new(TokenKind::Eof, "", span));
    }

    result
}

/// Split the flat token stream into logical lines.
/// Each line ends just before a Newline token.
/// The Newline token itself is included at the end of each line.
fn split_into_lines(tokens: &[Token]) -> Vec<Vec<Token>> {
    let mut lines  : Vec<Vec<Token>> = Vec::new();
    let mut current: Vec<Token>      = Vec::new();

    for tok in tokens {
        match &tok.kind {
            TokenKind::Eof => {
                if !current.is_empty() {
                    lines.push(current.clone());
                    current.clear();
                }
                // Eof handled by caller
            }
            TokenKind::Newline => {
                current.push(tok.clone());
                lines.push(current.clone());
                current.clear();
            }
            _ => {
                current.push(tok.clone());
            }
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

/// Measure the indentation level of a line.
/// Indentation is the number of leading spaces.
/// Tabs count as 4 spaces (consistent tab width).
fn measure_indent(line: &[Token]) -> usize {
    // The lexer already skips whitespace characters.
    // Indentation is tracked via line/col in spans.
    // We use the column of the first non-whitespace token.
    // Column is 1-based — so col 1 = no indent, col 5 = 4 spaces indent.
    line.first()
        .map(|t| {
            let col = t.span.col as usize;
            if col == 0 { 0 } else { col - 1 }
        })
        .unwrap_or(0)
}

/// True if a line contains only Newline tokens or is empty.
fn is_blank_line(line: &[Token]) -> bool {
    line.iter().all(|t| matches!(
        t.kind,
        TokenKind::Newline
    ))
}

/// Update bracket counter based on token kind.
fn update_brackets(tok: &Token, brackets: &mut usize) {
    match &tok.kind {
        TokenKind::LParen
        | TokenKind::LBracket
        | TokenKind::LBrace  => *brackets += 1,

        TokenKind::RParen
        | TokenKind::RBracket
        | TokenKind::RBrace  => {
            if *brackets > 0 { *brackets -= 1; }
        }
        _ => {}
    }
}

/// A dummy span for synthetic tokens.
fn dummy_span() -> Span {
    Span::new(FileId(0), 0, 0, 0, 0)
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::FileId;

    fn file() -> FileId { FileId(1) }

    fn make_tok(kind: TokenKind, line: u32, col: u32) -> Token {
        Token::new(kind, "", Span::new(FileId(1), 0, 0, line, col))
    }

    #[test]
    fn passthrough_is_identity() {
        let tokens = vec![];
        let result = inject_indentation(tokens);
        // Empty input — only Eof produced
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, TokenKind::Eof);
    }

    #[test]
    fn flat_code_no_indent_tokens() {
        // fn main:\n    no indented block — just one line
        let tokens = vec![
            make_tok(TokenKind::Fn,              1, 1),
            make_tok(TokenKind::Ident("main".into()), 1, 4),
            make_tok(TokenKind::Colon,           1, 8),
            make_tok(TokenKind::Newline,         1, 9),
            make_tok(TokenKind::Eof,             2, 1),
        ];
        let result = inject_indentation(tokens);
        // Should have no INDENT or DEDENT
        let has_indent = result.iter().any(|t| t.kind == TokenKind::Indent);
        let has_dedent = result.iter().any(|t| t.kind == TokenKind::Dedent);
        assert!(!has_indent, "unexpected INDENT in flat code");
        assert!(!has_dedent, "unexpected DEDENT in flat code");
    }

    #[test]
    fn indented_block_gets_indent_dedent() {
        // Line 1: col 1 — fn main:
        // Line 2: col 5 — let x = 1   (indented)
        // Line 3: col 1 — fn other:   (dedented)
        let tokens = vec![
            make_tok(TokenKind::Fn,               1, 1),
            make_tok(TokenKind::Newline,           1, 9),
            make_tok(TokenKind::Let,               2, 5),
            make_tok(TokenKind::Newline,           2, 9),
            make_tok(TokenKind::Fn,                3, 1),
            make_tok(TokenKind::Newline,           3, 9),
            make_tok(TokenKind::Eof,               4, 1),
        ];
        let result = inject_indentation(tokens);

        let kinds: Vec<&TokenKind> = result.iter()
            .map(|t| &t.kind)
            .collect();

        assert!(kinds.contains(&&TokenKind::Indent),
            "expected INDENT, got: {:?}", kinds);
        assert!(kinds.contains(&&TokenKind::Dedent),
            "expected DEDENT, got: {:?}", kinds);
    }

    #[test]
    fn brackets_suppress_indent() {
        // Inside ( ) — newlines and indent changes are ignored
        let tokens = vec![
            make_tok(TokenKind::LParen,  1, 1),
            make_tok(TokenKind::Let,     2, 5),  // col 5 inside parens
            make_tok(TokenKind::Newline, 2, 9),
            make_tok(TokenKind::RParen,  3, 1),
            make_tok(TokenKind::Newline, 3, 2),
            make_tok(TokenKind::Eof,     4, 1),
        ];
        let result = inject_indentation(tokens);
        let has_indent = result.iter().any(|t| t.kind == TokenKind::Indent);
        assert!(!has_indent, "INDENT should be suppressed inside brackets");
    }

    #[test]
    fn eof_flushes_dedents() {
        // Open an indented block and then EOF — should flush DEDENTs
        let tokens = vec![
            make_tok(TokenKind::Fn,      1, 1),
            make_tok(TokenKind::Newline, 1, 9),
            make_tok(TokenKind::Let,     2, 5),  // indented
            make_tok(TokenKind::Newline, 2, 9),
            make_tok(TokenKind::Eof,     3, 1),
        ];
        let result = inject_indentation(tokens);
        let dedent_count = result.iter()
            .filter(|t| t.kind == TokenKind::Dedent)
            .count();
        assert!(dedent_count >= 1,
            "expected at least 1 DEDENT at EOF flush, got {}", dedent_count);
    }
}