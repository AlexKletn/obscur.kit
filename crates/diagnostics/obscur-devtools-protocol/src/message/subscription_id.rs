/// A server-assigned identifier for an active subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SubscriptionId(pub u64);

impl SubscriptionId {
    /// Creates a subscription id from a raw value.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw identifier value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
