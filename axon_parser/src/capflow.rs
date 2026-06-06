// axon_parser/src/capflow.rs
// AXON Phase 9 — Transitive Capability Flow Analysis
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// Builds call graph and transitive capability sets from the PARSER AST,
// where function names are still strings (before HIR lowers idents to PlaceIds).
//
// Rule: required_caps(f) = declared_caps(f) ∪ ⋃ required_caps(callees(f))

use std::collections::{HashMap, HashSet};
use crate::parser::{Item, Expr, Stmt, ImplItem, TraitItem, FnSig};

// ============================================================
// CALL GRAPH
// ============================================================

#[derive(Debug, Default)]
pub struct CallGraph {
    pub edges: HashMap<String, Vec<String>>,
    pub declared_caps: HashMap<String, Vec<String>>,
}

impl CallGraph {
    pub fn build_from_items(items: &[Item]) -> CallGraph {
        let mut cg = CallGraph::default();
        for item in items {
            match item {
                Item::Fn(sig, body) => {
                    cg.add_from_sig_body(sig, body);
                }
                Item::Impl(i) => {
                    for ii in &i.items {
                        if let ImplItem::Fn(sig, body) = ii {
                            cg.add_from_impl_fn(sig, body);
                        }
                    }
                }
                Item::Trait(t) => {
                    for ti in &t.items {
                        if let TraitItem::Fn(sig, body) = ti {
                            cg.add_from_trait_fn(sig, body);
                        }
                    }
                }
                Item::Extern(_, fns, _) => {
                // P21-M1: register extern fn names with inferred caps
                for sig in fns {
                    let caps = infer_ffi_caps(&sig.name);
                    cg.declared_caps.entry(sig.name.clone()).or_default().extend(caps);
                }
            }
                _ => {}
            }
        }
        cg
    }

    fn add_from_impl_fn(&mut self, sig: &FnSig, body: &Expr) {
        self.add_from_sig_body(sig, body);
    }

    fn add_from_trait_fn(&mut self, sig: &FnSig, body: &Option<Expr>) {
        if let Some(b) = body {
            self.add_from_sig_body(sig, b);
        }
    }

    fn add_from_sig_body(&mut self, sig: &FnSig, body: &Expr) {
        let name = sig.name.name.clone();
        let caps: Vec<String> = sig.attrs.iter()
            .filter(|a| a.name == "cap" || a.name == "requires_cap" || a.name == "capability")
            .flat_map(|a| a.args.iter().cloned())
            .collect();
        self.declared_caps.entry(name.clone()).or_default().extend(caps);
        let mut callees: Vec<String> = Vec::new();
        collect_callees_from_expr(body, &mut callees);
        let mut seen: HashSet<String> = HashSet::new();
        let callees: Vec<String> = callees.into_iter().filter(|c| seen.insert(c.clone())).collect();
        self.edges.entry(name).or_default().extend(callees);
    }
}

fn collect_callees_from_expr(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::Call(func, args, _) => {
            if let Some(name) = callee_name_from_expr(func) {
                out.push(name);
            }
            collect_callees_from_expr(func, out);
            for arg in args { collect_callees_from_expr(arg, out); }
        }
        Expr::MethodCall(recv, method, args, _) => {
            out.push(method.name.clone());
            collect_callees_from_expr(recv, out);
            for arg in args { collect_callees_from_expr(arg, out); }
        }
        Expr::Block(stmts, tail, _) => {
            for stmt in stmts { collect_callees_from_stmt(stmt, out); }
            if let Some(t) = tail { collect_callees_from_expr(t, out); }
        }
        Expr::If(cond, then, else_, _) => {
            collect_callees_from_expr(cond, out);
            collect_callees_from_expr(then, out);
            if let Some(e) = else_ { collect_callees_from_expr(e, out); }
        }
        Expr::While(cond, body, _) => {
            collect_callees_from_expr(cond, out);
            collect_callees_from_expr(body, out);
        }
        Expr::Loop(body, _) => collect_callees_from_expr(body, out),
        Expr::For(_, iter, body, _) => {
            collect_callees_from_expr(iter, out);
            collect_callees_from_expr(body, out);
        }
        Expr::Match(scrutinee, arms, _) => {
            collect_callees_from_expr(scrutinee, out);
            for arm in arms {
                if let Some(g) = &arm.guard { collect_callees_from_expr(g, out); }
                collect_callees_from_expr(&arm.body, out);
            }
        }
        Expr::Return(Some(v), _) | Expr::Break(Some(v), _) => {
            collect_callees_from_expr(v, out);
        }
        Expr::Binary(_, l, r, _) => {
            collect_callees_from_expr(l, out);
            collect_callees_from_expr(r, out);
        }
        Expr::Unary(_, e, _) | Expr::Cast(e, _, _) | Expr::Deref(e, _) => {
            collect_callees_from_expr(e, out);
        }
        Expr::Field(e, _, _) => collect_callees_from_expr(e, out),
        Expr::Index(obj, idx, _) => {
            collect_callees_from_expr(obj, out);
            collect_callees_from_expr(idx, out);
        }
        Expr::Assign(lhs, rhs, _) | Expr::AssignOp(_, lhs, rhs, _) => {
            collect_callees_from_expr(lhs, out);
            collect_callees_from_expr(rhs, out);
        }
        Expr::Tuple(exprs, _) | Expr::Array(exprs, _) => {
            for e in exprs { collect_callees_from_expr(e, out); }
        }
        Expr::Struct(_, fields, _) => {
            for (_, e) in fields { collect_callees_from_expr(e, out); }
        }
        Expr::Ref(_, e, _) => collect_callees_from_expr(e, out),
        Expr::Range(lo, hi, _, _) => {
            if let Some(e) = lo { collect_callees_from_expr(e, out); }
            if let Some(e) = hi { collect_callees_from_expr(e, out); }
        }
        Expr::Lit(_, _) | Expr::Ident(_) | Expr::Path(_, _)
        | Expr::Continue(_) | Expr::Return(None, _) | Expr::Break(None, _) => {}
        Expr::Closure(_, body, _) => collect_callees_from_expr(body, out),
        Expr::Try(e, _) => collect_callees_from_expr(e, out),
    }
}

fn collect_callees_from_stmt(stmt: &Stmt, out: &mut Vec<String>) {
    match stmt {
        Stmt::Expr(e, _) => collect_callees_from_expr(e, out),
        Stmt::Let(_, _, Some(e), _) => collect_callees_from_expr(e, out),
        Stmt::Let(_, _, None, _) | Stmt::Item(_) => {}
    }
}

fn callee_name_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(i) => Some(i.name.clone()),
        Expr::Path(segs, _) if !segs.is_empty() => Some(segs.last().unwrap().name.clone()),
        _ => None,
    }
}

// ============================================================
// TRANSITIVE CAPABILITY PROPAGATION
// ============================================================

#[derive(Debug, Clone)]
pub struct FnCapInfo {
    pub transitive_caps: HashSet<String>,
    pub cap_chains: HashMap<String, Vec<String>>,
}

pub type TransitiveCaps = HashMap<String, FnCapInfo>;

pub fn propagate(cg: &CallGraph) -> TransitiveCaps {
    let mut caps: HashMap<String, HashSet<String>> = HashMap::new();
    let mut chains: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

    for (fname, dcaps) in &cg.declared_caps {
        let entry = caps.entry(fname.clone()).or_default();
        entry.extend(dcaps.iter().cloned());
        let chain_entry = chains.entry(fname.clone()).or_default();
        for cap in dcaps {
            chain_entry.entry(cap.clone()).or_default();
        }
    }
    for fname in cg.edges.keys() {
        caps.entry(fname.clone()).or_default();
        chains.entry(fname.clone()).or_default();
    }

    let mut changed = true;
    while changed {
        changed = false;
        let callers: Vec<String> = cg.edges.keys().cloned().collect();
        for caller in &callers {
            let callees = cg.edges[caller].clone();
            for callee in &callees {
                let callee_caps: HashSet<String> = caps.get(callee).cloned().unwrap_or_default();
                let callee_chains: HashMap<String, Vec<String>> = chains.get(callee).cloned().unwrap_or_default();
                for cap in &callee_caps {
                    if caps.entry(caller.clone()).or_default().insert(cap.clone()) {
                        changed = true;
                        let mut new_chain = vec![caller.clone()];
                        match callee_chains.get(cap) {
                            Some(sub) if sub.is_empty() => { new_chain.push(callee.clone()); }
                            Some(sub) => { new_chain.extend(sub.iter().cloned()); }
                            None => { new_chain.push(callee.clone()); }
                        }
                        chains.entry(caller.clone()).or_default().insert(cap.clone(), new_chain);
                    }
                }
            }
        }
    }

    caps.keys().map(|fname| {
        let transitive_caps = caps[fname].clone();
        let cap_chains = chains.get(fname).cloned().unwrap_or_default();
        (fname.clone(), FnCapInfo { transitive_caps, cap_chains })
    }).collect()
}

// ============================================================
// ERROR FORMATTING
// ============================================================

pub fn format_chain(chain: &[String]) -> String {
    chain.join(" \u{2192} ")
}

// ============================================================
// FFI CAP INFERENCE
// ============================================================

/// Infer capability requirements from a C foreign function name.
/// Called during Phase 21 extern block parsing; available as a
/// standalone lookup now so the table is built and tested in Phase 15.
///
/// Rule: unrecognised foreign symbols are tagged with the most
/// restrictive cap that matches their name pattern.
/// Unknown names return an empty vec — caller decides policy.
pub fn infer_ffi_caps(fn_name: &str) -> Vec<String> {
    let name = fn_name.to_lowercase();
    let mut caps: Vec<String> = Vec::new();

    // Network: connect/socket/send/recv family
    let network_patterns = [
        "connect", "socket", "send", "recv",
        "bind", "listen", "sendto", "recvfrom",
        "gethostbyname", "getaddrinfo", "inet_",
    ];
    if network_patterns.iter().any(|p| name == *p || name.starts_with(p)) {
        caps.push("network_connect".to_string());
    }

    // Filesystem write: open/write/unlink/rename family
    let file_write_patterns = [
        "write", "fwrite", "pwrite", "writev",
        "unlink", "rename", "mkdir", "rmdir",
        "chmod", "chown", "truncate", "ftruncate",
        "creat", "mknod",
    ];
    if file_write_patterns.iter().any(|p| name == *p || name.starts_with(p)) {
        caps.push("file_write".to_string());
    }

    // Filesystem read: open/read/stat family
    // Note: "open" can read or write — conservatively tag file_read;
    // writers already caught above by creat/write patterns.
    let file_read_patterns = [
        "open", "read", "fread", "pread", "readv",
        "stat", "fstat", "lstat", "readdir", "opendir",
        "access", "realpath", "getcwd",
    ];
    if file_read_patterns.iter().any(|p| name == *p || name.starts_with(p))
        && !caps.contains(&"file_read".to_string())
    {
        caps.push("file_read".to_string());
    }

    // seL4 IPC intrinsics — axon_ipc_* require ipc_send or ipc_receive
    // Prefix match: catches axon_ipc_call, axon_ipc_send, axon_ipc_call_async etc.
    if name.starts_with("axon_ipc_call") || name.starts_with("axon_ipc_send") {
        caps.push("ipc_send".to_string());
    }
    if name.starts_with("axon_ipc_recv") {
        caps.push("ipc_receive".to_string());
    }

    // Process spawn: fork/exec family
    let spawn_patterns = [
        "fork", "exec", "execve", "execvp", "execle",
        "posix_spawn", "clone", "vfork",
    ];
    if spawn_patterns.iter().any(|p| name == *p || name.starts_with(p)) {
        caps.push("spawn".to_string());
    }

    caps
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn build_cg(src: &str) -> CallGraph {
        let items = parse(src).expect("parse failed");
        CallGraph::build_from_items(&items)
    }

    fn build_trans(src: &str) -> TransitiveCaps {
        let items = parse(src).expect("parse failed");
        let cg = CallGraph::build_from_items(&items);
        propagate(&cg)
    }

    #[test]
    fn tcf1_no_calls_empty_edges() {
        let cg = build_cg("fn add(x: i32, y: i32) -> i32 { return x; }");
        let callees = cg.edges.get("add").map(|v| v.len()).unwrap_or(0);
        assert_eq!(callees, 0);
    }

    #[test]
    fn tcf2_direct_call_captured() {
        let src = "fn inner(x: i32) -> i32 { return x; } fn outer(x: i32) -> i32 { return inner(x); }";
        let cg = build_cg(src);
        let callees = cg.edges.get("outer").expect("outer must exist");
        assert!(callees.contains(&"inner".to_string()));
    }

    #[test]
    fn tcf3_multiple_callees() {
        let src = "fn a(x: i32) -> i32 { return x; } fn b(x: i32) -> i32 { return x; } fn c(x: i32) -> i32 { return a(b(x)); }";
        let cg = build_cg(src);
        let callees = cg.edges.get("c").expect("c must exist");
        assert!(callees.contains(&"a".to_string()));
        assert!(callees.contains(&"b".to_string()));
    }

    #[test]
    fn tcf4_declared_caps_stored() {
        let src = "#[cap(network_connect)] fn send(x: i32) -> i32 { return x; }";
        let cg = build_cg(src);
        let caps = cg.declared_caps.get("send").expect("send must exist");
        assert!(caps.contains(&"network_connect".to_string()));
    }

    #[test]
    fn tcf5_no_duplicate_callees() {
        let src = "fn g(x: i32) -> i32 { return x; } fn f(x: i32) -> i32 { return g(g(x)); }";
        let cg = build_cg(src);
        let callees = cg.edges.get("f").expect("f must exist");
        let count = callees.iter().filter(|c| c.as_str() == "g").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn tcf6_direct_cap_present_in_transitive() {
        let src = "#[cap(network_connect)] fn send(x: i32) -> i32 { return x; }";
        let trans = build_trans(src);
        let info = trans.get("send").expect("send must be in result");
        assert!(info.transitive_caps.contains("network_connect"));
    }

    #[test]
    fn tcf7_transitive_cap_propagates_up() {
        let src = "#[cap(network_connect)] fn send(x: i32) -> i32 { return x; } fn wrapper(x: i32) -> i32 { return send(x); }";
        let trans = build_trans(src);
        let info = trans.get("wrapper").expect("wrapper must exist");
        assert!(info.transitive_caps.contains("network_connect"), "wrapper must inherit network_connect from send");
    }

    #[test]
    fn tcf8_chain_two_hops() {
        let src = "#[cap(file_write)] fn inner(x: i32) -> i32 { return x; } fn mid(x: i32) -> i32 { return inner(x); } fn outer(x: i32) -> i32 { return mid(x); }";
        let trans = build_trans(src);
        let info = trans.get("outer").expect("outer must exist");
        assert!(info.transitive_caps.contains("file_write"));
    }

    #[test]
    fn tcf9_no_spurious_caps() {
        let src = "fn clean(x: i32) -> i32 { return x; }";
        let trans = build_trans(src);
        let info = trans.get("clean").expect("clean must exist");
        assert!(info.transitive_caps.is_empty());
    }

    #[test]
    fn tcf10_cycle_stabilises() {
        let src = "#[cap(ipc_send)] fn ping(x: i32) -> i32 { return pong(x); } fn pong(x: i32) -> i32 { return ping(x); }";
        let trans = build_trans(src);
        assert!(trans.get("ping").unwrap().transitive_caps.contains("ipc_send"));
        assert!(trans.get("pong").unwrap().transitive_caps.contains("ipc_send"));
    }

    #[test]
    fn tcf11_chain_recorded() {
        let src = "#[cap(network_connect)] fn send(x: i32) -> i32 { return x; } fn wrapper(x: i32) -> i32 { return send(x); }";
        let trans = build_trans(src);
        let info = trans.get("wrapper").expect("wrapper must exist");
        let chain = info.cap_chains.get("network_connect").expect("chain must be recorded");
        assert!(chain.contains(&"wrapper".to_string()));
        assert!(chain.contains(&"send".to_string()));
    }

    // ── Phase 20 M3 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_ipc_ffi_infers_ipc_send() {
        // axon_ipc_call must infer ipc_send cap
        let caps = super::infer_ffi_caps("axon_ipc_call");
        assert!(caps.contains(&"ipc_send".to_string()),
            "axon_ipc_call must infer ipc_send, got: {:?}", caps);
    }

    #[test]
    fn tc_ipc_ffi_send_infers_ipc_send() {
        let caps = super::infer_ffi_caps("axon_ipc_send");
        assert!(caps.contains(&"ipc_send".to_string()),
            "axon_ipc_send must infer ipc_send, got: {:?}", caps);
    }

    #[test]
    fn tc_ipc_ffi_recv_infers_ipc_receive() {
        let caps = super::infer_ffi_caps("axon_ipc_recv");
        assert!(caps.contains(&"ipc_receive".to_string()),
            "axon_ipc_recv must infer ipc_receive, got: {:?}", caps);
    }

    #[test]
    fn tc_ipc_cap_transitive_propagates() {
        // fn caller calls fn with #[cap(ipc_send)] — ipc_send propagates transitively.
        // All profiles allow ipc_send so no violation; but the cap must propagate
        // in the call graph (transitive_caps of caller must include ipc_send).
        let src = r#"
            #[cap(ipc_send)]
            fn ipc_fn(x: i32) -> i32 { return x; }
            fn caller(x: i32) -> i32 { return ipc_fn(x); }
        "#;
        let items = crate::parser::parse(src).expect("parse failed");
        let cg = super::CallGraph::build_from_items(&items);
        let trans = super::propagate(&cg);
        let caller_caps = &trans.get("caller").expect("caller must exist").transitive_caps;
        assert!(
            caller_caps.contains("ipc_send"),
            "ipc_send must propagate to caller, got: {:?}", caller_caps
        );
    }

    // ── Phase 15 M3 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_cap_ffi_connect_infers_network() {
        let caps = super::infer_ffi_caps("connect");
        assert!(caps.contains(&"network_connect".to_string()),
            "connect must infer network_connect, got: {:?}", caps);
    }

    #[test]
    fn tc_cap_ffi_socket_infers_network() {
        let caps = super::infer_ffi_caps("socket");
        assert!(caps.contains(&"network_connect".to_string()),
            "socket must infer network_connect, got: {:?}", caps);
    }

    #[test]
    fn tc_cap_ffi_open_infers_file_read() {
        let caps = super::infer_ffi_caps("open");
        assert!(caps.contains(&"file_read".to_string()),
            "open must infer file_read, got: {:?}", caps);
    }

    #[test]
    fn tc_cap_ffi_write_infers_file_write() {
        let caps = super::infer_ffi_caps("write");
        assert!(caps.contains(&"file_write".to_string()),
            "write must infer file_write, got: {:?}", caps);
    }

    #[test]
    fn tc_cap_ffi_fork_infers_spawn() {
        let caps = super::infer_ffi_caps("fork");
        assert!(caps.contains(&"spawn".to_string()),
            "fork must infer spawn, got: {:?}", caps);
    }

    #[test]
    fn tc_cap_ffi_execve_infers_spawn() {
        let caps = super::infer_ffi_caps("execve");
        assert!(caps.contains(&"spawn".to_string()),
            "execve must infer spawn, got: {:?}", caps);
    }

    #[test]
    fn tc_cap_ffi_unknown_fn_no_caps() {
        let caps = super::infer_ffi_caps("memcpy");
        assert!(caps.is_empty(),
            "memcpy must infer no caps, got: {:?}", caps);
    }

    #[test]
    fn tc_cap_ffi_case_insensitive() {
        let caps = super::infer_ffi_caps("Connect");
        assert!(caps.contains(&"network_connect".to_string()),
            "Connect (uppercase) must infer network_connect, got: {:?}", caps);
    }

    #[test]
    fn tcf12_format_chain_output() {
        let chain = vec!["f".to_string(), "g".to_string(), "h".to_string()];
        assert_eq!(format_chain(&chain), "f \u{2192} g \u{2192} h");
    }
}
