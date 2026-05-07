// ============================================================
// AXON Lexer — span.rs
// Source location tracking for all AST nodes and tokens
// ============================================================

/// Opaque file identifier — index into the compiler's file table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

/// A source location — byte offsets into the original source string.
/// Every token and AST node carries a Span for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub file  : FileId,
    pub start : usize,   // byte offset — inclusive
    pub end   : usize,   // byte offset — exclusive
    pub line  : u32,     // 1-indexed line number
    pub col   : u32,     // 1-indexed column number
}

impl Span {
    /// Construct a new span.
    pub fn new(file: FileId, start: usize, end: usize, line: u32, col: u32) -> Self {
        Span { file, start, end, line, col }
    }

    /// A dummy span used for synthetic/generated nodes.
    pub fn dummy() -> Self {
        Span { file: FileId(0), start: 0, end: 0, line: 0, col: 0 }
    }

    /// Merge two spans into one covering both.
    /// Used to compute parent AST node spans from child spans.
    pub fn merge(self, other: Span) -> Span {
        Span {
            file  : self.file,
            start : self.start.min(other.start),
            end   : self.end.max(other.end),
            line  : self.line.min(other.line),
            col   : self.col,
        }
    }

    /// Length of the span in bytes.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// True if this span covers zero bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_merge() {
        let a = Span::new(FileId(0), 0, 5, 1, 1);
        let b = Span::new(FileId(0), 3, 10, 1, 4);
        let merged = a.merge(b);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 10);
    }

    #[test]
    fn test_span_len() {
        let s = Span::new(FileId(0), 2, 7, 1, 3);
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn test_span_dummy() {
        let d = Span::dummy();
        assert_eq!(d.start, 0);
        assert_eq!(d.end, 0);
        assert!(d.is_empty());
    }
}
