#![doc = include_str!("unmount_context.doc/_module.md")]

use core::ops::{Deref, DerefMut};

use crate::component::ComponentContext;

#[doc = include_str!("unmount_context.doc/UnmountContext.md")]
#[derive(Debug)]
pub struct UnmountContext<'ctx, 'scope> {
    context: &'ctx mut ComponentContext<'scope>,
}

impl<'ctx, 'scope> UnmountContext<'ctx, 'scope> {
    #[doc = include_str!("unmount_context.doc/new.md")]
    #[must_use]
    pub fn new(context: &'ctx mut ComponentContext<'scope>) -> Self {
        Self { context }
    }

    #[doc = include_str!("unmount_context.doc/context.md")]
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
    use crate::state::{SignalGenerations, SubscriptionScope};

    #[test]
    fn forwards_to_context() {
        let mut scope = SubscriptionScope::new();
        let mut flag = false;
        let signals = SignalGenerations::new();
        let mut ctx =
            ComponentContext::new(&mut scope, LifecycleState::Unmounted, &mut flag, &signals);
        let mut unmount = UnmountContext::new(&mut ctx);
        assert_eq!(unmount.context().phase(), LifecycleState::Unmounted);
    }
}
