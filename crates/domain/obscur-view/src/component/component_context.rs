#![doc = include_str!("component_context.doc/_module.md")]

use obscur_identity::SignalId;

use crate::lifecycle::LifecycleState;
use crate::state::{SignalGenerations, SubscriptionScope};

#[doc = include_str!("component_context.doc/ComponentContext.md")]
#[derive(Debug)]
pub struct ComponentContext<'a> {
    #[doc = include_str!("component_context.doc/scope.md")]
    scope: &'a mut SubscriptionScope,
    #[doc = include_str!("component_context.doc/signals.md")]
    signals: &'a SignalGenerations,
    #[doc = include_str!("component_context.doc/phase.md")]
    phase: LifecycleState,
    #[doc = include_str!("component_context.doc/needs_update.md")]
    needs_update: &'a mut bool,
}

impl<'a> ComponentContext<'a> {
    #[doc = include_str!("component_context.doc/new.md")]
    #[must_use]
    pub fn new(
        scope: &'a mut SubscriptionScope,
        phase: LifecycleState,
        needs_update: &'a mut bool,
        signals: &'a SignalGenerations,
    ) -> Self {
        Self {
            scope,
            signals,
            phase,
            needs_update,
        }
    }

    #[doc = include_str!("component_context.doc/track.md")]
    pub fn track(&mut self, source: SignalId) {
        let observed = self.signals.generation(source);
        self.scope.subscribe(source, observed);
    }

    #[doc = include_str!("component_context.doc/phase_2.md")]
    #[must_use]
    pub const fn phase(&self) -> LifecycleState {
        self.phase
    }

    #[doc = include_str!("component_context.doc/subscriptions.md")]
    #[must_use]
    pub fn subscriptions(&self) -> &SubscriptionScope {
        &*self.scope
    }

    #[doc = include_str!("component_context.doc/subscriptions_mut.md")]
    #[must_use]
    pub fn subscriptions_mut(&mut self) -> &mut SubscriptionScope {
        &mut *self.scope
    }

    #[doc = include_str!("component_context.doc/request_update.md")]
    pub fn request_update(&mut self) {
        *self.needs_update = true;
    }

    #[doc = include_str!("component_context.doc/needs_update_2.md")]
    #[must_use]
    pub fn needs_update(&self) -> bool {
        *self.needs_update
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentContext;
    use crate::lifecycle::LifecycleState;
    use crate::state::SignalGenerations;
    use crate::state::{StateGeneration, SubscriptionScope};
    use obscur_identity::SignalId;

    #[test]
    fn exposes_phase_and_scope() {
        let mut scope = SubscriptionScope::new();
        let mut flag = false;
        let signals = SignalGenerations::new();
        let mut ctx =
            ComponentContext::new(&mut scope, LifecycleState::Mounted, &mut flag, &signals);
        assert_eq!(ctx.phase(), LifecycleState::Mounted);
        assert!(ctx.subscriptions().is_empty());
        ctx.subscriptions_mut()
            .subscribe(SignalId::from_raw(1), StateGeneration::INITIAL);
        assert_eq!(ctx.subscriptions().len(), 1);
    }

    #[test]
    fn request_update_sets_flag() {
        let mut scope = SubscriptionScope::new();
        let mut flag = false;
        let signals = SignalGenerations::new();
        let mut ctx =
            ComponentContext::new(&mut scope, LifecycleState::Updating, &mut flag, &signals);
        assert!(!ctx.needs_update());
        ctx.request_update();
        assert!(ctx.needs_update());
        // The flag is owned outside the context, so the runtime can observe it.
        assert!(flag);
    }
}
