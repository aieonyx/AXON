// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_pkg P72 — Local package registry (installed packages index)

use serde::{Deserialize, Serialize};
use crate::manifest::{AxpkgManifest, Capability};
use crate::hash::{sovereign_hash_hex, HASH_LEN};

/// Registry entry for an installed package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub version: String,
    pub author: String,
    pub content_hash_hex: String,
    pub signed: bool,
    pub install_path: String,
    pub capabilities: Vec<Capability>,
}

/// Local installed package registry.
#[derive(Debug, Default)]
pub struct AxpkgRegistry {
    pub entries: Vec<RegistryEntry>,
}

impl AxpkgRegistry {
    pub fn new() -> Self { Self::default() }

    /// Register an installed package.
    pub fn register(
        &mut self,
        manifest: &AxpkgManifest,
        content_hash: &[u8; HASH_LEN],
        install_path: &str,
    ) {
        // Remove any existing entry for same package name
        self.entries.retain(|e| e.name != manifest.name);
        self.entries.push(RegistryEntry {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            author: manifest.author.clone(),
            content_hash_hex: sovereign_hash_hex(content_hash),
            signed: manifest.signed,
            install_path: install_path.to_string(),
            capabilities: manifest.capabilities.clone(),
        });
    }

    /// Look up a package by name.
    pub fn lookup(&self, name: &str) -> Option<&RegistryEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Remove a package from the registry.
    pub fn unregister(&mut self, name: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.name != name);
        self.entries.len() < before
    }

    /// Check if a package has a specific capability.
    pub fn has_capability(&self, name: &str, cap: &Capability) -> bool {
        self.lookup(name)
            .map(|e| e.capabilities.contains(cap))
            .unwrap_or(false)
    }

    pub fn package_count(&self) -> usize { self.entries.len() }

    /// Serialize to .axreg JSON format.
    pub fn serialize(&self) -> String {
        serde_json::to_string_pretty(&self.entries).unwrap_or_default()
    }
}
