//! `Component`: the author-facing trait a component type implements.

use crate::boundary::ErrorBoundaryPolicy;
use crate::component::ComponentContext;
use crate::lifecycle::{MountContext, UnmountContext, UpdateContext};
use crate::view::View;

/// The author-facing contract every component type implements (spec §2.1).
///
/// A component:
///
/// * receives typed inputs as [`Props`](Self::Props) — values flow *down*
///   (spec §3.1); there is no two-way binding;
/// * produces a declarative [`View`] from [`render`](Self::render);
/// * may hold *optional local view state* as
///   [`LocalState<T>`](crate::state::LocalState) **fields on the implementor**.
///   The toolkit provides no global store; a component owns its own state and
///   the runtime never reaches in to mutate it (spec §2.1, §3.1);
/// * may open *scoped subscriptions* and *scoped async tasks* through the
///   [`ComponentContext`], all automatically cleaned up when the instance's
///   scope drops;
/// * has minimal lifecycle hooks ([`on_mount`](Self::on_mount),
///   [`on_update`](Self::on_update), [`on_unmount`](Self::on_unmount)) whose
///   default bodies do nothing;
/// * is an error boundary by default, customizable via
///   [`error_boundary`](Self::error_boundary).
///
/// # Not object-safe by design
///
/// `Component` has an associated `Props` type and a generic-free but
/// `Self`-typed API, so it is intentionally **not** object-safe. Heterogeneous
/// storage and type-erasure (holding many differently-typed components behind
/// one handle, dispatching by [`ComponentId`](obscur_identity::ComponentId)) is
/// the concern of the component registry and runtime in later sub-steps — not of
/// this authoring trait. Keeping props strongly typed here is the whole point;
/// erasing them is a separate, deliberate boundary crossing.
pub trait Component {
    /// The typed inputs this component receives. Values flow down through this
    /// type; it is supplied by the parent on every render (spec §3.1).
    type Props;

    /// Produces the component's desired [`View`] for the given `props`.
    ///
    /// This is a pure description of the intended output: it returns plain view
    /// data and performs no rendering, reconciliation, or dispatch itself. Use
    /// `ctx` to read the lifecycle phase, open scoped subscriptions, or
    /// [`request_update`](ComponentContext::request_update).
    fn render(&mut self, ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View;

    /// Called once when the instance is first mounted. Default: no-op.
    fn on_mount(&mut self, ctx: &mut MountContext<'_, '_>) {
        let _ = ctx;
    }

    /// Called when the instance's inputs may have changed. Default: no-op.
    ///
    /// Consult [`UpdateContext::changed`] to skip work when nothing the
    /// component depends on actually advanced.
    fn on_update(&mut self, ctx: &mut UpdateContext<'_, '_>) {
        let _ = ctx;
    }

    /// Called as the instance is torn down. Default: no-op.
    ///
    /// Releasing scoped subscriptions and local state is automatic (it happens
    /// when the instance's scope and fields drop); override this only for
    /// side-effects that must run *before* that cleanup.
    fn on_unmount(&mut self, ctx: &mut UnmountContext<'_, '_>) {
        let _ = ctx;
    }

    /// The error-boundary policy for this component. Default: an enabled
    /// boundary with a small retry budget. Override to customize or return
    /// [`ErrorBoundaryPolicy::disabled`] to opt out (spec §1.7).
    fn error_boundary(&self) -> ErrorBoundaryPolicy {
        ErrorBoundaryPolicy::default()
    }

    /// The fallback UI this component's boundary shows in place of its subtree
    /// once a contained failure is out of retries (spec §1.7). Default: `None`
    /// — the subtree is replaced by nothing, so a **contained failure that is
    /// out of retries** never leaves the last good output on screen pretending
    /// to be current.
    ///
    /// Two paths do still leave it standing, and neither is a fallback's to
    /// answer: a retry, for the frames its budget buys, and a propagated
    /// failure that reaches no enabled boundary. Both are reported. See
    /// `render_components`' "Where the previous output still stands".
    ///
    /// Read once, when the runtime creates the instance, and never asked for
    /// again: it must not depend on the render that failed, which is exactly the
    /// state a fallback cannot trust.
    fn fallback(&self) -> Option<View> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Component;
    use crate::boundary::ErrorBoundaryPolicy;
    use crate::component::ComponentContext;
    use crate::lifecycle::{LifecycleState, MountContext};
    use crate::state::{LocalState, SignalGenerations, SubscriptionScope};
    use crate::view::View;

    /// A minimal component that holds local view state and renders an empty
    /// view, exercising the trait's defaults.
    struct Counter {
        count: LocalState<u32>,
    }

    struct CounterProps {
        step: u32,
    }

    impl Component for Counter {
        type Props = CounterProps;

        fn render(&mut self, ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
            self.count.set(*self.count.get() + props.step);
            ctx.request_update();
            View::empty()
        }

        fn on_mount(&mut self, ctx: &mut MountContext<'_, '_>) {
            // Uses the default-overriding hook to prove the borrow threads.
            let _ = ctx.phase();
        }
    }

    #[test]
    fn default_error_boundary_is_enabled() {
        let component = Counter {
            count: LocalState::new(0),
        };
        assert_eq!(component.error_boundary(), ErrorBoundaryPolicy::default());
    }

    #[test]
    fn render_and_hooks_thread_the_context() {
        let mut component = Counter {
            count: LocalState::new(0),
        };
        let mut scope = SubscriptionScope::new();
        let mut flag = false;
        let signals = SignalGenerations::new();
        let mut ctx =
            ComponentContext::new(&mut scope, LifecycleState::Mounted, &mut flag, &signals);

        let view = component.render(&mut ctx, &CounterProps { step: 2 });
        assert!(view.root().is_empty());
        assert_eq!(*component.count.get(), 2);
        assert!(ctx.needs_update());

        let mut mount = MountContext::new(&mut ctx);
        component.on_mount(&mut mount);
    }
}
