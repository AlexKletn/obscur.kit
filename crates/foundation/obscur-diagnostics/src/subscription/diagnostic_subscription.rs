#![doc = include_str!("diagnostic_subscription.doc/_module.md")]

use std::collections::VecDeque;

use super::diagnostic_buffer_policy::{DiagnosticBufferPolicy, OverflowStrategy};
use super::diagnostic_filter::DiagnosticFilter;
use crate::diagnostic::Diagnostic;

#[doc = include_str!("diagnostic_subscription.doc/SubscriptionId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    #[doc = include_str!("diagnostic_subscription.doc/new.md")]
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("diagnostic_subscription.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[doc = include_str!("diagnostic_subscription.doc/DiagnosticSubscription.md")]
#[derive(Debug, Clone)]
pub struct DiagnosticSubscription {
    id: SubscriptionId,
    filter: DiagnosticFilter,
    policy: DiagnosticBufferPolicy,
    buffer: VecDeque<Diagnostic>,
    enabled: bool,
}

impl DiagnosticSubscription {
    #[doc = include_str!("diagnostic_subscription.doc/new_2.md")]
    #[must_use]
    pub fn new(
        id: SubscriptionId,
        filter: DiagnosticFilter,
        policy: DiagnosticBufferPolicy,
    ) -> Self {
        Self {
            id,
            filter,
            policy,
            buffer: VecDeque::new(),
            enabled: true,
        }
    }

    #[doc = include_str!("diagnostic_subscription.doc/id.md")]
    #[must_use]
    pub fn id(&self) -> SubscriptionId {
        self.id
    }

    #[doc = include_str!("diagnostic_subscription.doc/filter.md")]
    #[must_use]
    pub fn filter(&self) -> DiagnosticFilter {
        self.filter
    }

    #[doc = include_str!("diagnostic_subscription.doc/policy.md")]
    #[must_use]
    pub fn policy(&self) -> DiagnosticBufferPolicy {
        self.policy
    }

    #[doc = include_str!("diagnostic_subscription.doc/is_enabled.md")]
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[doc = include_str!("diagnostic_subscription.doc/set_enabled.md")]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    #[doc = include_str!("diagnostic_subscription.doc/offer.md")]
    pub fn offer(&mut self, diagnostic: &Diagnostic) -> bool {
        if !self.enabled || !self.filter.admits(diagnostic) {
            return false;
        }

        // A zero-capacity buffer can never accept anything.
        if self.policy.capacity == 0 {
            return false;
        }

        if self.buffer.len() < self.policy.capacity {
            self.buffer.push_back(diagnostic.clone());
            return true;
        }

        // Buffer is at capacity: apply the overflow strategy.
        match self.policy.overflow {
            OverflowStrategy::DropOldest => {
                self.buffer.pop_front();
                self.buffer.push_back(diagnostic.clone());
                true
            }
            OverflowStrategy::DropNewest => false,
            OverflowStrategy::Coalesce => {
                if let Some(slot) = self
                    .buffer
                    .iter_mut()
                    .find(|d| d.code == diagnostic.code && d.subsystem == diagnostic.subsystem)
                {
                    // Merge: retain the newest occurrence in place.
                    *slot = diagnostic.clone();
                    true
                } else {
                    // No coalescable entry — fall back to dropping the oldest.
                    self.buffer.pop_front();
                    self.buffer.push_back(diagnostic.clone());
                    true
                }
            }
        }
    }

    #[doc = include_str!("diagnostic_subscription.doc/drain.md")]
    pub fn drain(&mut self) -> Vec<Diagnostic> {
        self.buffer.drain(..).collect()
    }

    #[doc = include_str!("diagnostic_subscription.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[doc = include_str!("diagnostic_subscription.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

    fn diag(severity: DiagnosticSeverity, code: &'static str) -> Diagnostic {
        Diagnostic::new(
            DiagnosticCode::new(code),
            severity,
            DiagnosticSubsystem::Layout,
            "msg",
        )
    }

    fn sub(policy: DiagnosticBufferPolicy) -> DiagnosticSubscription {
        DiagnosticSubscription::new(
            SubscriptionId::new(1),
            DiagnosticFilter::min(DiagnosticSeverity::Info),
            policy,
        )
    }

    #[test]
    fn filter_rejects_below_min() {
        let mut s = sub(DiagnosticBufferPolicy::bounded(4));
        assert!(!s.offer(&diag(DiagnosticSeverity::Trace, "a")));
        assert!(s.is_empty());
        assert!(s.offer(&diag(DiagnosticSeverity::Warning, "b")));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn disabled_rejects() {
        let mut s = sub(DiagnosticBufferPolicy::bounded(4));
        s.set_enabled(false);
        assert!(!s.is_enabled());
        assert!(!s.offer(&diag(DiagnosticSeverity::Error, "a")));
        assert!(s.is_empty());
        s.set_enabled(true);
        assert!(s.offer(&diag(DiagnosticSeverity::Error, "a")));
    }

    #[test]
    fn buffers_to_capacity() {
        let mut s = sub(DiagnosticBufferPolicy::bounded(2));
        assert!(s.offer(&diag(DiagnosticSeverity::Info, "a")));
        assert!(s.offer(&diag(DiagnosticSeverity::Info, "b")));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn drop_oldest_evicts_oldest() {
        let mut s = sub(DiagnosticBufferPolicy::new(2, OverflowStrategy::DropOldest));
        s.offer(&diag(DiagnosticSeverity::Info, "a"));
        s.offer(&diag(DiagnosticSeverity::Info, "b"));
        assert!(s.offer(&diag(DiagnosticSeverity::Info, "c")));
        let drained = s.drain();
        let codes: Vec<&str> = drained.iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, vec!["b", "c"]);
    }

    #[test]
    fn drop_newest_rejects_incoming() {
        let mut s = sub(DiagnosticBufferPolicy::new(2, OverflowStrategy::DropNewest));
        s.offer(&diag(DiagnosticSeverity::Info, "a"));
        s.offer(&diag(DiagnosticSeverity::Info, "b"));
        assert!(!s.offer(&diag(DiagnosticSeverity::Info, "c")));
        let codes: Vec<&str> = s.drain().iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, vec!["a", "b"]);
    }

    #[test]
    fn coalesce_merges_same_code_and_subsystem() {
        let mut s = sub(DiagnosticBufferPolicy::new(2, OverflowStrategy::Coalesce));
        s.offer(&diag(DiagnosticSeverity::Info, "a"));
        s.offer(&diag(DiagnosticSeverity::Info, "b"));
        // "a" already buffered -> coalesced, no growth.
        assert!(s.offer(&diag(DiagnosticSeverity::Warning, "a")));
        assert_eq!(s.len(), 2);
        let drained = s.drain();
        // "a" retained in place, now updated to Warning severity.
        assert_eq!(drained[0].code.as_str(), "a");
        assert_eq!(drained[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(drained[1].code.as_str(), "b");
    }

    #[test]
    fn coalesce_falls_back_to_drop_oldest() {
        let mut s = sub(DiagnosticBufferPolicy::new(2, OverflowStrategy::Coalesce));
        s.offer(&diag(DiagnosticSeverity::Info, "a"));
        s.offer(&diag(DiagnosticSeverity::Info, "b"));
        // No matching code -> drop oldest ("a").
        assert!(s.offer(&diag(DiagnosticSeverity::Info, "c")));
        let codes: Vec<&str> = s.drain().iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, vec!["b", "c"]);
    }

    #[test]
    fn drain_empties() {
        let mut s = sub(DiagnosticBufferPolicy::bounded(4));
        s.offer(&diag(DiagnosticSeverity::Info, "a"));
        assert_eq!(s.drain().len(), 1);
        assert!(s.is_empty());
        assert!(s.drain().is_empty());
    }

    #[test]
    fn id_accessor() {
        let s = DiagnosticSubscription::new(
            SubscriptionId::new(7),
            DiagnosticFilter::min(DiagnosticSeverity::Info),
            DiagnosticBufferPolicy::bounded(1),
        );
        assert_eq!(s.id(), SubscriptionId::new(7));
        assert_eq!(s.id().raw(), 7);
    }
}
