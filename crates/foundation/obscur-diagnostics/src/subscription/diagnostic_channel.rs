//! The functional diagnostics channel (spec §7.8).

use super::diagnostic_buffer_policy::DiagnosticBufferPolicy;
use super::diagnostic_filter::DiagnosticFilter;
use super::{DiagnosticSubscription, SubscriptionId};
use crate::diagnostic::Diagnostic;

#[doc = include_str!("../../docs/api/subscription/diagnostic_channel.doc/DiagnosticChannel.md")]
#[derive(Debug, Default)]
pub struct DiagnosticChannel {
    subscriptions: Vec<DiagnosticSubscription>,
    next_id: u64,
}

impl DiagnosticChannel {
    /// Creates an empty channel with no subscriptions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
            next_id: 0,
        }
    }

    /// Registers a subscription and returns its freshly minted id.
    pub fn subscribe(
        &mut self,
        filter: DiagnosticFilter,
        policy: DiagnosticBufferPolicy,
    ) -> SubscriptionId {
        let id = SubscriptionId::new(self.next_id);
        self.next_id += 1;
        self.subscriptions
            .push(DiagnosticSubscription::new(id, filter, policy));
        id
    }

    /// Removes the subscription with `id`, returning whether one was removed.
    pub fn unsubscribe(&mut self, id: SubscriptionId) -> bool {
        let before = self.subscriptions.len();
        self.subscriptions.retain(|s| s.id() != id);
        self.subscriptions.len() != before
    }

    /// Publishes `diagnostic` to the channel, offering it to every enabled,
    /// matching subscription. Each subscription applies its own buffer policy,
    /// so backpressure is independent per subscriber.
    pub fn publish(&mut self, diagnostic: &Diagnostic) {
        for subscription in &mut self.subscriptions {
            subscription.offer(diagnostic);
        }
    }

    /// Drains all buffered diagnostics from the subscription with `id`, or
    /// returns an empty vector if no such subscription exists.
    pub fn drain(&mut self, id: SubscriptionId) -> Vec<Diagnostic> {
        self.subscription_mut(id)
            .map(DiagnosticSubscription::drain)
            .unwrap_or_default()
    }

    /// Returns a mutable reference to the subscription with `id`, if any, so
    /// callers can toggle it enabled/disabled or inspect its buffer.
    #[must_use]
    pub fn subscription_mut(&mut self, id: SubscriptionId) -> Option<&mut DiagnosticSubscription> {
        self.subscriptions.iter_mut().find(|s| s.id() == id)
    }

    /// Returns the number of active subscriptions.
    #[must_use]
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ChannelSink;
    use super::*;
    use crate::OverflowStrategy;
    use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};
    use crate::port::DiagnosticSink;

    fn diag(severity: DiagnosticSeverity, code: &'static str) -> Diagnostic {
        Diagnostic::new(
            DiagnosticCode::new(code),
            severity,
            DiagnosticSubsystem::Layout,
            "msg",
        )
    }

    #[test]
    fn subscribe_mints_unique_ids() {
        let mut ch = DiagnosticChannel::new();
        let a = ch.subscribe(
            DiagnosticFilter::min(DiagnosticSeverity::Info),
            DiagnosticBufferPolicy::bounded(4),
        );
        let b = ch.subscribe(
            DiagnosticFilter::min(DiagnosticSeverity::Info),
            DiagnosticBufferPolicy::bounded(4),
        );
        assert_ne!(a, b);
        assert_eq!(ch.subscription_count(), 2);
    }

    #[test]
    fn publish_fans_to_matching_only() {
        let mut ch = DiagnosticChannel::new();
        let low = ch.subscribe(
            DiagnosticFilter::min(DiagnosticSeverity::Trace),
            DiagnosticBufferPolicy::bounded(8),
        );
        let high = ch.subscribe(
            DiagnosticFilter::min(DiagnosticSeverity::Error),
            DiagnosticBufferPolicy::bounded(8),
        );

        ch.publish(&diag(DiagnosticSeverity::Info, "a"));

        assert_eq!(ch.drain(low).len(), 1);
        assert_eq!(ch.drain(high).len(), 0);
    }

    #[test]
    fn unsubscribe_removes() {
        let mut ch = DiagnosticChannel::new();
        let id = ch.subscribe(
            DiagnosticFilter::min(DiagnosticSeverity::Info),
            DiagnosticBufferPolicy::bounded(4),
        );
        assert!(ch.unsubscribe(id));
        assert!(!ch.unsubscribe(id));
        assert_eq!(ch.subscription_count(), 0);
        // Draining a removed subscription yields nothing.
        assert!(ch.drain(id).is_empty());
    }

    #[test]
    fn per_subscription_backpressure_is_independent() {
        let mut ch = DiagnosticChannel::new();
        let small = ch.subscribe(
            DiagnosticFilter::min(DiagnosticSeverity::Info),
            DiagnosticBufferPolicy::new(1, OverflowStrategy::DropOldest),
        );
        let big = ch.subscribe(
            DiagnosticFilter::min(DiagnosticSeverity::Info),
            DiagnosticBufferPolicy::bounded(8),
        );

        ch.publish(&diag(DiagnosticSeverity::Info, "a"));
        ch.publish(&diag(DiagnosticSeverity::Info, "b"));

        // Small keeps only the newest; big keeps both.
        let small_codes: Vec<&str> = ch.drain(small).iter().map(|d| d.code.as_str()).collect();
        assert_eq!(small_codes, vec!["b"]);
        assert_eq!(ch.drain(big).len(), 2);
    }

    #[test]
    fn disabled_subscription_skipped() {
        let mut ch = DiagnosticChannel::new();
        let id = ch.subscribe(
            DiagnosticFilter::min(DiagnosticSeverity::Info),
            DiagnosticBufferPolicy::bounded(4),
        );
        ch.subscription_mut(id).unwrap().set_enabled(false);
        ch.publish(&diag(DiagnosticSeverity::Error, "a"));
        assert!(ch.drain(id).is_empty());
    }

    #[test]
    fn channel_sink_publishes() {
        let mut ch = DiagnosticChannel::new();
        let id = ch.subscribe(
            DiagnosticFilter::min(DiagnosticSeverity::Info),
            DiagnosticBufferPolicy::bounded(4),
        );
        let sink = ChannelSink::new(ch);
        sink.emit(&diag(DiagnosticSeverity::Warning, "a"));
        let drained = sink.channel_mut().drain(id);
        assert_eq!(drained.len(), 1);
    }
}
