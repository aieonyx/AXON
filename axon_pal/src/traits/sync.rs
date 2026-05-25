use axon_core::prelude::*;
use crate::types::RawHandle;
pub trait PalSync {
    fn mutex_new() -> AxonResult<RawHandle>;
    fn mutex_lock(handle: RawHandle) -> AxonResult<()>;
    fn mutex_unlock(handle: RawHandle) -> AxonResult<()>;
    fn mutex_destroy(handle: RawHandle) -> AxonResult<()>;
    fn thread_spawn(f: fn()) -> AxonResult<RawHandle>;
    fn thread_join(handle: RawHandle) -> AxonResult<()>;
    fn thread_yield();
}
