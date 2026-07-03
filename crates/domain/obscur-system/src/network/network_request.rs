//! An HTTP-like network request a host loader fulfils (spec §5.15).
//!
//! The toolkit performs no network I/O of its own: a [`NetworkRequest`] is a
//! pure, backend-agnostic *description* of a fetch that the host resolves
//! through its own loader (spec §5.15). It carries an HTTP-shaped method, a URL,
//! ordered headers, and an optional body — never a live connection.

use std::sync::Arc;

/// The HTTP method of a [`NetworkRequest`] (spec §5.15).
///
/// Only the common methods a host loader is expected to honor are modeled; the
/// predicates classify a method without the host having to match on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// Retrieve a resource without side effects.
    Get,
    /// Retrieve only the headers of a resource.
    Head,
    /// Submit an entity to a resource.
    Post,
    /// Replace the target resource with the request body.
    Put,
    /// Delete the target resource.
    Delete,
    /// Apply a partial modification to the target resource.
    Patch,
}

impl HttpMethod {
    /// Returns whether this method is *safe* — read-only with no side effects,
    /// i.e. [`Get`](Self::Get) or [`Head`](Self::Head) (spec §5.15).
    #[must_use]
    pub const fn is_safe(&self) -> bool {
        matches!(self, Self::Get | Self::Head)
    }

    /// Returns whether this method conventionally carries a request body, i.e.
    /// [`Post`](Self::Post), [`Put`](Self::Put), or [`Patch`](Self::Patch)
    /// (spec §5.15).
    #[must_use]
    pub const fn has_body(&self) -> bool {
        matches!(self, Self::Post | Self::Put | Self::Patch)
    }
}

/// An HTTP-like request the host loader fulfils on the kit's behalf
/// (spec §5.15).
///
/// The kit itself opens no sockets: it hands this description to a
/// host-provided [`NetworkService`](crate::network::NetworkService), which
/// resolves the URL through its own loader. Headers are kept in insertion order
/// (a [`Vec`] of name/value pairs) because HTTP permits repeated header names
/// and preserves their order; the body is an [`Arc<[u8]>`] so clones are cheap
/// and the bytes can be shared without copying.
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

    /// Appends a `name: value` header, preserving order (fluent; spec §5.15).
    ///
    /// Existing headers are never replaced, so repeated names accumulate in the
    /// order they were added — matching HTTP's multi-valued header semantics.
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

    /// Returns the value of the first header whose name matches `name`
    /// case-insensitively, or `None` (spec §5.15).
    ///
    /// The match is ASCII-case-insensitive because HTTP header names are
    /// case-insensitive; the *first* matching header wins, mirroring how a
    /// well-behaved client reads a single-valued header.
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

    #[test]
    fn method_predicates_classify() {
        assert!(HttpMethod::Get.is_safe());
        assert!(HttpMethod::Head.is_safe());
        assert!(!HttpMethod::Post.is_safe());
        assert!(HttpMethod::Post.has_body());
        assert!(HttpMethod::Put.has_body());
        assert!(HttpMethod::Patch.has_body());
        assert!(!HttpMethod::Get.has_body());
        assert!(!HttpMethod::Delete.has_body());
    }
}
