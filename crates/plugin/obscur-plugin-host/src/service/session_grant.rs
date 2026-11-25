use super::{ServiceHandle, ServiceRevoker};

pub(super) struct SessionGrant {
    pub(super) handle: ServiceHandle,
    pub(super) revoker: ServiceRevoker,
}
