//! AuditSink — trait and built-in implementations.

use super::{AuditError, AuditResult, event::AuditEvent};

/// A sink that receives and records audit events.
pub trait AuditSink {
    /// Record a single audit event.
    fn emit(&mut self, event: &AuditEvent) -> AuditResult<()>;
    /// Flush any buffered events.
    fn flush(&mut self) -> AuditResult<()> { Ok(()) }
}

/// In-memory audit sink — stores all events in a Vec.
/// Use for testing and short-lived processes.
#[derive(Debug, Default)]
pub struct MemorySink {
    events: Vec<AuditEvent>,
}

impl MemorySink {
    pub fn new() -> Self { Self::default() }
    pub fn events(&self) -> &[AuditEvent] { &self.events }
    pub fn len(&self) -> usize { self.events.len() }
    pub fn is_empty(&self) -> bool { self.events.is_empty() }
    pub fn clear(&mut self) { self.events.clear(); }
}

impl AuditSink for MemorySink {
    fn emit(&mut self, event: &AuditEvent) -> AuditResult<()> {
        self.events.push(event.clone());
        Ok(())
    }
}

/// Stdout audit sink — prints one JSON line per event.
/// Use for development and debugging.
#[derive(Debug, Default)]
pub struct StdoutSink;

impl StdoutSink {
    pub fn new() -> Self { Self }
}

impl AuditSink for StdoutSink {
    fn emit(&mut self, event: &AuditEvent) -> AuditResult<()> {
        let json = serde_json::to_string(event)
            .map_err(|e| AuditError::SerializationError(e.to_string()))?;
        println!("{json}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::event::{AuditEvent, EventKind};

    fn sample() -> AuditEvent {
        AuditEvent {
            id: 1, kind: EventKind::Custom,
            label: "test".to_string(), payload: vec![],
            prev_hash: [0u8; 32], timestamp: 0,
        }
    }

    #[test]
    fn memory_sink_stores_events() {
        let mut s = MemorySink::new();
        s.emit(&sample()).unwrap();
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn memory_sink_clear() {
        let mut s = MemorySink::new();
        s.emit(&sample()).unwrap();
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn memory_sink_multiple_events() {
        let mut s = MemorySink::new();
        for _ in 0..5 { s.emit(&sample()).unwrap(); }
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn stdout_sink_emits_without_panic() {
        let mut s = StdoutSink::new();
        s.emit(&sample()).unwrap();
    }
}
