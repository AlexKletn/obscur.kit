#![doc = include_str!("signal_id.doc/_module.md")]

#[doc = include_str!("signal_id.doc/SignalId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalId(u64);

impl SignalId {
    #[doc = include_str!("signal_id.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("signal_id.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SignalId;

    #[test]
    fn round_trips_its_raw_handle() {
        assert_eq!(SignalId::from_raw(7).raw(), 7);
    }

    #[test]
    fn distinct_handles_are_distinct_signals() {
        assert_eq!(SignalId::from_raw(1), SignalId::from_raw(1));
        assert_ne!(SignalId::from_raw(1), SignalId::from_raw(2));
    }
}
