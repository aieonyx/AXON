// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_media error types.
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum MediaError {
    InvalidSampleRate(u32),
    InvalidChannels(u8),
    BufferTooSmall { needed: usize, got: usize },
    InvalidFrame,
    RtpSequenceError,
    EncodingFailed(String),
    DecodingFailed(String),
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MediaError::InvalidSampleRate(r)         => write!(f, "invalid sample rate: {}", r),
            MediaError::InvalidChannels(c)           => write!(f, "invalid channels: {}", c),
            MediaError::BufferTooSmall { needed, got } =>
                write!(f, "buffer too small: needed {}, got {}", needed, got),
            MediaError::InvalidFrame                 => write!(f, "invalid frame"),
            MediaError::RtpSequenceError             => write!(f, "RTP sequence error"),
            MediaError::EncodingFailed(s)            => write!(f, "encoding failed: {}", s),
            MediaError::DecodingFailed(s)            => write!(f, "decoding failed: {}", s),
        }
    }
}

pub type MediaResult<T> = Result<T, MediaError>;
