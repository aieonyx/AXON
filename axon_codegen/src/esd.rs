// axon_codegen::esd — Encrypted Segment Declarations. SPEC: 6C-03
// Three memory segment states: exposed, sealed, coiled.
// coiled = encrypted at rest (AES-256-GCM). PHASE7: actual encryption.
// Default state is locked for sovereignty-critical types.

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum SegmentState{Exposed,Sealed,Coiled}

impl std::fmt::Display for SegmentState{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        match self{
            SegmentState::Exposed=>write!(f,"exposed"),
            SegmentState::Sealed=>write!(f,"sealed"),
            SegmentState::Coiled=>write!(f,"coiled"),
        }
    }
}

#[derive(Debug,Clone)]
pub struct ESDAnnotation{
    pub symbol:String,
    pub state:SegmentState,
    pub key_ref:Option<String>,
    pub has_override:bool,
    pub line:usize,
}

#[derive(Debug,Clone)]
pub struct ESDViolation{
    pub symbol:String,
    pub declared_state:SegmentState,
    pub required_state:SegmentState,
    pub message:String,
}

impl std::fmt::Display for ESDViolation{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        write!(f,"ESD violation: {} declared as {} but requires {} — {}",
            self.symbol,self.declared_state,self.required_state,self.message)
    }
}

#[derive(Debug)]
pub struct ESDReport{
    pub segments:Vec<ESDAnnotation>,
    pub violations:Vec<ESDViolation>,
}

impl ESDReport{
    pub fn is_clean(&self)->bool{self.violations.is_empty()}
    pub fn coiled_count(&self)->usize{
        self.segments.iter().filter(|s|s.state==SegmentState::Coiled).count()
    }
}

/// Types that MUST use @segment(coiled) by default.
/// Compiler error if declared as exposed or sealed without @override_segment_requirement.
const MANDATORY_COILED_TYPES:&[&str]=&[
    "CryptoKey","PrivateKey","SecretKey","ModelWeights",
    "SovereignData","KeyMaterial","AuthToken","PasswordHash",
];

pub struct ESDAnalyser;

impl ESDAnalyser{
    /// Scan source for @segment annotations and enforce mandatory-coiled policy.
    /// SPEC: 6C-03
    pub fn analyse(source:&str)->ESDReport{
        let annotations=Self::collect_annotations(source);
        let mut violations=Vec::new();
        for ann in &annotations{
            if ann.state!=SegmentState::Coiled&&!ann.has_override{
                if let Some(typ)=Self::mandatory_coiled_type(source,&ann.symbol){
                    violations.push(ESDViolation{
                        symbol:ann.symbol.clone(),
                        declared_state:ann.state.clone(),
                        required_state:SegmentState::Coiled,
                        message:format!("{} requires @segment(coiled). Add @override_segment_requirement with justification to bypass.",typ),
                    });
                }
            }
        }
        // Check static items of mandatory-coiled types without any @segment annotation
        for line in source.lines(){
            let t=line.trim();
            if t.starts_with("static ")||t.starts_with("pub static "){
                for typ in MANDATORY_COILED_TYPES{
                    if t.contains(typ){
                        let name=Self::extract_static_name(t);
                        let already_annotated=annotations.iter().any(|a|a.symbol==name);
                        if !already_annotated{
                            violations.push(ESDViolation{
                                symbol:name,
                                declared_state:SegmentState::Exposed,
                                required_state:SegmentState::Coiled,
                                message:format!("{} must use @segment(coiled). No segment annotation found.",typ),
                            });
                        }
                    }
                }
            }
        }
        ESDReport{segments:annotations,violations}
    }

    fn collect_annotations(source:&str)->Vec<ESDAnnotation>{
        let mut annotations=Vec::new();
        let lines:Vec<&str>=source.lines().collect();
        for (i,line) in lines.iter().enumerate(){
            let t=line.trim();
            if t.starts_with("@segment"){
                let inner=t.trim_start_matches("@segment")
                    .trim().trim_matches('(').trim_matches(')');
                let state=if inner.contains("coiled"){SegmentState::Coiled}
                    else if inner.contains("sealed"){SegmentState::Sealed}
                    else{SegmentState::Exposed};
                let key_ref=if inner.contains("key:"){
                    inner.split("key:").nth(1).map(|s|s.trim().trim_matches('"').to_string())
                }else{None};
                let has_override=lines[..i].iter().rev().take(3)
                    .any(|l|l.trim().starts_with("@override_segment_requirement"));
                let symbol=lines.get(i+1)
                    .map(|l|Self::extract_static_name(l.trim())).unwrap_or_default();
                annotations.push(ESDAnnotation{symbol,state,key_ref,has_override,line:i+1});
            }
        }
        annotations
    }

    fn mandatory_coiled_type(source:&str,symbol:&str)->Option<&'static str>{
        for line in source.lines(){
            if line.contains(symbol){
                for typ in MANDATORY_COILED_TYPES{
                    if line.contains(typ){return Some(typ);}
                }
            }
        }
        None
    }

    fn extract_static_name(line:&str)->String{
        let rest=line.trim_start_matches("pub ").trim_start_matches("static ");
        rest.split([':','=',]).next().unwrap_or("").trim().to_string()
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_coiled_clean(){
        let s="@segment(coiled)\nstatic W:ModelWeights=load();";
        let r=ESDAnalyser::analyse(s);
        assert!(r.is_clean());
        assert_eq!(r.coiled_count(),1);
    }

    #[test]
    fn test_exposed_normal_type_clean(){
        let s="@segment(exposed)\nstatic CFG:AppConfig=default();";
        assert!(ESDAnalyser::analyse(s).is_clean());
    }

    #[test]
    fn test_mandatory_no_annotation_violation(){
        let s="static KEY:CryptoKey=load();";
        let r=ESDAnalyser::analyse(s);
        assert!(!r.is_clean());
    }

    #[test]
    fn test_sealed_on_cryptokey_violation(){
        let s="@segment(sealed)\nstatic KEY:CryptoKey=load();";
        assert!(!ESDAnalyser::analyse(s).is_clean());
    }

    #[test]
    fn test_override_bypasses(){
        let s="@override_segment_requirement\n@segment(sealed)\nstatic K:CryptoKey=load();";
        assert!(ESDAnalyser::analyse(s).is_clean());
    }

    #[test]
    fn test_sealed_normal_type(){
        let s="@segment(sealed)\nstatic ST:ProcessState=new();";
        let r=ESDAnalyser::analyse(s);
        assert!(r.is_clean());
        assert_eq!(r.segments[0].state,SegmentState::Sealed);
    }

    #[test]
    fn test_coiled_key_ref(){
        let s="@segment(coiled, key: project/key)\nstatic W:ModelWeights=load();";
        let r=ESDAnalyser::analyse(s);
        assert!(r.is_clean());
        assert!(r.segments[0].key_ref.is_some());
    }

    #[test]
    fn test_violation_display(){
        let v=ESDViolation{
            symbol:"K".to_string(),
            declared_state:SegmentState::Exposed,
            required_state:SegmentState::Coiled,
            message:"test".to_string(),
        };
        assert!(v.to_string().contains("ESD violation"));
    }
}
