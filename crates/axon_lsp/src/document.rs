// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P64 — Document store: tracks open .ax files and their parsed state

use std::collections::HashMap;
use crate::types::Diagnostic;

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: String,
    pub text: String,
    pub version: i32,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default)]
pub struct DocumentStore {
    pub docs: HashMap<String, Document>,
}

impl DocumentStore {
    pub fn open(&mut self, uri: String, text: String, version: i32) -> &Document {
        let diags = parse_diagnostics(&text);
        self.docs.insert(uri.clone(), Document {
            uri: uri.clone(), text, version, diagnostics: diags,
        });
        self.docs.get(&uri).unwrap()
    }

    pub fn update(&mut self, uri: &str, text: String, version: i32) -> Option<&Document> {
        if let Some(doc) = self.docs.get_mut(uri) {
            doc.text = text.clone();
            doc.version = version;
            doc.diagnostics = parse_diagnostics(&text);
            Some(self.docs.get(uri).unwrap())
        } else {
            None
        }
    }

    pub fn close(&mut self, uri: &str) {
        self.docs.remove(uri);
    }

    pub fn get(&self, uri: &str) -> Option<&Document> {
        self.docs.get(uri)
    }
}

/// Parse an .ax source string and return LSP diagnostics for any errors.
/// Delegates to axon_parser::parse() — the top-level sovereign entry point.
pub fn parse_diagnostics(source: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    match axon_parser::parser::parse(source) {
        Ok(_) => {}
        Err(e) => {
            let msg = format!("{}", e);
            let line = extract_line_from_error(&msg);
            diags.push(Diagnostic::error(line, 0, msg));
        }
    }
    diags
}

/// Try to extract a 0-based line number from a parser error message.
/// Errors often contain "line N" or "at line N".
fn extract_line_from_error(msg: &str) -> u32 {
    // Look for "line N" pattern
    let lower = msg.to_lowercase();
    if let Some(pos) = lower.find("line ") {
        let rest = &msg[pos + 5..];
        let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(n) = num.parse::<u32>() {
            return n.saturating_sub(1); // convert to 0-based
        }
    }
    0
}
