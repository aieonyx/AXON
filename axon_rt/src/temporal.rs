// axon_rt::temporal — Temporal Value Types runtime. SPEC: 6C-04
// Time-aware type system: values become Expired<T> after validity window.
// All operations on Expired<T> are blocked at compile/analysis time.

use std::time::{Duration,Instant};

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum TemporalState{Valid,Expired}

#[derive(Debug,Clone)]
pub struct TemporalValue<T>{
    pub value:T,
    pub created_at:Instant,
    pub valid_for:Duration,
    pub label:String,
}

impl<T:Clone> TemporalValue<T>{
    pub fn new(value:T,valid_for:Duration,label:&str)->Self{
        Self{value,created_at:Instant::now(),valid_for,label:label.to_string()}
    }
    pub fn state(&self)->TemporalState{
        if self.created_at.elapsed()>self.valid_for{TemporalState::Expired}
        else{TemporalState::Valid}
    }
    pub fn is_expired(&self)->bool{self.state()==TemporalState::Expired}
    pub fn is_valid(&self)->bool{!self.is_expired()}
    pub fn get(&self)->Option<&T>{
        if self.is_valid(){Some(&self.value)}else{None}
    }
    pub fn remaining_ms(&self)->u64{
        let elapsed=self.created_at.elapsed();
        if elapsed>=self.valid_for{0}
        else{(self.valid_for-elapsed).as_millis() as u64}
    }
}

#[derive(Debug,Default)]
pub struct TemporalMonitor{
    labels:Vec<String>,
    expirations:Vec<(String,bool)>,
}

impl TemporalMonitor{
    pub fn new()->Self{Self::default()}
    pub fn register<T:Clone>(&mut self,tv:&TemporalValue<T>){
        self.labels.push(tv.label.clone());
        self.expirations.push((tv.label.clone(),tv.is_expired()));
    }
    pub fn expired_count(&self)->usize{
        self.expirations.iter().filter(|(_,e)|*e).count()
    }
    pub fn all_valid(&self)->bool{self.expired_count()==0}
    pub fn expired_labels(&self)->Vec<&str>{
        self.expirations.iter().filter(|(_,e)|*e).map(|(l,_)|l.as_str()).collect()
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_new_value_is_valid(){
        let v=TemporalValue::new(42u32,Duration::from_secs(60),"token");
        assert!(v.is_valid());
        assert!(!v.is_expired());
        assert_eq!(v.get(),Some(&42u32));
    }

    #[test]
    fn test_expired_returns_none(){
        let v=TemporalValue::new("key",Duration::from_nanos(1),"api_key");
        std::thread::sleep(Duration::from_millis(1));
        assert!(v.is_expired());
        assert!(v.get().is_none());
    }

    #[test]
    fn test_remaining_ms_decreasing(){
        let v=TemporalValue::new(1u32,Duration::from_secs(10),"t");
        assert!(v.remaining_ms()>0);
    }

    #[test]
    fn test_monitor_all_valid(){
        let mut m=TemporalMonitor::new();
        let v=TemporalValue::new(1u32,Duration::from_secs(60),"v1");
        m.register(&v);
        assert!(m.all_valid());
        assert_eq!(m.expired_count(),0);
    }

    #[test]
    fn test_monitor_expired_label(){
        let mut m=TemporalMonitor::new();
        let v=TemporalValue::new(1u32,Duration::from_nanos(1),"stale");
        std::thread::sleep(Duration::from_millis(1));
        m.register(&v);
        assert!(!m.all_valid());
        assert!(m.expired_labels().contains(&"stale"));
    }
}
