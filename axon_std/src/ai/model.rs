//! axon::ai::model — model management via Ollama.

use serde::Deserialize;
use super::{AiError, AiResult, client};

/// An opaque handle to a loaded Ollama model.
#[derive(Debug, Clone)]
pub struct ModelHandle {
    /// The model name as known to Ollama (e.g. "llama3.2", "nomic-embed-text").
    pub name: String,
    /// Parameter count string if known (e.g. "3.2B").
    pub size: Option<String>,
}

impl ModelHandle {
    /// Return the model name.
    pub fn name(&self) -> &str { &self.name }
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Option<Vec<ModelInfo>>,
}

#[derive(Deserialize)]
struct ModelInfo {
    name:    String,
    #[serde(rename = "parameter_size")]
    size:    Option<String>,
}

/// List all models currently available in Ollama.
///
/// Returns model names that can be passed to [`infer`][super::infer]
/// or [`embed`][super::embed].
pub fn model_list() -> AiResult<Vec<String>> {
    let body = client::ollama_get("/api/tags")?;
    let resp: TagsResponse = serde_json::from_str(&body)
        .map_err(|e| AiError::MalformedResponse(format!("JSON parse: {e}")))?;

    Ok(resp.models
        .unwrap_or_default()
        .into_iter()
        .map(|m| m.name)
        .collect())
}

/// Load (pull) a model by name into Ollama.
///
/// If the model is already present, this is a no-op.
/// Returns a [`ModelHandle`] for use in inference calls.
pub fn model_load(name: &str) -> AiResult<ModelHandle> {
    #[derive(serde::Serialize)]
    struct PullRequest<'a> { name: &'a str, stream: bool }

    let body = serde_json::to_string(&PullRequest { name, stream: false })
        .map_err(|e| AiError::IoError(e.to_string()))?;

    let response = client::ollama_post("/api/pull", &body)?;

    // Ollama returns {"status":"success"} on completion
    if response.contains("success") || response.contains("already") {
        Ok(ModelHandle { name: name.to_string(), size: None })
    } else {
        Err(AiError::ModelNotFound(format!("pull failed for '{name}': {response}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::client;

    #[test]
    fn model_list_without_ollama_returns_err() {
        if !client::ollama_available() {
            assert!(model_list().is_err());
        }
    }

    #[test]
    fn model_list_with_ollama_returns_vec() {
        if client::ollama_available() {
            let models = model_list().unwrap_or_default();
            // Just verify it doesn't panic — may be empty if no models loaded
            let _ = models.len();
        }
    }

    #[test]
    fn model_handle_name() {
        let h = ModelHandle { name: "llama3.2".into(), size: Some("3.2B".into()) };
        assert_eq!(h.name(), "llama3.2");
    }
}
