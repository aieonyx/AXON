// axon_parser::tvt — Temporal Value Type analyser. SPEC: 6C-04
// Detects @temporal annotations and Expired<T> violations at analysis time.

#[derive(Debug,Clone)]
pub struct TemporalSpec{
    pub symbol:String,
    pub type_name:String,
    pub duration_ms:u64,
    pub line:usize,
}

#[derive(Debug,Clone)]
pub struct TemporalViolation{
    pub symbol:String,
    pub message:String,
    pub line:usize,
}

impl std::fmt::Display for TemporalViolation{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        write!(f,"TVT violation at line {}: {} — {}",self.line,self.symbol,self.message)
    }
}

#[derive(Debug)]
pub struct TVTReport{
    pub specs:Vec<TemporalSpec>,
    pub violations:Vec<TemporalViolation>,
}

impl TVTReport{
    pub fn is_clean(&self)->bool{self.violations.is_empty()}
    pub fn temporal_count(&self)->usize{self.specs.len()}
}

pub struct TVTAnalyser;

impl TVTAnalyser{
    /// Analyse source for @temporal annotations and Expired<T> usage.
    /// SPEC: 6C-04
    pub fn analyse(source:&str)->TVTReport{
        let specs=Self::collect_specs(source);
        let violations=Self::detect_violations(source);
        TVTReport{specs,violations}
    }

    fn collect_specs(source:&str)->Vec<TemporalSpec>{
        let mut specs=Vec::new();
        let lines:Vec<&str>=source.lines().collect();
        for (i,line) in lines.iter().enumerate(){
            let t=line.trim();
            if t.starts_with("@temporal"){
                let inner=t.trim_start_matches("@temporal")
                    .trim().trim_matches('(').trim_matches(')');
                let duration_ms=inner.split("ms").next()
                    .and_then(|s|s.trim().parse::<u64>().ok()).unwrap_or(0);
                if let Some(next)=lines.get(i+1){
                    let nt=next.trim();
                    let symbol=Self::extract_symbol(nt);
                    let type_name=Self::extract_type(nt);
                    specs.push(TemporalSpec{symbol,type_name,duration_ms,line:i+1});
                }
            }
        }
        specs
    }

    fn detect_violations(source:&str)->Vec<TemporalViolation>{
        let mut violations=Vec::new();
        for (i,line) in source.lines().enumerate(){
            if line.contains("Expired<"){
                let sym=line.trim().split(':').next().unwrap_or("").trim();
                violations.push(TemporalViolation{
                    symbol:sym.to_string(),
                    message:"Expired<T> value used — all operations on expired temporal values are blocked".to_string(),
                    line:i+1,
                });
            }
        }
        violations
    }

    fn extract_symbol(line:&str)->String{
        let rest=line.trim_start_matches("pub ").trim_start_matches("let ");
        rest.split([':','=']).next().unwrap_or("").trim().to_string()
    }

    fn extract_type(line:&str)->String{
        if let Some(after_colon)=line.split(':').nth(1){
            after_colon.split('=').next().unwrap_or("").trim().to_string()
        }else{"".to_string()}
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_collect_temporal_spec(){
        let s="@temporal(5000ms)\nlet token:AuthToken=generate();";
        let r=TVTAnalyser::analyse(s);
        assert_eq!(r.temporal_count(),1);
        assert_eq!(r.specs[0].duration_ms,5000);
        assert!(r.is_clean());
    }

    #[test]
    fn test_no_temporal_no_specs(){
        let r=TVTAnalyser::analyse("fn f(){let x=1;}");
        assert_eq!(r.temporal_count(),0);
        assert!(r.is_clean());
    }

    #[test]
    fn test_expired_usage_is_violation(){
        let s="let result=use_expired(Expired<Token>);";
        let r=TVTAnalyser::analyse(s);
        assert!(!r.is_clean());
        assert_eq!(r.violations.len(),1);
    }

    #[test]
    fn test_violation_display(){
        let v=TemporalViolation{symbol:"tok".to_string(),
            message:"expired".to_string(),line:5};
        assert!(v.to_string().contains("TVT violation"));
        assert!(v.to_string().contains("line 5"));
    }

    #[test]
    fn test_multiple_temporal_specs(){
        let s="@temporal(1000ms)\nlet a:AuthToken=t1();\n@temporal(2000ms)\nlet b:SessionKey=t2();";
        let r=TVTAnalyser::analyse(s);
        assert_eq!(r.temporal_count(),2);
    }

    #[test]
    fn test_zero_duration_spec(){
        let s="@temporal(0ms)\nlet x:Token=t();";
        let r=TVTAnalyser::analyse(s);
        assert_eq!(r.specs[0].duration_ms,0);
    }
}
