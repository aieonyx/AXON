//! AXON alloc prelude — `use axon_alloc::prelude::*`
pub use axon_core::prelude::*;
pub use crate::boxed::AxonBox;
pub use crate::collections::btree::AxonBTreeMap;
pub use crate::collections::map::AxonHashMap;
pub use crate::collections::string::AxonString;
pub use crate::collections::vec::AxonVec;
pub use crate::sync::{AxonArc, AxonRc};

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn prelude_imports_compile() {
        let _: AxonInt = 0;
        assert!(AxonResult::<i32>::Ok(1).is_ok());
        let mut v: AxonVec<i64> = AxonVec::new(); v.push(42); assert_eq!(v[0], 42);
        let mut m: AxonHashMap<&str,i64> = AxonHashMap::new(); m.insert("x",1); assert_eq!(m["x"],1);
        let mut bt: AxonBTreeMap<i32,i32> = AxonBTreeMap::new(); bt.insert(1,10); assert_eq!(bt[&1],10);
        let s: AxonString = AxonString::from("axon"); assert_eq!(s, "axon");
        let b: AxonBox<i64> = AxonBox::new(99); assert_eq!(*b, 99);
        let a = AxonArc::new(7_i64); assert_eq!(*a, 7);
        let r = AxonRc::new(3_i64); assert_eq!(*r, 3);
    }
}
