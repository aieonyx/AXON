//! Minimal seL4 ABI layer — sovereign, zero external dependencies.
//!
//! AXON owns its seL4 bindings. Only the subset needed by the PAL
//! is defined here. The seL4 ABI is formally specified and stable.
//!
//! Reference: seL4 Reference Manual (NICTA/Data61/seL4 Foundation)
//! ABI: aarch64 — syscall via `svc #0`, registers x0-x7.

pub mod ipc;
pub mod syscall;

// ── Capability pointers ───────────────────────────────────────────────────────

/// A seL4 capability pointer (slot index in a CNode).
/// The kernel validates all capabilities — no raw pointer arithmetic needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CPtr(pub usize);

impl CPtr {
    /// The null capability — always invalid.
    pub const NULL: CPtr = CPtr(0);
    /// seL4 initial capability: root CNode.
    pub const INIT_CNODE: CPtr = CPtr(1);
    /// seL4 initial capability: TCB of root task.
    pub const INIT_TCB: CPtr = CPtr(2);
    /// seL4 initial capability: root VSpace (page table root).
    pub const INIT_VSPACE: CPtr = CPtr(3);
    /// seL4 initial capability: IRQ control.
    pub const IRQ_CTRL: CPtr = CPtr(4);
    /// seL4 initial capability: ASID control.
    pub const ASID_CTRL: CPtr = CPtr(5);
}

// ── Message registers ─────────────────────────────────────────────────────────

/// Number of message registers in a seL4 IPC message.
pub const SEL4_MSG_MAX_LENGTH: usize = 120;

/// seL4 MessageInfo — encodes length, caps, and extra caps in a single word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageInfo(pub usize);

impl MessageInfo {
    /// Construct a MessageInfo with the given length and zero caps.
    #[inline]
    pub const fn new(length: usize) -> Self {
        // Bits [6:0] = length, bits [9:7] = extra_caps, bits [11:10] = caps_unwrapped
        Self(length & 0x7F)
    }
    /// Return the message length (number of MRs).
    #[inline]
    pub const fn length(self) -> usize { self.0 & 0x7F }
    /// Return true if this message carries zero capabilities.
    #[inline]
    pub const fn is_data_only(self) -> bool { (self.0 >> 7) == 0 }
}

// ── IPC Buffer ────────────────────────────────────────────────────────────────

/// seL4 IPC buffer layout (aarch64, 4KB page).
///
/// The IPC buffer is a per-thread shared memory region mapped by the kernel.
/// The root task's IPC buffer virtual address is provided at boot.
#[repr(C, align(512))]
pub struct IPCBuffer {
    /// Message tag (MessageInfo encoded as a word).
    pub tag: usize,
    /// Message registers MR[0..120].
    pub msg: [usize; SEL4_MSG_MAX_LENGTH],
    /// User data word (arbitrary).
    pub user_data: usize,
    /// Caps or badge slots.
    pub caps_or_badges: [usize; SEL4_MSG_MAX_LENGTH],
    /// Receive slot CPtr.
    pub receive_cnode: CPtr,
    pub receive_index: usize,
    pub receive_depth: usize,
}

/// Canonical IPC buffer virtual address for the AXON root task on seL4/QEMU virt.
/// This is set at boot by the seL4 kernel and communicated via the BootInfo struct.
/// For AXON's root task we use the standard seL4 convention: 0x10_000_000.
pub const IPC_BUFFER_VADDR: usize = 0x1000_0000;

/// Get a mutable reference to the IPC buffer.
///
/// # Safety
///
/// The IPC buffer must have been mapped by the seL4 kernel at `IPC_BUFFER_VADDR`
/// before this function is called. Valid only in a running seL4 root task context.
#[inline]
pub unsafe fn ipc_buffer() -> &'static mut IPCBuffer {
    &mut *(IPC_BUFFER_VADDR as *mut IPCBuffer)
}
