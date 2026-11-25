//! `MountContext`: the per-hook context handed to `on_mount`.

use core::ops::{Deref, DerefMut};

use crate::component::ComponentContext;

#[doc = include_str!("../../docs/api/lifecycle/mount_context.doc/MountContext.md")]
#[derive(Debug)]
pub struct MountContext<'ctx, 'scope> {
    context: &'ctx mut ComponentContext<'scope>,
}

impl<'ctx, 'scope> MountContext<'ctx, 'scope> {
    /// Wraps a borrowed [`ComponentContext`] for the mount hook.
    #[must_use]
    pub fn new(context: &'ctx mut ComponentContext<'scope>) -> Self {
        Self { context }
    }

    /// Mutable access to the underlying [`ComponentContext`].
    #[must_use]
    pub fn context(&mut self) -> &mut ComponentContext<'scope> {
        self.context
    }
}

impl<'scope> Deref for MountContext<'_, 'scope> {
    type Target = ComponentContext<'scope>;

    fn deref(&self) -> &Self::Target {
        &*self.context
    }
}

impl DerefMut for MountContext<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.context
    }
}

#[cfg(test)]
mod tests {
    use super::MountContext;
    use crate::component::ComponentContext;
    use crate::lifecycle::LifecycleState;
    use crate::state::{SignalGenerations, SubscriptionScope};

    #[test]
    fn forwards_to_context() {
        let mut scope = SubscriptionScope::new();
        let mut flag = false;
        let signals = SignalGenerations::new();
        let mut ctx =
            ComponentContext::new(&mut scope, LifecycleState::Mounted, &mut flag, &signals);
        let mut mount = MountContext::new(&mut ctx);
        assert_eq!(mount.phase(), LifecycleState::Mounted);
        mount.context().request_update();
        assert!(mount.needs_update());
    }
}
