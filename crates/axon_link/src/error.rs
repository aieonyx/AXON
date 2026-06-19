// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use axon_std_string::AxString;

#[derive(Debug)]
pub enum LinkError {
    IoError(AxString),
    ElfFormatError(AxString),
    LayoutError(AxString),
    SigningError(AxString),
    Sel4Error(AxString),
}

impl core::fmt::Display for LinkError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LinkError::IoError(m)        => write!(f, "link: io error: {}", m.as_str()),
            LinkError::ElfFormatError(m) => write!(f, "link: elf format error: {}", m.as_str()),
            LinkError::LayoutError(m)    => write!(f, "link: layout error: {}", m.as_str()),
            LinkError::SigningError(m)   => write!(f, "link: signing error: {}", m.as_str()),
            LinkError::Sel4Error(m)      => write!(f, "link: sel4 error: {}", m.as_str()),
        }
    }
}

pub type LinkResult<T> = Result<T, LinkError>;
