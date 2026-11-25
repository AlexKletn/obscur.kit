/// A client-assigned identifier correlating a command with its response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CommandId(pub u64);

impl CommandId {
    /// Creates a command id from a raw value.
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
