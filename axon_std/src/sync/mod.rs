// Copyright (c) 2026 Edison Lepinet / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// sync/mod.rs -- Sovereign synchronisation primitives.
// Channel<T>: bounded MPSC, maps to seL4 IPC endpoints.
// SovereignMutex<T>: auditable mutual exclusion.
// SovereignOnce: one-time init barrier for capability registration.

pub mod channel;
pub mod mutex;
pub mod once;

pub use channel::{channel, Sender, Receiver, ChannelError};
pub use mutex::{SovereignMutex, MutexError};
pub use once::SovereignOnce;
