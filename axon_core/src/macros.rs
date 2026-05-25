//! Core macros: axon_try!, axon_assert!, axon_unreachable!, axon_todo!
#[macro_export]
macro_rules! axon_try {
    ($expr:expr) => { match $expr {
        $crate::result::AxonResult::Ok(v)  => v,
        $crate::result::AxonResult::Err(e) => return $crate::result::AxonResult::Err(e),
    }};
}
#[macro_export]
macro_rules! axon_assert {
    ($c:expr)          => { if !$c { core::panic!("axon_assert! failed: {}", core::stringify!($c)); } };
    ($c:expr, $m:literal) => { if !$c { core::panic!("axon_assert! failed: {}", $m); } };
}
#[macro_export]
macro_rules! axon_unreachable {
    ()           => { core::panic!("axon_unreachable!() at {}:{}", file!(), line!()) };
    ($m:literal) => { core::panic!("axon_unreachable!(): {}", $m) };
}
#[macro_export]
macro_rules! axon_todo {
    ()           => { core::panic!("axon_todo!() at {}:{}", file!(), line!()) };
    ($m:literal) => { core::panic!("axon_todo!(): {}", $m) };
}
#[cfg(test)]
mod tests {
    use crate::prelude::*;
    fn ok()  -> AxonResult<i64> { AxonResult::Ok(7) }
    fn err() -> AxonResult<i64> { AxonResult::Err(AxonError::not_found("x")) }
    fn chain_ok()  -> AxonResult<i64> { let v = axon_try!(ok());  AxonResult::Ok(v+1) }
    fn chain_err() -> AxonResult<i64> { let _v = axon_try!(err()); AxonResult::Ok(99) }
    #[test] fn axon_try_ok_path()  { assert_eq!(chain_ok(), AxonResult::Ok(8)); }
    #[test] fn axon_try_err_path() { assert!(chain_err().is_err()); }
    #[test] fn axon_assert_passes(){ axon_assert!(1+1==2); }
    #[test] #[should_panic(expected="axon_assert! failed")] fn axon_assert_fails() { axon_assert!(1==2); }
    #[test] #[should_panic(expected="axon_unreachable")] fn axon_unreachable_panics() { axon_unreachable!(); }
}
