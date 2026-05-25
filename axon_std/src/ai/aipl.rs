//! AIPL — AI Postcondition Lifter (P6+ feature integration).
//!
//! AIPL takes a function signature and asks Ollama to suggest
//! formal `@ensures` postconditions for it. The output is advisory —
//! the developer reviews and adds the annotations manually.
//! The deterministic verifier (axon::verify) then enforces them.
//!
//! Pipeline:
//!   fn signature (AXON source) → aipl_suggest() → AiplSuggestion
//!                                                      ↓
//!                                               Developer reviews
//!                                                      ↓
//!                                        @ensures annotation added
//!                                                      ↓
//!                                     axon::verify enforces at compile time

use super::{AiError, AiResult, InferenceWeight, inference};

/// A suggested set of formal postconditions from AIPL.
#[derive(Debug, Clone)]
pub struct AiplSuggestion {
    /// The function signature AIPL analysed.
    pub fn_signature: String,
    /// Suggested `@ensures` annotations in AXON syntax.
    pub suggestions: Vec<String>,
    /// Confidence score 0.0–1.0 (higher = more confident).
    pub confidence: f32,
    /// Raw explanation from the model.
    pub explanation: String,
}

const AIPL_SYSTEM_PROMPT: &str = r#"You are AIPL, the AXON AI Postcondition Lifter.
Given an AXON function signature, suggest formal @ensures postconditions.
Output ONLY a JSON object with this schema:
{"suggestions": ["@ensures ...", ...], "confidence": 0.0-1.0, "explanation": "..."}
Do not output anything else. No markdown. No prose."#;

/// Suggest formal `@ensures` postconditions for an AXON function signature.
///
/// Uses SecurityCritical weight — formal spec proposals require full
/// inference depth and determinism (low temperature).
///
/// # Examples
///
/// ```rust,ignore
/// use axon_std::ai::aipl_suggest;
/// let suggestion = aipl_suggest("fn add(a: Int, b: Int) -> Int", "llama3.2").unwrap();
/// for s in &suggestion.suggestions {
///     println!("{}", s);  // e.g. "@ensures result == a + b"
/// }
/// ```
pub fn aipl_suggest(fn_signature: &str, model: &str) -> AiResult<AiplSuggestion> {
    let prompt = format!(
        "{AIPL_SYSTEM_PROMPT}\n\nFunction signature:\n{fn_signature}"
    );

    // SecurityCritical: 4096 tokens, temperature 0.1 — formal spec needs determinism
    let raw = inference::infer_weighted(&prompt, model, InferenceWeight::SecurityCritical)?;

    // Parse the JSON response
    parse_aipl_response(fn_signature, &raw)
}

fn parse_aipl_response(fn_signature: &str, raw: &str) -> AiResult<AiplSuggestion> {
    // Find the JSON object in the response
    let json_start = raw.find('{').ok_or_else(||
        AiError::MalformedResponse("AIPL: no JSON object in response".into())
    )?;
    let json_end = raw.rfind('}').ok_or_else(||
        AiError::MalformedResponse("AIPL: unclosed JSON object".into())
    )?;
    let json_str = &raw[json_start..=json_end];

    let v: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| AiError::MalformedResponse(format!("AIPL JSON parse: {e}")))?;

    let suggestions = v["suggestions"]
        .as_array()
        .map(|arr| arr.iter()
            .filter_map(|s| s.as_str().map(String::from))
            .collect())
        .unwrap_or_default();

    let confidence = v["confidence"].as_f64().unwrap_or(0.5) as f32;
    let explanation = v["explanation"]
        .as_str()
        .unwrap_or("No explanation provided")
        .to_string();

    Ok(AiplSuggestion {
        fn_signature: fn_signature.to_string(),
        suggestions,
        confidence,
        explanation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::client;

    #[test]
    fn aipl_without_ollama_returns_err() {
        if !client::ollama_available() {
            let r = aipl_suggest("fn add(a: Int, b: Int) -> Int", "llama3.2");
            assert!(r.is_err());
        }
    }

    #[test]
    fn parse_aipl_response_valid_json() {
        let raw = r#"{"suggestions":["@ensures result >= 0"],"confidence":0.9,"explanation":"always non-negative"}"#;
        let s = parse_aipl_response("fn abs(x: Int) -> Int", raw).unwrap();
        assert_eq!(s.suggestions.len(), 1);
        assert_eq!(s.suggestions[0], "@ensures result >= 0");
        assert!((s.confidence - 0.9).abs() < 0.001);
    }

    #[test]
    fn parse_aipl_response_no_json_returns_err() {
        let r = parse_aipl_response("fn f()", "no json here");
        assert!(r.is_err());
    }

    #[test]
    fn aipl_suggestion_fields() {
        let s = AiplSuggestion {
            fn_signature: "fn f()".into(),
            suggestions: vec!["@ensures true".into()],
            confidence: 0.8,
            explanation: "trivially true".into(),
        };
        assert_eq!(s.suggestions.len(), 1);
        assert!((s.confidence - 0.8).abs() < 0.001);
    }
}
