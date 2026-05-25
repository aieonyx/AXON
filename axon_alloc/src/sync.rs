//! AxonArc<T> and AxonRc<T> — reference-counted heap values.
pub type AxonArc<T> = alloc::sync::Arc<T>;
pub type AxonRc<T>  = alloc::rc::Rc<T>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn arc_clone_and_deref() {
        let a = AxonArc::new(42_i64); let b = AxonArc::clone(&a);
        assert_eq!(*a, 42); assert_eq!(*b, 42); assert_eq!(AxonArc::strong_count(&a), 2);
    }
    #[test] fn arc_drops_correctly() {
        let a = AxonArc::new(99_i64);
        { let _b = AxonArc::clone(&a); assert_eq!(AxonArc::strong_count(&a), 2); }
        assert_eq!(AxonArc::strong_count(&a), 1);
    }
    #[test] fn rc_clone_and_deref() {
        let r = AxonRc::new("sovereign"); let r2 = AxonRc::clone(&r);
        assert_eq!(*r, "sovereign"); assert_eq!(AxonRc::strong_count(&r), 2);
        drop(r2); assert_eq!(AxonRc::strong_count(&r), 1);
    }
    #[test] fn arc_shared_state() {
        let shared: AxonArc<alloc::vec::Vec<i32>> = AxonArc::new(alloc::vec![1,2,3]);
        let clone = AxonArc::clone(&shared);
        assert_eq!(shared.len(), clone.len()); assert_eq!(shared[0], clone[0]);
    }
}
