// ============================================================
// AXON Codegen — apr.rs
// Annotation Preservation Report
// Tracks every decorator annotation through transpilation
// Copyright © 2026 Edison Lepiten — AIEONYX
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum APRStatus {
    Preserved,            // kept in output as comment
    Translated,           // converted to Rust equivalent (debug_assert!)
    DroppedUnsupported,   // no Rust equivalent yet
    DroppedError(String), // failed to process
}

#[derive(Debug, Clone)]
pub struct APREntry {
    pub fn_name    : String,
    pub annotation : String,
    pub status     : APRStatus,
}

#[derive(Debug, Default)]
pub struct APR {
    pub entries: Vec<APREntry>,
}

impl APR {
    pub fn new() -> Self { APR { entries: Vec::new() } }

    pub fn add(&mut self, fn_name: &str, annotation: &str, status: APRStatus) {
        self.entries.push(APREntry {
            fn_name    : fn_name.to_string(),
            annotation : annotation.to_string(),
            status,
        });
    }

    pub fn preserved_count(&self) -> usize {
        self.entries.iter()
            .filter(|e| matches!(e.status, APRStatus::Preserved | APRStatus::Translated))
            .count()
    }

    pub fn translated_count(&self) -> usize {
        self.entries.iter()
            .filter(|e| e.status == APRStatus::Translated)
            .count()
    }

    pub fn dropped_count(&self) -> usize {
        self.entries.iter()
            .filter(|e| matches!(e.status,
                APRStatus::DroppedUnsupported | APRStatus::DroppedError(_)))
            .count()
    }

    pub fn has_dropped(&self) -> bool {
        self.dropped_count() > 0
    }

    pub fn summary(&self) -> String {
        format!(
            "Annotations: {} preserved, {} translated, {} dropped",
            self.preserved_count(),
            self.translated_count(),
            self.dropped_count()
        )
    }

    pub fn warnings(&self) -> Vec<String> {
        self.entries.iter()
            .filter(|e| matches!(e.status,
                APRStatus::DroppedUnsupported | APRStatus::DroppedError(_)))
            .map(|e| format!(
                "warning[W301]: annotation dropped without translation\n  fn {}: {}",
                e.fn_name, e.annotation
            ))
            .collect()
    }

    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::File::create(path)?;
        writeln!(f, "function_name | annotation_text | status")?;
        for entry in &self.entries {
            let status_str = match &entry.status {
                APRStatus::Preserved          => "PRESERVED".to_string(),
                APRStatus::Translated         => "TRANSLATED".to_string(),
                APRStatus::DroppedUnsupported => "DROPPED_UNSUPPORTED".to_string(),
                APRStatus::DroppedError(e)    => format!("DROPPED_ERROR({})", e),
            };
            writeln!(f, "{} | {} | {}",
                entry.fn_name, entry.annotation, status_str)?;
        }
        Ok(())
    }
}
