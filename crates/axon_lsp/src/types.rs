// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P64 — LSP 3.17 protocol types (minimal sovereign subset)

use serde::{Deserialize, Serialize};

// ── JSON-RPC ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

impl RpcResponse {
    pub fn ok(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }
    pub fn err(id: Option<serde_json::Value>, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(RpcError { code, message: message.into() }),
        }
    }
    pub fn notification(method: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        })
    }
}

// ── LSP position / range ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

// ── Diagnostic ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: u32,  // 1=Error 2=Warning 3=Info 4=Hint
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl Diagnostic {
    pub fn error(line: u32, character: u32, message: String) -> Self {
        Self {
            range: Range {
                start: Position { line, character },
                end:   Position { line, character: character + 1 },
            },
            severity: 1,
            message,
            source: Some("axonyx".into()),
        }
    }
}

// ── TextDocumentItem ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentItem {
    pub uri: String,
    #[serde(rename = "languageId")]
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

// ── Hover ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hover {
    pub contents: MarkupContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkupContent {
    pub kind: String,   // "plaintext" | "markdown"
    pub value: String,
}

// ── ServerCapabilities ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(rename = "textDocumentSync")]
    pub text_document_sync: u32,  // 1 = Full
    #[serde(rename = "hoverProvider")]
    pub hover_provider: bool,
    #[serde(rename = "diagnosticProvider")]
    pub diagnostic_provider: DiagnosticOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticOptions {
    pub identifier: String,
    #[serde(rename = "interFileDependencies")]
    pub inter_file_dependencies: bool,
    #[serde(rename = "workspaceDiagnostics")]
    pub workspace_diagnostics: bool,
}
