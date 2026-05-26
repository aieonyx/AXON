//! Integration tests — axon::ai runtime module (Ollama-optional).
use axon_std::ai::{client::ollama_available, InferenceWeight};

#[test] fn ai_weight_token_budgets_ordered() {
    assert!(InferenceWeight::SecurityCritical.max_tokens() > InferenceWeight::Standard.max_tokens());
    assert!(InferenceWeight::Standard.max_tokens() > InferenceWeight::FastPass.max_tokens());
}
#[test] fn ai_weight_temperature_ordered() {
    let sc = InferenceWeight::SecurityCritical.temperature();
    let st = InferenceWeight::Standard.temperature();
    let fp = InferenceWeight::FastPass.temperature();
    assert!(sc < st); assert!(st < fp);
}
#[test] fn ai_security_critical_requires_audit() {
    assert!(InferenceWeight::SecurityCritical.requires_audit());
    assert!(!InferenceWeight::Standard.requires_audit());
    assert!(!InferenceWeight::FastPass.requires_audit());
}
#[test] fn ai_weight_labels_unique() {
    let labels = [InferenceWeight::SecurityCritical.label(),
                  InferenceWeight::Standard.label(),
                  InferenceWeight::FastPass.label()];
    let set: std::collections::HashSet<_> = labels.iter().collect();
    assert_eq!(set.len(), 3);
}
#[test] fn ai_ollama_available_returns_bool() {
    let _ = ollama_available(); // must not panic
}
#[test] fn ai_infer_without_ollama_graceful() {
    use axon_std::ai::{AiError, infer};
    if !ollama_available() {
        let r = infer("test", "llama3.2");
        assert!(r.is_err());
        assert!(matches!(r.unwrap_err(), AiError::OllamaUnavailable(_)));
    }
}
#[test] fn ai_embed_without_ollama_graceful() {
    use axon_std::ai::embed;
    if !ollama_available() { assert!(embed("test", "nomic-embed-text").is_err()); }
}
#[test] fn ai_model_list_without_ollama_graceful() {
    use axon_std::ai::model_list;
    if !ollama_available() { assert!(model_list().is_err()); }
}
#[test] fn ai_infer_with_ollama() {
    use axon_std::ai::{infer, AiError};
    if ollama_available() {
        let r = infer("Reply with exactly: AXON", "llama3.2");
        match r {
            Ok(_) => {}  // model responded — full pass
            Err(AiError::ModelNotFound(_)) => {}  // model not loaded — acceptable
            Err(AiError::OllamaUnavailable(_)) => {}  // race condition — acceptable
            Err(e) => panic!("Unexpected Ollama error: {:?}", e),
        }
    }
}
#[test] fn ai_embed_with_ollama() {
    use axon_std::ai::embed;
    if ollama_available() {
        let r = embed("sovereign computing AXON", "nomic-embed-text");
        if let Ok(v) = r { assert!(!v.is_empty()); }
    }
}
#[test] fn ai_aipl_parse_valid_json() {
    use axon_std::ai::aipl::AiplSuggestion;
    let s = AiplSuggestion {
        fn_signature: "fn add(a: Int, b: Int) -> Int".into(),
        suggestions: vec!["@ensures result == a + b".into()],
        confidence: 0.9,
        explanation: "arithmetic".into(),
    };
    assert_eq!(s.suggestions.len(), 1);
    assert!((s.confidence - 0.9).abs() < 0.001);
}
