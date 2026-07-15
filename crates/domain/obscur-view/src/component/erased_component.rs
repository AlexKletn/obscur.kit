//! `ErasedComponent`: calling a typed [`Component`] from behind a `dyn` handle.

use crate::boundary::ErrorBoundaryPolicy;
use crate::component::{Component, ComponentContext, FromProps, PropsError};
use crate::lifecycle::{MountContext, UnmountContext, UpdateContext};
use crate::view::{View, ViewProperty};

/// The object-safe face of a [`Component`], so a runtime can hold many
/// differently-typed components behind one handle and render them (spec §2.1,
/// §2.14).
///
/// [`Component`] is deliberately **not** object-safe: it has an associated
/// `Props` type, which is the whole point — props stay strongly typed. That
/// leaves a gap the runtime cannot cross on its own, because a retained node
/// carries only an untyped `&[ViewProperty]`. This trait is the crossing, and it
/// is the *only* place the erasure happens.
///
/// # You do not implement this
///
/// There is a blanket impl for every `C: Component` whose `C::Props: FromProps`,
/// so deriving `Contract` on the props struct is enough to make a component
/// erasable. The blanket impl decodes the properties into `C::Props` and then
/// calls the author's own typed [`render`](Component::render) — the component
/// never sees an untyped value, and the runtime never sees a typed one.
///
/// # Where local state lives
///
/// Nowhere here. A component owns its [`LocalState`](crate::state::LocalState)
/// as plain fields on `Self`, so keeping the same erased value alive across
/// frames is exactly what preserves that state; there is no store to consult and
/// the runtime never reaches in (spec §2.1, §3.1).
pub trait ErasedComponent {
    /// Decodes `props` into the component's typed props and renders it.
    ///
    /// # Errors
    ///
    /// Returns the [`PropsError`] from decoding when the node's properties do not
    /// satisfy the component's typed props. Render itself cannot fail — a
    /// component that wants to signal failure does so through its error boundary.
    fn render_erased(
        &mut self,
        ctx: &mut ComponentContext<'_>,
        props: &[ViewProperty],
    ) -> Result<View, PropsError>;

    /// Runs the component's [`on_mount`](Component::on_mount) hook.
    fn on_mount_erased(&mut self, ctx: &mut MountContext<'_, '_>);

    /// Runs the component's [`on_update`](Component::on_update) hook.
    fn on_update_erased(&mut self, ctx: &mut UpdateContext<'_, '_>);

    /// Runs the component's [`on_unmount`](Component::on_unmount) hook.
    fn on_unmount_erased(&mut self, ctx: &mut UnmountContext<'_, '_>);

    /// The component's [`error_boundary`](Component::error_boundary) policy.
    fn error_boundary_erased(&self) -> ErrorBoundaryPolicy;
}

impl<C> ErasedComponent for C
where
    C: Component,
    C::Props: FromProps,
{
    fn render_erased(
        &mut self,
        ctx: &mut ComponentContext<'_>,
        props: &[ViewProperty],
    ) -> Result<View, PropsError> {
        // Decode first: if the authored properties do not satisfy the typed
        // props, the component is never called with a half-built value.
        let typed = C::Props::from_props(props)?;
        Ok(self.render(ctx, &typed))
    }

    fn on_mount_erased(&mut self, ctx: &mut MountContext<'_, '_>) {
        self.on_mount(ctx);
    }

    fn on_update_erased(&mut self, ctx: &mut UpdateContext<'_, '_>) {
        self.on_update(ctx);
    }

    fn on_unmount_erased(&mut self, ctx: &mut UnmountContext<'_, '_>) {
        self.on_unmount(ctx);
    }

    fn error_boundary_erased(&self) -> ErrorBoundaryPolicy {
        self.error_boundary()
    }
}

#[cfg(test)]
mod tests {
    use super::ErasedComponent;
    use crate::component::{Component, ComponentContext, FromProps, PropsError, decode_required};
    use crate::lifecycle::{LifecycleState, MountContext};
    use crate::state::{LocalState, SignalGenerations, SubscriptionScope};
    use crate::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};
    use obscur_identity::{ComponentId, InternedSymbol};

    fn step_symbol() -> InternedSymbol {
        InternedSymbol::from_raw(0x8000_0001)
    }

    struct CounterProps {
        step: u32,
    }

    impl FromProps for CounterProps {
        fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
            Ok(Self {
                step: decode_required(props, step_symbol())?,
            })
        }
    }

    /// A component holding real local state, so the tests can prove the state
    /// survives across erased renders.
    struct Counter {
        count: LocalState<u32>,
        mounted: bool,
    }

    impl Component for Counter {
        type Props = CounterProps;

        fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
            self.count.set(*self.count.get() + props.step);
            View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text(
                self.count.get().to_string(),
            )]))
        }

        fn on_mount(&mut self, _ctx: &mut MountContext<'_, '_>) {
            self.mounted = true;
        }
    }

    fn counter() -> Counter {
        Counter {
            count: LocalState::new(0),
            mounted: false,
        }
    }

    fn props(step: i64) -> Vec<ViewProperty> {
        vec![ViewProperty::new(step_symbol(), ViewValue::Int(step))]
    }

    /// Renders `component` through the `dyn` boundary and returns its text.
    fn render_text(component: &mut dyn ErasedComponent, props: &[ViewProperty]) -> String {
        let mut scope = SubscriptionScope::new();
        let mut needs_update = false;
        let signals = SignalGenerations::new();
        let mut ctx = ComponentContext::new(&mut scope, LifecycleState::Mounted, &mut needs_update, &signals);
        let view = component
            .render_erased(&mut ctx, props)
            .expect("the props decode");
        let ViewNode::Text { content, .. } = &view.root().nodes()[0] else {
            panic!("the counter renders a text node");
        };
        content.to_string()
    }

    #[test]
    fn a_typed_component_renders_from_behind_a_dyn_handle() {
        // THE crossing: the runtime holds `Box<dyn ErasedComponent>` and passes
        // an untyped property slice; the author's `render` still receives typed
        // `CounterProps`.
        let mut erased: Box<dyn ErasedComponent> = Box::new(counter());
        assert_eq!(render_text(&mut *erased, &props(2)), "2");
    }

    #[test]
    fn local_state_survives_across_erased_renders() {
        // The reason the runtime must keep the same erased VALUE alive rather
        // than re-creating it: `count` lives on the component itself.
        let mut erased: Box<dyn ErasedComponent> = Box::new(counter());
        assert_eq!(render_text(&mut *erased, &props(2)), "2");
        assert_eq!(render_text(&mut *erased, &props(3)), "5");
        assert_eq!(render_text(&mut *erased, &props(1)), "6");
    }

    #[test]
    fn undecodable_props_never_reach_the_component() {
        // A wrongly-typed prop must surface as a decode error, not call `render`
        // with a defaulted value.
        let mut erased: Box<dyn ErasedComponent> = Box::new(counter());
        let mut scope = SubscriptionScope::new();
        let mut needs_update = false;
        let signals = SignalGenerations::new();
        let mut ctx = ComponentContext::new(&mut scope, LifecycleState::Mounted, &mut needs_update, &signals);

        let bad = vec![ViewProperty::new(step_symbol(), ViewValue::Text("2".into()))];
        assert_eq!(
            erased.render_erased(&mut ctx, &bad),
            Err(PropsError::Type {
                name: step_symbol(),
                expected: "u32"
            })
        );
        // And a missing required prop likewise.
        assert_eq!(
            erased.render_erased(&mut ctx, &[]),
            Err(PropsError::Missing {
                name: step_symbol()
            })
        );
    }

    #[test]
    fn lifecycle_hooks_dispatch_through_the_erasure() {
        let mut erased: Box<dyn ErasedComponent> = Box::new(counter());
        let mut scope = SubscriptionScope::new();
        let mut needs_update = false;
        let signals = SignalGenerations::new();
        let mut ctx = ComponentContext::new(&mut scope, LifecycleState::Created, &mut needs_update, &signals);
        let mut mount = MountContext::new(&mut ctx);
        erased.on_mount_erased(&mut mount);
        // The default `error_boundary` still answers through the erasure.
        assert_eq!(
            erased.error_boundary_erased(),
            crate::boundary::ErrorBoundaryPolicy::default()
        );
    }

    #[test]
    fn heterogeneous_components_share_one_handle_type() {
        // The point of erasure: two unrelated `Props` types, one collection.
        struct LabelProps;
        impl FromProps for LabelProps {
            fn from_props(_: &[ViewProperty]) -> Result<Self, PropsError> {
                Ok(Self)
            }
        }
        struct Label;
        impl Component for Label {
            type Props = LabelProps;
            fn render(&mut self, _: &mut ComponentContext<'_>, _: &Self::Props) -> View {
                View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(
                    ComponentId::from_raw(1),
                )]))
            }
        }

        let components: Vec<Box<dyn ErasedComponent>> = vec![Box::new(counter()), Box::new(Label)];
        assert_eq!(components.len(), 2);
    }
}
