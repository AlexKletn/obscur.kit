//! `ComponentContext`: the borrow threaded through render and lifecycle hooks.

use obscur_identity::SignalId;

use crate::lifecycle::LifecycleState;
use crate::state::{SignalGenerations, SubscriptionScope};

/// The context threaded through [`render`](crate::component::Component::render)
/// and the lifecycle hooks (spec §2.1).
///
/// It is deliberately minimal and *honest*: there is no reactive runtime yet,
/// so the context does not dispatch actions, resolve capabilities, or reach any
/// global store (there is none — local view state lives in the component's own
/// [`LocalState`](crate::state::LocalState) fields). It only exposes what a
/// component legitimately needs during a single render/lifecycle call:
///
/// * the component's [`SubscriptionScope`], so it can open scoped subscriptions
///   that are automatically released when the scope is dropped;
/// * the current [`LifecycleState`], so a hook can branch on the phase;
/// * a way to *request* a re-render ([`request_update`](Self::request_update)),
///   which merely sets a flag the runtime observes later via
///   [`needs_update`](Self::needs_update) — the context never renders anything
///   itself.
///
/// # Borrows
///
/// `ComponentContext<'a>` borrows the instance's subscription scope and its
/// "needs update" flag mutably for `'a`; it owns no state of its own. Because
/// it holds `&mut` borrows it is invariant in `'a` and therefore not `Clone`.
#[derive(Debug)]
pub struct ComponentContext<'a> {
    /// The instance's scoped subscriptions, borrowed for the call.
    scope: &'a mut SubscriptionScope,
    /// The current version of every observable source, so [`track`](Self::track)
    /// can record *which* generation a dependency was taken at. Read-only: a
    /// component observes sources, it never advances them (values flow down).
    signals: &'a SignalGenerations,
    /// The lifecycle phase this context was created for.
    phase: LifecycleState,
    /// Flag set by [`request_update`](Self::request_update); the runtime reads
    /// and clears it. Borrowed so the signal outlives the context.
    needs_update: &'a mut bool,
}

impl<'a> ComponentContext<'a> {
    /// Creates a context over the given subscription `scope`, lifecycle
    /// `phase`, and `needs_update` flag.
    ///
    /// The runtime constructs this immediately before calling `render` or a
    /// lifecycle hook and drops it immediately after, so the borrows are held
    /// only for the duration of that call.
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

    /// Records that this render depends on the observable `source` (spec §3.1).
    ///
    /// This is the whole of dependency tracking from the author's side: read a
    /// source, say so. The runtime then invalidates this component when — and
    /// only when — that source advances. A component that tracks nothing is never
    /// invalidated by a state change; a component that tracks `A` is untouched
    /// when `B` changes.
    ///
    /// Dependencies are re-collected on every render, so a component that reads a
    /// source only on some branch depends on it only when it actually did. The
    /// generation is captured here, not by the caller, so a source that advances
    /// *between* this render and the next is correctly seen as changed.
    pub fn track(&mut self, source: SignalId) {
        let observed = self.signals.generation(source);
        self.scope.subscribe(source, observed);
    }

    /// Returns the lifecycle phase this context is running in.
    #[must_use]
    pub const fn phase(&self) -> LifecycleState {
        self.phase
    }

    /// Shared access to the component's scoped subscriptions.
    #[must_use]
    pub fn subscriptions(&self) -> &SubscriptionScope {
        &*self.scope
    }

    /// Mutable access to the component's scoped subscriptions, e.g. to open or
    /// release one. Everything opened here is cleaned up when the scope drops.
    #[must_use]
    pub fn subscriptions_mut(&mut self) -> &mut SubscriptionScope {
        &mut *self.scope
    }

    /// Requests that the runtime re-render this component.
    ///
    /// This only raises a flag; it performs no rendering and schedules no work.
    /// The runtime observes the flag via [`needs_update`](Self::needs_update)
    /// after the current call returns and decides what to do.
    pub fn request_update(&mut self) {
        *self.needs_update = true;
    }

    /// Returns whether [`request_update`](Self::request_update) was called on
    /// this context.
    #[must_use]
    pub fn needs_update(&self) -> bool {
        *self.needs_update
    }
}

#[cfg(test)]
mod tests {
    use obscur_identity::SignalId;
    use crate::state::SignalGenerations;
    use super::ComponentContext;
    use crate::lifecycle::LifecycleState;
    use crate::state::{StateGeneration, SubscriptionScope};

    #[test]
    fn exposes_phase_and_scope() {
        let mut scope = SubscriptionScope::new();
        let mut flag = false;
        let signals = SignalGenerations::new();
        let mut ctx = ComponentContext::new(&mut scope, LifecycleState::Mounted, &mut flag, &signals);
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
        let mut ctx = ComponentContext::new(&mut scope, LifecycleState::Updating, &mut flag, &signals);
        assert!(!ctx.needs_update());
        ctx.request_update();
        assert!(ctx.needs_update());
        // The flag is owned outside the context, so the runtime can observe it.
        assert!(flag);
    }
}
