#![doc = include_str!("resource_state.doc/_module.md")]

use crate::state::{ResourceError, ResourceMetadata, ResourceVersion};

#[doc = include_str!("resource_state.doc/ResourceState.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceState {
    #[doc = include_str!("resource_state.doc/Pending.md")]
    Pending,
    #[doc = include_str!("resource_state.doc/Loading.md")]
    Loading {
        #[doc = include_str!("resource_state.doc/progress.md")]
        progress: Option<f32>,
    },
    #[doc = include_str!("resource_state.doc/Ready.md")]
    Ready {
        #[doc = include_str!("resource_state.doc/version.md")]
        version: ResourceVersion,
        #[doc = include_str!("resource_state.doc/metadata.md")]
        metadata: ResourceMetadata,
    },
    #[doc = include_str!("resource_state.doc/Failed.md")]
    Failed {
        #[doc = include_str!("resource_state.doc/error.md")]
        error: ResourceError,
        #[doc = include_str!("resource_state.doc/version_2.md")]
        version: ResourceVersion,
    },
}

impl ResourceState {
    #[doc = include_str!("resource_state.doc/pending.md")]
    #[must_use]
    pub const fn pending() -> Self {
        Self::Pending
    }

    #[doc = include_str!("resource_state.doc/loading.md")]
    #[must_use]
    pub const fn loading(progress: Option<f32>) -> Self {
        Self::Loading { progress }
    }

    #[doc = include_str!("resource_state.doc/ready.md")]
    #[must_use]
    pub const fn ready(version: ResourceVersion, metadata: ResourceMetadata) -> Self {
        Self::Ready { version, metadata }
    }

    #[doc = include_str!("resource_state.doc/failed.md")]
    #[must_use]
    pub const fn failed(error: ResourceError, version: ResourceVersion) -> Self {
        Self::Failed { error, version }
    }

    #[doc = include_str!("resource_state.doc/is_ready.md")]
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    #[doc = include_str!("resource_state.doc/is_failed.md")]
    #[must_use]
    pub const fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    #[doc = include_str!("resource_state.doc/is_loading.md")]
    #[must_use]
    pub const fn is_loading(&self) -> bool {
        matches!(self, Self::Loading { .. })
    }

    #[doc = include_str!("resource_state.doc/is_terminal.md")]
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Ready { .. } | Self::Failed { .. })
    }

    #[doc = include_str!("resource_state.doc/version_3.md")]
    #[must_use]
    pub const fn version(&self) -> Option<ResourceVersion> {
        match self {
            Self::Ready { version, .. } | Self::Failed { version, .. } => Some(*version),
            Self::Pending | Self::Loading { .. } => None,
        }
    }

    #[doc = include_str!("resource_state.doc/metadata_2.md")]
    #[must_use]
    pub const fn metadata(&self) -> Option<&ResourceMetadata> {
        match self {
            Self::Ready { metadata, .. } => Some(metadata),
            _ => None,
        }
    }

    #[doc = include_str!("resource_state.doc/error_2.md")]
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
            ResourceState::Loading {
                progress: Some(0.5)
            }
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
            ready_state()
                .metadata()
                .and_then(ResourceMetadata::byte_size),
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
