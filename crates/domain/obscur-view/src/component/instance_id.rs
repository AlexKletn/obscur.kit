#![doc = include_str!("instance_id.doc/_module.md")]

#[doc = include_str!("instance_id.doc/InstanceId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstanceId(u64);

impl InstanceId {
    #[doc = include_str!("instance_id.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("instance_id.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::InstanceId;

    #[test]
    fn round_trips_raw_handle() {
        let id = InstanceId::from_raw(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn distinct_handles_are_unequal() {
        assert_ne!(InstanceId::from_raw(1), InstanceId::from_raw(2));
    }
}
