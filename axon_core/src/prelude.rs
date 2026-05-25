//! AXON core prelude — `use axon_core::prelude::*`
pub use crate::error::{AxonError, ErrorKind};
pub use crate::result::AxonResult;
pub use crate::traits::convert::{AxonFrom, AxonInto};
pub use crate::traits::display::AxonDisplay;
pub use crate::traits::hash::{AxonHash, AxonHasher, FnvHasher};
pub use crate::types::{AxonBool,AxonByte,AxonFloat,AxonInt,AxonUint,F32,F64,I8,I16,I32,I64,I128,U8,U16,U32,U64,U128,Isize,Usize};
pub use crate::{axon_assert, axon_todo, axon_try, axon_unreachable};
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn prelude_imports_compile() {
        let _:AxonInt=0; let _:AxonFloat=0.0;
        let ok:AxonResult<i32>=AxonResult::Ok(1); assert!(ok.is_ok());
        let err:AxonResult<i32>=AxonResult::Err(AxonError::not_found("x")); assert!(err.is_err());
        fn nd<T:AxonDisplay>(_:T){} nd(42_i32);
        let mut hh=FnvHasher::new(); 42_u64.axon_hash(&mut hh); let _=hh.finish();
        axon_assert!(1+1==2);
    }
}
