#[doc = include_str!("../../docs/api/gesture/gesture_recognizer.doc/RecognizerId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecognizerId(u64);

impl RecognizerId {
    /// Creates a recognizer id from a raw `u64`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw `u64` backing this id.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::RecognizerId;

    #[test]
    fn recognizer_id_roundtrips() {
        assert_eq!(RecognizerId::from_raw(7).raw(), 7);
    }
}
