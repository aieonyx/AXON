//! AxonBox<T> — heap-allocated single value with unique ownership.
pub type AxonBox<T> = alloc::boxed::Box<T>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn box_allocates_and_deref() {
        let b: AxonBox<i64> = AxonBox::new(42); assert_eq!(*b, 42);
    }
    #[test] fn box_into_inner() {
        let b: AxonBox<i64> = AxonBox::new(99); let v = *b; assert_eq!(v, 99);
    }
    #[test] fn box_recursive_type() {
        enum List { Nil, Cons(i32, AxonBox<List>) }
        let list = List::Cons(1, AxonBox::new(List::Cons(2, AxonBox::new(List::Nil))));
        match list { List::Cons(v,_) => assert_eq!(v,1), List::Nil => panic!() }
    }
    #[test] fn box_trait_object() {
        trait Speak { fn speak(&self) -> &'static str; }
        struct Dog;
        impl Speak for Dog { fn speak(&self) -> &'static str { "woof" } }
        let b: AxonBox<dyn Speak> = AxonBox::new(Dog);
        assert_eq!(b.speak(), "woof");
    }
}
