// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P65 — Package index: publish, resolve, list, duplicate guard

use std::collections::HashMap;
use crate::manifest::{Manifest, Version};
use crate::hash::sha256_hex;

#[derive(Debug)]
pub enum RegistryError {
    AlreadyExists(String),
    NotFound(String),
    InvalidManifest(String),
    HashMismatch { expected: String, got: String },
    ValidationFailed(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AlreadyExists(s)    => write!(f, "package already exists: {}", s),
            Self::NotFound(s)         => write!(f, "package not found: {}", s),
            Self::InvalidManifest(s)  => write!(f, "invalid manifest: {}", s),
            Self::HashMismatch { expected, got } =>
                write!(f, "hash mismatch: expected {} got {}", expected, got),
            Self::ValidationFailed(s) => write!(f, "validation failed: {}", s),
        }
    }
}

/// A published package entry in the index.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub manifest: Manifest,
    /// Raw content bytes of the package
    pub content: Vec<u8>,
}

/// Key: "name@major.minor.patch"
fn entry_key(name: &str, version: &Version) -> String {
    format!("{}@{}", name, version.to_string())
}

/// The sovereign package index — in-memory for P65, persistent store in P65.1
#[derive(Debug, Default)]
pub struct PackageIndex {
    entries: HashMap<String, IndexEntry>,
}

impl PackageIndex {
    pub fn new() -> Self { Self::default() }

    /// Publish a package: validate manifest, verify content hash, store.
    pub fn publish(&mut self, mut manifest: Manifest, content: Vec<u8>)
        -> Result<String, RegistryError>
    {
        // Validate manifest fields
        manifest.validate().map_err(|e| RegistryError::InvalidManifest(e))?;

        // Compute and verify content hash
        let computed = sha256_hex(&content);
        if manifest.content_hash.is_empty() {
            // Auto-fill hash if not provided
            manifest.content_hash = computed.clone();
        } else if manifest.content_hash != computed {
            return Err(RegistryError::HashMismatch {
                expected: manifest.content_hash.clone(),
                got: computed,
            });
        }

        let key = entry_key(&manifest.name, &manifest.version);

        // Duplicate guard
        if self.entries.contains_key(&key) {
            return Err(RegistryError::AlreadyExists(key));
        }

        self.entries.insert(key.clone(), IndexEntry { manifest, content });
        Ok(key)
    }

    /// Resolve exact version.
    pub fn resolve(&self, name: &str, version: &Version) -> Option<&IndexEntry> {
        self.entries.get(&entry_key(name, version))
    }

    /// Resolve latest version of a package (highest semver).
    pub fn resolve_latest(&self, name: &str) -> Option<&IndexEntry> {
        self.entries.values()
            .filter(|e| e.manifest.name == name)
            .max_by_key(|e| e.manifest.version.clone())
    }

    /// List all published packages (name@version strings).
    pub fn list(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.entries.keys().cloned().collect();
        keys.sort();
        keys
    }

    /// Total number of published packages.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Verify integrity of a stored package (re-hash content).
    pub fn verify(&self, name: &str, version: &Version) -> Result<bool, RegistryError> {
        let entry = self.resolve(name, version)
            .ok_or_else(|| RegistryError::NotFound(entry_key(name, version)))?;
        let computed = sha256_hex(&entry.content);
        Ok(computed == entry.manifest.content_hash)
    }
}
