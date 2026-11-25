//! Protocol version and version-range negotiation (spec §7.6).

#[doc = include_str!("../../docs/api/message/protocol_version.doc/ProtocolVersion.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolVersion {
    /// Major version; a change here is a breaking wire change.
    pub major: u16,
    /// Minor version; a higher value is additive and backward compatible.
    pub minor: u16,
}

impl ProtocolVersion {
    /// The protocol version implemented by this crate.
    pub const CURRENT: Self = Self { major: 1, minor: 0 };

    /// Creates a protocol version.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    #[doc = include_str!("../../docs/api/message/protocol_version.doc/is_compatible_with.md")]
    #[must_use]
    pub const fn is_compatible_with(self, other: Self) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

#[cfg(test)]
mod tests {
    use super::super::VersionRange;
    use super::*;

    #[test]
    fn current_is_one_zero() {
        assert_eq!(ProtocolVersion::CURRENT, ProtocolVersion::new(1, 0));
    }

    #[test]
    fn compatible_same_major_higher_or_equal_minor() {
        let server = ProtocolVersion::new(1, 3);
        assert!(server.is_compatible_with(ProtocolVersion::new(1, 0)));
        assert!(server.is_compatible_with(ProtocolVersion::new(1, 3)));
    }

    #[test]
    fn incompatible_lower_minor_or_wrong_major() {
        let server = ProtocolVersion::new(1, 1);
        // Server minor too low for the peer's requirement.
        assert!(!server.is_compatible_with(ProtocolVersion::new(1, 4)));
        // Major mismatch is never compatible.
        assert!(!server.is_compatible_with(ProtocolVersion::new(2, 0)));
    }

    #[test]
    fn range_contains() {
        let r = VersionRange::new(ProtocolVersion::new(1, 0), ProtocolVersion::new(1, 5));
        assert!(r.contains(ProtocolVersion::new(1, 3)));
        assert!(!r.contains(ProtocolVersion::new(1, 6)));
        assert!(!r.contains(ProtocolVersion::new(2, 0)));
    }

    #[test]
    fn negotiate_picks_highest_common() {
        let server = VersionRange::new(ProtocolVersion::new(1, 0), ProtocolVersion::new(1, 5));
        let client = VersionRange::new(ProtocolVersion::new(1, 2), ProtocolVersion::new(2, 0));
        assert_eq!(server.negotiate(client), Some(ProtocolVersion::new(1, 5)));
    }

    #[test]
    fn negotiate_bounded_by_client_max() {
        let server = VersionRange::new(ProtocolVersion::new(1, 0), ProtocolVersion::new(1, 9));
        let client = VersionRange::new(ProtocolVersion::new(1, 0), ProtocolVersion::new(1, 4));
        assert_eq!(server.negotiate(client), Some(ProtocolVersion::new(1, 4)));
    }

    #[test]
    fn negotiate_major_mismatch_is_none() {
        let server = VersionRange::exact(ProtocolVersion::new(1, 0));
        let client = VersionRange::exact(ProtocolVersion::new(2, 0));
        assert_eq!(server.negotiate(client), None);
    }

    #[test]
    fn negotiate_is_symmetric() {
        let a = VersionRange::new(ProtocolVersion::new(1, 0), ProtocolVersion::new(1, 5));
        let b = VersionRange::new(ProtocolVersion::new(1, 2), ProtocolVersion::new(1, 8));
        assert_eq!(a.negotiate(b), b.negotiate(a));
        assert_eq!(a.negotiate(b), Some(ProtocolVersion::new(1, 5)));
    }
}
