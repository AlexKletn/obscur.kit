//! `UpdateContext`: the per-hook context handed to `on_update`.

use core::ops::{Deref, DerefMut};

use crate::component::ComponentContext;
use crate::state::StateGeneration;

/// The context passed to
/// [`Component::on_update`](crate::component::Component::on_update) when an
/// instance's inputs may have changed (spec §2.1, §3.1).
///
/// Besides borrowing the underlying [`ComponentContext`], it carries the
/// previous and next input [`StateGeneration`]s so the hook can early-out when
/// nothing it depends on actually changed. This ties into the spec's
/// "invalidate only changed consumers": [`changed`](Self::changed) is `true`
/// only when the next generation is strictly newer than the previous one.
///
/// # Borrows
///
/// `UpdateContext<'ctx, 'scope>` borrows a `ComponentContext<'scope>` mutably
/// for `'ctx` (two lifetimes for the same invariance reason as
/// [`MountContext`](crate::lifecycle::MountContext)) and additionally owns two
/// `StateGeneration` watermarks by value.
#[derive(Debug)]
pub struct UpdateContext<'ctx, 'scope> {
    context: &'ctx mut ComponentContext<'scope>,
    previous: StateGeneration,
    next: StateGeneration,
}

impl<'ctx, 'scope> UpdateContext<'ctx, 'scope> {
    /// Wraps a borrowed [`ComponentContext`] together with the input generation
    /// watermarks observed before (`previous`) and for this update (`next`).
    #[must_use]
    pub fn new(
        context: &'ctx mut ComponentContext<'scope>,
        previous: StateGeneration,
        next: StateGeneration,
    ) -> Self {
        Self {
            context,
            previous,
            next,
        }
    }

    /// Mutable access to the underlying [`ComponentContext`].
    #[must_use]
    pub fn context(&mut self) -> &mut ComponentContext<'scope> {
        self.context
    }

    /// The input generation the instance last observed before this update.
    #[must_use]
    pub const fn previous_generation(&self) -> StateGeneration {
        self.previous
    }

    /// The input generation this update is being applied for.
    #[must_use]
    pub const fn next_generation(&self) -> StateGeneration {
        self.next
    }

    /// Returns whether the inputs actually advanced since the last update.
    ///
    /// When this is `false` the hook may skip work entirely, matching the
    /// spec's "invalidate only consumers of changed values".
    #[must_use]
    pub fn changed(&self) -> bool {
        self.next.is_newer_than(self.previous)
    }
}

impl<'scope> Deref for UpdateContext<'_, 'scope> {
    type Target = ComponentContext<'scope>;

    fn deref(&self) -> &Self::Target {
        &*self.context
    }
}

impl DerefMut for UpdateContext<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.context
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateContext;
    use crate::component::ComponentContext;
    use crate::lifecycle::LifecycleState;
    use crate::state::SignalGenerations;
    use crate::state::{StateGeneration, SubscriptionScope};

    #[test]
    fn changed_reflects_generation_advance() {
        let mut scope = SubscriptionScope::new();
        let mut flag = false;
        let signals = SignalGenerations::new();
        let mut ctx =
            ComponentContext::new(&mut scope, LifecycleState::Updating, &mut flag, &signals);
        let prev = StateGeneration::from_raw(3);
        let next = prev.next();
        let update = UpdateContext::new(&mut ctx, prev, next);
        assert!(update.changed());
        assert_eq!(update.previous_generation(), prev);
        assert_eq!(update.next_generation(), next);
    }

    #[test]
    fn equal_generations_are_unchanged() {
        let mut scope = SubscriptionScope::new();
        let mut flag = false;
        let signals = SignalGenerations::new();
        let mut ctx =
            ComponentContext::new(&mut scope, LifecycleState::Updating, &mut flag, &signals);
        let g = StateGeneration::from_raw(5);
        let update = UpdateContext::new(&mut ctx, g, g);
        assert!(!update.changed());
    }
}
