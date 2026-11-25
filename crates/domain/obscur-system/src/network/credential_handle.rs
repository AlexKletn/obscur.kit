#![doc = include_str!("../../docs/api/network/credential_handle.doc/_module.md")]

#[doc = include_str!("../../docs/api/network/credential_handle.doc/CredentialHandle.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialHandle {
    scheme: Box<str>,
    reference: Box<str>,
}

impl CredentialHandle {
    #[doc = include_str!("../../docs/api/network/credential_handle.doc/new.md")]
    #[must_use]
    pub fn new(scheme: impl Into<Box<str>>, reference: impl Into<Box<str>>) -> Self {
        Self {
            scheme: scheme.into(),
            reference: reference.into(),
        }
    }

    /// Creates a `"Bearer"`-scheme handle referring to the opaque host-side
    /// `reference` (spec §5.15).
    #[must_use]
    pub fn bearer(reference: impl Into<Box<str>>) -> Self {
        Self::new("Bearer", reference)
    }

    /// Returns the authentication scheme (spec §5.15).
    #[must_use]
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// Returns the opaque host-side reference — **not** the secret (spec §5.15,
    /// §7.12).
    #[must_use]
    pub fn reference(&self) -> &str {
        &self.reference
    }

    #[doc = include_str!("../../docs/api/network/credential_handle.doc/redacted_debug.md")]
    #[must_use]
    pub fn redacted_debug(&self) -> String {
        format!("CredentialHandle({}, <redacted>)", self.scheme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_sets_scheme_and_reference() {
        let handle = CredentialHandle::bearer("host-token-id-42");
        assert_eq!(handle.scheme(), "Bearer");
        assert_eq!(handle.reference(), "host-token-id-42");
    }

    #[test]
    fn new_sets_arbitrary_scheme() {
        let handle = CredentialHandle::new("Basic", "creds-ref");
        assert_eq!(handle.scheme(), "Basic");
        assert_eq!(handle.reference(), "creds-ref");
    }

    #[test]
    fn redacted_debug_hides_the_reference() {
        let handle = CredentialHandle::bearer("super-secret-handle");
        let redacted = handle.redacted_debug();
        assert_eq!(redacted, "CredentialHandle(Bearer, <redacted>)");
        assert!(!redacted.contains("super-secret-handle"));
    }
}
