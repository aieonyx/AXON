use axon_core::prelude::*;
use axon_core::types::U32;
pub trait PalProcess {
    fn args() -> AxonResult<&'static [&'static str]>;
    fn env_var(key: &str) -> AxonResult<&'static str>;
    fn exit(code: U32) -> !;
    fn pid() -> AxonResult<U32>;
}
