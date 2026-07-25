// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_pkg P72 — Package manifest

use serde::{Deserialize, Serialize};

pub const AXPKG_MAGIC: [u8; 4] = [b'A', b'X', b'P', b'K'];
pub const AXPKG_VERSION: u8 = 1;

/// Declared capability — what the package is allowed to do.
/// Interpreter enforces: undeclared capabilities are denied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Capability {
    /// Can send AWP frames
    Awp,
    /// Can read from AXFS
    FsRead,
    /// Can write to AXFS
    FsWrite,
    /// Can access EdisonDB (future)
    Database,
    /// Can spawn sub-processes (future)
    Spawn,
}

impl Capability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Awp      => "awp",
            Self::FsRead   => "fs:read",
            Self::FsWrite  => "fs:write",
            Self::Database => "database",
            Self::Spawn    => "spawn",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "awp"      => Some(Self::Awp),
            "fs:read"  => Some(Self::FsRead),
            "fs:write" => Some(Self::FsWrite),
            "database" => Some(Self::Database),
            "spawn"    => Some(Self::Spawn),
            _          => None,
        }
    }
}

/// Package manifest — metadata and capability declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxpkgManifest {
    /// Package name (lowercase, hyphen-separated)
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Author identity
    pub author: String,
    /// Human-readable description
    pub description: String,
    /// Entry point .ax file name
    pub entry: String,
    /// Declared capabilities (deny-by-default)
    pub capabilities: Vec<Capability>,
    /// Minimum axon_interp version required
    pub min_interp_version: String,
    /// Signer public key hex (Ed25519, 32 bytes = 64 hex chars)
    pub signer_pubkey_hex: String,
    /// Whether this package is signed (false = unsigned stub)
    pub signed: bool,
}

impl AxpkgManifest {
    pub fn new(
        name: &str,
        version: &str,
        author: &str,
        description: &str,
        entry: &str,
        capabilities: Vec<Capability>,
    ) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            author: author.to_string(),
            description: description.to_string(),
            entry: entry.to_string(),
            capabilities,
            min_interp_version: "0.1.0".to_string(),
            signer_pubkey_hex: "0".repeat(64),
            signed: false,
        }
    }

    pub fn has_capability(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.name.is_empty() {
            return Err(ManifestError::EmptyName);
        }
        if self.version.is_empty() {
            return Err(ManifestError::EmptyVersion);
        }
        if self.entry.is_empty() {
            return Err(ManifestError::EmptyEntry);
        }
        if !self.entry.ends_with(".ax") {
            return Err(ManifestError::InvalidEntry(self.entry.clone()));
        }
        // Name must be lowercase alphanumeric + hyphens
        if !self.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(ManifestError::InvalidName(self.name.clone()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ManifestError {
    EmptyName,
    EmptyVersion,
    EmptyEntry,
    InvalidEntry(String),
    InvalidName(String),
    ParseError(String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::EmptyName           => write!(f, "package name is empty"),
            Self::EmptyVersion        => write!(f, "package version is empty"),
            Self::EmptyEntry          => write!(f, "entry point is empty"),
            Self::InvalidEntry(s)     => write!(f, "entry must end in .ax: {}", s),
            Self::InvalidName(s)      => write!(f, "invalid package name: {}", s),
            Self::ParseError(s)       => write!(f, "manifest parse error: {}", s),
        }
    }
}
