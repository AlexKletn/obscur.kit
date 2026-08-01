#![doc = include_str!("network_service.doc/_module.md")]

use obscur_capability::SupportLevel;

use crate::network::{CredentialHandle, NetworkRequest, NetworkResponse};

#[doc = include_str!("network_service.doc/NetworkError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkError {
    #[doc = include_str!("network_service.doc/Unsupported.md")]
    Unsupported,
    #[doc = include_str!("network_service.doc/Denied.md")]
    Denied,
    #[doc = include_str!("network_service.doc/Timeout.md")]
    Timeout,
    #[doc = include_str!("network_service.doc/InvalidUrl.md")]
    InvalidUrl(Box<str>),
    #[doc = include_str!("network_service.doc/Transport.md")]
    Transport(Box<str>),
}

impl NetworkError {
    #[doc = include_str!("network_service.doc/is_denied.md")]
    #[must_use]
    pub const fn is_denied(&self) -> bool {
        matches!(self, Self::Denied)
    }

    #[doc = include_str!("network_service.doc/is_unsupported.md")]
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

#[doc = include_str!("network_service.doc/NetworkService.md")]
pub trait NetworkService {
    #[doc = include_str!("network_service.doc/fetch.md")]
    fn fetch(&mut self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError>;

    #[doc = include_str!("network_service.doc/fetch_with_credential.md")]
    fn fetch_with_credential(
        &mut self,
        request: NetworkRequest,
        credential: &CredentialHandle,
    ) -> Result<NetworkResponse, NetworkError> {
        let _ = credential;
        self.fetch(request)
    }

    #[doc = include_str!("network_service.doc/availability.md")]
    #[must_use]
    fn availability(&self) -> SupportLevel;
}

#[doc = include_str!("network_service.doc/NullNetworkService.md")]
#[derive(Debug, Default, Clone)]
pub struct NullNetworkService;

impl NullNetworkService {
    #[doc = include_str!("network_service.doc/new.md")]
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

#[doc = include_str!("network_service.doc/StubNetworkService.md")]
#[derive(Debug, Default, Clone)]
pub struct StubNetworkService {
    #[doc = include_str!("network_service.doc/routes.md")]
    routes: Vec<(Box<str>, NetworkResponse)>,
    #[doc = include_str!("network_service.doc/requests.md")]
    requests: Vec<NetworkRequest>,
}

impl StubNetworkService {
    #[doc = include_str!("network_service.doc/new_2.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("network_service.doc/route.md")]
    #[must_use]
    pub fn route(mut self, url: impl Into<Box<str>>, response: NetworkResponse) -> Self {
        self.routes.push((url.into(), response));
        self
    }

    #[doc = include_str!("network_service.doc/requests_2.md")]
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
        let _: Box<dyn NetworkService> = Box::new(NullNetworkService);
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
        let mut service = StubNetworkService::new().route(
            "https://example.com/a",
            NetworkResponse::new(200, &b"A"[..]),
        );
        let response = service
            .fetch(NetworkRequest::get("https://example.com/a"))
            .expect("routed url");
        assert_eq!(response.status(), 200);
        assert_eq!(response.body(), b"A");
        assert_eq!(service.availability(), SupportLevel::Emulated);
    }

    #[test]
    fn stub_records_requests() {
        let mut service = StubNetworkService::new().route("u", NetworkResponse::new(200, &b""[..]));
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
        let mut service =
            StubNetworkService::new().route("u", NetworkResponse::new(200, &b"ok"[..]));
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

        let _: &dyn std::error::Error = &NetworkError::Timeout;
    }
}
