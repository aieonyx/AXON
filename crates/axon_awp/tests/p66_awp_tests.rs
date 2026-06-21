// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P66 — axon_awp tests (20 tests)

use axon_awp::types::{AwpStatus, AwpMethod, AwpRequest, CATEGORIES};
use axon_awp::parser::{parse, is_awp};
use axon_awp::router::AwpRouter;

// ── T1: is_awp detects awp:// prefix ─────────────────────────────────────────
#[test]
fn t1_is_awp() {
    assert!(is_awp("awp://mynode.dev"));
    assert!(!is_awp("https://example.com"));
    assert!(!is_awp("awp"));
}

// ── T2: parse global address ──────────────────────────────────────────────────
#[test]
fn t2_parse_global() {
    let addr = parse("awp://aieonyx.dev").unwrap();
    assert_eq!(addr.name, "aieonyx");
    assert_eq!(addr.category, "dev");
    assert!(addr.region.is_none());
    assert_eq!(addr.path, "/");
}

// ── T3: parse regional address ────────────────────────────────────────────────
#[test]
fn t3_parse_regional() {
    let addr = parse("awp://josebank.bank.ph").unwrap();
    assert_eq!(addr.name, "josebank");
    assert_eq!(addr.category, "bank");
    assert_eq!(addr.region.as_deref(), Some("ph"));
}

// ── T4: parse with path ───────────────────────────────────────────────────────
#[test]
fn t4_parse_with_path() {
    let addr = parse("awp://aieonyx.dev/dashboard").unwrap();
    assert_eq!(addr.path, "/dashboard");
    assert_eq!(addr.name, "aieonyx");
}

// ── T5: parse invalid scheme ─────────────────────────────────────────────────
#[test]
fn t5_parse_invalid_scheme() {
    let err = parse("https://aieonyx.dev").unwrap_err();
    assert!(matches!(err, axon_awp::types::AwpError::InvalidScheme(_)));
}

// ── T6: parse invalid category ───────────────────────────────────────────────
#[test]
fn t6_parse_invalid_category() {
    let err = parse("awp://aieonyx.cloud").unwrap_err();
    assert!(matches!(err, axon_awp::types::AwpError::InvalidCategory(_)));
}

// ── T7: parse invalid region ─────────────────────────────────────────────────
#[test]
fn t7_parse_invalid_region() {
    let err = parse("awp://aieonyx.dev.xx").unwrap_err();
    assert!(matches!(err, axon_awp::types::AwpError::InvalidRegion(_)));
}

// ── T8: parse invalid name (hyphen) ──────────────────────────────────────────
#[test]
fn t8_parse_invalid_name() {
    let err = parse("awp://my-node.dev").unwrap_err();
    assert!(matches!(err, axon_awp::types::AwpError::InvalidName(_)));
}

// ── T9: all categories are valid ─────────────────────────────────────────────
#[test]
fn t9_all_categories_valid() {
    for cat in CATEGORIES {
        let uri = format!("awp://testnode.{}", cat);
        assert!(parse(&uri).is_ok(), "category {} should be valid", cat);
    }
}

// ── T10: address to_string roundtrip ─────────────────────────────────────────
#[test]
fn t10_address_to_string() {
    let addr = parse("awp://aieonyx.dev").unwrap();
    assert_eq!(addr.to_string(), "awp://aieonyx.dev");
}

// ── T11: regional address to_string ──────────────────────────────────────────
#[test]
fn t11_regional_to_string() {
    let addr = parse("awp://josebank.bank.ph").unwrap();
    assert_eq!(addr.to_string(), "awp://josebank.bank.ph");
}

// ── T12: is_global / is_regional flags ───────────────────────────────────────
#[test]
fn t12_global_regional_flags() {
    let global = parse("awp://aieonyx.dev").unwrap();
    let regional = parse("awp://aieonyx.dev.cz").unwrap();
    assert!(global.is_global());
    assert!(!global.is_regional());
    assert!(regional.is_regional());
    assert!(!regional.is_global());
}

// ── T13: router register and dispatch ────────────────────────────────────────
#[test]
fn t13_router_register_dispatch() {
    let mut router = AwpRouter::new();
    router.register("aieonyx.dev", Box::new(|_req| {
        axon_awp::types::AwpResponse::ok(b"sovereign home".to_vec())
    })).unwrap();
    let resp = router.resolve("awp://aieonyx.dev").unwrap();
    assert_eq!(resp.status, AwpStatus::Ok);
    assert_eq!(resp.body, b"sovereign home");
}

// ── T14: router returns 404 for unknown route ────────────────────────────────
#[test]
fn t14_router_not_found() {
    let router = AwpRouter::new();
    let resp = router.resolve("awp://ghost.mesh").unwrap();
    assert_eq!(resp.status, AwpStatus::NotFound);
}

// ── T15: router global fallback for regional request ─────────────────────────
#[test]
fn t15_router_global_fallback() {
    let mut router = AwpRouter::new();
    // Register global only
    router.register("aieonyx.dev", Box::new(|_| {
        axon_awp::types::AwpResponse::ok(b"global".to_vec())
    })).unwrap();
    // Request regional — should fall back to global
    let resp = router.resolve("awp://aieonyx.dev.cz").unwrap();
    assert_eq!(resp.status, AwpStatus::Ok);
    assert_eq!(resp.body, b"global");
}

// ── T16: router regional takes priority over global ──────────────────────────
#[test]
fn t16_router_regional_priority() {
    let mut router = AwpRouter::new();
    router.register("aieonyx.dev", Box::new(|_| {
        axon_awp::types::AwpResponse::ok(b"global".to_vec())
    })).unwrap();
    router.register("aieonyx.dev.cz", Box::new(|_| {
        axon_awp::types::AwpResponse::ok(b"czech".to_vec())
    })).unwrap();
    let resp = router.resolve("awp://aieonyx.dev.cz").unwrap();
    assert_eq!(resp.body, b"czech");
    let resp2 = router.resolve("awp://aieonyx.dev").unwrap();
    assert_eq!(resp2.body, b"global");
}

// ── T17: router route_count ──────────────────────────────────────────────────
#[test]
fn t17_router_count() {
    let mut router = AwpRouter::new();
    assert_eq!(router.route_count(), 0);
    router.register("a.dev", Box::new(|_| axon_awp::types::AwpResponse::ok(b"".to_vec()))).unwrap();
    router.register("b.social", Box::new(|_| axon_awp::types::AwpResponse::ok(b"".to_vec()))).unwrap();
    assert_eq!(router.route_count(), 2);
}

// ── T18: router list_routes sorted ───────────────────────────────────────────
#[test]
fn t18_router_list_routes() {
    let mut router = AwpRouter::new();
    router.register("zzz.dev", Box::new(|_| axon_awp::types::AwpResponse::ok(b"".to_vec()))).unwrap();
    router.register("aaa.gov", Box::new(|_| axon_awp::types::AwpResponse::ok(b"".to_vec()))).unwrap();
    let routes = router.list_routes();
    assert_eq!(routes[0], "aaa.gov");
    assert_eq!(routes[1], "zzz.dev");
}

// ── T19: AwpResponse helpers ──────────────────────────────────────────────────
#[test]
fn t19_response_helpers() {
    let ok = axon_awp::types::AwpResponse::ok(b"data".to_vec());
    assert_eq!(ok.status, AwpStatus::Ok);
    let nf = axon_awp::types::AwpResponse::not_found();
    assert_eq!(nf.status, AwpStatus::NotFound);
    let fb = axon_awp::types::AwpResponse::forbidden();
    assert_eq!(fb.status, AwpStatus::Forbidden);
}

// ── T20: parse case-normalisation ────────────────────────────────────────────
#[test]
fn t20_parse_case_normalisation() {
    // AWP addresses are always lowercase
    let addr = parse("awp://AIEONYX.DEV").unwrap();
    assert_eq!(addr.name, "aieonyx");
    assert_eq!(addr.category, "dev");
}
