//! axon::ai::infer — text generation with security-weighted dispatch.

use serde::{Deserialize, Serialize};
use super::{AiError, AiResult, InferenceWeight, client};

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model:       &'a str,
    prompt:      &'a str,
    stream:      bool,
    options:     GenerateOptions,
}

#[derive(Serialize)]
struct GenerateOptions {
    num_predict:  u32,
    temperature:  f32,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: Option<String>,
    error:    Option<String>,
}

/// Generate a text response from a local Ollama model.
///
/// Uses [`InferenceWeight::Standard`] by default.
///
/// # Examples
///
/// ```rust,ignore
/// use axon_std::ai::infer;
/// let reply = infer("What is seL4?", "llama3.2").unwrap();
/// ```
pub fn infer(prompt: &str, model: &str) -> AiResult<String> {
    infer_weighted(prompt, model, InferenceWeight::Standard)
}

/// Generate a text response with explicit security weight.
///
/// # Bio DNA — Security-Weighted Inference
///
/// `SecurityCritical` → 4096 tokens, temperature 0.1, audit required
/// `Standard`         → 1024 tokens, temperature 0.7
/// `FastPass`         → 256 tokens,  temperature 0.9
///
/// # Examples
///
/// ```rust,ignore
/// use axon_std::ai::{infer_weighted, InferenceWeight};
/// let reply = infer_weighted(
///     "Audit this capability grant",
///     "llama3.2",
///     InferenceWeight::SecurityCritical,
/// ).unwrap();
/// ```
pub fn infer_weighted(
    prompt: &str,
    model:  &str,
    weight: InferenceWeight,
) -> AiResult<String> {
    let req = GenerateRequest {
        model,
        prompt,
        stream: false,
        options: GenerateOptions {
            num_predict: weight.max_tokens(),
            temperature: weight.temperature(),
        },
    };

    let body = serde_json::to_string(&req)
        .map_err(|e| AiError::IoError(e.to_string()))?;

    let response_body = client::ollama_post("/api/generate", &body)?;

    let resp: GenerateResponse = serde_json::from_str(&response_body)
        .map_err(|e| AiError::MalformedResponse(format!("JSON parse: {e} — body: {response_body}")))?;

    if let Some(err) = resp.error {
        return Err(AiError::ModelNotFound(err));
    }

    resp.response.ok_or_else(|| {
        AiError::MalformedResponse("no 'response' field in Ollama reply".into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_without_ollama_returns_unavailable() {
        // If Ollama isn't running, we expect OllamaUnavailable — not a panic
        if !client::ollama_available() {
            let r = infer("test", "llama3.2");
            assert!(r.is_err());
            assert!(matches!(r.unwrap_err(), AiError::OllamaUnavailable(_)));
        }
    }

    #[test]
    fn infer_weighted_security_critical_has_low_temp() {
        assert!(InferenceWeight::SecurityCritical.temperature() < 0.5);
    }

    #[test]
    fn infer_weighted_fast_pass_has_small_budget() {
        assert!(InferenceWeight::FastPass.max_tokens() <= 256);
    }
}
