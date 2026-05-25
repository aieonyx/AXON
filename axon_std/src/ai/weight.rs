//! InferenceWeight — Bio DNA: Transcription Factor Specificity.
//!
//! Just as transcription factors bind with different specificity to
//! regulate gene expression, AXON AI inference applies different depth
//! based on the security context of the calling code.

/// Controls inference depth for an axon::ai call.
///
/// # Bio DNA mapping
///
/// Transcription factors in biology bind to DNA promoters with varying
/// affinity — high-specificity TFs activate only the right genes at the
/// right time. `InferenceWeight` applies the same principle to AI:
/// high-security functions get full attention, low-risk calls get
/// fast-pass to preserve throughput.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceWeight {
    /// Full inference depth. Maximum token budget. Slowest.
    ///
    /// Use for: @security_critical functions, capability verification,
    /// audit-trail generation, formal spec proposals.
    ///
    /// Bio DNA: High-specificity transcription factor — full activation.
    SecurityCritical,

    /// Standard inference depth. Default for most AXON programs.
    ///
    /// Use for: general queries, code assistance, documentation generation.
    ///
    /// Bio DNA: Medium-affinity TF — normal expression level.
    Standard,

    /// Minimal inference. Lowest token budget. Fastest response.
    ///
    /// Use for: hot paths, low-risk classification, UI hints.
    /// Never use for security-sensitive operations.
    ///
    /// Bio DNA: Low-affinity TF — minimal expression, high throughput.
    FastPass,
}

impl InferenceWeight {
    /// Maximum tokens for this weight tier.
    pub const fn max_tokens(&self) -> u32 {
        match self {
            InferenceWeight::SecurityCritical => 4096,
            InferenceWeight::Standard         => 1024,
            InferenceWeight::FastPass         =>  256,
        }
    }

    /// Ollama temperature for this weight tier.
    /// Lower = more deterministic (better for security contexts).
    pub const fn temperature(&self) -> f32 {
        match self {
            InferenceWeight::SecurityCritical => 0.1,
            InferenceWeight::Standard         => 0.7,
            InferenceWeight::FastPass         => 0.9,
        }
    }

    /// Whether this weight tier requires audit logging via axon::audit.
    pub const fn requires_audit(&self) -> bool {
        matches!(self, InferenceWeight::SecurityCritical)
    }

    /// Human-readable tier name for logging.
    pub const fn label(&self) -> &'static str {
        match self {
            InferenceWeight::SecurityCritical => "security_critical",
            InferenceWeight::Standard         => "standard",
            InferenceWeight::FastPass         => "fast_pass",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_token_budgets_ordered() {
        assert!(InferenceWeight::SecurityCritical.max_tokens() >
                InferenceWeight::Standard.max_tokens());
        assert!(InferenceWeight::Standard.max_tokens() >
                InferenceWeight::FastPass.max_tokens());
    }

    #[test]
    fn weight_temperature_ordered() {
        let sc = InferenceWeight::SecurityCritical.temperature();
        let st = InferenceWeight::Standard.temperature();
        let fp = InferenceWeight::FastPass.temperature();
        assert!(sc < st);
        assert!(st < fp);
    }

    #[test]
    fn only_security_critical_requires_audit() {
        assert!(InferenceWeight::SecurityCritical.requires_audit());
        assert!(!InferenceWeight::Standard.requires_audit());
        assert!(!InferenceWeight::FastPass.requires_audit());
    }

    #[test]
    fn weight_labels_distinct() {
        assert_ne!(InferenceWeight::SecurityCritical.label(),
                   InferenceWeight::Standard.label());
        assert_ne!(InferenceWeight::Standard.label(),
                   InferenceWeight::FastPass.label());
    }
}
