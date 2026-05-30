// axon_parser::sec — Semantic Equivalence Classes. SPEC: 6C-02
// COMPILER-INTERNAL ONLY. Rule S4+i-R1 applies:
// SEC is a READ mechanism, never a WRITE mechanism.
// Equivalent forms are NEVER exposed in documentation,
// tooling output, or error messages.

/// A normalization event recorded at debug level only.
/// Never surfaced to the developer in error messages or tooling.
#[derive(Debug,Clone)]
pub struct SECNormalizationLog{
    pub class_id:&'static str,
    pub line:usize,
}

#[derive(Debug)]
pub struct SECReport{
    pub normalizations:Vec<SECNormalizationLog>,
    pub source_was_modified:bool,
}

impl SECReport{
    pub fn normalization_count(&self)->usize{self.normalizations.len()}
    pub fn is_canonical(&self)->bool{!self.source_was_modified}
}

/// Compiler-internal equivalence entry.
/// Not user-facing. S4+i-R1.
struct SECEntry{
    class_id:&'static str,
    variant:&'static str,
    canonical:&'static str,
}

/// Built-in equivalence table.
/// COMPILER-INTERNAL. Not exported. Not documented publicly.
/// S4+i-R1: never expose these forms in error messages or docs.
const SEC_TABLE:&[SECEntry]=&[
    SECEntry{class_id:"SEC-001",variant:"authorize ",canonical:"grant "},
    SECEntry{class_id:"SEC-002",variant:"allow ",canonical:"grant "},
    SECEntry{class_id:"SEC-003",variant:"permit ",canonical:"grant "},
    SECEntry{class_id:"SEC-004",variant:"@verify",canonical:"@ensures"},
    SECEntry{class_id:"SEC-005",variant:"@check",canonical:"@ensures"},
    SECEntry{class_id:"SEC-006",variant:"@require",canonical:"@requires"},
    SECEntry{class_id:"SEC-007",variant:"@must",canonical:"@requires"},
];

pub struct SECNormalizer;

impl SECNormalizer{
    /// Normalize source to canonical forms before parsing.
    /// Phase 6: source-level text normalization.
    /// PHASE7: token-sequence pattern matching before AST.
    /// SPEC: 6C-02 | Rule S4+i-R1
    pub fn normalize(source:&str)->( String,SECReport){
        let mut result=source.to_string();
        let mut logs=Vec::new();
        let mut modified=false;
        for entry in SEC_TABLE{
            if result.contains(entry.variant){
                let line=result.lines().enumerate()
                    .find(|(_,l)|l.contains(entry.variant))
                    .map(|(i,_)|i+1).unwrap_or(0);
                result=result.replace(entry.variant,entry.canonical);
                logs.push(SECNormalizationLog{class_id:entry.class_id,line});
                modified=true;
            }
        }
        (result,SECReport{normalizations:logs,source_was_modified:modified})
    }

    /// Returns true if source contains any variant forms requiring normalization.
    pub fn needs_normalization(source:&str)->bool{
        SEC_TABLE.iter().any(|e|source.contains(e.variant))
    }

    /// Returns the canonical form for a given variant, if known.
    /// Returns None if the input is already canonical or unknown.
    /// INTERNAL USE ONLY. S4+i-R1.
    pub fn canonical_for(variant:&str)->Option<&'static str>{
        SEC_TABLE.iter().find(|e|variant.contains(e.variant)).map(|e|e.canonical)
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_canonical_source_unchanged(){
        let src="fn f(){grant cap to process;}";
        let (out,report)=SECNormalizer::normalize(src);
        assert_eq!(out,src);
        assert!(report.is_canonical());
        assert_eq!(report.normalization_count(),0);
    }

    #[test]
    fn test_variant_normalized_to_canonical(){
        let src="authorize process with cap;";
        let (out,report)=SECNormalizer::normalize(src);
        assert!(out.contains("grant "));
        assert!(!report.is_canonical());
        assert_eq!(report.normalization_count(),1);
    }

    #[test]
    fn test_decorator_variant_normalized(){
        let src="@verify(result>=0)";
        let (out,_)=SECNormalizer::normalize(src);
        assert!(out.contains("@ensures"));
        assert!(!out.contains("@verify"));
    }

    #[test]
    fn test_needs_normalization_true(){
        assert!(SECNormalizer::needs_normalization("allow cap to proc;"));
    }

    #[test]
    fn test_needs_normalization_false(){
        assert!(!SECNormalizer::needs_normalization("grant cap to proc;"));
    }

    #[test]
    fn test_canonical_for_known_variant(){
        let c=SECNormalizer::canonical_for("authorize process");
        assert_eq!(c,Some("grant "));
    }

    #[test]
    fn test_canonical_for_unknown_returns_none(){
        let c=SECNormalizer::canonical_for("unknown_keyword");
        assert!(c.is_none());
    }

    #[test]
    fn test_multiple_variants_in_source(){
        let src="authorize cap;
@verify(x>0)";
        let (out,report)=SECNormalizer::normalize(src);
        assert!(out.contains("grant "));
        assert!(out.contains("@ensures"));
        assert!(report.normalization_count()>=2);
    }

    #[test]
    fn test_normalization_log_has_line_number(){
        let src="fn f(){}
authorize cap;";
        let (_,report)=SECNormalizer::normalize(src);
        assert!(!report.normalizations.is_empty());
        assert_eq!(report.normalizations[0].line,2);
    }
}
