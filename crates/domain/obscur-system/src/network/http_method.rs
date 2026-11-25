#[doc = include_str!("../../docs/api/network/network_request.doc/HttpMethod.md")]
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

#[cfg(test)]
mod tests {
    use super::HttpMethod;

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
