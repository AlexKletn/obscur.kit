use super::ProtocolVersion;

/// An inclusive range of protocol versions a peer supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionRange {
    /// Lowest supported version (inclusive).
    pub min: ProtocolVersion,
    /// Highest supported version (inclusive).
    pub max: ProtocolVersion,
}

impl VersionRange {
    /// Creates a version range.
    #[must_use]
    pub const fn new(min: ProtocolVersion, max: ProtocolVersion) -> Self {
        Self { min, max }
    }

    /// Creates a range covering exactly one version.
    #[must_use]
    pub const fn exact(version: ProtocolVersion) -> Self {
        Self {
            min: version,
            max: version,
        }
    }

    /// Returns whether `version` falls within this (inclusive) range.
    #[must_use]
    pub fn contains(self, version: ProtocolVersion) -> bool {
        self.min <= version && version <= self.max
    }

    #[doc = include_str!("../../docs/api/message/protocol_version.doc/negotiate.md")]
    #[must_use]
    pub fn negotiate(self, client: Self) -> Option<ProtocolVersion> {
        // The highest candidate both sides could speak is the lower of the two
        // maxima; it is valid only when it is at least both minima.
        let candidate = if self.max <= client.max {
            self.max
        } else {
            client.max
        };
        if self.min <= candidate && client.min <= candidate {
            Some(candidate)
        } else {
            None
        }
    }
}
