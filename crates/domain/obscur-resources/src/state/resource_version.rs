#![doc = include_str!("resource_version.doc/_module.md")]

use obscur_identity::VersionStamp;

#[doc = include_str!("resource_version.doc/ResourceVersion.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceVersion {
    #[doc = include_str!("resource_version.doc/stamp.md")]
    pub stamp: VersionStamp,
}

impl ResourceVersion {
    #[doc = include_str!("resource_version.doc/initial.md")]
    #[must_use]
    pub const fn initial() -> Self {
        Self {
            stamp: VersionStamp::from_raw(1),
        }
    }

    #[doc = include_str!("resource_version.doc/next.md")]
    #[must_use]
    pub const fn next(self) -> Self {
        Self {
            stamp: self.stamp.bump(),
        }
    }

    #[doc = include_str!("resource_version.doc/stamp_2.md")]
    #[must_use]
    pub const fn stamp(self) -> VersionStamp {
        self.stamp
    }

    #[doc = include_str!("resource_version.doc/is_newer_than.md")]
    #[must_use]
    pub fn is_newer_than(self, other: Self) -> bool {
        self.stamp > other.stamp
    }
}

impl Default for ResourceVersion {
    #[doc = include_str!("resource_version.doc/default.md")]
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
