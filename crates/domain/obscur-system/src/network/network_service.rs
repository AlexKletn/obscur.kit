#![doc = include_str!("../../docs/api/network/network_service.doc/_module.md")]

use obscur_capability::SupportLevel;

use crate::network::{CredentialHandle, NetworkRequest, NetworkResponse};

use super::NetworkError;

#[doc = include_str!("../../docs/api/network/network_service.doc/NetworkService.md")]
pub trait NetworkService {
    #[doc = include_str!("../../docs/api/network/network_service.doc/fetch.md")]
    fn fetch(&mut self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError>;

    #[doc = include_str!("../../docs/api/network/network_service.doc/fetch_with_credential.md")]
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

#[cfg(test)]
mod tests {
    use super::super::{NullNetworkService, StubNetworkService};
    use super::*;

    #[test]
    fn network_service_is_object_safe() {
        let _: Box<dyn NetworkService> = Box::new(NullNetworkService);
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
}
