//! `ValidationContext`: mutable state threaded through a validation pass
//! (spec §2.5 validation).

use std::collections::BTreeSet;

use obscur_capability::CapabilityId;

#[doc = include_str!("../../docs/api/validation/validation_context.doc/ValidationContext.md")]
#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    seen_ids: BTreeSet<u64>,
    available_capabilities: BTreeSet<Box<str>>,
}

impl ValidationContext {
    /// Creates an empty context: no ids seen and no capabilities granted (so
    /// capability checking is off).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a context whose granted capability set is populated from `caps`.
    #[must_use]
    pub fn with_capabilities(caps: impl IntoIterator<Item = CapabilityId>) -> Self {
        Self {
            seen_ids: BTreeSet::new(),
            available_capabilities: caps
                .into_iter()
                .map(|cap| Box::from(cap.as_str()))
                .collect(),
        }
    }

    #[doc = include_str!("../../docs/api/validation/validation_context.doc/record_id.md")]
    pub fn record_id(&mut self, id: u64) -> bool {
        self.seen_ids.insert(id)
    }

    /// Returns whether capability `id` is in the granted set.
    #[must_use]
    pub fn has_capability(&self, id: &str) -> bool {
        self.available_capabilities.contains(id)
    }

    /// Returns whether capability checking is active — i.e. the granted set is
    /// non-empty.
    #[must_use]
    pub fn checks_capabilities(&self) -> bool {
        !self.available_capabilities.is_empty()
    }

    /// Clears the seen-id bookkeeping so the context can drive a fresh walk.
    /// The granted capability set is preserved.
    pub fn reset(&mut self) {
        self.seen_ids.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_id_detects_duplicates() {
        let mut ctx = ValidationContext::new();
        assert!(ctx.record_id(1));
        assert!(ctx.record_id(2));
        // Re-inserting an existing id reports a duplicate.
        assert!(!ctx.record_id(1));
    }

    #[test]
    fn new_context_checks_no_capabilities() {
        let ctx = ValidationContext::new();
        assert!(!ctx.checks_capabilities());
        assert!(!ctx.has_capability("fs.read"));
    }

    #[test]
    fn with_capabilities_populates_granted_set() {
        let ctx = ValidationContext::with_capabilities([
            CapabilityId::new("fs.read"),
            CapabilityId::new("net.fetch"),
        ]);
        assert!(ctx.checks_capabilities());
        assert!(ctx.has_capability("fs.read"));
        assert!(ctx.has_capability("net.fetch"));
        assert!(!ctx.has_capability("clipboard.write"));
    }

    #[test]
    fn reset_clears_ids_but_keeps_capabilities() {
        let mut ctx = ValidationContext::with_capabilities([CapabilityId::new("fs.read")]);
        assert!(ctx.record_id(1));
        ctx.reset();
        // Id 1 is fresh again after reset.
        assert!(ctx.record_id(1));
        // Capabilities survive the reset.
        assert!(ctx.has_capability("fs.read"));
    }
}
