// ============================================================
// axon_ai — translator.rs
// NL → FormalSpec via local Ollama
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// ARCHITECTURE NOTE (post-peer-review):
// The translator is ADVISORY ONLY. It is not in the TCB.
// It proposes formal specs. The developer reviews them.
// The deterministic verifier (verifier.rs) enforces them.
//
// Non-determinism is controlled via:
//   - temperature = 0
//   - grammar-constrained JSON output
//   - output is cached by intent string hash
//   - compiler never rejects based on AI output alone
// ============================================================

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use crate::spec::{FormalSpec, Constraint, Effect};
use crate::error::AiError;
use serde_json;

// ── Translator config ─────────────────────────────────────────

pub struct TranslatorConfig {
    /// Ollama API endpoint
    pub ollama_host  : String,
    pub ollama_port  : u16,
    /// Model to use for translation
    pub model        : String,
    /// Temperature — must be 0 for determinism
    pub temperature  : f64,
    /// Cache translated specs to avoid redundant LLM calls
    pub cache        : HashMap<String, FormalSpec>,
}

impl Default for TranslatorConfig {
    fn default() -> Self {
        TranslatorConfig {
            ollama_host : "127.0.0.1".to_string(),
            ollama_port : 11434,
            model       : "qwen2.5-coder:7b".to_string(),
            temperature : 0.0,
            cache       : HashMap::new(),
        }
    }
}

// ── IntentTranslator ──────────────────────────────────────────

/// Translates natural language @ai.intent strings into FormalSpec.
/// Output is advisory — developer must review before formal verifier runs.
pub struct IntentTranslator {
    pub config: TranslatorConfig,
}

impl IntentTranslator {
    pub fn new() -> Self {
        IntentTranslator { config: TranslatorConfig::default() }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.config.model = model.to_string(); self
    }

    /// Translate a natural language intent string to a FormalSpec proposal.
    ///
    /// This is ADVISORY. Never call this as a compilation gate.
    /// Always run the deterministic verifier (verifier.rs) on the result.
    pub fn translate(&mut self, intent_nl: &str) -> Result<FormalSpec, AiError> {
        // Check cache first (determinism + performance)
        let cache_key = intent_nl.trim().to_lowercase();
        if let Some(cached) = self.config.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Try to call Ollama
        match self.call_ollama(intent_nl) {
            Ok(spec) => {
                self.config.cache.insert(cache_key, spec.clone());
                Ok(spec)
            }
            Err(AiError::OllamaUnavailable(_)) => {
                // Ollama not running — use rule-based fallback
                // This is NOT a compile error
                let spec = self.rule_based_fallback(intent_nl);
                self.config.cache.insert(cache_key, spec.clone());
                Ok(spec)
            }
            Err(e) => Err(e),
        }
    }

    /// Attempt to call local Ollama instance.
    fn call_ollama(&self, intent_nl: &str) -> Result<FormalSpec, AiError> {
        let prompt = self.build_prompt(intent_nl);
        let body   = serde_json::json!({
            "model"       : self.config.model,
            "prompt"      : prompt,
            "temperature" : self.config.temperature,
            "stream"      : false,
            "format"      : "json"
        });
        let body_str = body.to_string();

        // Simple HTTP POST to Ollama (no async needed)
        // Short timeout so fallback triggers quickly if Ollama is slow
        use std::time::Duration;
        let addr = format!("{}:{}", self.config.ollama_host, self.config.ollama_port);
        let socket_addr: std::net::SocketAddr = addr.parse()
            .map_err(|e: std::net::AddrParseError| AiError::OllamaUnavailable(e.to_string()))?;
        let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_millis(500))
            .map_err(|e| AiError::OllamaUnavailable(e.to_string()))?;
        // 8 second read timeout — model inference limit
        stream.set_read_timeout(Some(Duration::from_secs(8)))
            .map_err(|e| AiError::OllamaUnavailable(e.to_string()))?;

        let request = format!(
            "POST /api/generate HTTP/1.0\r\n\
             Host: {addr}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {len}\r\n\
             \r\n\
             {body}",
            addr = addr,
            len  = body_str.len(),
            body = body_str
        );

        stream.write_all(request.as_bytes())
            .map_err(|e| AiError::OllamaUnavailable(e.to_string()))?;

        let mut response = String::new();
        stream.read_to_string(&mut response)
            .map_err(|e| AiError::OllamaUnavailable(e.to_string()))?;

        // Extract JSON body from HTTP response
        let json_start = response.find('{')
            .ok_or_else(|| AiError::MalformedResponse("no JSON in response".into()))?;
        let json_str = &response[json_start..];

        // Parse Ollama response
        let ollama_resp: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| AiError::MalformedResponse(e.to_string()))?;

        let response_text = ollama_resp["response"].as_str()
            .unwrap_or("")
            .trim();

        // Parse the structured spec from LLM output
        self.parse_llm_output(intent_nl, response_text)
    }

    /// Build the structured prompt for the LLM.
    /// Uses grammar-constrained JSON output to ensure parseability.
    fn build_prompt(&self, intent_nl: &str) -> String {
        format!(r#"You are a formal specification assistant for a systems programming language.

Your task: translate a natural language function intent into a formal specification.

Natural language intent: "{intent}"

Output ONLY valid JSON matching this exact schema. No explanation, no markdown.

{{
  "ensures": [list of constraint strings from: "result >= 0", "result > 0", "result != null", "no_allocation", "no_io", "pure_inputs", or "custom: <expression>"],
  "requires": [list of precondition strings],
  "effects": [list from: "pure", "readonly", "writes_audit_log", "may_allocate", "no_allocate"],
  "confidence": <float 0.0-1.0>,
  "explanation": "<one sentence explaining the translation>"
}}

Examples:
- "always returns non-negative" → {{"ensures": ["result >= 0"], "requires": [], "effects": [], "confidence": 0.95, "explanation": "Non-negative means result >= 0"}}
- "never returns null" → {{"ensures": ["result != null"], "requires": [], "effects": [], "confidence": 0.98, "explanation": "Never null expressed as result != null"}}
- "pure function, no side effects" → {{"ensures": [], "requires": [], "effects": ["pure"], "confidence": 0.99, "explanation": "Pure function declaration"}}
- "does not allocate memory" → {{"ensures": ["no_allocation"], "requires": [], "effects": ["no_allocate"], "confidence": 0.97, "explanation": "No heap allocation permitted"}}

JSON output:"#,
            intent = intent_nl
        )
    }

    /// Parse LLM JSON output into a FormalSpec.
    fn parse_llm_output(
        &self,
        intent_nl   : &str,
        llm_output  : &str,
    ) -> Result<FormalSpec, AiError> {
        // Find JSON in output
        let json_start = llm_output.find('{')
            .ok_or_else(|| AiError::MalformedResponse(
                format!("no JSON found in: {}", &llm_output[..llm_output.len().min(100)])
            ))?;
        let json_end = llm_output.rfind('}')
            .ok_or_else(|| AiError::MalformedResponse("unclosed JSON".into()))?;
        let json_str = &llm_output[json_start..=json_end];

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| AiError::MalformedResponse(e.to_string()))?;

        let mut spec = FormalSpec::new(intent_nl);
        spec.ai_confidence = parsed["confidence"].as_f64().unwrap_or(0.5);

        // Parse ensures
        if let Some(ensures) = parsed["ensures"].as_array() {
            for e in ensures {
                if let Some(s) = e.as_str() {
                    if let Some(c) = Self::parse_constraint_str(s) {
                        spec.ensures.push(c);
                    }
                }
            }
        }

        // Parse effects
        if let Some(effects) = parsed["effects"].as_array() {
            for ef in effects {
                if let Some(s) = ef.as_str() {
                    if let Some(e) = Self::parse_effect_str(s) {
                        spec.effects.push(e);
                    }
                }
            }
        }

        // Parse requires
        if let Some(requires) = parsed["requires"].as_array() {
            for r in requires {
                if let Some(s) = r.as_str() {
                    if let Some(c) = Self::parse_constraint_str(s) {
                        spec.requires.push(c);
                    }
                }
            }
        }

        Ok(spec)
    }

    fn parse_constraint_str(s: &str) -> Option<Constraint> {
        match s.trim() {
            "result >= 0" | "result_non_negative" => Some(Constraint::ResultNonNegative),
            "result > 0"  | "result_positive"     => Some(Constraint::ResultPositive),
            "result != null" | "result_non_null"  => Some(Constraint::ResultNonNull),
            "no_allocation"                        => Some(Constraint::NoHeapAllocation),
            "no_io"                                => Some(Constraint::NoIO),
            "pure_inputs"                          => Some(Constraint::PureInputs),
            s if s.starts_with("result >= ") => {
                s[10..].trim().parse().ok().map(Constraint::ResultAtLeast)
            }
            s if s.starts_with("result <= ") => {
                s[10..].trim().parse().ok().map(Constraint::ResultAtMost)
            }
            s if s.starts_with("result == ") => {
                s[10..].trim().parse().ok().map(Constraint::ResultEquals)
            }
            s if s.starts_with("custom: ") => {
                Some(Constraint::Custom(s[8..].to_string()))
            }
            s => Some(Constraint::Custom(s.to_string())),
        }
    }

    fn parse_effect_str(s: &str) -> Option<Effect> {
        match s.trim() {
            "pure"            => Some(Effect::Pure),
            "readonly"        => Some(Effect::ReadOnly),
            "writes_audit_log"=> Some(Effect::WritesAuditLog),
            "may_allocate"    => Some(Effect::MayAllocate),
            "no_allocate"     => Some(Effect::NoAllocate),
            s                 => Some(Effect::Custom(s.to_string())),
        }
    }

    /// Rule-based fallback when Ollama is unavailable.
    /// Uses pattern matching on common intent strings.
    /// This ensures compilation continues even without AI.
    pub fn rule_based_fallback(&self, intent_nl: &str) -> FormalSpec {
        let lower = intent_nl.to_lowercase();
        let mut spec = FormalSpec::new(intent_nl);
        spec.ai_confidence = 0.7; // lower confidence for rule-based

        // Pattern matching on common intent phrases
        if lower.contains("non-negative") || lower.contains("nonnegative")
            || lower.contains("not negative") {
            spec.ensures.push(Constraint::ResultNonNegative);
        }
        if lower.contains("positive") && !lower.contains("non-positive") {
            spec.ensures.push(Constraint::ResultPositive);
        }
        if lower.contains("not null") || lower.contains("non-null")
            || lower.contains("nonnull") || lower.contains("never null") {
            spec.ensures.push(Constraint::ResultNonNull);
        }
        if lower.contains("no allocation") || lower.contains("no heap")
            || lower.contains("does not allocate") {
            spec.ensures.push(Constraint::NoHeapAllocation);
            spec.effects.push(Effect::NoAllocate);
        }
        if lower.contains("pure") || lower.contains("no side effect")
            || lower.contains("deterministic") {
            spec.effects.push(Effect::Pure);
        }
        if lower.contains("read only") || lower.contains("readonly")
            || lower.contains("only reads") || lower.contains("does not write") {
            spec.effects.push(Effect::ReadOnly);
        }
        if lower.contains("no io") || lower.contains("no i/o")
            || lower.contains("no input") || lower.contains("no output") {
            spec.ensures.push(Constraint::NoIO);
        }
        if lower.contains("audit") || lower.contains("always logs")
            || lower.contains("log every") {
            spec.effects.push(Effect::WritesAuditLog);
            spec.ensures.push(Constraint::AlwaysReaches("audit_log".to_string()));
        }

        spec
    }
}
