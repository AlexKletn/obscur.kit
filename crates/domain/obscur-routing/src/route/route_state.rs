//! Opaque typed state attached to a navigation entry (spec §6.13).

#[doc = include_str!("../../docs/api/route/route_state.doc/RouteState.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteState {
    /// The application-chosen type tag identifying the payload's shape.
    kind: Box<str>,
    /// The application's serialized state blob, opaque to the toolkit.
    payload: Box<str>,
}

impl RouteState {
    /// Creates an empty state with an empty kind and payload.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates a state from an application `kind` tag and serialized `payload`.
    #[must_use]
    pub fn new(kind: impl Into<Box<str>>, payload: impl Into<Box<str>>) -> Self {
        Self {
            kind: kind.into(),
            payload: payload.into(),
        }
    }

    /// Returns the application-chosen type tag.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Returns the serialized payload.
    #[must_use]
    pub fn payload(&self) -> &str {
        &self.payload
    }

    /// Returns whether both the kind and payload are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.kind.is_empty() && self.payload.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_empty() {
        let state = RouteState::empty();
        assert_eq!(state.kind(), "");
        assert_eq!(state.payload(), "");
        assert!(state.is_empty());
        assert_eq!(state, RouteState::default());
    }

    #[test]
    fn new_carries_kind_and_payload() {
        let state = RouteState::new("scroll", "{\"y\":120}");
        assert_eq!(state.kind(), "scroll");
        assert_eq!(state.payload(), "{\"y\":120}");
        assert!(!state.is_empty());
    }

    #[test]
    fn kind_only_is_not_empty() {
        assert!(!RouteState::new("scroll", "").is_empty());
        assert!(!RouteState::new("", "blob").is_empty());
    }
}
