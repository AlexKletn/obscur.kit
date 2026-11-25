#![doc = include_str!("../../docs/api/network/network_request.doc/_module.md")]

use std::sync::Arc;

use super::HttpMethod;

#[doc = include_str!("../../docs/api/network/network_request.doc/NetworkRequest.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkRequest {
    method: HttpMethod,
    url: Box<str>,
    headers: Vec<(Box<str>, Box<str>)>,
    body: Option<Arc<[u8]>>,
}

impl NetworkRequest {
    /// Creates a bodyless request with `method` targeting `url` (spec §5.15).
    #[must_use]
    pub fn new(method: HttpMethod, url: impl Into<Box<str>>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Creates a [`HttpMethod::Get`] request for `url` (spec §5.15).
    #[must_use]
    pub fn get(url: impl Into<Box<str>>) -> Self {
        Self::new(HttpMethod::Get, url)
    }

    /// Creates a [`HttpMethod::Post`] request for `url` carrying `body`
    /// (spec §5.15).
    #[must_use]
    pub fn post(url: impl Into<Box<str>>, body: impl Into<Arc<[u8]>>) -> Self {
        Self::new(HttpMethod::Post, url).with_body(body)
    }

    #[doc = include_str!("../../docs/api/network/network_request.doc/header.md")]
    #[must_use]
    pub fn header(mut self, name: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Attaches `body` to the request, replacing any existing body (fluent;
    /// spec §5.15).
    #[must_use]
    pub fn with_body(mut self, body: impl Into<Arc<[u8]>>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Returns the request method (spec §5.15).
    #[must_use]
    pub const fn method(&self) -> HttpMethod {
        self.method
    }

    /// Returns the target URL (spec §5.15).
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the ordered header name/value pairs (spec §5.15).
    #[must_use]
    pub fn headers(&self) -> &[(Box<str>, Box<str>)] {
        &self.headers
    }

    /// Returns the request body bytes, or `None` when the request has no body
    /// (spec §5.15).
    #[must_use]
    pub fn body(&self) -> Option<&[u8]> {
        self.body.as_deref()
    }

    #[doc = include_str!("../../docs/api/network/network_request.doc/header_value.md")]
    #[must_use]
    pub fn header_value(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_builds_bodyless_request() {
        let request = NetworkRequest::get("https://example.com/data");
        assert_eq!(request.method(), HttpMethod::Get);
        assert_eq!(request.url(), "https://example.com/data");
        assert_eq!(request.body(), None);
        assert!(request.headers().is_empty());
    }

    #[test]
    fn post_carries_body() {
        let request = NetworkRequest::post("https://example.com/submit", &b"payload"[..]);
        assert_eq!(request.method(), HttpMethod::Post);
        assert_eq!(request.body(), Some(&b"payload"[..]));
    }

    #[test]
    fn with_body_replaces_body() {
        let request = NetworkRequest::new(HttpMethod::Put, "u")
            .with_body(&b"first"[..])
            .with_body(&b"second"[..]);
        assert_eq!(request.body(), Some(&b"second"[..]));
    }

    #[test]
    fn headers_are_ordered_and_appended() {
        let request = NetworkRequest::get("u")
            .header("Accept", "text/plain")
            .header("Accept", "text/html");
        let names: Vec<&str> = request
            .headers()
            .iter()
            .map(|(_, value)| value.as_ref())
            .collect();
        assert_eq!(names, ["text/plain", "text/html"]);
    }

    #[test]
    fn header_value_is_case_insensitive_first_match() {
        let request = NetworkRequest::get("u")
            .header("Content-Type", "application/json")
            .header("content-type", "text/plain");
        assert_eq!(
            request.header_value("CONTENT-TYPE"),
            Some("application/json")
        );
        assert_eq!(request.header_value("missing"), None);
    }
}
