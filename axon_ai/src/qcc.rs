// axon_ai::qcc — Quantum Contract Checker. SPEC: 6A-06
// Bio: Superoxide Dismutase — simultaneously neutralises contradictory pairs.
// Detects @ensures contracts that cannot both be true.

use crate::spec::{Constraint,Effect,FormalSpec};

#[derive(Debug,Clone)]
pub struct ContractContradiction{
    pub fn_name:String,
    pub constraint_a:Constraint,
    pub constraint_b:Constraint,
    pub reason:String,
}
impl std::fmt::Display for ContractContradiction{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        write!(f,"QCC contradiction in fn {}: {}",self.fn_name,self.reason)
    }
}
#[derive(Debug,Clone)]
pub struct QCCReport{
    pub fn_name:String,
    pub contradictions:Vec<ContractContradiction>,
    pub unsatisfiable:bool,
}
impl QCCReport{
    pub fn is_clean(&self)->bool{self.contradictions.is_empty()}
}
pub struct QCCAnalyser;
impl QCCAnalyser{
    pub fn check(fn_name:&str,spec:&FormalSpec)->QCCReport{
        let mut contradictions=Vec::new();
        let ensures=&spec.ensures;
        let effects=&spec.effects;
        for i in 0..ensures.len(){
            for j in (i+1)..ensures.len(){
                if let Some(r)=Self::constraints_contradict(&ensures[i],&ensures[j]){
                    contradictions.push(ContractContradiction{
                        fn_name:fn_name.to_string(),
                        constraint_a:ensures[i].clone(),
                        constraint_b:ensures[j].clone(),
                        reason:r,
                    });
                }
            }
        }
        for i in 0..effects.len(){
            for j in (i+1)..effects.len(){
                if let Some(r)=Self::effects_contradict(&effects[i],&effects[j]){
                    contradictions.push(ContractContradiction{
                        fn_name:fn_name.to_string(),
                        constraint_a:Constraint::Custom(format!("{:?}",effects[i])),
                        constraint_b:Constraint::Custom(format!("{:?}",effects[j])),
                        reason:r,
                    });
                }
            }
        }
        let unsatisfiable=!contradictions.is_empty();
        QCCReport{fn_name:fn_name.to_string(),contradictions,unsatisfiable}
    }
    pub fn check_all(specs:&[(String,FormalSpec)])->Vec<QCCReport>{
        specs.iter().map(|(n,s)|Self::check(n,s)).filter(|r|!r.is_clean()).collect()
    }
    fn constraints_contradict(a:&Constraint,b:&Constraint)->Option<String>{
        match (a,b){
            (Constraint::ResultNonNegative,Constraint::ResultAtLeast(n)) if *n<0=>
                Some(format!("result>=0 contradicts result>={}",n)),
            (Constraint::ResultAtLeast(n),Constraint::ResultNonNegative) if *n<0=>
                Some(format!("result>={} contradicts result>=0",n)),
            (Constraint::ResultPositive,Constraint::ResultAtLeast(n)) if *n<=0=>
                Some(format!("result>0 contradicts result>={}",n)),
            (Constraint::ResultAtLeast(a),Constraint::ResultAtMost(b)) if a>b=>
                Some(format!("result>={} contradicts result<={}: empty range",a,b)),
            (Constraint::ResultEquals(a),Constraint::ResultEquals(b)) if a!=b=>
                Some(format!("result=={} contradicts result=={}",a,b)),
            (Constraint::ResultEquals(n),Constraint::ResultNonNegative) if *n<0=>
                Some(format!("result=={} contradicts result>=0",n)),
            _=>None,
        }
    }
    fn effects_contradict(a:&Effect,b:&Effect)->Option<String>{
        match (a,b){
            (Effect::Pure,Effect::MayAllocate)=>Some("Pure contradicts MayAllocate".to_string()),
            (Effect::MayAllocate,Effect::Pure)=>Some("MayAllocate contradicts Pure".to_string()),
            (Effect::NoAllocate,Effect::MayAllocate)=>Some("NoAllocate contradicts MayAllocate".to_string()),
            (Effect::MayAllocate,Effect::NoAllocate)=>Some("MayAllocate contradicts NoAllocate".to_string()),
            _=>None,
        }
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    use crate::spec::{FormalSpec,Constraint,Effect};
    #[test]
    fn test_clean_spec(){
        let s=FormalSpec::new("ok").with_ensures(Constraint::ResultNonNegative);
        assert!(QCCAnalyser::check("f",&s).is_clean());
    }
    #[test]
    fn test_nonneg_contradicts_negative_atleast(){
        let s=FormalSpec::new("bad").with_ensures(Constraint::ResultNonNegative).with_ensures(Constraint::ResultAtLeast(-5));
        let r=QCCAnalyser::check("f",&s);
        assert!(!r.is_clean());
    }
    #[test]
    fn test_empty_range(){
        let s=FormalSpec::new("bad").with_ensures(Constraint::ResultAtLeast(10)).with_ensures(Constraint::ResultAtMost(5));
        assert!(!QCCAnalyser::check("f",&s).is_clean());
    }
    #[test]
    fn test_two_equals_contradict(){
        let s=FormalSpec::new("bad").with_ensures(Constraint::ResultEquals(1)).with_ensures(Constraint::ResultEquals(2));
        assert!(!QCCAnalyser::check("f",&s).is_clean());
    }
    #[test]
    fn test_pure_contradicts_may_allocate(){
        let s=FormalSpec::new("bad").with_effect(Effect::Pure).with_effect(Effect::MayAllocate);
        assert!(!QCCAnalyser::check("f",&s).is_clean());
    }
    #[test]
    fn test_check_all_filters_clean(){
        let specs=vec![
            ("f".to_string(),FormalSpec::new("ok").with_ensures(Constraint::ResultNonNegative)),
            ("g".to_string(),FormalSpec::new("bad").with_ensures(Constraint::ResultEquals(1)).with_ensures(Constraint::ResultEquals(2))),
        ];
        let r=QCCAnalyser::check_all(&specs);
        assert_eq!(r.len(),1);
        assert_eq!(r[0].fn_name,"g");
    }
    #[test]
    fn test_contradiction_display(){
        let c=ContractContradiction{fn_name:"f".to_string(),constraint_a:Constraint::ResultNonNegative,constraint_b:Constraint::ResultAtLeast(-1),reason:"test".to_string()};
        assert!(c.to_string().contains("QCC"));
    }
}
