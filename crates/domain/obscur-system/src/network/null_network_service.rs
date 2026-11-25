use obscur_capability::SupportLevel;

use super::{NetworkError, NetworkRequest, NetworkResponse, NetworkService};

#[doc = include_str!("../../docs/api/network/network_service.doc/NullNetworkService.md")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_service_fetch_is_unsupported() {
        let mut service = NullNetworkService::new();
        assert_eq!(
            service.fetch(NetworkRequest::get("https://example.com")),
            Err(NetworkError::Unsupported)
        );
        assert_eq!(service.availability(), SupportLevel::Unavailable);
    }
}
