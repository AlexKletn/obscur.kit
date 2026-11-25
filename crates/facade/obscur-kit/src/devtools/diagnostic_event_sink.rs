use obscur_devtools_protocol::message::{EventKind, EventSeverity, Topic};
use obscur_devtools_server::transport::EventQueue;

use crate::diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticSink};

#[doc = include_str!("../../docs/api/devtools.doc/DiagnosticEventSink.md")]
pub struct DiagnosticEventSink {
    events: EventQueue,
}

impl DiagnosticEventSink {
    /// Creates a sink publishing onto `events`.
    #[must_use]
    pub fn new(events: EventQueue) -> Self {
        Self { events }
    }
}

impl DiagnosticSink for DiagnosticEventSink {
    fn emit(&self, diagnostic: &Diagnostic) {
        // A full queue drops its oldest and counts the loss in
        // `EventQueue::dropped`, which is where a host reads it; failing the
        // emit here would only turn a reported gap into an unreported one.
        let _ = self.events.push(
            Topic::Diagnostics,
            EventKind::Diagnostic {
                severity: event_severity(diagnostic.severity),
                code: diagnostic.code.as_str().into(),
                message: diagnostic.message.as_str().into(),
            },
        );
    }
}

/// Maps the runtime's severity ladder onto the protocol's identical one.
const fn event_severity(severity: DiagnosticSeverity) -> EventSeverity {
    match severity {
        DiagnosticSeverity::Trace => EventSeverity::Trace,
        DiagnosticSeverity::Info => EventSeverity::Info,
        DiagnosticSeverity::Warning => EventSeverity::Warning,
        DiagnosticSeverity::Error => EventSeverity::Error,
        DiagnosticSeverity::Critical => EventSeverity::Critical,
    }
}
