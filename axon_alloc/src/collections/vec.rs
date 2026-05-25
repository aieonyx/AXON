//! AxonVec<T> — growable heap-allocated sequence.
pub type AxonVec<T> = alloc::vec::Vec<T>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn vec_push_and_index() {
        let mut v: AxonVec<i64> = AxonVec::new();
        v.push(1); v.push(2); v.push(3);
        assert_eq!(v[0], 1); assert_eq!(v[2], 3); assert_eq!(v.len(), 3);
    }
    #[test] fn vec_pop() {
        let mut v: AxonVec<i64> = AxonVec::new();
        v.push(42); assert_eq!(v.pop(), Some(42)); assert!(v.is_empty());
    }
    #[test] fn vec_with_capacity() {
        let v: AxonVec<u8> = AxonVec::with_capacity(64);
        assert_eq!(v.len(), 0); assert!(v.capacity() >= 64);
    }
    #[test] fn vec_iter() {
        let v: AxonVec<i32> = alloc::vec![1,2,3,4,5];
        assert_eq!(v.iter().sum::<i32>(), 15);
    }
    #[test] fn vec_retain() {
        let mut v: AxonVec<i32> = alloc::vec![1,2,3,4,5,6];
        v.retain(|&x| x % 2 == 0);
        assert_eq!(v, alloc::vec![2,4,6]);
    }
    #[test] fn vec_dedup() {
        let mut v: AxonVec<i32> = alloc::vec![1,1,2,3,3,3,4];
        v.dedup(); assert_eq!(v, alloc::vec![1,2,3,4]);
    }
    #[test] fn vec_extend() {
        let mut v: AxonVec<i32> = AxonVec::new();
        v.extend([10,20,30]); assert_eq!(v.len(), 3); assert_eq!(v[1], 20);
    }
    #[test] fn vec_truncate() {
        let mut v: AxonVec<i32> = alloc::vec![1,2,3,4,5];
        v.truncate(3); assert_eq!(v, alloc::vec![1,2,3]);
    }
    #[test] fn vec_sort() {
        let mut v: AxonVec<i32> = alloc::vec![5,3,1,4,2];
        v.sort(); assert_eq!(v, alloc::vec![1,2,3,4,5]);
    }
    #[test] fn vec_contains() {
        let v: AxonVec<i32> = alloc::vec![10,20,30];
        assert!(v.contains(&20)); assert!(!v.contains(&99));
    }
}
