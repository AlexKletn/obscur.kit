use super::ServiceSessionRevoker;

pub(super) struct PendingDelivery(pub(super) Option<ServiceSessionRevoker>);

impl Drop for PendingDelivery {
    fn drop(&mut self) {
        if let Some(revoker) = &self.0 {
            revoker.revoke();
        }
    }
}
