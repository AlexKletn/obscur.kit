//! Load lifecycle state of a resource (spec §5.15 loading/ready/failed).

use crate::state::{ResourceError, ResourceMetadata, ResourceVersion};

/// The load lifecycle state of a resource (spec §5.15).
///
/// Models the mutually exclusive stages a resource passes through as it is
/// requested, loaded/decoded/compiled, and either becomes usable or fails:
/// *pending* → *loading* → *ready* or *failed*. This is pure data; state
/// transitions are driven by the [`crate::hub`] `ResourceHub` (a later sub-step),
/// not by this type. A [`ResourceState::Ready`] or [`ResourceState::Failed`]
/// carries a [`ResourceVersion`] so a newer reload can supersede it.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceState {
    /// The resource has been requested but loading has not started.
    Pending,
    /// The resource is in flight. `progress` is `0.0..=1.0` when known (spec
    /// §5.16 progressive decode), or `None` when indeterminate.
    Loading {
        /// Fractional load progress in `0.0..=1.0`, or `None` if indeterminate.
        progress: Option<f32>,
    },
    /// The resource is usable, carrying the version it was loaded at and its
    /// descriptive metadata (spec §5.15 ready state).
    Ready {
        /// The version this ready result was produced at.
        version: ResourceVersion,
        /// Descriptive metadata about the loaded resource.
        metadata: ResourceMetadata,
    },
    /// The resource failed at a given version, so a newer reload can supersede
    /// it (spec §5.15 failed state).
    Failed {
        /// Why the resource failed.
        error: ResourceError,
        /// The version this failure was produced at.
        version: ResourceVersion,
    },
}

impl ResourceState {
    /// A [`ResourceState::Pending`] state.
    #[must_use]
    pub const fn pending() -> Self {
        Self::Pending
    }

    /// A [`ResourceState::Loading`] state with the given optional progress.
    #[must_use]
    pub const fn loading(progress: Option<f32>) -> Self {
        Self::Loading { progress }
    }

    /// A [`ResourceState::Ready`] state at the given version, carrying metadata.
    #[must_use]
    pub const fn ready(version: ResourceVersion, metadata: ResourceMetadata) -> Self {
        Self::Ready { version, metadata }
    }

    /// A [`ResourceState::Failed`] state with the given error and version.
    #[must_use]
    pub const fn failed(error: ResourceError, version: ResourceVersion) -> Self {
        Self::Failed { error, version }
    }

    /// Returns whether the resource is ready to use.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    /// Returns whether the resource failed to load.
    #[must_use]
    pub const fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    /// Returns whether the resource is currently loading.
    #[must_use]
    pub const fn is_loading(&self) -> bool {
        matches!(self, Self::Loading { .. })
    }

    /// Returns whether the resource has reached a terminal state
    /// ([`ResourceState::Ready`] or [`ResourceState::Failed`]).
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Ready { .. } | Self::Failed { .. })
    }

    /// Returns the version this state was produced at.
    ///
    /// [`ResourceState::Ready`] and [`ResourceState::Failed`] carry one;
    /// [`ResourceState::Pending`] and [`ResourceState::Loading`] return `None`.
    #[must_use]
    pub const fn version(&self) -> Option<ResourceVersion> {
        match self {
            Self::Ready { version, .. } | Self::Failed { version, .. } => Some(*version),
            Self::Pending | Self::Loading { .. } => None,
        }
    }

    /// Returns the metadata for a [`ResourceState::Ready`] state, else `None`.
    #[must_use]
    pub const fn metadata(&self) -> Option<&ResourceMetadata> {
        match self {
            Self::Ready { metadata, .. } => Some(metadata),
            _ => None,
        }
    }

    /// Returns the error for a [`ResourceState::Failed`] state, else `None`.
    #[must_use]
    pub const fn error(&self) -> Option<&ResourceError> {
        match self {
            Self::Failed { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready_state() -> ResourceState {
        ResourceState::ready(
            ResourceVersion::initial(),
            ResourceMetadata::new().with_byte_size(64),
        )
    }

    fn failed_state() -> ResourceState {
        ResourceState::failed(ResourceError::NotFound, ResourceVersion::initial().next())
    }

    #[test]
    fn constructors_build_expected_variants() {
        assert_eq!(ResourceState::pending(), ResourceState::Pending);
        assert_eq!(
            ResourceState::loading(Some(0.5)),
            ResourceState::Loading { progress: Some(0.5) }
        );
        assert!(matches!(ready_state(), ResourceState::Ready { .. }));
        assert!(matches!(failed_state(), ResourceState::Failed { .. }));
    }

    #[test]
    fn predicates_across_variants() {
        assert!(!ResourceState::pending().is_loading());
        assert!(ResourceState::loading(None).is_loading());
        assert!(ready_state().is_ready());
        assert!(failed_state().is_failed());

        assert!(!ResourceState::pending().is_terminal());
        assert!(!ResourceState::loading(None).is_terminal());
        assert!(ready_state().is_terminal());
        assert!(failed_state().is_terminal());
    }

    #[test]
    fn version_only_for_terminal_states() {
        assert_eq!(ResourceState::pending().version(), None);
        assert_eq!(ResourceState::loading(None).version(), None);
        assert_eq!(ready_state().version(), Some(ResourceVersion::initial()));
        assert_eq!(
            failed_state().version(),
            Some(ResourceVersion::initial().next())
        );
    }

    #[test]
    fn metadata_only_for_ready() {
        assert_eq!(
            ready_state().metadata().and_then(ResourceMetadata::byte_size),
            Some(64)
        );
        assert!(ResourceState::pending().metadata().is_none());
        assert!(failed_state().metadata().is_none());
    }

    #[test]
    fn error_only_for_failed() {
        assert_eq!(failed_state().error(), Some(&ResourceError::NotFound));
        assert!(ready_state().error().is_none());
        assert!(ResourceState::pending().error().is_none());
    }
}
