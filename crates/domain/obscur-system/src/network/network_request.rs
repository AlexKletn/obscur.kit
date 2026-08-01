#![doc = include_str!("network_request.doc/_module.md")]

use std::sync::Arc;

#[doc = include_str!("network_request.doc/HttpMethod.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    #[doc = include_str!("network_request.doc/Get.md")]
    Get,
    #[doc = include_str!("network_request.doc/Head.md")]
    Head,
    #[doc = include_str!("network_request.doc/Post.md")]
    Post,
    #[doc = include_str!("network_request.doc/Put.md")]
    Put,
    #[doc = include_str!("network_request.doc/Delete.md")]
    Delete,
    #[doc = include_str!("network_request.doc/Patch.md")]
    Patch,
}

impl HttpMethod {
    #[doc = include_str!("network_request.doc/is_safe.md")]
    #[must_use]
    pub const fn is_safe(&self) -> bool {
        matches!(self, Self::Get | Self::Head)
    }

    #[doc = include_str!("network_request.doc/has_body.md")]
    #[must_use]
    pub const fn has_body(&self) -> bool {
        matches!(self, Self::Post | Self::Put | Self::Patch)
    }
}

#[doc = include_str!("network_request.doc/NetworkRequest.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkRequest {
    method: HttpMethod,
    url: Box<str>,
    headers: Vec<(Box<str>, Box<str>)>,
    body: Option<Arc<[u8]>>,
}

impl NetworkRequest {
    #[doc = include_str!("network_request.doc/new.md")]
    #[must_use]
    pub fn new(method: HttpMethod, url: impl Into<Box<str>>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    #[doc = include_str!("network_request.doc/get.md")]
    #[must_use]
    pub fn get(url: impl Into<Box<str>>) -> Self {
        Self::new(HttpMethod::Get, url)
    }

    #[doc = include_str!("network_request.doc/post.md")]
    #[must_use]
    pub fn post(url: impl Into<Box<str>>, body: impl Into<Arc<[u8]>>) -> Self {
        Self::new(HttpMethod::Post, url).with_body(body)
    }

    #[doc = include_str!("network_request.doc/header.md")]
    #[must_use]
    pub fn header(mut self, name: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    #[doc = include_str!("network_request.doc/with_body.md")]
    #[must_use]
    pub fn with_body(mut self, body: impl Into<Arc<[u8]>>) -> Self {
        self.body = Some(body.into());
        self
    }

    #[doc = include_str!("network_request.doc/method.md")]
    #[must_use]
    pub const fn method(&self) -> HttpMethod {
        self.method
    }

    #[doc = include_str!("network_request.doc/url.md")]
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    #[doc = include_str!("network_request.doc/headers.md")]
    #[must_use]
    pub fn headers(&self) -> &[(Box<str>, Box<str>)] {
        &self.headers
    }

    #[doc = include_str!("network_request.doc/body.md")]
    #[must_use]
    pub fn body(&self) -> Option<&[u8]> {
        self.body.as_deref()
    }

    #[doc = include_str!("network_request.doc/header_value.md")]
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
