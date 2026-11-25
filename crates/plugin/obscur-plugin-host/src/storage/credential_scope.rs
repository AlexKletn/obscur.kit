/// Exact destination at which a host service may use a credential.
#[derive(Clone, PartialEq, Eq)]
pub struct CredentialScope {
    service: Box<str>,
    operation: Box<str>,
    origin: Box<str>,
}

impl CredentialScope {
    /// Uses identifiers canonicalized by the trusted host service.
    #[must_use]
    pub fn new(
        service: impl Into<Box<str>>,
        operation: impl Into<Box<str>>,
        origin: impl Into<Box<str>>,
    ) -> Self {
        Self {
            service: service.into(),
            operation: operation.into(),
            origin: origin.into(),
        }
    }
}
