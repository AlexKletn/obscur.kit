//! The host-provided port for satisfying network fetches (spec §5.15).
//!
//! The toolkit performs **no network I/O of its own**: [`NetworkService`] is a
//! port a host/application implements to resolve a
//! [`NetworkRequest`](crate::network::NetworkRequest) into a
//! [`NetworkResponse`](crate::network::NetworkResponse) through its own loader
//! (spec §5.15). The honest kit default — [`NullNetworkService`] — provides
//! nothing, enforcing the no-network policy; [`StubNetworkService`] is a
//! deterministic test host with canned routes.

use obscur_capability::SupportLevel;

use crate::network::{CredentialHandle, NetworkRequest, NetworkResponse};

/// Why a [`NetworkService`] fetch failed (spec §5.15).
///
/// Each variant is a pure, backend-agnostic value; the informational variants
/// carry a human-readable detail. This type implements [`std::error::Error`] so
/// it composes with the standard error ecosystem while remaining I/O-free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkError {
    /// This service performs no network access at all — the honest kit default
    /// (spec §5.15).
    Unsupported,
    /// The host declined the fetch by policy (permission, sandbox, allowlist).
    Denied,
    /// The fetch did not complete within the host's time budget.
    Timeout,
    /// The URL was malformed or unroutable; the payload echoes the offending
    /// URL.
    InvalidUrl(Box<str>),
    /// An underlying transport failed; the payload describes the failure.
    Transport(Box<str>),
}

impl NetworkError {
    /// Returns whether this is the policy-refusal [`Denied`](Self::Denied)
    /// variant (spec §5.15).
    #[must_use]
    pub const fn is_denied(&self) -> bool {
        matches!(self, Self::Denied)
    }

    /// Returns whether this is the no-network [`Unsupported`](Self::Unsupported)
    /// variant (spec §5.15).
    #[must_use]
    pub const fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported)
    }
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("network access is unsupported"),
            Self::Denied => f.write_str("network access denied"),
            Self::Timeout => f.write_str("network request timed out"),
            Self::InvalidUrl(url) => write!(f, "invalid url: {url}"),
            Self::Transport(reason) => write!(f, "network transport error: {reason}"),
        }
    }
}

impl std::error::Error for NetworkError {}

/// The host-provided port for satisfying network fetches (spec §5.15).
///
/// The kit opens no sockets: an adapter implements this once per host (a real
/// HTTP client, a sandboxed loader, a test double) and the domain drives it as
/// `Box<dyn NetworkService>`. The trait is therefore deliberately
/// **object-safe** — no generic methods and no `Self`-returning methods.
/// Fetches take `&mut self` because a real host mutates connection/pool state
/// as it works; failures are reported through [`NetworkError`] rather than
/// panics.
pub trait NetworkService {
    /// Fetches `request` and returns the host's [`NetworkResponse`]
    /// (spec §5.15).
    ///
    /// A service that performs no network access returns
    /// [`NetworkError::Unsupported`] rather than panicking.
    fn fetch(&mut self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError>;

    /// Fetches `request`, authenticating with `credential` (spec §5.15).
    ///
    /// The default implementation simply delegates to [`Self::fetch`]: the
    /// [`CredentialHandle`] carries only an opaque reference (§7.12), so the
    /// host is responsible for resolving it to the real secret and injecting
    /// the actual `Authorization` header when it performs the fetch — the kit
    /// never holds the secret. An adapter may override this to thread the
    /// credential through its own authentication flow.
    fn fetch_with_credential(
        &mut self,
        request: NetworkRequest,
        credential: &CredentialHandle,
    ) -> Result<NetworkResponse, NetworkError> {
        let _ = credential;
        self.fetch(request)
    }

    /// Reports the honest [`SupportLevel`] at which this service offers network
    /// access (spec §5.15).
    #[must_use]
    fn availability(&self) -> SupportLevel;
}

/// The honest kit default: a [`NetworkService`] that performs **no** network
/// access (spec §5.15).
///
/// `NullNetworkService` enforces the §5.15 policy that the toolkit does no
/// network I/O by itself: every [`fetch`](NetworkService::fetch) returns
/// [`NetworkError::Unsupported`] and [`availability`](NetworkService::availability)
/// reports [`SupportLevel::Unavailable`]. Its refusals are correct behavior,
/// not stubs waiting to be filled in — a host that wants network access
/// supplies its own adapter.
#[derive(Debug, Default, Clone)]
pub struct NullNetworkService;

impl NullNetworkService {
    /// Creates the no-network default service (spec §5.15).
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl NetworkService for NullNetworkService {
    fn fetch(&mut self, _request: NetworkRequest) -> Result<NetworkResponse, NetworkError> {
        // Honest: the kit performs no network access on its own (spec §5.15).
        Err(NetworkError::Unsupported)
    }

    fn availability(&self) -> SupportLevel {
        SupportLevel::Unavailable
    }
}

/// A deterministic test host [`NetworkService`] backed by canned routes
/// (spec §5.15).
///
/// `StubNetworkService` maps an exact request URL to a pre-built
/// [`NetworkResponse`] and records every request it receives, with no real
/// I/O, clock, or randomness. A [`fetch`](NetworkService::fetch) returns a
/// clone of the response routed for the request's URL, or
/// [`NetworkError::InvalidUrl`] when no route matches. It reports
/// [`SupportLevel::Emulated`] because it emulates a host loader rather than
/// reaching a real network.
#[derive(Debug, Default, Clone)]
pub struct StubNetworkService {
    /// Exact-URL to canned-response routes, consulted in order.
    routes: Vec<(Box<str>, NetworkResponse)>,
    /// Every request received, in arrival order — inspectable by tests.
    requests: Vec<NetworkRequest>,
}

impl StubNetworkService {
    /// Creates an empty stub host with no routes and no recorded requests
    /// (spec §5.15).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a canned `response` for the exact `url`, returning `self`
    /// (fluent; spec §5.15).
    #[must_use]
    pub fn route(mut self, url: impl Into<Box<str>>, response: NetworkResponse) -> Self {
        self.routes.push((url.into(), response));
        self
    }

    /// Returns the requests recorded so far, in arrival order (spec §5.15).
    #[must_use]
    pub fn requests(&self) -> &[NetworkRequest] {
        &self.requests
    }
}

impl NetworkService for StubNetworkService {
    fn fetch(&mut self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError> {
        let matched = self
            .routes
            .iter()
            .find(|(url, _)| url.as_ref() == request.url())
            .map(|(_, response)| response.clone());
        let url = Box::from(request.url());
        self.requests.push(request);
        matched.ok_or(NetworkError::InvalidUrl(url))
    }

    fn availability(&self) -> SupportLevel {
        SupportLevel::Emulated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::HttpMethod;

    /// Compile-time proof the port is object-safe (spec §5.15 replaceable port).
    #[test]
    fn network_service_is_object_safe() {
        let _: Box<dyn NetworkService> = Box::new(NullNetworkService::default());
    }

    #[test]
    fn null_service_fetch_is_unsupported() {
        let mut service = NullNetworkService::new();
        assert_eq!(
            service.fetch(NetworkRequest::get("https://example.com")),
            Err(NetworkError::Unsupported)
        );
        assert_eq!(service.availability(), SupportLevel::Unavailable);
    }

    #[test]
    fn stub_routes_return_canned_response() {
        let mut service = StubNetworkService::new()
            .route("https://example.com/a", NetworkResponse::new(200, &b"A"[..]));
        let response = service
            .fetch(NetworkRequest::get("https://example.com/a"))
            .expect("routed url");
        assert_eq!(response.status(), 200);
        assert_eq!(response.body(), b"A");
        assert_eq!(service.availability(), SupportLevel::Emulated);
    }

    #[test]
    fn stub_records_requests() {
        let mut service = StubNetworkService::new()
            .route("u", NetworkResponse::new(200, &b""[..]));
        let _ = service.fetch(NetworkRequest::get("u"));
        let _ = service.fetch(NetworkRequest::post("u", &b"body"[..]));
        assert_eq!(service.requests().len(), 2);
        assert_eq!(service.requests()[0].method(), HttpMethod::Get);
        assert_eq!(service.requests()[1].method(), HttpMethod::Post);
    }

    #[test]
    fn stub_unknown_url_is_invalid_url() {
        let mut service = StubNetworkService::new();
        assert_eq!(
            service.fetch(NetworkRequest::get("https://ghost")),
            Err(NetworkError::InvalidUrl(Box::from("https://ghost")))
        );
        // The unmatched request is still recorded.
        assert_eq!(service.requests().len(), 1);
    }

    #[test]
    fn fetch_with_credential_delegates_to_fetch() {
        let mut service = StubNetworkService::new()
            .route("u", NetworkResponse::new(200, &b"ok"[..]));
        let credential = CredentialHandle::bearer("host-ref");
        let response = service
            .fetch_with_credential(NetworkRequest::get("u"), &credential)
            .expect("routed url");
        assert_eq!(response.body(), b"ok");
    }

    #[test]
    fn network_error_display_and_predicates() {
        assert_eq!(
            NetworkError::Unsupported.to_string(),
            "network access is unsupported"
        );
        assert_eq!(NetworkError::Denied.to_string(), "network access denied");
        assert_eq!(
            NetworkError::Timeout.to_string(),
            "network request timed out"
        );
        assert_eq!(
            NetworkError::InvalidUrl(Box::from("bad")).to_string(),
            "invalid url: bad"
        );
        assert_eq!(
            NetworkError::Transport(Box::from("reset")).to_string(),
            "network transport error: reset"
        );

        assert!(NetworkError::Denied.is_denied());
        assert!(!NetworkError::Unsupported.is_denied());
        assert!(NetworkError::Unsupported.is_unsupported());
        assert!(!NetworkError::Denied.is_unsupported());

        let _as_error: &dyn std::error::Error = &NetworkError::Timeout;
    }
}
