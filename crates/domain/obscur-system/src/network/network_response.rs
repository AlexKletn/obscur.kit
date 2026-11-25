#![doc = include_str!("../../docs/api/network/network_response.doc/_module.md")]

use std::sync::Arc;

#[doc = include_str!("../../docs/api/network/network_response.doc/NetworkResponse.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkResponse {
    status: u16,
    headers: Vec<(Box<str>, Box<str>)>,
    body: Arc<[u8]>,
}

impl NetworkResponse {
    /// Creates a response with HTTP `status` and `body`, and no headers
    /// (spec §5.15).
    #[must_use]
    pub fn new(status: u16, body: impl Into<Arc<[u8]>>) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: body.into(),
        }
    }

    /// Appends a `name: value` response header, preserving order (fluent;
    /// spec §5.15).
    #[must_use]
    pub fn header(mut self, name: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Returns the HTTP status code (spec §5.15).
    #[must_use]
    pub const fn status(&self) -> u16 {
        self.status
    }

    /// Returns the ordered response header name/value pairs (spec §5.15).
    #[must_use]
    pub fn headers(&self) -> &[(Box<str>, Box<str>)] {
        &self.headers
    }

    /// Returns the response body bytes (spec §5.15).
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    #[doc = include_str!("../../docs/api/network/network_response.doc/header_value.md")]
    #[must_use]
    pub fn header_value(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_ref())
    }

    /// Returns whether the status is a 2xx success code (spec §5.15).
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self.status, 200..=299)
    }

    /// Returns whether the status is a 4xx client-error code (spec §5.15).
    #[must_use]
    pub const fn is_client_error(&self) -> bool {
        matches!(self.status, 400..=499)
    }

    /// Returns whether the status is a 5xx server-error code (spec §5.15).
    #[must_use]
    pub const fn is_server_error(&self) -> bool {
        matches!(self.status, 500..=599)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_status_and_body() {
        let response = NetworkResponse::new(200, &b"ok"[..]);
        assert_eq!(response.status(), 200);
        assert_eq!(response.body(), b"ok");
        assert!(response.headers().is_empty());
    }

    #[test]
    fn status_classifiers() {
        assert!(NetworkResponse::new(200, &b""[..]).is_success());
        assert!(NetworkResponse::new(204, &b""[..]).is_success());
        assert!(!NetworkResponse::new(200, &b""[..]).is_client_error());

        assert!(NetworkResponse::new(404, &b""[..]).is_client_error());
        assert!(!NetworkResponse::new(404, &b""[..]).is_success());

        assert!(NetworkResponse::new(500, &b""[..]).is_server_error());
        assert!(!NetworkResponse::new(500, &b""[..]).is_client_error());
    }

    #[test]
    fn header_value_is_case_insensitive() {
        let response = NetworkResponse::new(200, &b""[..])
            .header("Content-Type", "application/json")
            .header("X-Trace", "abc");
        assert_eq!(
            response.header_value("content-type"),
            Some("application/json")
        );
        assert_eq!(response.header_value("X-TRACE"), Some("abc"));
        assert_eq!(response.header_value("absent"), None);
    }
}
