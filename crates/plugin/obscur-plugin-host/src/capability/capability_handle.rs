use std::{fmt, sync::Arc};

/// An opaque, non-cloneable authorization for a plugin service.
pub struct CapabilityHandle {
    issuer: Arc<()>,
    pub(super) id: u64,
}

impl CapabilityHandle {
    pub(super) const fn new(issuer: Arc<()>, id: u64) -> Self {
        Self { issuer, id }
    }

    pub(super) fn belongs_to(&self, issuer: &Arc<()>) -> bool {
        Arc::ptr_eq(issuer, &self.issuer)
    }
}

impl fmt::Debug for CapabilityHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CapabilityHandle").finish_non_exhaustive()
    }
}
