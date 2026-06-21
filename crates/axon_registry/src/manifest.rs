// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P65 — .axpkg manifest: name, version, author, hashes, signature

use serde::{Deserialize, Serialize};

/// Semantic version — major.minor.patch
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(3, '.').collect();
        if parts.len() != 3 { return None; }
        Some(Self {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major.cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

/// Package manifest — the sovereign identity of an .axpkg
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Package name (snake_case, no spaces)
    pub name: String,
    /// Semantic version
    pub version: Version,
    /// Human-readable description
    pub description: String,
    /// Author: "Full Name <email>"
    pub author: String,
    /// AXONYX language version required (e.g. "0.63")
    pub axon_version: String,
    /// SHA-256 hex digest of the package content bytes
    pub content_hash: String,
    /// Ed25519 signature (hex) of content_hash by author key
    /// Empty string = unsigned (dev packages only)
    pub signature: String,
    /// License identifier
    pub license: String,
    /// Dependencies: Vec<"name@version">
    pub dependencies: Vec<String>,
}

impl Manifest {
    pub fn new(
        name: impl Into<String>,
        version: Version,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version,
            description: description.into(),
            author: author.into(),
            axon_version: "0.63".into(),
            content_hash: String::new(),
            signature: String::new(),
            license: "Apache-2.0".into(),
            dependencies: vec![],
        }
    }

    /// Validate manifest fields — returns Err with reason if invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("name must not be empty".into());
        }
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!("name '{}' must be snake_case alphanumeric+underscore", self.name));
        }
        if self.author.is_empty() {
            return Err("author must not be empty".into());
        }
        if self.content_hash.len() != 64 && !self.content_hash.is_empty() {
            return Err(format!("content_hash must be 64 hex chars, got {}", self.content_hash.len()));
        }
        Ok(())
    }

    /// Serialize to canonical JSON bytes (sorted keys via serde).
    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("manifest parse error: {}", e))
    }
}
