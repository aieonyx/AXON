// ============================================================
// AXON Lexer — token.rs
// Complete TokenKind enum — v0.3
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
// ============================================================

use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind   : TokenKind,
    pub lexeme : String,
    pub span   : Span,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, span: Span) -> Self {
        Token { kind, lexeme: lexeme.into(), span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    IntLit(i64), FloatLit(f64), StrLit(String),
    StrInterpStart, StrInterpPart(String), StrInterpExprStart, StrInterpExprEnd, StrInterpEnd,
    BytesLit(Vec<u8>), BoolLit(bool),

    // Identifier
    Ident(String),

    // Declaration keywords
    Fn, Task, Struct, Enum, Type, Const, Impl, Trait, Module, Import, As,
    Actor,    // [v0.3]
    Handle,   // [v0.3]
    Reply,    // [v0.3]
    Opaque,   // [v0.3]

    // Binding keywords
    Let,
    LetAt,      // let@ [v0.3] ephemeral binding
    Mut, Own, Borrow, MutBorrow, Share,

    // Effect keyword
    Uses,       // [v0.3] effect declarations

    // Control flow
    If, Else, For,
    Foreach,    // [v0.3]
    While, Match, Return, Break, Continue, Pass, Then, In,
    Yield,      // [v0.3]

    // Intent keywords [v0.3]
    Intent, Secure, Performant, Auditable, Verifiable, MinimalRuntime,

    // Resource management keywords [v0.3.1]
    Defer,      // defer
    With,       // with

    // Concurrency
    Spawn, Await,

    // Memory
    Raw, Allocate, Free,

    // Values
    True, False, None, Some,

    // Logical
    And, Or, Not,

    // Type keywords
    TInt, TInt32, TInt64, TInt8,
    TUInt, TUInt32, TUInt64, TUInt8,
    TFloat, TFloat32,
    TBool, TChar, TStr, TBytes, TUnit,
    TOption, TResult, TList, TCap,
    TSend, TSync,
    TTainted,    // [v0.3]
    TClean,      // [v0.3]
    TProvenance, // [v0.3]

    // Temporal [v0.3]
    TemporalNow,
    TemporalLifetime,
    TemporalEpoch,

    // Program-level intent [v0.3.1]
    ProgramIntentDecl,   // @program_intent

    // Punctuation
    Dot, Comma, Colon, DoubleColon, Semicolon,
    LParen, RParen, LBracket, RBracket, LBrace, RBrace,
    Arrow, FatArrow, Question, At, Pipe, Ampersand, Star, Hash, Tilde,
    Bang,        // ! [v0.3] capability pin

    // v0.3 operators
    PipeForward, // |>
    TildeArrow,  // ~>

    // Arithmetic
    Plus, Minus, Slash, Percent, Caret,

    // Shift
    ShiftLeft, ShiftRight,

    // Comparison
    EqEq, BangEq, Lt, Gt, LtEq, GtEq,

    // Assignment
    Assign, PlusAssign, MinusAssign, StarAssign, SlashAssign,
    PercentAssign, AmpAssign, PipeAssign, CaretAssign, ShlAssign, ShrAssign,

    // Range
    DotDot, DotDotEq,

    // Indentation
    Indent, Dedent, Newline,

    // Turbofish
    TurboStart, // ::<

    // Meta
    Eof,
    Error(String),
    Comment(String),
    DocComment(String),
}

impl TokenKind {
    pub fn is_keyword(&self) -> bool {
        matches!(self,
            TokenKind::Fn | TokenKind::Task | TokenKind::Struct |
            TokenKind::Enum | TokenKind::Type | TokenKind::Const |
            TokenKind::Impl | TokenKind::Trait | TokenKind::Module |
            TokenKind::Import | TokenKind::As | TokenKind::Actor |
            TokenKind::Handle | TokenKind::Reply | TokenKind::Opaque |
            TokenKind::Let | TokenKind::LetAt | TokenKind::Mut |
            TokenKind::Own | TokenKind::Borrow | TokenKind::MutBorrow |
            TokenKind::Share | TokenKind::Uses |
            TokenKind::If | TokenKind::Else | TokenKind::For |
            TokenKind::Foreach | TokenKind::While | TokenKind::Match |
            TokenKind::Return | TokenKind::Break | TokenKind::Continue |
            TokenKind::Pass | TokenKind::Then | TokenKind::In |
            TokenKind::Yield | TokenKind::Defer | TokenKind::With | TokenKind::Intent | TokenKind::Secure |
            TokenKind::Performant | TokenKind::Auditable |
            TokenKind::Verifiable | TokenKind::MinimalRuntime |
            TokenKind::Spawn | TokenKind::Await |
            TokenKind::Raw | TokenKind::Allocate | TokenKind::Free |
            TokenKind::And | TokenKind::Or | TokenKind::Not
        )
    }

    pub fn is_decl_start(&self) -> bool {
        matches!(self,
            TokenKind::Fn | TokenKind::Task | TokenKind::Struct |
            TokenKind::Enum | TokenKind::Type | TokenKind::Const |
            TokenKind::Impl | TokenKind::Trait | TokenKind::Actor |
            TokenKind::Opaque | TokenKind::At | TokenKind::Eof
        )
    }

    pub fn is_stmt_start(&self) -> bool {
        matches!(self,
            TokenKind::Let | TokenKind::LetAt | TokenKind::Mut |
            TokenKind::Return | TokenKind::If | TokenKind::For |
            TokenKind::Foreach | TokenKind::While | TokenKind::Match |
            TokenKind::Break | TokenKind::Continue | TokenKind::Pass |
            TokenKind::Spawn | TokenKind::Raw | TokenKind::Intent |
            TokenKind::Dedent | TokenKind::Ident(_)
        )
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TokenKind::Fn           => "'fn'",
            TokenKind::Task         => "'task'",
            TokenKind::Struct       => "'struct'",
            TokenKind::Enum         => "'enum'",
            TokenKind::Actor        => "'actor'",
            TokenKind::Handle       => "'handle'",
            TokenKind::Reply        => "'reply'",
            TokenKind::Opaque       => "'opaque'",
            TokenKind::Let          => "'let'",
            TokenKind::LetAt        => "'let@'",
            TokenKind::Mut          => "'mut'",
            TokenKind::Uses         => "'uses'",
            TokenKind::If           => "'if'",
            TokenKind::Else         => "'else'",
            TokenKind::For          => "'for'",
            TokenKind::Foreach      => "'foreach'",
            TokenKind::While        => "'while'",
            TokenKind::Match        => "'match'",
            TokenKind::Return       => "'return'",
            TokenKind::Yield        => "'yield'",
            TokenKind::Intent       => "'intent'",
            TokenKind::Spawn        => "'spawn'",
            TokenKind::Await        => "'await'",
            TokenKind::Colon        => "':'",
            TokenKind::Arrow        => "'->'",
            TokenKind::FatArrow     => "'=>'",
            TokenKind::PipeForward  => "'|>'",
            TokenKind::TildeArrow   => "'~>'",
            TokenKind::Bang         => "'!'",
            TokenKind::LParen       => "'('",
            TokenKind::RParen       => "')'",
            TokenKind::LBracket     => "'['",
            TokenKind::RBracket     => "']'",
            TokenKind::LBrace       => "'{'",
            TokenKind::RBrace       => "'}'",
            TokenKind::Comma        => "','",
            TokenKind::Dot          => "'.'",
            TokenKind::Assign       => "'='",
            TokenKind::Indent       => "INDENT",
            TokenKind::Dedent       => "DEDENT",
            TokenKind::Newline      => "NEWLINE",
            TokenKind::Eof          => "end of file",
            TokenKind::Ident(_)     => "identifier",
            TokenKind::IntLit(_)    => "integer literal",
            TokenKind::FloatLit(_)  => "float literal",
            TokenKind::StrLit(_)    => "string literal",
            TokenKind::BoolLit(_)   => "boolean literal",
            TokenKind::TemporalNow       => "'@now'",
            TokenKind::TemporalLifetime  => "'@lifetime'",
            TokenKind::TemporalEpoch     => "'@epoch'",
            TokenKind::ProgramIntentDecl => "'@program_intent'",
            _                       => "token",
        }
    }
}

pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
    match s {
        "fn" => Some(TokenKind::Fn), "task" => Some(TokenKind::Task),
        "struct" => Some(TokenKind::Struct), "enum" => Some(TokenKind::Enum),
        "type" => Some(TokenKind::Type), "const" => Some(TokenKind::Const),
        "impl" => Some(TokenKind::Impl), "trait" => Some(TokenKind::Trait),
        "module" => Some(TokenKind::Module), "import" => Some(TokenKind::Import),
        "as" => Some(TokenKind::As), "actor" => Some(TokenKind::Actor),
        "handle" => Some(TokenKind::Handle), "reply" => Some(TokenKind::Reply),
        "opaque" => Some(TokenKind::Opaque),
        "let" => Some(TokenKind::Let), "mut" => Some(TokenKind::Mut),
        "own" => Some(TokenKind::Own), "borrow" => Some(TokenKind::Borrow),
        "mutborrow" => Some(TokenKind::MutBorrow), "share" => Some(TokenKind::Share),
        "uses" => Some(TokenKind::Uses),
        "if" => Some(TokenKind::If), "else" => Some(TokenKind::Else),
        "for" => Some(TokenKind::For), "foreach" => Some(TokenKind::Foreach),
        "while" => Some(TokenKind::While), "match" => Some(TokenKind::Match),
        "return" => Some(TokenKind::Return), "break" => Some(TokenKind::Break),
        "continue" => Some(TokenKind::Continue), "pass" => Some(TokenKind::Pass),
        "then" => Some(TokenKind::Then), "in" => Some(TokenKind::In),
        "yield" => Some(TokenKind::Yield),
        "defer" => Some(TokenKind::Defer),
        "with"  => Some(TokenKind::With),
        "intent" => Some(TokenKind::Intent), "secure" => Some(TokenKind::Secure),
        "performant" => Some(TokenKind::Performant), "auditable" => Some(TokenKind::Auditable),
        "verifiable" => Some(TokenKind::Verifiable),
        "minimal_runtime" => Some(TokenKind::MinimalRuntime),
        "spawn" => Some(TokenKind::Spawn), "await" => Some(TokenKind::Await),
        "raw" => Some(TokenKind::Raw), "allocate" => Some(TokenKind::Allocate),
        "free" => Some(TokenKind::Free),
        "true" => Some(TokenKind::BoolLit(true)),
        "false" => Some(TokenKind::BoolLit(false)),
        "None" => Some(TokenKind::None), "Some" => Some(TokenKind::Some),
        "and" => Some(TokenKind::And), "or" => Some(TokenKind::Or),
        "not" => Some(TokenKind::Not),
        "Int" => Some(TokenKind::TInt), "Int32" => Some(TokenKind::TInt32),
        "Int64" => Some(TokenKind::TInt64), "Int8" => Some(TokenKind::TInt8),
        "UInt" => Some(TokenKind::TUInt), "UInt32" => Some(TokenKind::TUInt32),
        "UInt64" => Some(TokenKind::TUInt64), "UInt8" => Some(TokenKind::TUInt8),
        "Float" => Some(TokenKind::TFloat), "Float32" => Some(TokenKind::TFloat32),
        "Bool" => Some(TokenKind::TBool), "Char" => Some(TokenKind::TChar),
        "Str" => Some(TokenKind::TStr), "Bytes" => Some(TokenKind::TBytes),
        "Unit" => Some(TokenKind::TUnit), "Option" => Some(TokenKind::TOption),
        "Result" => Some(TokenKind::TResult), "List" => Some(TokenKind::TList),
        "cap" => Some(TokenKind::TCap), "Send" => Some(TokenKind::TSend),
        "Sync" => Some(TokenKind::TSync), "Tainted" => Some(TokenKind::TTainted),
        "Clean" => Some(TokenKind::TClean), "Provenance" => Some(TokenKind::TProvenance),
        _ => None,
    }
}

pub fn temporal_from_str(s: &str) -> Option<TokenKind> {
    match s {
        "now"            => Some(TokenKind::TemporalNow),
        "lifetime"       => Some(TokenKind::TemporalLifetime),
        "epoch"          => Some(TokenKind::TemporalEpoch),
        "program_intent" => Some(TokenKind::ProgramIntentDecl),
        _                => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v02_keywords_still_recognized() {
        let kws = vec!["fn","task","struct","enum","let","mut","if","else",
            "for","while","match","return","break","continue","pass","spawn",
            "await","raw","and","or","not","Int","Float","Bool","Str","Option",
            "Result","List","cap","Send","Sync"];
        for kw in kws { assert!(keyword_from_str(kw).is_some(), "missing: {}", kw); }
    }

    #[test]
    fn test_v03_new_keywords_recognized() {
        let kws = vec!["actor","handle","reply","opaque","uses","foreach",
            "yield","intent","secure","performant","auditable","verifiable",
            "minimal_runtime","Tainted","Clean","Provenance"];
        for kw in kws { assert!(keyword_from_str(kw).is_some(), "missing: {}", kw); }
    }

    #[test]
    fn test_temporal_keywords() {
        assert_eq!(temporal_from_str("now"),      Some(TokenKind::TemporalNow));
        assert_eq!(temporal_from_str("lifetime"), Some(TokenKind::TemporalLifetime));
        assert_eq!(temporal_from_str("epoch"),    Some(TokenKind::TemporalEpoch));
        assert_eq!(temporal_from_str("other"),    None);
    }

    #[test]
    fn test_let_at_distinct_from_let() {
        assert_ne!(
            std::mem::discriminant(&TokenKind::Let),
            std::mem::discriminant(&TokenKind::LetAt)
        );
    }

    #[test]
    fn test_pipe_forward_distinct_from_pipe() {
        assert_ne!(
            std::mem::discriminant(&TokenKind::Pipe),
            std::mem::discriminant(&TokenKind::PipeForward)
        );
    }

    #[test]
    fn test_bool_literals() {
        assert_eq!(keyword_from_str("true"),  Some(TokenKind::BoolLit(true)));
        assert_eq!(keyword_from_str("false"), Some(TokenKind::BoolLit(false)));
    }

    #[test]
    fn test_non_keywords_not_recognized() {
        assert!(keyword_from_str("foo").is_none());
        assert!(keyword_from_str("").is_none());
    }

    #[test]
    fn test_program_intent_token() {
        // @program_intent is lexed via temporal_from_str after seeing @
        assert_eq!(
            temporal_from_str("program_intent"),
            Some(TokenKind::ProgramIntentDecl)
        );
        // Distinguish from temporal tokens
        assert_ne!(
            temporal_from_str("program_intent"),
            temporal_from_str("now")
        );
    }

    #[test]
    fn test_v031_new_keywords_recognized() {
        assert!(keyword_from_str("defer").is_some(), "defer missing");
        assert!(keyword_from_str("with").is_some(),  "with missing");
    }
}
