// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_media -- sovereign audio/video codec layer.
// P59.0: PCM audio, RTP framing, RGB/YUV frame buffer.
// P59.1: G.711 compression, GPU-accelerated encoding.
pub mod error;
pub mod frame;
pub mod pcm;
pub mod rtp;
pub use error::{MediaError, MediaResult};
pub use frame::{VideoFrame, PixelFormat};
pub use pcm::{PcmConfig, PcmFrame, SAMPLE_RATE_8KHZ, SAMPLE_RATE_44KHZ, SAMPLE_RATE_48KHZ};
pub use rtp::{RtpHeader, RtpPacket, RtpSession, RTP_VERSION, PT_PCMU, PT_L16_MONO};
