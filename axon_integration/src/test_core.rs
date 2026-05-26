//! Integration tests — axon_core foundation layer.
use axon_core::prelude::*;

// ── AxonResult combinators ────────────────────────────────────────────────────
#[test] fn core_result_ok_chain() {
    let r: AxonResult<i64> = AxonResult::Ok(1)
        .and_then(|v| AxonResult::Ok(v + 1))
        .and_then(|v| AxonResult::Ok(v * 10))
        .map(|v| v + 5);
    assert_eq!(r, AxonResult::Ok(25));
}
#[test] fn core_result_err_short_circuits() {
    let mut steps = 0u32;
    let r: AxonResult<i64> = AxonResult::Err(AxonError::not_found("x"))
        .and_then(|v| { steps += 1; AxonResult::Ok(v) })
        .map(|v|       { steps += 1; v });
    assert!(r.is_err()); assert_eq!(steps, 0);
}
#[test] fn core_result_unwrap_or_else() {
    let v = AxonResult::<i64>::Err(AxonError::io("fail"))
        .unwrap_or_else(|e| { assert_eq!(e.kind, ErrorKind::Io); -1 });
    assert_eq!(v, -1);
}
#[test] fn core_error_with_code_preserves_kind() {
    let e = AxonError::io("syscall failed").with_code(9);
    assert_eq!(e.kind, ErrorKind::Io);
    assert_eq!(e.code, Some(9));
}
#[test] fn core_error_kind_discriminants_stable() {
    assert_eq!(ErrorKind::Io as u8, 0);
    assert_eq!(ErrorKind::NotFound as u8, 1);
    assert_eq!(ErrorKind::Unknown as u8, 255);
}
#[test] fn core_axon_try_ok() {
    fn f() -> AxonResult<i64> { let v = axon_try!(AxonResult::Ok(42)); AxonResult::Ok(v * 2) }
    assert_eq!(f(), AxonResult::Ok(84));
}
#[test] fn core_axon_try_err() {
    fn f() -> AxonResult<i64> { axon_try!(AxonResult::Err(AxonError::not_found("x"))); AxonResult::Ok(0) }
    assert!(f().is_err());
}
#[test] fn core_types_sizes() {
    use core::mem::size_of;
    assert_eq!(size_of::<AxonInt>(), 8);
    assert_eq!(size_of::<AxonFloat>(), 8);
    assert_eq!(size_of::<AxonBool>(), 1);
}
#[test] fn core_fnv_hasher_deterministic() {
    let mut h1 = FnvHasher::new(); "axon_sovereign".axon_hash(&mut h1);
    let mut h2 = FnvHasher::new(); "axon_sovereign".axon_hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}
#[test] fn core_fnv_different_inputs_differ() {
    let mut h1 = FnvHasher::new(); "alpha".axon_hash(&mut h1);
    let mut h2 = FnvHasher::new(); "beta".axon_hash(&mut h2);
    assert_ne!(h1.finish(), h2.finish());
}
#[test] fn core_axon_from_reflexive() { assert_eq!(i64::axon_from(42_i64), 42); }
#[test] fn core_axon_into_reflexive() { let v: i64 = 99_i64.axon_into(); assert_eq!(v, 99); }
#[test] fn core_prelude_all_items_accessible() {
    let _: AxonInt = 0; let _: AxonFloat = 0.0;
    let ok: AxonResult<i32> = AxonResult::Ok(1); assert!(ok.is_ok());
    let mut h = FnvHasher::new(); 42u64.axon_hash(&mut h); let _ = h.finish();
}
#[test] fn core_error_is_kind_exhaustive() {
    let variants = [ErrorKind::Io, ErrorKind::NotFound, ErrorKind::PermissionDenied,
        ErrorKind::InvalidInput, ErrorKind::TimedOut, ErrorKind::Overflow,
        ErrorKind::Underflow, ErrorKind::InvalidState, ErrorKind::NotImplemented,
        ErrorKind::Verification, ErrorKind::AiInference, ErrorKind::Audit, ErrorKind::Unknown];
    for v in variants { let e = AxonError::new(v, "t"); assert!(e.is_kind(v)); }
}
#[test] fn core_result_as_ref() { assert_eq!(AxonResult::Ok(42i64).as_ref(), AxonResult::Ok(&42)); }
#[test] fn core_result_map_err() {
    let r = AxonResult::<i64>::Err(AxonError::not_found("x"))
        .map_err(|_| AxonError::io("remapped"));
    assert_eq!(r.err().unwrap().kind, ErrorKind::Io);
}
