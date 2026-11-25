#[doc = include_str!("../../docs/api/state/subscription.doc/SubscriptionId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    /// Wraps a raw handle value.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
