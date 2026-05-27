// Copyright 2026 Edison Lepiten. SPEC: 6A-01 DWC
use crossbeam_queue::ArrayQueue;
use std::sync::OnceLock;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub struct ContractId(pub u64);
impl ContractId {
    pub const fn from_hash(h:u64)->Self{Self(h)}
    pub const TRIVIAL:Self=Self(0);
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum WitnessKind{Pre,Post,Invariant,Refine}

#[derive(Debug,Clone,Copy)]
pub struct SourceLocation{
    pub file:&'static str,pub line:u32,pub column:u32,
}
pub struct WitnessPayload{
    pub predicate_src:&'static str,
    pub snapshot:Option<Box<dyn erased_serde::Serialize+Send>>,
    pub source_loc:SourceLocation,
}
impl std::fmt::Debug for WitnessPayload{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        f.debug_struct("WitnessPayload")
            .field("predicate_src",&self.predicate_src)
            .field("snapshot",&self.snapshot.as_ref().map(|_|"<captured>"))
            .field("source_loc",&self.source_loc).finish()
    }
}

#[derive(Debug)]
pub enum Verdict{Pass,Fail(WitnessPayload),Panic{message:&'static str}}
impl Verdict{
    pub fn is_pass(&self)->bool{matches!(self,Verdict::Pass)}
    pub fn is_fail(&self)->bool{!self.is_pass()}
}
#[derive(Debug)]
pub struct WitnessRecord{
    pub contract_id:ContractId,
    pub kind:WitnessKind,
    pub verdict:Verdict,
    pub call_site:SourceLocation,
    pub timestamp:u64,
}
impl WitnessRecord{
    pub const fn trivial()->Self{Self{
        contract_id:ContractId::TRIVIAL,
        kind:WitnessKind::Pre,
        verdict:Verdict::Pass,
        call_site:SourceLocation{file:"",line:0,column:0},
        timestamp:0,}}
    pub fn is_trivial(&self)->bool{
        self.contract_id==ContractId::TRIVIAL&&self.timestamp==0}
    pub fn is_pass(&self)->bool{self.verdict.is_pass()}
    pub fn is_fail(&self)->bool{self.verdict.is_fail()}
}
#[derive(Debug)]
pub struct ContractViolation{pub record:WitnessRecord}
impl std::fmt::Display for ContractViolation{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        write!(f,"contract violation [{:?} @ {:?}] at {}:{}:{}",
            self.record.kind,self.record.contract_id,
            self.record.call_site.file,
            self.record.call_site.line,
            self.record.call_site.column)
    }
}
impl std::error::Error for ContractViolation{}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum OverflowPolicy{DropOldest,DropNewest,FlushToAegis}
pub struct WitnessStore{
    ring:ArrayQueue<WitnessRecord>,// PHASE7: seqlock ring
    overflow_policy:OverflowPolicy,
}
impl WitnessStore{
    pub fn new(policy:OverflowPolicy)->Self{
        Self{ring:ArrayQueue::new(4096),overflow_policy:policy}}
    pub fn record(&self,w:WitnessRecord){
        match self.ring.push(w){
            Ok(())=>{}
            Err(w)=>match self.overflow_policy{
                OverflowPolicy::DropNewest=>{}
                OverflowPolicy::DropOldest|OverflowPolicy::FlushToAegis=>{
                    let _=self.ring.pop();
                    let _=self.ring.push(w);
                }
            },
        }
    }
    pub fn drain_failures(&self)->Vec<WitnessRecord>{
        let mut out=Vec::new();
        while let Some(r)=self.ring.pop(){if r.is_fail(){out.push(r);}}
        out}
    pub fn snapshot(&self)->Vec<WitnessRecord>{
        let mut out=Vec::new();
        while let Some(r)=self.ring.pop(){out.push(r);}
        out}
    pub fn clear(&self){while self.ring.pop().is_some(){}}
    pub fn len(&self)->usize{self.ring.len()}
    pub fn is_empty(&self)->bool{self.ring.is_empty()}
}
static WITNESS_STORE:OnceLock<WitnessStore>=OnceLock::new();
pub fn store()->&'static WitnessStore{
    WITNESS_STORE.get_or_init(||WitnessStore::new(OverflowPolicy::DropOldest))}

#[cfg(test)]
mod tests{
    use super::*;
    fn pr()->WitnessRecord{WitnessRecord{
        contract_id:ContractId::from_hash(1),kind:WitnessKind::Pre,
        verdict:Verdict::Pass,timestamp:1,
        call_site:SourceLocation{file:"t",line:1,column:1}}}
    fn fr()->WitnessRecord{WitnessRecord{
        contract_id:ContractId::from_hash(2),kind:WitnessKind::Pre,
        verdict:Verdict::Fail(WitnessPayload{
            predicate_src:"x>0",snapshot:None,
            source_loc:SourceLocation{file:"t",line:1,column:1}}),
        call_site:SourceLocation{file:"t",line:2,column:1},timestamp:2}}
    #[test] fn pass_not_in_drain(){
        let s=WitnessStore::new(OverflowPolicy::DropOldest);
        s.record(pr());assert!(s.drain_failures().is_empty());}
    #[test] fn fail_in_drain(){
        let s=WitnessStore::new(OverflowPolicy::DropOldest);
        s.record(fr());assert_eq!(s.drain_failures().len(),1);}
    #[test] fn overflow_oldest(){
        let s=WitnessStore::new(OverflowPolicy::DropOldest);
        for _ in 0..4096{s.record(pr());} s.record(fr());}
    #[test] fn overflow_newest(){
        let s=WitnessStore::new(OverflowPolicy::DropNewest);
        for _ in 0..4096{s.record(pr());} s.record(fr());
        assert!(s.drain_failures().is_empty());}
    #[test] fn trivial(){
        let r=WitnessRecord::trivial();
        assert!(r.is_trivial()&&r.is_pass());}
    #[test] fn violation_display(){
        let v=ContractViolation{record:fr()};
        assert!(v.to_string().contains("contract violation"));}
}
