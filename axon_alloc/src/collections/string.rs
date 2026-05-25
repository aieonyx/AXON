//! AxonString — owned heap-allocated UTF-8 string.
pub type AxonString = alloc::string::String;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    #[test] fn string_push_str() {
        let mut s = AxonString::from("Hello");
        s.push_str(", AXON!"); assert_eq!(s, "Hello, AXON!");
    }
    #[test] fn string_len_and_empty() {
        let s = AxonString::new(); assert!(s.is_empty()); assert_eq!(s.len(), 0);
    }
    #[test] fn string_from_int() { assert_eq!(42_i64.to_string(), "42"); }
    #[test] fn string_contains() {
        let s = AxonString::from("sovereign digital");
        assert!(s.contains("digital")); assert!(!s.contains("cloud"));
    }
    #[test] fn string_to_uppercase() {
        assert_eq!(AxonString::from("axon").to_uppercase(), "AXON");
    }
    #[test] fn string_trim() {
        assert_eq!(AxonString::from("  axon  ").trim(), "axon");
    }
    #[test] fn string_split_collect() {
        let owned = AxonString::from("a,b,c");
        let parts: alloc::vec::Vec<&str> = owned.split(',').collect();
        assert_eq!(parts, alloc::vec!["a", "b", "c"]);
    }
    #[test] fn string_replace() {
        assert_eq!(AxonString::from("hello world").replace("world", "AXON"), "hello AXON");
    }
}
