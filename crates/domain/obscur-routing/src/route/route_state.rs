#![doc = include_str!("route_state.doc/_module.md")]

#[doc = include_str!("route_state.doc/RouteState.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteState {
    #[doc = include_str!("route_state.doc/kind.md")]
    kind: Box<str>,
    #[doc = include_str!("route_state.doc/payload.md")]
    payload: Box<str>,
}

impl RouteState {
    #[doc = include_str!("route_state.doc/empty.md")]
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[doc = include_str!("route_state.doc/new.md")]
    #[must_use]
    pub fn new(kind: impl Into<Box<str>>, payload: impl Into<Box<str>>) -> Self {
        Self {
            kind: kind.into(),
            payload: payload.into(),
        }
    }

    #[doc = include_str!("route_state.doc/kind_2.md")]
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    #[doc = include_str!("route_state.doc/payload_2.md")]
    #[must_use]
    pub fn payload(&self) -> &str {
        &self.payload
    }

    #[doc = include_str!("route_state.doc/is_empty.md")]
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
