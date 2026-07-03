//! `UnmountContext`: the per-hook context handed to `on_unmount`.

use core::ops::{Deref, DerefMut};

use crate::component::ComponentContext;

/// The context passed to
/// [`Component::on_unmount`](crate::component::Component::on_unmount) as an
/// instance is torn down (spec §2.1).
///
/// It borrows the underlying [`ComponentContext`] so the hook can perform any
/// last read of its subscription scope or phase. It carries no extra data: the
/// actual teardown is *automatic*. Dropping the component's
/// [`SubscriptionScope`](crate::state::SubscriptionScope) releases every scoped
/// subscription, and dropping its [`LocalState`](crate::state::LocalState)
/// fields releases its local view state. This hook exists only for side-effects
/// the author wants to run *before* that automatic cleanup happens; it is not
/// required for correctness.
///
/// # Borrows
///
/// `UnmountContext<'ctx, 'scope>` borrows a `ComponentContext<'scope>` mutably
/// for `'ctx`, using two lifetimes for the same invariance reason as
/// [`MountContext`](crate::lifecycle::MountContext).
#[derive(Debug)]
pub struct UnmountContext<'ctx, 'scope> {
    context: &'ctx mut ComponentContext<'scope>,
}

impl<'ctx, 'scope> UnmountContext<'ctx, 'scope> {
    /// Wraps a borrowed [`ComponentContext`] for the unmount hook.
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

impl<'scope> Deref for UnmountContext<'_, 'scope> {
    type Target = ComponentContext<'scope>;

    fn deref(&self) -> &Self::Target {
        &*self.context
    }
}

impl DerefMut for UnmountContext<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.context
    }
}

#[cfg(test)]
mod tests {
    use super::UnmountContext;
    use crate::component::ComponentContext;
    use crate::lifecycle::LifecycleState;
    use crate::state::SubscriptionScope;

    #[test]
    fn forwards_to_context() {
        let mut scope = SubscriptionScope::new();
        let mut flag = false;
        let mut ctx = ComponentContext::new(&mut scope, LifecycleState::Unmounted, &mut flag);
        let mut unmount = UnmountContext::new(&mut ctx);
        assert_eq!(unmount.context().phase(), LifecycleState::Unmounted);
    }
}
