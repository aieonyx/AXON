//! AXON unified error type.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum ErrorKind {
    Io = 0, NotFound = 1, PermissionDenied = 2, InvalidInput = 3,
    TimedOut = 4, Overflow = 5, Underflow = 6, InvalidState = 7,
    NotImplemented = 8, Verification = 9, AiInference = 10,
    Audit = 11, Unknown = 255,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AxonError {
    pub kind: ErrorKind,
    pub message: &'static str,
    pub code: Option<u32>,
}

impl AxonError {
    pub const fn new(kind: ErrorKind, message: &'static str) -> Self {
        Self { kind, message, code: None }
    }
    pub const fn with_code(self, code: u32) -> Self { Self { code: Some(code), ..self } }
    pub const fn not_found(m: &'static str)       -> Self { Self::new(ErrorKind::NotFound, m) }
    pub const fn invalid_input(m: &'static str)   -> Self { Self::new(ErrorKind::InvalidInput, m) }
    pub const fn not_implemented(m: &'static str) -> Self { Self::new(ErrorKind::NotImplemented, m) }
    pub const fn permission_denied(m: &'static str) -> Self { Self::new(ErrorKind::PermissionDenied, m) }
    pub const fn io(m: &'static str)              -> Self { Self::new(ErrorKind::Io, m) }
    pub const fn timed_out(m: &'static str)       -> Self { Self::new(ErrorKind::TimedOut, m) }
    pub const fn verification(m: &'static str)    -> Self { Self::new(ErrorKind::Verification, m) }
    pub const fn invalid_state(m: &'static str)   -> Self { Self::new(ErrorKind::InvalidState, m) }
    pub const fn unknown(m: &'static str)         -> Self { Self::new(ErrorKind::Unknown, m) }
    pub const fn is_kind(&self, kind: ErrorKind) -> bool {
        (self.kind as u8) == (kind as u8)
    }
}

impl core::fmt::Display for AxonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.code {
            Some(c) => write!(f, "[{:?}({})] {}", self.kind, c, self.message),
            None    => write!(f, "[{:?}] {}", self.kind, self.message),
        }
    }
}

#[cfg(feature = "error_in_core")]
impl core::error::Error for AxonError {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn error_kind_not_found() {
        let e = AxonError::not_found("missing"); assert_eq!(e.kind, ErrorKind::NotFound); assert_eq!(e.code, None);
    }
    #[test] fn error_const_constructors() {
        const E: AxonError = AxonError::not_found("x"); assert_eq!(E.kind, ErrorKind::NotFound);
    }
    #[test] fn error_with_code() {
        let e = AxonError::io("x").with_code(5); assert_eq!(e.code, Some(5));
    }
    #[test] fn error_is_kind() {
        let e = AxonError::timed_out("x"); assert!(e.is_kind(ErrorKind::TimedOut)); assert!(!e.is_kind(ErrorKind::NotFound));
    }
    #[test] fn error_kind_repr_u8() {
        assert_eq!(ErrorKind::Io as u8, 0); assert_eq!(ErrorKind::NotFound as u8, 1); assert_eq!(ErrorKind::Unknown as u8, 255);
    }
    #[test] fn errorkind_discriminants_are_stable() {
        assert_eq!(ErrorKind::Io as u8, 0); assert_eq!(ErrorKind::PermissionDenied as u8, 2);
        assert_eq!(ErrorKind::Verification as u8, 9); assert_eq!(ErrorKind::Unknown as u8, 255);
    }
    #[test] fn is_kind_covers_all_variants() {
        let variants = [ErrorKind::Io,ErrorKind::NotFound,ErrorKind::PermissionDenied,
            ErrorKind::InvalidInput,ErrorKind::TimedOut,ErrorKind::Overflow,ErrorKind::Underflow,
            ErrorKind::InvalidState,ErrorKind::NotImplemented,ErrorKind::Verification,
            ErrorKind::AiInference,ErrorKind::Audit,ErrorKind::Unknown];
        for &v in &variants { let e = AxonError::new(v,"t"); assert!(e.is_kind(v)); }
    }
    #[test] fn error_code_round_trips() {
        let e = AxonError::io("x"); assert_eq!(e.code,None);
        let e2 = e.with_code(9); assert_eq!(e2.code,Some(9)); assert_eq!(e.code,None);
        assert_eq!(AxonError::unknown("x").with_code(u32::MAX).code, Some(u32::MAX));
    }
    #[test] fn error_display() {
        use std::format;
        let s = format!("{}", AxonError::not_found("gone")); assert!(s.contains("NotFound")); assert!(s.contains("gone"));
        let s2 = format!("{}", AxonError::io("x").with_code(5)); assert!(s2.contains("5"));
    }
    #[test] fn error_copy() { let e = AxonError::io("x"); let e2 = e; assert_eq!(e,e2); }
    #[cfg(feature = "error_in_core")]
    #[test] fn axon_error_implements_core_error() {
        use core::error::Error; let e = AxonError::not_found("x");
        let _: &dyn Error = &e; assert!(e.source().is_none());
    }
}
