//! Monotonic version of a resource's content (spec §5.15 versioning).

use obscur_identity::VersionStamp;

#[doc = include_str!("../../docs/api/state/resource_version.doc/ResourceVersion.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceVersion {
    /// The underlying version stamp.
    pub stamp: VersionStamp,
}

impl ResourceVersion {
    #[doc = include_str!("../../docs/api/state/resource_version.doc/initial.md")]
    #[must_use]
    pub const fn initial() -> Self {
        Self {
            stamp: VersionStamp::from_raw(1),
        }
    }

    /// Returns the next version, advancing the content version by one (spec
    /// §5.15). Used when a resource is reloaded.
    #[must_use]
    pub const fn next(self) -> Self {
        Self {
            stamp: self.stamp.bump(),
        }
    }

    /// Returns the underlying [`VersionStamp`].
    #[must_use]
    pub const fn stamp(self) -> VersionStamp {
        self.stamp
    }

    /// Returns whether this version is strictly newer than `other` (spec §5.15
    /// stale-job rejection: a job older than the current version is rejected).
    #[must_use]
    pub fn is_newer_than(self, other: Self) -> bool {
        self.stamp > other.stamp
    }
}

impl Default for ResourceVersion {
    /// The default version is [`ResourceVersion::initial`].
    fn default() -> Self {
        Self::initial()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_starts_at_one() {
        assert_eq!(ResourceVersion::initial().stamp().raw(), 1);
        assert_eq!(ResourceVersion::default(), ResourceVersion::initial());
    }

    #[test]
    fn next_advances_by_one() {
        let v = ResourceVersion::initial();
        assert_eq!(v.next().stamp().raw(), 2);
        assert_eq!(v.next().next().stamp().raw(), 3);
    }

    #[test]
    fn ordering_is_monotonic() {
        let v1 = ResourceVersion::initial();
        let v2 = v1.next();
        assert!(v1 < v2);
        assert!(v2 > v1);
    }

    #[test]
    fn is_newer_than_compares_versions() {
        let old = ResourceVersion::initial();
        let new = old.next();
        assert!(new.is_newer_than(old));
        assert!(!old.is_newer_than(new));
        assert!(!old.is_newer_than(old));
    }

    #[test]
    fn stale_job_rejection() {
        // A resource is now at its third version.
        let current = ResourceVersion::initial().next().next();
        // A job issued for the first version completes late; it is stale.
        let stale_job = ResourceVersion::initial();
        assert!(!stale_job.is_newer_than(current));
        // A job for a version at least as new supersedes.
        assert!(current.next().is_newer_than(current));
    }
}
