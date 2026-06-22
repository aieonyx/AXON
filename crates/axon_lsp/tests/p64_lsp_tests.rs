// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P64 — axon_lsp tests (20 tests)

use axon_lsp::types::*;
use axon_lsp::handler::{LspHandler, word_at};
use axon_lsp::document::{DocumentStore, parse_diagnostics};
use serde_json::json;

fn make_req(method: &str, id: Option<i64>, params: serde_json::Value) -> RpcRequest {
    RpcRequest {
        jsonrpc: "2.0".into(),
        id: id.map(|i| json!(i)),
        method: method.into(),
        params,
    }
}

// ── T1: RpcResponse::ok serializes correctly ──────────────────────────────────
#[test]
fn t1_rpc_response_ok() {
    let r = RpcResponse::ok(Some(json!(1)), json!({"foo": "bar"}));
    assert_eq!(r.jsonrpc, "2.0");
    assert!(r.error.is_none());
    assert!(r.result.is_some());
}

// ── T2: RpcResponse::err serializes correctly ─────────────────────────────────
#[test]
fn t2_rpc_response_err() {
    let r = RpcResponse::err(Some(json!(1)), -32601, "method not found");
    assert!(r.result.is_none());
    assert_eq!(r.error.as_ref().unwrap().code, -32601);
}

// ── T3: initialize returns capabilities ──────────────────────────────────────
#[test]
fn t3_initialize() {
    let mut h = LspHandler::new();
    let req = make_req("initialize", Some(1), json!({"capabilities": {}}));
    let resp = h.handle(&req).unwrap();
    let result = resp.result.unwrap();
    assert!(result["capabilities"]["textDocumentSync"].as_u64().unwrap() == 1);
    assert_eq!(result["serverInfo"]["name"], "axon_lsp");
}

// ── T4: initialized notification returns None ────────────────────────────────
#[test]
fn t4_initialized_notification() {
    let mut h = LspHandler::new();
    let req = make_req("initialized", None, json!({}));
    assert!(h.handle(&req).is_none());
    assert!(h.initialized);
}

// ── T5: shutdown returns null result ─────────────────────────────────────────
#[test]
fn t5_shutdown() {
    let mut h = LspHandler::new();
    let req = make_req("shutdown", Some(2), json!(null));
    let resp = h.handle(&req).unwrap();
    assert!(resp.result.unwrap().is_null());
    assert!(h.shutdown);
}

// ── T6: unknown request returns MethodNotFound ────────────────────────────────
#[test]
fn t6_unknown_request() {
    let mut h = LspHandler::new();
    let req = make_req("textDocument/nonexistent", Some(3), json!({}));
    let resp = h.handle(&req).unwrap();
    assert_eq!(resp.error.unwrap().code, -32601);
}

// ── T7: unknown notification returns None ────────────────────────────────────
#[test]
fn t7_unknown_notification() {
    let mut h = LspHandler::new();
    let req = make_req("$/someNotification", None, json!({}));
    assert!(h.handle(&req).is_none());
}

// ── T8: didOpen stores document ───────────────────────────────────────────────
#[test]
fn t8_did_open() {
    let mut h = LspHandler::new();
    let req = make_req("textDocument/didOpen", None, json!({
        "textDocument": {
            "uri": "file:///test.ax",
            "languageId": "axonyx",
            "version": 1,
            "text": "fn main() -> i64 { 42 }"
        }
    }));
    h.handle(&req);
    assert!(h.store.get("file:///test.ax").is_some());
}

// ── T9: didChange updates document ───────────────────────────────────────────
#[test]
fn t9_did_change() {
    let mut h = LspHandler::new();
    // Open first
    let open = make_req("textDocument/didOpen", None, json!({
        "textDocument": {
            "uri": "file:///test.ax",
            "languageId": "axonyx",
            "version": 1,
            "text": "fn main() -> i64 { 42 }"
        }
    }));
    h.handle(&open);
    // Change
    let change = make_req("textDocument/didChange", None, json!({
        "textDocument": { "uri": "file:///test.ax", "version": 2 },
        "contentChanges": [{ "text": "fn foo() -> i64 { 0 }" }]
    }));
    h.handle(&change);
    let doc = h.store.get("file:///test.ax").unwrap();
    assert_eq!(doc.version, 2);
    assert!(doc.text.contains("foo"));
}

// ── T10: didClose removes document ───────────────────────────────────────────
#[test]
fn t10_did_close() {
    let mut h = LspHandler::new();
    let open = make_req("textDocument/didOpen", None, json!({
        "textDocument": {
            "uri": "file:///test.ax",
            "languageId": "axonyx",
            "version": 1,
            "text": "fn main() -> i64 { 42 }"
        }
    }));
    h.handle(&open);
    let close = make_req("textDocument/didClose", None, json!({
        "textDocument": { "uri": "file:///test.ax" }
    }));
    h.handle(&close);
    assert!(h.store.get("file:///test.ax").is_none());
}

// ── T11: hover on known word returns content ──────────────────────────────────
#[test]
fn t11_hover_word() {
    let mut h = LspHandler::new();
    let open = make_req("textDocument/didOpen", None, json!({
        "textDocument": {
            "uri": "file:///test.ax",
            "languageId": "axonyx",
            "version": 1,
            "text": "fn main() -> i64 { 42 }"
        }
    }));
    h.handle(&open);
    let hover = make_req("textDocument/hover", Some(4), json!({
        "textDocument": { "uri": "file:///test.ax" },
        "position": { "line": 0, "character": 3 }
    }));
    let resp = h.handle(&hover).unwrap();
    let result = resp.result.unwrap();
    assert!(!result.is_null());
    assert!(result["contents"]["value"].as_str().unwrap().contains("main"));
}

// ── T12: hover on whitespace returns null ─────────────────────────────────────
#[test]
fn t12_hover_whitespace() {
    let mut h = LspHandler::new();
    let open = make_req("textDocument/didOpen", None, json!({
        "textDocument": {
            "uri": "file:///test.ax",
            "languageId": "axonyx",
            "version": 1,
            "text": "fn main() -> i64 { 42 }"
        }
    }));
    h.handle(&open);
    let hover = make_req("textDocument/hover", Some(5), json!({
        "textDocument": { "uri": "file:///test.ax" },
        "position": { "line": 0, "character": 2 }  // space between 'fn' and 'main'
    }));
    let resp = h.handle(&hover).unwrap();
    assert!(resp.result.unwrap().is_null());
}

// ── T13: diagnostic pull on clean source returns empty ───────────────────────
#[test]
fn t13_diagnostic_clean() {
    let mut h = LspHandler::new();
    let open = make_req("textDocument/didOpen", None, json!({
        "textDocument": {
            "uri": "file:///clean.ax",
            "languageId": "axonyx",
            "version": 1,
            "text": "fn answer() -> i64 { 42 }"
        }
    }));
    h.handle(&open);
    let diag = make_req("textDocument/diagnostic", Some(6), json!({
        "textDocument": { "uri": "file:///clean.ax" }
    }));
    let resp = h.handle(&diag).unwrap();
    let items = &resp.result.unwrap()["items"];
    assert!(items.as_array().unwrap().is_empty(),
        "clean source must have 0 diagnostics, got: {:?}", items);
}

// ── T14: diagnostic pull on unknown uri returns empty ────────────────────────
#[test]
fn t14_diagnostic_unknown_uri() {
    let mut h = LspHandler::new();
    let diag = make_req("textDocument/diagnostic", Some(7), json!({
        "textDocument": { "uri": "file:///nonexistent.ax" }
    }));
    let resp = h.handle(&diag).unwrap();
    let items = &resp.result.unwrap()["items"];
    assert!(items.as_array().unwrap().is_empty());
}

// ── T15: word_at extracts identifier ─────────────────────────────────────────
#[test]
fn t15_word_at_ident() {
    let src = "fn sovereign_main() -> i64 { 0 }";
    assert_eq!(word_at(src, 0, 3), "sovereign_main");
    assert_eq!(word_at(src, 0, 10), "sovereign_main");
}

// ── T16: word_at returns empty for punctuation ────────────────────────────────
#[test]
fn t16_word_at_punct() {
    let src = "fn main() -> i64 { 0 }";
    assert_eq!(word_at(src, 0, 8), ""); // '('
}

// ── T17: word_at handles multi-line source ────────────────────────────────────
#[test]
fn t17_word_at_multiline() {
    let src = "fn main() -> i64 {\n    let x = 42\n    x\n}";
    assert_eq!(word_at(src, 1, 8), "x");
}

// ── T18: DocumentStore open/get roundtrip ─────────────────────────────────────
#[test]
fn t18_doc_store_roundtrip() {
    let mut store = DocumentStore::default();
    store.open("file:///a.ax".into(), "fn f() -> i64 { 1 }".into(), 1);
    let doc = store.get("file:///a.ax").unwrap();
    assert_eq!(doc.version, 1);
    assert!(doc.text.contains("fn f"));
}

// ── T19: DocumentStore update changes text ────────────────────────────────────
#[test]
fn t19_doc_store_update() {
    let mut store = DocumentStore::default();
    store.open("file:///b.ax".into(), "fn old() -> i64 { 0 }".into(), 1);
    store.update("file:///b.ax", "fn new() -> i64 { 1 }".into(), 2);
    let doc = store.get("file:///b.ax").unwrap();
    assert_eq!(doc.version, 2);
    assert!(doc.text.contains("new"));
}

// ── T20: parse_diagnostics clean source returns no errors ────────────────────
#[test]
fn t20_parse_diagnostics_clean() {
    let diags = parse_diagnostics("fn answer() -> i64 { 42 }");
    assert!(diags.is_empty(),
        "clean source must have 0 diagnostics, got: {:?}", diags);
}
