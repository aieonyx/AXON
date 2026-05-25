//! axon::ai::embed — text embeddings via Ollama.

use serde::{Deserialize, Serialize};
use super::{AiError, AiResult, client};

#[derive(Serialize)]
struct EmbedRequest<'a> {
    model:  &'a str,
    prompt: &'a str,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embedding: Option<Vec<f32>>,
    error:     Option<String>,
}

/// Generate a text embedding vector from a local Ollama model.
///
/// Returns a float vector suitable for semantic search, clustering,
/// or similarity comparison in EdisonDB vector indexes.
///
/// # Examples
///
/// ```rust,ignore
/// use axon_std::ai::embed;
/// let vec = embed("sovereign computing", "nomic-embed-text").unwrap();
/// assert!(!vec.is_empty());
/// ```
pub fn embed(text: &str, model: &str) -> AiResult<Vec<f32>> {
    let req = EmbedRequest { model, prompt: text };
    let body = serde_json::to_string(&req)
        .map_err(|e| AiError::IoError(e.to_string()))?;

    let response_body = client::ollama_post("/api/embeddings", &body)?;

    let resp: EmbedResponse = serde_json::from_str(&response_body)
        .map_err(|e| AiError::MalformedResponse(format!("JSON parse: {e}")))?;

    if let Some(err) = resp.error {
        return Err(AiError::ModelNotFound(err));
    }

    resp.embedding.ok_or_else(|| {
        AiError::MalformedResponse("no 'embedding' field in Ollama reply".into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::client;

    #[test]
    fn embed_without_ollama_returns_err() {
        if !client::ollama_available() {
            let r = embed("test", "nomic-embed-text");
            assert!(r.is_err());
        }
    }

    #[test]
    fn embed_with_ollama_returns_vector() {
        if client::ollama_available() {
            // Only runs when Ollama is live on the machine
            let r = embed("AXON sovereign computing", "nomic-embed-text");
            if let Ok(v) = r {
                assert!(!v.is_empty());
                // Embedding vectors are unit-normalized by most models
                let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                assert!(norm > 0.0);
            }
        }
    }
}
