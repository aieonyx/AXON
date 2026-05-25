//! AXON primary result type.
//!
//! Use [`axon_try!`] for error propagation — it is the canonical AXON
//! equivalent of Rust's `?` operator.

use crate::error::AxonError;

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AxonResult<T> { Ok(T), Err(AxonError) }

impl<T> AxonResult<T> {
    pub const fn is_ok(&self)  -> bool { matches!(self, AxonResult::Ok(_)) }
    pub const fn is_err(&self) -> bool { !self.is_ok() }

    #[track_caller]
    pub fn unwrap(self) -> T {
        match self {
            AxonResult::Ok(v)  => v,
            AxonResult::Err(e) => panic!("AxonResult::unwrap() called on Err: {}", e),
        }
    }
    pub fn unwrap_or(self, default: T) -> T {
        match self { AxonResult::Ok(v) => v, AxonResult::Err(_) => default }
    }
    pub fn unwrap_or_else<F: FnOnce(AxonError) -> T>(self, f: F) -> T {
        match self { AxonResult::Ok(v) => v, AxonResult::Err(e) => f(e) }
    }
    #[track_caller]
    pub fn unwrap_err(self) -> AxonError where T: core::fmt::Debug {
        match self {
            AxonResult::Ok(v)  => panic!("AxonResult::unwrap_err() called on Ok: {:?}", v),
            AxonResult::Err(e) => e,
        }
    }
    pub fn ok(self)  -> Option<T>         { match self { AxonResult::Ok(v) => Some(v), AxonResult::Err(_) => None } }
    pub fn err(self) -> Option<AxonError> { match self { AxonResult::Ok(_) => None, AxonResult::Err(e) => Some(e) } }
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> AxonResult<U> {
        match self { AxonResult::Ok(v) => AxonResult::Ok(f(v)), AxonResult::Err(e) => AxonResult::Err(e) }
    }
    pub fn and_then<U, F: FnOnce(T) -> AxonResult<U>>(self, f: F) -> AxonResult<U> {
        match self { AxonResult::Ok(v) => f(v), AxonResult::Err(e) => AxonResult::Err(e) }
    }
    pub fn and<U>(self, other: AxonResult<U>) -> AxonResult<U> {
        match self { AxonResult::Ok(_) => other, AxonResult::Err(e) => AxonResult::Err(e) }
    }
    pub fn or_else<F: FnOnce(AxonError) -> AxonResult<T>>(self, f: F) -> AxonResult<T> {
        match self { AxonResult::Ok(_) => self, AxonResult::Err(e) => f(e) }
    }
    pub fn map_err<F: FnOnce(AxonError) -> AxonError>(self, f: F) -> AxonResult<T> {
        match self { AxonResult::Ok(v) => AxonResult::Ok(v), AxonResult::Err(e) => AxonResult::Err(f(e)) }
    }
    pub fn as_ref(&self) -> AxonResult<&T> {
        match self { AxonResult::Ok(v) => AxonResult::Ok(v), AxonResult::Err(e) => AxonResult::Err(*e) }
    }
}

impl<T> From<AxonResult<T>> for Result<T, AxonError> {
    fn from(r: AxonResult<T>) -> Self {
        match r { AxonResult::Ok(v) => Ok(v), AxonResult::Err(e) => Err(e) }
    }
}
impl<T> From<Result<T, AxonError>> for AxonResult<T> {
    fn from(r: Result<T, AxonError>) -> Self {
        match r { Ok(v) => AxonResult::Ok(v), Err(e) => AxonResult::Err(e) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{AxonError, ErrorKind};
    fn ok()  -> AxonResult<i64> { AxonResult::Ok(42) }
    fn err() -> AxonResult<i64> { AxonResult::Err(AxonError::not_found("x")) }

    #[test] fn result_ok_is_ok()  { assert!(ok().is_ok());  assert!(!ok().is_err()); }
    #[test] fn result_err_is_err(){ assert!(err().is_err()); assert!(!err().is_ok()); }
    #[test] fn result_map_ok()    { assert_eq!(ok().map(|v| v*2), AxonResult::Ok(84)); }
    #[test] fn result_map_err_passthrough() { assert!(err().map(|v| v*2).is_err()); }
    #[test] fn result_and_then_chains() {
        assert_eq!(ok().and_then(|v| AxonResult::Ok(v+8)).and_then(|v| AxonResult::Ok(v*2)), AxonResult::Ok(100));
    }
    #[test] fn result_and_then_short_circuits() {
        let mut called = false;
        let r = err().and_then(|_| { called=true; AxonResult::Ok(99_i64) });
        assert!(!called); assert!(r.is_err());
    }
    #[test] fn result_unwrap_or() { assert_eq!(err().unwrap_or(-1),-1); assert_eq!(ok().unwrap_or(-1),42); }
    #[test] fn result_ok_conversion()  { assert_eq!(ok().ok(),Some(42)); assert_eq!(err().ok(),None); }
    #[test] fn result_err_conversion() { assert!(ok().err().is_none()); assert_eq!(err().err().unwrap().kind, ErrorKind::NotFound); }
    #[test] fn result_map_err()  { assert_eq!(err().map_err(|_| AxonError::io("x")).err().unwrap().kind, ErrorKind::Io); }
    #[test] fn result_as_ref()   { assert_eq!(ok().as_ref(), AxonResult::Ok(&42)); }
    #[test] fn result_from_std() {
        let a: AxonResult<i64> = Ok(7_i64).map_err(|e: AxonError| e).into();
        assert_eq!(a, AxonResult::Ok(7));
    }
    #[test] fn result_unwrap_no_debug_bound() {
        struct Opaque(i32);
        assert_eq!(AxonResult::Ok(Opaque(7)).unwrap().0, 7);
    }
    #[test] #[should_panic(expected="NotFound")]
    fn panic_message_shows_kind_name() { err().unwrap(); }
    #[test] fn result_with_pal_error_code() {
        let e = AxonError::io("x").with_code(5);
        assert_eq!(AxonResult::<i64>::Err(e).err().unwrap().code, Some(5));
    }
    #[test] fn fuzz_result_all_variants_all_combinators() {
        use crate::error::ErrorKind::*;
        let kinds = [Io,NotFound,PermissionDenied,InvalidInput,TimedOut,
                     Overflow,Underflow,InvalidState,NotImplemented,
                     Verification,AiInference,Audit,Unknown];
        for k in kinds {
            let r: AxonResult<u64> = AxonResult::Err(AxonError::new(k,"fuzz"));
            assert!(r.is_err()); assert!(!r.is_ok());
            let mut hit = false;
            let _ = r.map(|v| { hit=true; v });   assert!(!hit);
            let _ = r.and_then(|v| { hit=true; AxonResult::Ok(v) }); assert!(!hit);
            assert_eq!(r.unwrap_or(0), 0);
            assert!(r.ok().is_none());
            assert_eq!(r.err().unwrap().kind, k);
        }
        for i in 0u64..100 {
            let r: AxonResult<u64> = AxonResult::Ok(i);
            assert!(r.is_ok());
            assert_eq!(r.map(|v| v*2), AxonResult::Ok(i*2));
            assert_eq!(r.unwrap_or(9999), i);
        }
    }
}
