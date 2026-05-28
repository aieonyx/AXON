// axon_codegen::dwc — DWC lowering pass. SPEC: 6A-01
// Phase 6: pre inline, post at fn end. PHASE7: PostGuard RAII.
use axon_parser::ast::{ContractClauseKind,Expr,FnDecl};
use crate::generator::CodeGen;

fn cid(fn_name:&str,label:&str)->u64{
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash,Hasher};
    let mut h=DefaultHasher::new();
    fn_name.hash(&mut h); label.hash(&mut h); h.finish()
}

fn collect_olds<'a>(expr:&'a Expr,out:&mut Vec<&'a Expr>){
    match expr{
        Expr::Call(c)=>{
            if c.callee.name=="old"{
                if let Some(a)=c.args.first(){out.push(&a.value);}
            }else{
                for a in &c.args{collect_olds(&a.value,out);}
            }
        }
        Expr::BinOp(b)=>{collect_olds(&b.lhs,out);collect_olds(&b.rhs,out);}
        Expr::UnaryOp(u)=>collect_olds(&u.expr,out),
        Expr::MethodCall(m)=>{collect_olds(&m.receiver,out);for a in &m.args{collect_olds(&a.value,out);}}
        Expr::FieldAccess(fa)=>collect_olds(&fa.object,out),
        Expr::Index(i)=>{collect_olds(&i.object,out);collect_olds(&i.index,out);}
        _=>{}
    }
}

fn rewrite_olds(s:String,olds:&[(usize,String)])->String{
    let mut r=s;
    for (i,e) in olds{
        r=r.replace(&format!("old ( {} )",e),&format!("__dwc_old_{}",i));
        r=r.replace(&format!("old({})",e),&format!("__dwc_old_{}",i));
    }
    r
}

impl CodeGen{
    pub fn has_contracts(f:&FnDecl)->bool{!f.contracts.is_empty()}

    fn dwc_pre(&mut self,fn_name:&str,label:&str,pred:&str){
        let h=cid(fn_name,label);
        self.line(&format!("// SPEC:6A-01 pre[{}]",label));
        self.line(&format!("let __dwc_pre_pass={};"  ,pred));
        self.line("axon_rt::store().record(axon_rt::WitnessRecord{");
        self.indent();
        self.line(&format!("contract_id:axon_rt::ContractId::from_hash(0x{:016X}u64),",h));
        self.line("kind:axon_rt::WitnessKind::Pre,");
        self.line("verdict:if __dwc_pre_pass{axon_rt::Verdict::Pass}else{");
        self.indent();
        self.line("axon_rt::Verdict::Fail(axon_rt::WitnessPayload{");
        self.indent();
        self.line(&format!("predicate_src:\"{}\"  ,",label));
        self.line("snapshot:None,");
        self.line("source_loc:axon_rt::SourceLocation{file:file!(),line:line!(),column:column!()}})");
        self.dedent();
        self.line("source_loc:axon_rt::SourceLocation{file:file!(),line:line!(),column:column!()}})");
        self.dedent();
        self.line("},");
        self.line("call_site:axon_rt::SourceLocation{file:file!(),line:line!(),column:column!()},");
        self.line("timestamp:axon_rt::monotonic_ns(),");
        self.dedent();
        self.line("}));");
    }

    fn dwc_post(&mut self,fn_name:&str,label:&str,pred:&str){
        let h=cid(fn_name,label);
        self.line(&format!("// SPEC:6A-01 post[{}] PHASE7:PostGuard",label));
        self.line(&format!("let __dwc_post_pass={};",pred));
        self.line("axon_rt::store().record(axon_rt::WitnessRecord{");
        self.indent();
        self.line(&format!("contract_id:axon_rt::ContractId::from_hash(0x{:016X}u64),",h));
        self.line("kind:axon_rt::WitnessKind::Post,");
        self.line("verdict:if __dwc_post_pass{axon_rt::Verdict::Pass}else{");
        self.indent();
        self.line("axon_rt::Verdict::Fail(axon_rt::WitnessPayload{");
        self.indent();
        self.line(&format!("predicate_src:\"{}\"  ,",label));
        self.line("snapshot:None,");
        self.line("source_loc:axon_rt::SourceLocation{file:file!(),line:line!(),column:column!()}})");
        self.dedent();
        self.line("source_loc:axon_rt::SourceLocation{file:file!(),line:line!(),column:column!()}})");
        self.dedent();
        self.line("},");
        self.line("call_site:axon_rt::SourceLocation{file:file!(),line:line!(),column:column!()},");
        self.line("timestamp:axon_rt::monotonic_ns(),");
        self.dedent();
        self.line("}));");
    }

    pub fn emit_fn_with_dwc(&mut self,f:&FnDecl){
        let fn_name=f.name.name.clone();
        let params=self.emit_params(&f.params);
        let ret=f.ret_type.as_ref().map(|t|format!(" -> {}",crate::types::rust_type(t))).unwrap_or_default();
        self.line("// @contract SPEC:6A-01");
        self.line(&format!("pub fn {}({}){}{{",fn_name,params,ret));
        self.indent();
        let mut olds:Vec<(usize,String)>=Vec::new();
        for c in f.contracts.iter().filter(|c|c.kind==ContractClauseKind::Post){
            let mut refs=Vec::new();
            collect_olds(&c.predicate,&mut refs);
            for r in refs{
                let s=self.emit_expr_str(r);
                if !olds.iter().any(|(_,e)|e==&s){
                    let idx=olds.len();
                    olds.push((idx,s));
                }
            }
        }
        for (i,e) in &olds{
            self.line(&format!("let __dwc_old_{}={}.clone();",i,e));
        }
        for c in f.contracts.iter().filter(|c|c.kind==ContractClauseKind::Pre){
            let pred=self.emit_expr_str(&c.predicate);
            self.dwc_pre(&fn_name,&c.label.name,&pred);
        }
        self.emit_block(&f.body);
        for c in f.contracts.iter().filter(|c|c.kind==ContractClauseKind::Post){
            let raw=self.emit_expr_str(&c.predicate);
            let pred=rewrite_olds(raw,&olds);
            self.dwc_post(&fn_name,&c.label.name,&pred);
        }
        self.dedent();
        self.line("}");
    }
}
