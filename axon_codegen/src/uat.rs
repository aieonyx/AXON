// axon_codegen::uat — Audited Dead Code Elimination. SPEC: 6B-04
// Dead code in AXON is never silently dropped.
// Every eliminated function produces an audit record.
// Security-critical paths require explicit retirement.

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum UATTier{
    /// Normal function — tagged then eliminated with audit log.
    Normal,
    /// @security_path — cannot be silently eliminated.
    SecurityPath,
    /// @audit_retain — never eliminated regardless of usage.
    AuditRetain,
}

#[derive(Debug,Clone)]
pub struct DeadCodeCandidate{
    pub fn_name:String,
    pub tier:UATTier,
    pub line:usize,
    pub has_retire_declaration:bool,
}

#[derive(Debug,Clone)]
pub struct UATLog{
    pub fn_name:String,
    pub tier:UATTier,
    pub action:UATAction,
    pub reason:String,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum UATAction{
    /// Function eliminated, audit record kept.
    Eliminated,
    /// Function retained by @audit_retain.
    Retained,
    /// Elimination blocked — @security_path requires @retire.
    Blocked,
}

#[derive(Debug,Clone)]
pub struct UATViolation{
    pub fn_name:String,
    pub message:String,
}

impl std::fmt::Display for UATViolation{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        write!(f,"UAT violation: fn {} — {}",self.fn_name,self.message)
    }
}

#[derive(Debug)]
pub struct UATReport{
    pub logs:Vec<UATLog>,
    pub violations:Vec<UATViolation>,
}

impl UATReport{
    pub fn is_clean(&self)->bool{self.violations.is_empty()}
    pub fn retained_count(&self)->usize{
        self.logs.iter().filter(|l|l.action==UATAction::Retained).count()
    }
    pub fn eliminated_count(&self)->usize{
        self.logs.iter().filter(|l|l.action==UATAction::Eliminated).count()
    }
}

pub struct UATAnalyser;

impl UATAnalyser{
    /// Scan source for dead code candidates and run the three-tier UAT policy.
    /// Phase 6: source-level call detection.
    /// PHASE7: full CFG-based dead code analysis.
    /// SPEC: 6B-04
    pub fn analyse(source:&str)->UATReport{
        let candidates=Self::find_dead_code(source);
        let mut logs=Vec::new();
        let mut violations=Vec::new();
        for c in candidates{
            match c.tier{
                UATTier::AuditRetain=>{
                    logs.push(UATLog{
                        fn_name:c.fn_name.clone(),
                        tier:UATTier::AuditRetain,
                        action:UATAction::Retained,
                        reason:"@audit_retain: function preserved regardless of usage".to_string(),
                    });
                }
                UATTier::SecurityPath=>{
                    if c.has_retire_declaration{
                        logs.push(UATLog{
                            fn_name:c.fn_name.clone(),
                            tier:UATTier::SecurityPath,
                            action:UATAction::Eliminated,
                            reason:"@retire declared — explicit retirement authorized".to_string(),
                        });
                    }else{
                        logs.push(UATLog{
                            fn_name:c.fn_name.clone(),
                            tier:UATTier::SecurityPath,
                            action:UATAction::Blocked,
                            reason:"@security_path requires explicit @retire declaration".to_string(),
                        });
                        violations.push(UATViolation{
                            fn_name:c.fn_name.clone(),
                            message:"@security_path function is unused but has no @retire declaration. Add @retire(reason: ...) to authorize elimination.".to_string(),
                        });
                    }
                }
                UATTier::Normal=>{
                    logs.push(UATLog{
                        fn_name:c.fn_name.clone(),
                        tier:UATTier::Normal,
                        action:UATAction::Eliminated,
                        reason:"dead code — not reachable from any call site".to_string(),
                    });
                }
            }
        }
        UATReport{logs,violations}
    }

    fn find_dead_code(source:&str)->Vec<DeadCodeCandidate>{
        let mut candidates=Vec::new();
        let lines:Vec<&str>=source.lines().collect();
        for (i,line) in lines.iter().enumerate(){
            let trimmed=line.trim();
            if trimmed.starts_with("fn ")||trimmed.starts_with("pub fn "){
                let fn_name=Self::extract_fn_name(trimmed);
                if fn_name.is_empty()||fn_name=="main"{continue;}
                let tier=Self::detect_tier(&lines,i);
                let call_count=source.matches(&format!("{}(",fn_name)).count();
                // A function is dead if it appears only in its own definition (count==1)
                let is_dead=call_count<=1;
                if is_dead{
                    let has_retire=i>0&&lines[..i].iter().rev().take(5)
                        .any(|l|l.trim().starts_with("@retire"));
                    candidates.push(DeadCodeCandidate{
                        fn_name,tier,line:i+1,has_retire_declaration:has_retire,
                    });
                }
            }
        }
        candidates
    }

    fn detect_tier(lines:&[&str],fn_line:usize)->UATTier{
        if fn_line==0{return UATTier::Normal;}
        for l in lines[..fn_line].iter().rev().take(5){
            let t=l.trim();
            if t=="@audit_retain"{return UATTier::AuditRetain;}
            if t=="@security_path"{return UATTier::SecurityPath;}
        }
        UATTier::Normal
    }

    fn extract_fn_name(line:&str)->String{
        let rest=line.trim_start_matches("pub ").trim_start_matches("fn ");
        rest.split('(').next().unwrap_or("").trim().to_string()
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_clean_source_no_violations(){
        let src="fn main(){greet();}
fn greet(){println!(\"hi\");}";
        let r=UATAnalyser::analyse(src);
        assert!(r.is_clean());
    }

    #[test]
    fn test_normal_dead_code_logged(){
        let src="fn main(){}
fn unused(){}";
        let r=UATAnalyser::analyse(src);
        assert!(r.logs.iter().any(|l|l.fn_name=="unused"&&l.action==UATAction::Eliminated));
    }

    #[test]
    fn test_security_path_without_retire_is_violation(){
        let src="fn main(){}
@security_path
fn verify_cap(){}";
        let r=UATAnalyser::analyse(src);
        assert!(!r.is_clean());
        assert!(r.violations.iter().any(|v|v.fn_name=="verify_cap"));
    }

    #[test]
    fn test_security_path_with_retire_is_clean(){
        let src="fn main(){}
@retire(reason: replaced)
@security_path
fn verify_cap(){}";
        let r=UATAnalyser::analyse(src);
        assert!(r.is_clean());
        assert!(r.logs.iter().any(|l|l.fn_name=="verify_cap"&&l.action==UATAction::Eliminated));
    }

    #[test]
    fn test_audit_retain_always_kept(){
        let src="fn main(){}
@audit_retain
fn critical_path(){}";
        let r=UATAnalyser::analyse(src);
        assert!(r.is_clean());
        assert!(r.logs.iter().any(|l|l.fn_name=="critical_path"&&l.action==UATAction::Retained));
    }

    #[test]
    fn test_retained_count(){
        let src="fn main(){}
@audit_retain
fn a(){}
@audit_retain
fn b(){}";
        let r=UATAnalyser::analyse(src);
        assert_eq!(r.retained_count(),2);
    }

    #[test]
    fn test_eliminated_count(){
        let src="fn main(){}
fn dead1(){}
fn dead2(){}";
        let r=UATAnalyser::analyse(src);
        assert_eq!(r.eliminated_count(),2);
    }

    #[test]
    fn test_violation_display(){
        let v=UATViolation{fn_name:"f".to_string(),message:"test".to_string()};
        assert!(v.to_string().contains("UAT violation"));
    }

    #[test]
    fn test_called_function_not_dead(){
        let src="fn main(){safe_fn();}
fn safe_fn(){}";
        let r=UATAnalyser::analyse(src);
        assert!(!r.logs.iter().any(|l|l.fn_name=="safe_fn"));
    }
}
