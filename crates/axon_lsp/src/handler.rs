// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P64 — LSP method handler: dispatches JSON-RPC methods

use serde_json::{json, Value};
use crate::types::*;
use crate::document::DocumentStore;

pub struct LspHandler {
    pub store: DocumentStore,
    pub initialized: bool,
    pub shutdown: bool,
}

impl LspHandler {
    pub fn new() -> Self {
        Self {
            store: DocumentStore::default(),
            initialized: false,
            shutdown: false,
        }
    }

    /// Dispatch a JSON-RPC request/notification. Returns Some(response) for
    /// requests, None for notifications.
    pub fn handle(&mut self, req: &RpcRequest) -> Option<RpcResponse> {
        match req.method.as_str() {
            // ── Lifecycle ─────────────────────────────────────────────────────
            "initialize" => Some(self.handle_initialize(req)),
            "initialized" => { self.initialized = true; None }
            "shutdown" => {
                self.shutdown = true;
                Some(RpcResponse::ok(req.id.clone(), Value::Null))
            }
            "exit" => None, // handled at transport level

            // ── Text document sync ────────────────────────────────────────────
            "textDocument/didOpen" => {
                self.handle_did_open(req);
                None
            }
            "textDocument/didChange" => {
                self.handle_did_change(req);
                None
            }
            "textDocument/didClose" => {
                self.handle_did_close(req);
                None
            }

            // ── Hover ─────────────────────────────────────────────────────────
            "textDocument/hover" => Some(self.handle_hover(req)),

            // ── Diagnostics (pull model) ──────────────────────────────────────
            "textDocument/diagnostic" => Some(self.handle_diagnostic(req)),

            // ── Unknown ───────────────────────────────────────────────────────
            _ => {
                if req.id.is_some() {
                    // Request (has id) — must respond with MethodNotFound
                    Some(RpcResponse::err(req.id.clone(), -32601, "method not found"))
                } else {
                    // Notification (no id) — silently ignore
                    None
                }
            }
        }
    }

    fn handle_initialize(&mut self, req: &RpcRequest) -> RpcResponse {
        self.initialized = true;
        let caps = ServerCapabilities {
            text_document_sync: 1,  // Full sync
            hover_provider: true,
            diagnostic_provider: DiagnosticOptions {
                identifier: "axonyx".into(),
                inter_file_dependencies: false,
                workspace_diagnostics: false,
            },
        };
        RpcResponse::ok(req.id.clone(), json!({
            "capabilities": caps,
            "serverInfo": {
                "name": "axon_lsp",
                "version": "0.64.0"
            }
        }))
    }

    fn handle_did_open(&mut self, req: &RpcRequest) {
        if let Ok(item) = serde_json::from_value::<TextDocumentItem>(
            req.params["textDocument"].clone()
        ) {
            self.store.open(item.uri, item.text, item.version);
        }
    }

    fn handle_did_change(&mut self, req: &RpcRequest) {
        let uri = req.params["textDocument"]["uri"]
            .as_str().unwrap_or("").to_string();
        let version = req.params["textDocument"]["version"]
            .as_i64().unwrap_or(0) as i32;
        // Full sync: take first contentChange
        if let Some(text) = req.params["contentChanges"][0]["text"].as_str() {
            self.store.update(&uri, text.to_string(), version);
        }
    }

    fn handle_did_close(&mut self, req: &RpcRequest) {
        let uri = req.params["textDocument"]["uri"]
            .as_str().unwrap_or("").to_string();
        self.store.close(&uri);
    }

    fn handle_hover(&mut self, req: &RpcRequest) -> RpcResponse {
        let uri = req.params["textDocument"]["uri"]
            .as_str().unwrap_or("").to_string();
        let line = req.params["position"]["line"].as_u64().unwrap_or(0) as u32;
        let col  = req.params["position"]["character"].as_u64().unwrap_or(0) as u32;

        if let Some(doc) = self.store.get(&uri) {
            let word = word_at(&doc.text, line, col);
            if !word.is_empty() {
                let hover = Hover {
                    contents: MarkupContent {
                        kind: "markdown".into(),
                        value: format!("**AXONYX** — `{}`\n\nSovereign identifier.", word),
                    },
                    range: None,
                };
                return RpcResponse::ok(req.id.clone(), json!(hover));
            }
        }
        RpcResponse::ok(req.id.clone(), Value::Null)
    }

    fn handle_diagnostic(&mut self, req: &RpcRequest) -> RpcResponse {
        let uri = req.params["textDocument"]["uri"]
            .as_str().unwrap_or("").to_string();

        let diags = if let Some(doc) = self.store.get(&uri) {
            doc.diagnostics.clone()
        } else {
            vec![]
        };

        RpcResponse::ok(req.id.clone(), json!({
            "kind": "full",
            "items": diags,
        }))
    }
}

/// Extract the word at (line, col) from source text.
pub fn word_at(source: &str, line: u32, col: u32) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let l = line as usize;
    if l >= lines.len() { return String::new(); }
    let chars: Vec<char> = lines[l].chars().collect();
    let c = col as usize;
    if c >= chars.len() { return String::new(); }

    // Find word boundaries
    let is_ident = |ch: char| ch.is_alphanumeric() || ch == '_';
    if !is_ident(chars[c]) { return String::new(); }

    let start = (0..=c).rev().find(|&i| !is_ident(chars[i])).map(|i| i + 1).unwrap_or(0);
    let end = (c..chars.len()).find(|&i| !is_ident(chars[i])).unwrap_or(chars.len());
    chars[start..end].iter().collect()
}
