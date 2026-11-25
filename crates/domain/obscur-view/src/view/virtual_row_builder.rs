//! Row construction for compiled Rust virtual lists.

use std::{fmt, sync::Arc};

use super::ViewFragment;

#[derive(Clone)]
/// An owned row factory shared by clones of a virtual-list marker. Equality
/// compares the identity of the factory because captured values are opaque.
pub(super) struct VirtualRowBuilder(Arc<dyn Fn(usize) -> ViewFragment + Send + Sync>);

impl VirtualRowBuilder {
    pub(super) fn new(build: impl Fn(usize) -> ViewFragment + Send + Sync + 'static) -> Self {
        Self(Arc::new(build))
    }

    pub(super) fn build(&self, index: usize) -> ViewFragment {
        (self.0)(index)
    }
}

impl fmt::Debug for VirtualRowBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("VirtualRowBuilder")
    }
}

impl PartialEq for VirtualRowBuilder {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
