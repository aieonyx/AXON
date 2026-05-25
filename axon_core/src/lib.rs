#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(missing_docs)] // TODO: doc pass before crates.io publish

//! # axon_core
//! AXON language core primitives — `no_std`, zero dependencies.
//!
//! Import everything with `use axon_core::prelude::*`.

#[cfg(test)]
extern crate std;

pub mod error;
pub mod macros;
pub mod prelude;
pub mod result;
pub mod traits;
pub mod types;

pub use error::{AxonError, ErrorKind};
pub use result::AxonResult;
