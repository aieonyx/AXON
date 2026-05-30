// axon_ai::ocu — Observer Contract Units. SPEC: 6A-07
// Bio: Proprioceptors — observe and report, never intervene.
// Non-interventionist: OCUs record to WitnessStore, never block execution.

use std::collections::HashMap;

#[derive(Debug,Clone)]
pub struct ObserverUnit{
    pub observer_id:String,
    pub fn_name:String,
    pub observed_contracts:Vec<String>,
    pub active:bool,
}
impl ObserverUnit{
    pub fn new(id:&str,fn_name:&str,contracts:Vec<String>)->Self{
        Self{observer_id:id.to_string(),fn_name:fn_name.to_string(),observed_contracts:contracts,active:true}
    }
    pub fn deactivate(&mut self){self.active=false;}
    pub fn observes(&self,label:&str)->bool{
        self.active&&self.observed_contracts.iter().any(|c|c==label)
    }
}
#[derive(Debug,Clone)]
pub struct OCUEvent{
    pub observer_id:String,
    pub fn_name:String,
    pub contract_label:String,
    pub passed:bool,
    pub timestamp_ns:u64,
}
#[derive(Debug,Default)]
pub struct OCURegistry{
    units:HashMap<String,ObserverUnit>,
    events:Vec<OCUEvent>,
}
impl OCURegistry{
    pub fn new()->Self{Self::default()}
    pub fn register(&mut self,unit:ObserverUnit){
        self.units.insert(unit.observer_id.clone(),unit);
    }
    pub fn deactivate(&mut self,id:&str){
        if let Some(u)=self.units.get_mut(id){u.deactivate();}
    }
    pub fn record_event(&mut self,fn_name:&str,label:&str,passed:bool,ts:u64){
        for unit in self.units.values(){
            if unit.fn_name==fn_name&&unit.observes(label){
                self.events.push(OCUEvent{
                    observer_id:unit.observer_id.clone(),
                    fn_name:fn_name.to_string(),
                    contract_label:label.to_string(),
                    passed,timestamp_ns:ts,
                });
            }
        }
    }
    pub fn events(&self)->&[OCUEvent]{&self.events}
    pub fn events_for(&self,fn_name:&str)->Vec<&OCUEvent>{
        self.events.iter().filter(|e|e.fn_name==fn_name).collect()
    }
    pub fn failures(&self)->Vec<&OCUEvent>{
        self.events.iter().filter(|e|!e.passed).collect()
    }
    pub fn from_source(source:&str)->Self{
        let mut reg=Self::new();
        for line in source.lines(){
            let t=line.trim();
            if t.starts_with("@observes"){
                if let Some(inner)=t.strip_prefix("@observes(").and_then(|s|s.strip_suffix(")")){
                    let parts:Vec<&str>=inner.split(',').map(|s|s.trim()).collect();
                    let fn_name=parts.iter().find_map(|p|p.strip_prefix("fn:"))
                        .unwrap_or("").trim().trim_matches('"').to_string();
                    let contracts:Vec<String>=parts.iter()
                        .find_map(|p|p.strip_prefix("contracts:"))
                        .map(|s|s.trim().trim_matches('[').trim_matches(']').split(',')
                            .map(|c|c.trim().trim_matches('"').to_string()).collect())
                        .unwrap_or_default();
                    if !fn_name.is_empty(){
                        let id=format!("ocu-{}",fn_name);
                        reg.register(ObserverUnit::new(&id,&fn_name,contracts));
                    }
                }
            }
        }
        reg
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_register_and_observes(){
        let mut reg=OCURegistry::new();
        reg.register(ObserverUnit::new("o1","transfer",vec!["balance_sufficient".to_string()]));
        assert!(reg.units["o1"].observes("balance_sufficient"));
        assert!(!reg.units["o1"].observes("other"));
    }
    #[test]
    fn test_record_pass_event(){
        let mut reg=OCURegistry::new();
        reg.register(ObserverUnit::new("o1","f",vec!["c".to_string()]));
        reg.record_event("f","c",true,1000);
        assert_eq!(reg.events().len(),1);
        assert!(reg.events()[0].passed);
    }
    #[test]
    fn test_record_fail_event(){
        let mut reg=OCURegistry::new();
        reg.register(ObserverUnit::new("o1","f",vec!["c".to_string()]));
        reg.record_event("f","c",false,2000);
        assert_eq!(reg.failures().len(),1);
    }
    #[test]
    fn test_deactivate_stops_recording(){
        let mut reg=OCURegistry::new();
        reg.register(ObserverUnit::new("o1","f",vec!["c".to_string()]));
        reg.deactivate("o1");
        reg.record_event("f","c",false,1000);
        assert!(reg.events().is_empty());
    }
    #[test]
    fn test_events_for_fn(){
        let mut reg=OCURegistry::new();
        reg.register(ObserverUnit::new("o1","f",vec!["c1".to_string()]));
        reg.register(ObserverUnit::new("o2","g",vec!["c2".to_string()]));
        reg.record_event("f","c1",true,1);
        reg.record_event("g","c2",true,2);
        assert_eq!(reg.events_for("f").len(),1);
    }
    #[test]
    fn test_from_source(){
        let src="@observes(fn: transfer, contracts: [balance_sufficient])
fn transfer():
    pass
";
        let reg=OCURegistry::from_source(src);
        assert!(!reg.units.is_empty());
    }
    #[test]
    fn test_unobserved_contract_no_event(){
        let mut reg=OCURegistry::new();
        reg.register(ObserverUnit::new("o1","f",vec!["watched".to_string()]));
        reg.record_event("f","unwatched",false,1);
        assert!(reg.events().is_empty());
    }
}
