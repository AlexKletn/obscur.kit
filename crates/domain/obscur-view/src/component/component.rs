//! `Component`: the author-facing trait a component type implements.

use crate::boundary::ErrorBoundaryPolicy;
use crate::component::ComponentContext;
use crate::lifecycle::{MountContext, UnmountContext, UpdateContext};
use crate::view::View;

#[doc = include_str!("../../docs/api/component/component.doc/Component.md")]
pub trait Component {
    /// The typed inputs this component receives. Values flow down through this
    /// type; it is supplied by the parent on every render (spec §3.1).
    type Props;

    #[doc = include_str!("../../docs/api/component/component.doc/render.md")]
    fn render(&mut self, ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View;

    /// Called once when the instance is first mounted. Default: no-op.
    fn on_mount(&mut self, ctx: &mut MountContext<'_, '_>) {
        let _ = ctx;
    }

    #[doc = include_str!("../../docs/api/component/component.doc/on_update.md")]
    fn on_update(&mut self, ctx: &mut UpdateContext<'_, '_>) {
        let _ = ctx;
    }

    #[doc = include_str!("../../docs/api/component/component.doc/on_unmount.md")]
    fn on_unmount(&mut self, ctx: &mut UnmountContext<'_, '_>) {
        let _ = ctx;
    }

    /// The error-boundary policy for this component. Default: an enabled
    /// boundary with a small retry budget. Override to customize or return
    /// [`ErrorBoundaryPolicy::disabled`] to opt out (spec §1.7).
    fn error_boundary(&self) -> ErrorBoundaryPolicy {
        ErrorBoundaryPolicy::default()
    }

    #[doc = include_str!("../../docs/api/component/component.doc/fallback.md")]
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
