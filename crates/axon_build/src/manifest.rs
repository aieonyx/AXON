// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Sovereign .axbuild manifest parser.
// Hand-written — zero external toml crate dependency.
// Parses a minimal TOML-like manifest into AxManifest.

use axon_std_string::AxString;
use crate::error::{BuildError, BuildResult};

#[derive(Debug, Clone, PartialEq)]
pub enum Backend { Posix, Sel4 }

#[derive(Debug, Clone)]
pub struct AxDep {
    pub name: AxString,
    pub path: AxString,
}

#[derive(Debug, Clone)]
pub struct AxProfile {
    pub opt: u8,
    pub debug: bool,
    pub lto: bool,
}

impl Default for AxProfile {
    fn default() -> Self {
        AxProfile { opt: 0, debug: true, lto: false }
    }
}

#[derive(Debug, Clone)]
pub struct AxManifest {
    pub name: AxString,
    pub version: AxString,
    pub src_dir: AxString,
    pub entry: AxString,
    pub target_dir: AxString,
    pub backend: Backend,
    pub dependencies: Vec<AxDep>,
    pub profile: AxProfile,
}

/// Parse a .axbuild manifest from a file path.
pub fn manifest_parse(path: &str) -> BuildResult<AxManifest> {
    if !axon_std_io::path_exists(path) {
        return Err(BuildError::ManifestNotFound);
    }
    let content = axon_std_io::read_to_string(path)
        .map_err(|e| BuildError::ManifestParseError(
            AxString::ax_from_str(&format!("io error: {}", e))
        ))?;
    parse_content(&content)
}

/// Parse manifest content from a string (used in tests).
pub fn parse_content(content: &str) -> BuildResult<AxManifest> {
    let mut name        = AxString::new();
    let mut version     = AxString::new();
    let mut src_dir     = AxString::ax_from_str("src/");
    let mut entry       = AxString::ax_from_str("src/lib.ax");
    let mut target_dir  = AxString::ax_from_str("target/axon/");
    let mut backend     = Backend::Posix;
    let mut dependencies: Vec<AxDep> = Vec::new();
    let mut profile     = AxProfile::default();

    let mut current_section = AxString::new();
    let mut dep_name    = AxString::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();

        // Skip comments and blank lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len() - 1];
            // Flush pending dep
            if current_section.as_str() == "dep" && !dep_name.is_empty() {
                dep_name = AxString::new();
            }
            current_section = AxString::ax_from_str(section);
            // Extract dep name from [dependencies.xxx]
            if let Some(stripped) = section.strip_prefix("dependencies.") {
                dep_name = AxString::ax_from_str(stripped);
                current_section = AxString::ax_from_str("dep");
            }
            continue;
        }

        // Key = value
        let Some(eq) = line.find('=') else { continue };
        let key = line[..eq].trim();
        let val = line[eq + 1..].trim().trim_matches('"');

        match current_section.as_str() {
            "crate" => match key {
                "name"    => name    = AxString::ax_from_str(val),
                "version" => version = AxString::ax_from_str(val),
                _ => {}
            },
            "build" => match key {
                "src"     => src_dir    = AxString::ax_from_str(val),
                "entry"   => entry      = AxString::ax_from_str(val),
                "target"  => target_dir = AxString::ax_from_str(val),
                "backend" => backend    = if val == "sel4" { Backend::Sel4 } else { Backend::Posix },
                _ => {}
            },
            "dep" => {
                if key == "path" && !dep_name.is_empty() {
                    dependencies.push(AxDep {
                        name: dep_name.clone(),
                        path: AxString::ax_from_str(val),
                    });
                }
            },
            "profile.dev" => match key {
                "opt"   => profile.opt   = val.parse().unwrap_or(0),
                "debug" => profile.debug = val == "true",
                "lto"   => profile.lto   = val == "true",
                _ => {}
            },
            "profile.release" => match key {
                "opt"   => profile.opt   = val.parse().unwrap_or(3),
                "debug" => profile.debug = val == "true",
                "lto"   => profile.lto   = val == "true",
                _ => {}
            },
            _ => {}
        }
    }

    if name.is_empty() {
        return Err(BuildError::ManifestParseError(
            AxString::ax_from_str("missing required field: [crate] name")
        ));
    }
    if version.is_empty() {
        return Err(BuildError::ManifestParseError(
            AxString::ax_from_str("missing required field: [crate] version")
        ));
    }

    Ok(AxManifest { name, version, src_dir, entry, target_dir, backend, dependencies, profile })
}
