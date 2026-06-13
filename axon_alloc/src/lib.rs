// Copyright (c) 2026 Edison Lepitel / AIEONYX
#![no_std]
#![allow(missing_docs)]

extern crate alloc;
pub mod allocator;
pub mod heap;
pub mod slab;

pub mod boxed;
pub mod collections;
pub mod prelude;
pub mod sync;

pub use boxed::AxonBox;
pub use collections::btree::AxonBTreeMap;
pub use collections::map::AxonHashMap;
pub use collections::string::AxonString;
pub use collections::vec::AxonVec;
pub use sync::{AxonArc, AxonRc};
pub use allocator::{SovereignAllocator, AllocStats};
pub use heap::{SovereignHeap, HostHeap};
pub use slab::SlabPool;
