// axon_ai::csr — Compiler Self-Review Protocol. SPEC: 6A-02
// Second AI pass catches suggestions that contradict existing contracts.
// Skips gracefully with audit log if no AI model is available.

use std::collections::HashMap;
use crate::spec::{FormalSpec,Constraint,Effect};

#[derive(Debug,Clone)]
pub struct AIIntentSuggestion{
    pub fn_name:String,
    pub suggestion_nl:String,
    pub proposed_spec:FormalSpec,
    pub source_line:Option<u32>,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ConflictKind{Relaxes,Contradicts}

#[derive(Debug,Clone)]
pub struct CSRConflict{
    pub fn_name:String,
    pub suggestion_nl:String,
    pub existing_constraint:Constraint,
    pub kind:ConflictKind,
    pub source_line:Option<u32>,
}

#[derive(Debug,Clone)]
pub struct CSRReport{
    pub fn_name:String,
    pub suggestions_reviewed:usize,
    pub conflicts:Vec<CSRConflict>,
    pub skipped:bool,
}

impl CSRReport{
    pub fn is_clean(&self)->bool{self.conflicts.is_empty()}
    pub fn has_contradictions(&self)->bool{
        self.conflicts.iter().any(|c|c.kind==ConflictKind::Contradicts)
    }
    pub fn format_report(&self)->String{
        let mut out=String::new();
        out.push_str(&format!("AXON CSR REPORT — {}
",self.fn_name));
        if self.skipped{
            out.push_str("  CSR skipped — no model loaded
");
            return out;
        }
        out.push_str(&format!("  Pass 1 (AI inference):  {} suggestion(s) generated
",self.suggestions_reviewed));
        out.push_str(&format!("  Pass 2 (Self-review):   {} conflict(s) detected
",self.conflicts.len()));
        for c in &self.conflicts{
            out.push_str("\n  CONFLICT: @ai.intent suggestion");
            if let Some(l)=c.source_line{
                out.push_str(&format!(" at line {}",l));
            }
            out.push_str(&format!("
    Suggestion: {}
",c.suggestion_nl));
            out.push_str(&format!("    {:?}: {:?}
",c.kind,c.existing_constraint));
            out.push_str("    Resolution: COMPILE BLOCKED — resolve conflict manually
");
        }
        if self.is_clean(){
            out.push_str(&format!("
  {} suggestion(s) clean. 0 compile warnings.
",self.suggestions_reviewed));
        }
        out
    }
}

/// Returns true when an AI model endpoint is reachable.
/// Phase 6: checks if IntentTranslator has a configured endpoint.
/// SPEC: 6A-02
pub fn ai_model_available()->bool{
    std::env::var("AXON_AI_ENDPOINT").is_ok()||
    std::env::var("OLLAMA_API_BASE").is_ok()
}

/// Returns true if suggestion relaxes (weakens) an existing constraint.
/// SPEC: 6A-02
fn relaxes(suggested:&Constraint,existing:&Constraint)->bool{
    match (suggested,existing){
        (Constraint::ResultAtLeast(a),Constraint::ResultAtLeast(b))=>a<b,
        _=>false,
    }
}

/// Returns true if suggestion contradicts an existing constraint.
/// SPEC: 6A-02
fn contradicts(suggested:&FormalSpec,existing:&Constraint)->bool{
    match existing{
        Constraint::ResultNonNegative=>{
            suggested.ensures.iter().any(|c|matches!(c,Constraint::ResultAtLeast(n) if *n<0))
        }
        Constraint::NoHeapAllocation=>{
            suggested.effects.iter().any(|e|matches!(e,Effect::MayAllocate))
        }
        Constraint::ResultPositive=>{
            suggested.ensures.iter().any(|c|matches!(c,Constraint::ResultAtLeast(n) if *n<=0))
        }
        _=>false,
    }
}

pub struct CSRPass;

impl CSRPass{
    /// Run the second-pass self-review.
    /// SPEC: 6A-02
    pub fn run(
        suggestions:&[AIIntentSuggestion],
        contracts:&HashMap<String,Vec<Constraint>>,
        ai_available:bool,
    )->Vec<CSRReport>{
        if !ai_available{
            return suggestions.iter().map(|s|CSRReport{
                fn_name:s.fn_name.clone(),
                suggestions_reviewed:0,
                conflicts:Vec::new(),
                skipped:true,
            }).collect();
        }
        suggestions.iter().map(|s|{
            let existing=contracts.get(&s.fn_name).map(|v|v.as_slice()).unwrap_or(&[]);
            let mut conflicts=Vec::new();
            for existing_c in existing{
                if contradicts(&s.proposed_spec,existing_c){
                    conflicts.push(CSRConflict{
                        fn_name:s.fn_name.clone(),
                        suggestion_nl:s.suggestion_nl.clone(),
                        existing_constraint:existing_c.clone(),
                        kind:ConflictKind::Contradicts,
                        source_line:s.source_line,
                    });
                }
                for suggested_c in &s.proposed_spec.ensures{
                    if relaxes(suggested_c,existing_c){
                        conflicts.push(CSRConflict{
                            fn_name:s.fn_name.clone(),
                            suggestion_nl:s.suggestion_nl.clone(),
                            existing_constraint:existing_c.clone(),
                            kind:ConflictKind::Relaxes,
                            source_line:s.source_line,
                        });
                    }
                }
            }
            CSRReport{
                fn_name:s.fn_name.clone(),
                suggestions_reviewed:1,
                conflicts,
                skipped:false,
            }
        }).collect()
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::spec::{FormalSpec,Constraint,Effect};

    fn make_suggestion(fn_name:&str,nl:&str,spec:FormalSpec)->AIIntentSuggestion{
        AIIntentSuggestion{fn_name:fn_name.to_string(),suggestion_nl:nl.to_string(),proposed_spec:spec,source_line:Some(47)}
    }

    fn contracts(fn_name:&str,c:Constraint)->HashMap<String,Vec<Constraint>>{
        let mut m=HashMap::new();
        m.insert(fn_name.to_string(),vec![c]);
        m
    }

    #[test]
    fn test_clean_suggestion_no_conflict(){
        let s=make_suggestion("f","returns non-negative",FormalSpec::new("non-neg").with_ensures(Constraint::ResultNonNegative));
        let c=contracts("f",Constraint::ResultNonNegative);
        let reports=CSRPass::run(&[s],&c,true);
        assert!(reports[0].is_clean());
        assert!(!reports[0].skipped);
    }

    #[test]
    fn test_contradicting_suggestion_detected(){
        let s=make_suggestion("f","grant unconditionally",FormalSpec::new("relaxed").with_ensures(Constraint::ResultAtLeast(-1)));
        let c=contracts("f",Constraint::ResultNonNegative);
        let reports=CSRPass::run(&[s],&c,true);
        assert!(reports[0].has_contradictions());
        assert_eq!(reports[0].conflicts[0].kind,ConflictKind::Contradicts);
    }

    #[test]
    fn test_relaxing_suggestion_detected(){
        let s=make_suggestion("f","result at least zero",FormalSpec::new("weak").with_ensures(Constraint::ResultAtLeast(0)));
        let c=contracts("f",Constraint::ResultAtLeast(10));
        let reports=CSRPass::run(&[s],&c,true);
        assert!(!reports[0].is_clean());
        assert_eq!(reports[0].conflicts[0].kind,ConflictKind::Relaxes);
    }

    #[test]
    fn test_skip_when_no_ai(){
        let s=make_suggestion("f","anything",FormalSpec::new("x"));
        let c=contracts("f",Constraint::ResultNonNegative);
        let reports=CSRPass::run(&[s],&c,false);
        assert!(reports[0].skipped);
        assert!(reports[0].is_clean());
    }

    #[test]
    fn test_report_format_clean(){
        let s=make_suggestion("monitor","pure",FormalSpec::new("pure").with_effect(Effect::Pure));
        let c=HashMap::new();
        let reports=CSRPass::run(&[s],&c,true);
        let txt=reports[0].format_report();
        assert!(txt.contains("AXON CSR REPORT"));
        assert!(txt.contains("clean"));
    }

    #[test]
    fn test_report_format_conflict(){
        let s=make_suggestion("f","grant unconditionally",FormalSpec::new("bad").with_ensures(Constraint::ResultAtLeast(-1)));
        let c=contracts("f",Constraint::ResultNonNegative);
        let reports=CSRPass::run(&[s],&c,true);
        let txt=reports[0].format_report();
        assert!(txt.contains("CONFLICT"));
        assert!(txt.contains("COMPILE BLOCKED"));
    }

    #[test]
    fn test_skip_report_format(){
        let s=make_suggestion("f","x",FormalSpec::new("x"));
        let reports=CSRPass::run(&[s],&HashMap::new(),false);
        let txt=reports[0].format_report();
        assert!(txt.contains("CSR skipped"));
    }

    #[test]
    fn test_no_conflict_when_no_contracts(){
        let s=make_suggestion("f","do anything",FormalSpec::new("any").with_ensures(Constraint::ResultAtLeast(-99)));
        let reports=CSRPass::run(&[s],&HashMap::new(),true);
        assert!(reports[0].is_clean());
    }

    #[test]
    fn test_no_alloc_contradiction(){
        let s=make_suggestion("f","may allocate",FormalSpec::new("alloc").with_effect(Effect::MayAllocate));
        let c=contracts("f",Constraint::NoHeapAllocation);
        let reports=CSRPass::run(&[s],&c,true);
        assert!(reports[0].has_contradictions());
    }

}
