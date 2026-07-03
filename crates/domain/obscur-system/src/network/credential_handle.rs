//! An opaque, host-resolved reference to a credential — never the secret itself
//! (spec §5.15, §7.12).
//!
//! When a network fetch needs authentication, the kit must never hold the raw
//! secret: sensitive material stays with the host (privacy §7.12). A
//! [`CredentialHandle`] carries only an authentication *scheme* and an opaque
//! *reference* (a host-side handle/id) that the host — and only the host —
//! resolves to the actual secret when it fulfils the request (spec §5.15).

/// An opaque reference to a credential the host resolves, **never** the raw
/// secret (spec §5.15, §7.12).
///
/// The `reference` is not a password, token, or key: it is an opaque handle/id
/// the host maps to the real secret on its own side when performing a fetch.
/// The secret never enters the pure domain, honoring the privacy rule that
/// sensitive material stays with the host (§7.12). Even so the reference is
/// treated as sensitive — use [`Self::redacted_debug`] for logging so the
/// handle never leaks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialHandle {
    scheme: Box<str>,
    reference: Box<str>,
}

impl CredentialHandle {
    /// Creates a handle for authentication `scheme` (e.g. `"Bearer"`,
    /// `"Basic"`) referring to the opaque host-side `reference` (spec §5.15).
    ///
    /// `reference` must be an opaque handle the host resolves — never the raw
    /// secret (§7.12).
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

    /// Returns a log-safe description that names the scheme but **redacts** the
    /// reference, e.g. `"CredentialHandle(Bearer, <redacted>)"` (spec §7.12).
    ///
    /// The reference is opaque but still treated as sensitive, so it is never
    /// emitted here; use this instead of the [`Debug`] representation when
    /// logging so the handle can never leak.
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
