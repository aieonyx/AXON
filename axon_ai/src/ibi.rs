// axon_ai::ibi — Immortal Boundary Invariants. SPEC: 6A-04
// Telomere protection: critical boundaries cannot be weakened.
// Rule S4+i-R3: IBIs protect sovereignty, they do not override it.

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum IBITier{
    /// Tier 1 — AIEONYX-defined. Absolute. Hard error on suppression.
    Constitutional,
    /// Tier 2 — Node owner-defined. Cannot be overridden by external parties.
    Sovereign,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum IBIViolationKind{
    /// @suppress_invariant applied to an @immortal_invariant function.
    SuppressAttempted,
    /// External code attempts to override a Tier 2 @sovereign_invariant.
    ExternalOverride,
    /// Post-optimization weakening detected. PHASE7: full codegen check.
    InvariantWeakened,
}

#[derive(Debug,Clone)]
pub struct ImmortalBoundaryInvariant{
    /// Function name this IBI is attached to.
    pub fn_name:String,
    /// Tier 1 (Constitutional) or Tier 2 (Sovereign).
    pub tier:IBITier,
    /// Tier 2 only: the sovereign tag (e.g. "no-external-telemetry").
    pub tag:Option<String>,
    /// The @ensures expressions this IBI enforces.
    pub ensures:Vec<String>,
}

#[derive(Debug,Clone)]
pub struct IBIViolation{
    pub fn_name:String,
    pub tier:IBITier,
    pub kind:IBIViolationKind,
    pub message:String,
}

impl std::fmt::Display for IBIViolation{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        write!(f,"IBI VIOLATION [{:?}|{:?}] fn {}: {}",
            self.tier,self.kind,self.fn_name,self.message)
    }
}

/// Registry of all IBIs active in a compilation unit.
/// SPEC: 6A-04
#[derive(Debug,Default)]
pub struct ImmortalInvariantRegistry{
    pub ibis:Vec<ImmortalBoundaryInvariant>,
}

impl ImmortalInvariantRegistry{
    pub fn new()->Self{Self{ibis:Vec::new()}}

    /// Register a new IBI.
    pub fn register(&mut self,ibi:ImmortalBoundaryInvariant){
        self.ibis.push(ibi);
    }

    /// Check if a function has a Tier 1 Constitutional IBI.
    pub fn is_constitutional(&self,fn_name:&str)->bool{
        self.ibis.iter().any(|i|i.fn_name==fn_name&&i.tier==IBITier::Constitutional)
    }

    /// Check if a function has a Tier 2 Sovereign IBI.
    pub fn is_sovereign(&self,fn_name:&str)->bool{
        self.ibis.iter().any(|i|i.fn_name==fn_name&&i.tier==IBITier::Sovereign)
    }

    /// Detect @suppress_invariant on a Constitutional IBI function.
    /// Hard error — Tier 1 cannot be suppressed.
    /// SPEC: 6A-04
    pub fn check_suppression(&self,fn_name:&str,decorators:&[&str])->Option<IBIViolation>{
        let has_suppress=decorators.contains(&"suppress_invariant");
        if has_suppress&&self.is_constitutional(fn_name){
            return Some(IBIViolation{
                fn_name:fn_name.to_string(),
                tier:IBITier::Constitutional,
                kind:IBIViolationKind::SuppressAttempted,
                message:format!("cannot suppress @immortal_invariant on fn {}: Constitutional IBIs are absolute",fn_name),
            });
        }
        None
    }

    /// Check if external code attempts to override a Sovereign IBI.
    /// SPEC: 6A-04
    pub fn check_external_override(&self,fn_name:&str,is_external:bool)->Option<IBIViolation>{
        if is_external&&self.is_sovereign(fn_name){
            return Some(IBIViolation{
                fn_name:fn_name.to_string(),
                tier:IBITier::Sovereign,
                kind:IBIViolationKind::ExternalOverride,
                message:format!("external code cannot override @sovereign_invariant on fn {}",fn_name),
            });
        }
        None
    }

    /// Build registry from AXON source decorators.
    /// Detects @immortal_invariant and @sovereign_invariant decorators.
    /// SPEC: 6A-04
    pub fn from_source(source:&str)->Self{
        let mut reg=Self::new();
        let lines:Vec<&str>=source.lines().collect();
        for (i,line) in lines.iter().enumerate(){
            let trimmed=line.trim();
            if trimmed=="@immortal_invariant"{
                if let Some(fn_line)=lines[i+1..].iter().find(|l|l.trim().starts_with("fn ")||l.trim().starts_with("@ensures")){
                    let fn_name=extract_fn_name(fn_line);
                    reg.register(ImmortalBoundaryInvariant{
                        fn_name,tier:IBITier::Constitutional,tag:None,ensures:Vec::new(),
                    });
                }
            }else if trimmed.starts_with("@sovereign_invariant"){
                let tag=trimmed.trim_start_matches("@sovereign_invariant")
                    .trim().trim_matches('(').trim_matches(')').trim_matches('"').to_string();
                if let Some(fn_line)=lines[i+1..].iter().find(|l|l.trim().starts_with("fn ")||l.trim().starts_with("@ensures")){
                    let fn_name=extract_fn_name(fn_line);
                    reg.register(ImmortalBoundaryInvariant{
                        fn_name,tier:IBITier::Sovereign,
                        tag:if tag.is_empty(){None}else{Some(tag)},
                        ensures:Vec::new(),
                    });
                }
            }
        }
        reg
    }

    /// Load Tier 2 IBIs from sovereign.axon manifest.
    /// PHASE7: full manifest parsing with cryptographic verification.
    pub fn load_sovereign_manifest(_path:&str)->Vec<ImmortalBoundaryInvariant>{
        // Phase 6 stub — sovereign.axon parsing is Phase 7
        Vec::new()
    }
}

fn extract_fn_name(line:&str)->String{
    let trimmed=line.trim();
    if let Some(rest)=trimmed.strip_prefix("fn "){
        rest.split('(').next().unwrap_or("").trim().to_string()
    }else{
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    fn make_constitutional(fn_name:&str)->ImmortalBoundaryInvariant{
        ImmortalBoundaryInvariant{fn_name:fn_name.to_string(),tier:IBITier::Constitutional,tag:None,ensures:Vec::new()}
    }
    fn make_sovereign(fn_name:&str,tag:&str)->ImmortalBoundaryInvariant{
        ImmortalBoundaryInvariant{fn_name:fn_name.to_string(),tier:IBITier::Sovereign,tag:Some(tag.to_string()),ensures:Vec::new()}
    }

    #[test]
    fn test_register_and_query(){
        let mut reg=ImmortalInvariantRegistry::new();
        reg.register(make_constitutional("grant_capability"));
        assert!(reg.is_constitutional("grant_capability"));
        assert!(!reg.is_sovereign("grant_capability"));
    }

    #[test]
    fn test_suppress_constitutional_is_violation(){
        let mut reg=ImmortalInvariantRegistry::new();
        reg.register(make_constitutional("grant_capability"));
        let v=reg.check_suppression("grant_capability",&["suppress_invariant"]);
        assert!(v.is_some());
        assert_eq!(v.unwrap().kind,IBIViolationKind::SuppressAttempted);
    }

    #[test]
    fn test_suppress_sovereign_is_allowed(){
        let mut reg=ImmortalInvariantRegistry::new();
        reg.register(make_sovereign("send_metrics","no-external-telemetry"));
        // Tier 2 can be relaxed by owner — suppress is NOT a violation
        let v=reg.check_suppression("send_metrics",&["suppress_invariant"]);
        assert!(v.is_none());
    }

    #[test]
    fn test_external_override_sovereign(){
        let mut reg=ImmortalInvariantRegistry::new();
        reg.register(make_sovereign("send_metrics","no-telemetry"));
        let v=reg.check_external_override("send_metrics",true);
        assert!(v.is_some());
        assert_eq!(v.unwrap().kind,IBIViolationKind::ExternalOverride);
    }

    #[test]
    fn test_external_override_own_code_allowed(){
        let mut reg=ImmortalInvariantRegistry::new();
        reg.register(make_sovereign("send_metrics","no-telemetry"));
        let v=reg.check_external_override("send_metrics",false);
        assert!(v.is_none());
    }

    #[test]
    fn test_from_source_detects_immortal(){
        let src="@immortal_invariant
fn grant_capability():
    pass
";
        let reg=ImmortalInvariantRegistry::from_source(src);
        assert!(reg.is_constitutional("grant_capability"));
    }

    #[test]
    fn test_from_source_detects_sovereign(){
        let src="@sovereign_invariant(\"no-telemetry\")\nfn send_metrics():\n    pass\n";
        let reg=ImmortalInvariantRegistry::from_source(src);
        assert!(reg.is_sovereign("send_metrics"));
    }

    #[test]
    fn test_sovereign_manifest_stub_returns_empty(){
        let ibis=ImmortalInvariantRegistry::load_sovereign_manifest("sovereign.axon");
        assert!(ibis.is_empty());
    }

    #[test]
    fn test_violation_display(){
        let v=IBIViolation{fn_name:"f".to_string(),tier:IBITier::Constitutional,
            kind:IBIViolationKind::SuppressAttempted,message:"test".to_string()};
        assert!(v.to_string().contains("IBI VIOLATION"));
    }
}
