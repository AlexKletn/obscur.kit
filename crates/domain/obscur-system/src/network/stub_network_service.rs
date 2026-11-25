use obscur_capability::SupportLevel;

use super::{NetworkError, NetworkRequest, NetworkResponse, NetworkService};

#[doc = include_str!("../../docs/api/network/network_service.doc/StubNetworkService.md")]
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
    use super::super::HttpMethod;
    use super::*;

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
}
