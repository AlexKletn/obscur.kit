/// A stable identifier for a [`DiagnosticSubscription`](super::DiagnosticSubscription), minted by the
/// [`DiagnosticChannel`](crate::DiagnosticChannel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    /// Creates a subscription id from a raw value.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
