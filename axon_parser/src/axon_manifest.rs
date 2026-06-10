// axon_manifest.rs — P28: axon.toml manifest parser
// Copyright (c) 2026 Edison Lepiten / AIEONYX
// Apache-2.0
//
// Parses a minimal axon.toml project manifest:
//
//   [project]
//   name    = "my_pd"
//   version = "0.1.0"
//   entry   = "src/main.axon"
//
//   [build]
//   target  = "aarch64-sel4"
//   profile = "seL4-strict"
//
//   [capabilities]
//   required = ["ipc_send", "ipc_receive"]

/// Parsed axon.toml manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct AxonManifest {
    /// Project name
    pub name: String,
    /// Semantic version string
    pub version: String,
    /// Entry source file (relative path)
    pub entry: String,
    /// Compilation target (e.g. "aarch64-sel4", "x86_64-linux")
    pub target: String,
    /// CCP profile (e.g. "seL4-strict", "sovereign-offline")
    pub profile: String,
    /// Required capabilities declared in manifest
    pub required_caps: Vec<String>,
}

impl Default for AxonManifest {
    fn default() -> Self {
        AxonManifest {
            name:         "unnamed".to_string(),
            version:      "0.1.0".to_string(),
            entry:        "src/main.axon".to_string(),
            target:       "x86_64-linux".to_string(),
            profile:      "sovereign-offline".to_string(),
            required_caps: Vec::new(),
        }
    }
}

/// Error returned by the manifest parser.
#[derive(Debug, Clone)]
pub struct ManifestError {
    pub msg: String,
    pub line: usize,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "axon.toml:{}: {}", self.line, self.msg)
    }
}

/// Parse an `axon.toml` string into an `AxonManifest`.
///
/// Supports:
/// - `[section]` headers
/// - `key = "value"` string assignments
/// - `key = ["a", "b"]` string array assignments
/// - `#` line comments
/// - Blank lines
pub fn parse_manifest(src: &str) -> Result<AxonManifest, ManifestError> {
    let mut manifest = AxonManifest::default();
    let mut section = String::new();

    for (lineno, raw_line) in src.lines().enumerate() {
        let line_num = lineno + 1;
        // Strip inline comments and whitespace
        let line = if let Some(pos) = raw_line.find('#') {
            &raw_line[..pos]
        } else {
            raw_line
        }.trim();

        if line.is_empty() { continue; }

        // Section header: [section]
        if line.starts_with('[') {
            if !line.ends_with(']') {
                return Err(ManifestError { msg: "malformed section header".to_string(), line: line_num });
            }
            section = line[1..line.len()-1].trim().to_string();
            continue;
        }

        // Key-value pair
        let eq_pos = line.find('=').ok_or_else(|| ManifestError {
            msg: format!("expected '=' in: {}", line),
            line: line_num,
        })?;
        let key = line[..eq_pos].trim();
        let val_raw = line[eq_pos+1..].trim();

        // Array value: ["a", "b", ...]
        if val_raw.starts_with('[') {
            let caps = parse_string_array(val_raw).map_err(|e| ManifestError { msg: e, line: line_num })?;
            if section.as_str() == "capabilities" && key == "required" {
                manifest.required_caps = caps;
            }
            continue;
        }

        // String value: "value"
        let val = parse_string_value(val_raw).map_err(|e| ManifestError { msg: e, line: line_num })?;

        match (section.as_str(), key) {
            ("project", "name")    => manifest.name    = val,
            ("project", "version") => manifest.version = val,
            ("project", "entry")   => manifest.entry   = val,
            ("build",   "target")  => manifest.target  = val,
            ("build",   "profile") => manifest.profile = val,
            _ => {} // Unknown key — ignore for forward compatibility
        }
    }

    Ok(manifest)
}

/// Parse a quoted string value: `"hello"` → `hello`
fn parse_string_value(s: &str) -> Result<String, String> {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        Ok(s[1..s.len()-1].to_string())
    } else {
        Err(format!("expected quoted string, got: {}", s))
    }
}

/// Parse a string array: `["a", "b"]` → `["a", "b"]`
fn parse_string_array(s: &str) -> Result<Vec<String>, String> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(format!("expected array [...], got: {}", s));
    }
    let inner = &s[1..s.len()-1];
    let mut result = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        result.push(parse_string_value(part)?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TOML: &str = r#"
[project]
name    = "bastion_init"
version = "0.1.0"
entry   = "src/main.axon"

[build]
target  = "aarch64-sel4"
profile = "seL4-strict"

[capabilities]
required = ["ipc_send", "ipc_receive", "memory_grant"]
"#;

    #[test]
    fn tp28_01_parse_project_fields() {
        let m = parse_manifest(SAMPLE_TOML).expect("parse failed");
        assert_eq!(m.name, "bastion_init");
        assert_eq!(m.version, "0.1.0");
        assert_eq!(m.entry, "src/main.axon");
    }

    #[test]
    fn tp28_02_parse_build_fields() {
        let m = parse_manifest(SAMPLE_TOML).expect("parse failed");
        assert_eq!(m.target, "aarch64-sel4");
        assert_eq!(m.profile, "seL4-strict");
    }

    #[test]
    fn tp28_03_parse_capabilities() {
        let m = parse_manifest(SAMPLE_TOML).expect("parse failed");
        assert_eq!(m.required_caps, vec!["ipc_send", "ipc_receive", "memory_grant"]);
    }

    #[test]
    fn tp28_04_defaults_on_empty() {
        let m = parse_manifest("").expect("parse failed");
        assert_eq!(m.name, "unnamed");
        assert_eq!(m.target, "x86_64-linux");
        assert_eq!(m.profile, "sovereign-offline");
    }

    #[test]
    fn tp28_05_comments_ignored() {
        let src = r#"
# This is a comment
[project] # inline comment
name = "test_pd" # name comment
[build]
target = "aarch64-sel4"
"#;
        let m = parse_manifest(src).expect("parse failed");
        assert_eq!(m.name, "test_pd");
        assert_eq!(m.target, "aarch64-sel4");
    }

    #[test]
    fn tp28_06_unknown_keys_ignored() {
        let src = r#"
[project]
name = "foo"
unknown_key = "bar"
[unknown_section]
whatever = "ignored"
"#;
        let m = parse_manifest(src).expect("unknown keys must not cause error");
        assert_eq!(m.name, "foo");
    }

    #[test]
    fn tp28_07_sel4_pd_manifest() {
        // Full seL4 PD manifest pattern
        let src = r#"
[project]
name    = "axon_framebuffer_pd"
version = "0.2.0"
entry   = "src/fb_pd.axon"

[build]
target  = "aarch64-sel4"
profile = "seL4-strict"

[capabilities]
required = ["framebuffer_write", "ipc_receive"]
"#;
        let m = parse_manifest(src).expect("parse failed");
        assert_eq!(m.name, "axon_framebuffer_pd");
        assert_eq!(m.target, "aarch64-sel4");
        assert!(m.required_caps.contains(&"framebuffer_write".to_string()));
        assert!(m.required_caps.contains(&"ipc_receive".to_string()));
    }
}
