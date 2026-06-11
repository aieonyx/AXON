// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_compute — Kani Formal Verification Harnesses
// Phase 34 | dispatch correctness + checkpoint round-trip

#[cfg(kani)]
mod verify {
    use axon_compute::dispatch::{ComputeBackend, LaunchConfig, BufferDescriptor, KernelLaunch};
    use axon_compute::checkpoint::{ModelCheckpoint, save_checkpoint, load_checkpoint};

    // ------------------------------------------------------------------
    // CPU is always available
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_cpu_always_available() {
        assert!(ComputeBackend::Cpu.is_available());
    }

    // ------------------------------------------------------------------
    // LaunchConfig::linear covers all elements
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_launch_config_covers_n() {
        let n: usize = kani::any();
        let block: usize = kani::any();
        kani::assume(block > 0 && block <= 1024);
        kani::assume(n > 0 && n <= 65536);
        let cfg = LaunchConfig::linear(n, block);
        let total_threads = cfg.grid_x * cfg.block_x;
        assert!(total_threads >= n);
    }

    // ------------------------------------------------------------------
    // BufferDescriptor byte_size == numel * elem_bytes
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_buffer_byte_size() {
        let n: usize = kani::any();
        kani::assume(n > 0 && n <= 1_000_000);
        let buf = BufferDescriptor::f32_rw(n, "test");
        assert_eq!(buf.byte_size(), n * 4);
    }

    // ------------------------------------------------------------------
    // KernelLaunch validate: empty id → Err
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_empty_kernel_id_invalid() {
        let launch = KernelLaunch::new(
            "",
            ComputeBackend::Cpu,
            LaunchConfig::linear(256, 256),
        );
        assert!(launch.validate().is_err());
    }

    // ------------------------------------------------------------------
    // Checkpoint round-trip: step and loss preserved
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_checkpoint_roundtrip_metadata() {
        let step: u64 = kani::any();
        let loss: f32 = kani::any();
        kani::assume(loss.is_finite());
        let ckpt = ModelCheckpoint::new(step, loss);
        let bytes = save_checkpoint(&ckpt);
        let loaded = load_checkpoint(&bytes).unwrap();
        assert_eq!(loaded.step, step);
        assert!((loaded.loss - loss).abs() < 1e-6);
    }

    // ------------------------------------------------------------------
    // Checkpoint magic bytes always "ONYX"
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_checkpoint_magic() {
        let ckpt = ModelCheckpoint::new(0, 0.0);
        let bytes = save_checkpoint(&ckpt);
        assert_eq!(&bytes[0..4], b"ONYX");
    }
}
