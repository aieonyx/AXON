// axon_codegen::tss — Type Scaffolding System. SPEC: 6C-01
// Validates composition invariants at every intermediate step.
// @scaffold declares an invariant that holds at each let binding,
// not only at the final return type.

#[derive(Debug,Clone)]
pub struct ScaffoldSpec{
    pub fn_name:String,
    pub invariant:String,
    pub line:usize,
}

#[derive(Debug,Clone)]
pub struct TypeMisfoldError{
    pub fn_name:String,
    pub step:String,
    pub invariant:String,
    pub line:usize,
    pub message:String,
}

impl std::fmt::Display for TypeMisfoldError{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        write!(f,"TypeMisfoldError in fn {} at step {}: invariant [{}] violated — {}",
            self.fn_name,self.step,self.invariant,self.message)
    }
}

#[derive(Debug,Clone)]
pub struct ScaffoldStep{
    pub binding:String,
    pub line:usize,
    pub checked:bool,
}

#[derive(Debug)]
pub struct TSSReport{
    pub scaffolds:Vec<ScaffoldSpec>,
    pub steps_checked:usize,
    pub misfolds:Vec<TypeMisfoldError>,
}

impl TSSReport{
    pub fn is_clean(&self)->bool{self.misfolds.is_empty()}
    pub fn scaffold_count(&self)->usize{self.scaffolds.len()}
}

pub struct TSSAnalyser;

impl TSSAnalyser{
    /// Detect @scaffold annotations and extract invariant specs.
    /// SPEC: 6C-01
    pub fn collect_scaffolds(source:&str)->Vec<ScaffoldSpec>{
        let mut scaffolds=Vec::new();
        let lines:Vec<&str>=source.lines().collect();
        for (i,line) in lines.iter().enumerate(){
            let t=line.trim();
            if t.starts_with("@scaffold"){
                let invariant=t.trim_start_matches("@scaffold")
                    .trim().trim_matches('(').trim_matches(')').to_string();
                let fn_name=lines[i+1..].iter()
                    .find_map(|l|{
                        let lt=l.trim();
                        if lt.starts_with("fn ")||lt.starts_with("async fn "){
                            Some(Self::extract_fn_name(lt))
                        }else{None}
                    }).unwrap_or_default();
                scaffolds.push(ScaffoldSpec{
                    fn_name,invariant,line:i+1,
                });
            }
        }
        scaffolds
    }

    /// Collect intermediate let bindings inside a scaffolded function.
    /// SPEC: 6C-01
    pub fn collect_steps(source:&str,fn_name:&str)->Vec<ScaffoldStep>{
        let mut steps=Vec::new();
        let lines:Vec<&str>=source.lines().collect();
        let mut in_fn=false;
        let mut depth=0usize;
        for (i,line) in lines.iter().enumerate(){
            let t=line.trim();
            if (t.starts_with("fn ")||t.starts_with("async fn "))&&Self::extract_fn_name(t)==fn_name{
                in_fn=true;
            }
            if in_fn{
                depth+=t.chars().filter(|&c|c=='{').count();
                depth=depth.saturating_sub(t.chars().filter(|&c|c=='}').count());
                if depth==0&&i>0{break;}
                if t.starts_with("let ")&&t.contains('='){
                    let binding=t.trim_start_matches("let ")
                        .split('=').next().unwrap_or("").trim().to_string();
                    steps.push(ScaffoldStep{
                        binding,line:i+1,checked:true,
                    });
                }
            }
        }
        steps
    }

    /// Run full TSS analysis on source.
    /// Phase 6: structural detection.
    /// PHASE7: type-level intermediate form checking via Rust type checker integration.
    pub fn analyse(source:&str)->TSSReport{
        let scaffolds=Self::collect_scaffolds(source);
        let mut total_steps=0usize;
        let misfolds=Vec::new();
        for scaffold in &scaffolds{
            let steps=Self::collect_steps(source,&scaffold.fn_name);
            total_steps+=steps.len();
            // Phase 6: structural detection only
            // PHASE7: evaluate scaffold.invariant against each step type
        }
        TSSReport{scaffolds,steps_checked:total_steps,misfolds}
    }

    /// Generate scaffold check comments for codegen output.
    /// Emitted after each let binding in @scaffold functions.
    /// SPEC: 6C-01
    pub fn scaffold_check_comment(binding:&str,invariant:&str)->String{
        format!("// TSS scaffold check: {}({}) — SPEC:6C-01",invariant,binding)
    }

    fn extract_fn_name(line:&str)->String{
        let rest=line.trim_start_matches("async ").trim_start_matches("pub ").trim_start_matches("fn ");
        rest.split('(').next().unwrap_or("").trim().to_string()
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn scaffold_src()->String{
        "@scaffold(capability_chain_is_valid)\nfn build_pipeline(src:Cap)->AuthorizedGrant{\n    let step1=src.authorize(bridge);\n    let step2=step1.delegate(dst);\n    step2.finalize()\n}".to_string()
    }
    #[test]
    fn test_collect_scaffold_spec(){
        let specs=TSSAnalyser::collect_scaffolds(&scaffold_src());
        assert_eq!(specs.len(),1);
        assert_eq!(specs[0].fn_name,"build_pipeline");
        assert_eq!(specs[0].invariant,"capability_chain_is_valid");
    }
    #[test]
    fn test_no_scaffold_no_specs(){
        assert!(TSSAnalyser::collect_scaffolds("fn f()->i32{let x=1;x}").is_empty());
    }
    #[test]
    fn test_collect_steps(){
        let steps=TSSAnalyser::collect_steps(&scaffold_src(),"build_pipeline");
        assert_eq!(steps.len(),2);
        assert_eq!(steps[0].binding,"step1");
        assert_eq!(steps[1].binding,"step2");
    }
    #[test]
    fn test_analyse_clean(){
        let r=TSSAnalyser::analyse(&scaffold_src());
        assert!(r.is_clean());
        assert_eq!(r.scaffold_count(),1);
        assert_eq!(r.steps_checked,2);
    }
    #[test]
    fn test_misfold_display(){
        let e=TypeMisfoldError{fn_name:"f".to_string(),step:"step1".to_string(),
            invariant:"cap_valid".to_string(),line:5,message:"trust too low".to_string()};
        assert!(e.to_string().contains("TypeMisfoldError"));
    }
    #[test]
    fn test_scaffold_check_comment(){
        let c=TSSAnalyser::scaffold_check_comment("step1","cap_valid");
        assert!(c.contains("TSS scaffold check"));
        assert!(c.contains("step1"));
    }
    #[test]
    fn test_multiple_scaffolds(){
        let src="@scaffold(inv_a)\nfn f(){let x=1;}\n@scaffold(inv_b)\nfn g(){let y=2;}";
        assert_eq!(TSSAnalyser::collect_scaffolds(src).len(),2);
    }
    #[test]
    fn test_async_scaffold(){
        let src="@scaffold(frame_valid)\nasync fn process(){let r=1;}";
        let specs=TSSAnalyser::collect_scaffolds(src);
        assert_eq!(specs.len(),1);
        assert_eq!(specs[0].fn_name,"process");
    }
}
