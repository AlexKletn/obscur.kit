//! A request to schedule a frame: [`FrameRequest`].

use super::FrameReason;

/// A request to schedule a frame for a given [`FrameReason`].
///
/// A request records only *why* a frame is wanted. Turning a stream of requests
/// into actual frames — coalescing several requests into one frame, and ordering
/// them by priority — is the scheduler's job in a later phase; a `FrameRequest`
/// itself carries no timing or priority state.
///
/// The type is a plain value: `Copy`, equatable, and hashable, so pending
/// requests can be deduplicated in a set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameRequest {
    reason: FrameReason,
}

impl FrameRequest {
    /// Creates a request to schedule a frame for `reason`.
    #[must_use]
    pub const fn new(reason: FrameReason) -> Self {
        Self { reason }
    }

    /// Returns the reason this frame is being requested for.
    #[must_use]
    pub const fn reason(self) -> FrameReason {
        self.reason
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_reason() {
        let request = FrameRequest::new(FrameReason::Input);
        assert_eq!(request.reason(), FrameReason::Input);
    }

    #[test]
    fn equal_requests_compare_and_hash_equal() {
        use std::collections::HashSet;
        let a = FrameRequest::new(FrameReason::Animation);
        let b = FrameRequest::new(FrameReason::Animation);
        let c = FrameRequest::new(FrameReason::WindowChange);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut set = HashSet::new();
        set.insert(a);
        // A duplicate reason does not create a second entry.
        assert!(!set.insert(b));
        assert!(set.insert(c));
        assert_eq!(set.len(), 2);
    }
}
