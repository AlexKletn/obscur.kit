//! `ComponentFactory`: the object-safe type-erasure seam for instantiating one
//! component kind (spec §2.14).

use obscur_identity::ComponentId;

use crate::component::{ComponentContract, ComponentInstance, ErasedComponent, InstanceId};

/// Object-safe description of how to instantiate one kind of component.
///
/// The author-facing [`Component`](crate::component::Component) trait is
/// intentionally **not** object-safe (it has an associated `Props` type and a
/// `Self`-typed API), so a heterogeneous [`ComponentRegistry`] cannot store
/// `Component` values directly. `ComponentFactory` is the erasure seam: each
/// concrete component kind is fronted by a factory that reports its identity,
/// hands out an owned copy of its [`ComponentContract`], and mints fresh runtime
/// [`ComponentInstance`] records — all behind a `dyn` handle. Because every
/// method here is object-safe, factories are stored as
/// `Box<dyn ComponentFactory>`.
///
/// The [`core::fmt::Debug`] supertrait is required so that a boxed factory (and,
/// transitively, the [`ComponentRegistration`](crate::component::ComponentRegistration)
/// that owns one) is `Debug` without a hand-written impl.
///
/// # The two halves of an instance
///
/// Instantiating a component mints **two** values, deliberately kept apart:
///
/// * [`create_instance`](Self::create_instance) mints the
///   [`ComponentInstance`] — plain bookkeeping (identity, lifecycle state,
///   subscription scope) that stays `Debug` and inspectable;
/// * [`create_component`](Self::create_component) mints the author's component
///   value behind an [`ErasedComponent`] handle — that is where the component's
///   [`LocalState`](crate::state::LocalState) fields live, so it is the value a
///   runtime must keep alive across frames to preserve state.
///
/// Splitting them keeps the record free of the erased object (and therefore
/// `Debug`), while still giving a runtime everything it needs to render.
pub trait ComponentFactory: core::fmt::Debug {
    /// The stable identity of the component kind this factory instantiates.
    ///
    /// Must equal the [`id`](ComponentContract::id) of the contract returned by
    /// [`contract`](Self::contract) and the `component_id` of every instance
    /// produced by [`create_instance`](Self::create_instance).
    fn component_id(&self) -> ComponentId;

    /// Returns an owned clone of this component kind's contract.
    ///
    /// Cloning (rather than borrowing) keeps the method object-safe and lets a
    /// factory compute or store its contract however it likes.
    fn contract(&self) -> ComponentContract;

    /// Mints the runtime record for a new instance of this kind.
    ///
    /// The returned [`ComponentInstance`] is in
    /// [`LifecycleState::Created`](crate::lifecycle::LifecycleState::Created)
    /// and carries the given [`InstanceId`] and this factory's
    /// [`component_id`](Self::component_id). It is bookkeeping only; the
    /// renderable value comes from [`create_component`](Self::create_component).
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance;

    /// Mints a fresh component value for this kind, behind the object-safe
    /// [`ErasedComponent`] handle.
    ///
    /// This is the renderable half: the returned value holds the component's
    /// [`LocalState`](crate::state::LocalState) fields, so a runtime keeps **one**
    /// of these alive per mounted instance and re-renders through it. Dropping it
    /// drops the component's local state and scoped subscriptions — which is
    /// exactly the spec's "automatic cleanup" (§2.1).
    ///
    /// A typical implementation is `Box::new(MyComponent::default())`; the
    /// blanket impl on [`ErasedComponent`] makes any `Component` whose
    /// `Props: FromProps` usable here without extra glue.
    ///
    /// [`FromProps`]: crate::component::FromProps
    fn create_component(&self) -> Box<dyn ErasedComponent>;
}

#[cfg(test)]
mod tests {
    use super::ComponentFactory;
    use crate::component::{
        Component, ComponentContext, ComponentContract, ComponentInstance, ErasedComponent,
        FromProps, InstanceId, PropsError,
    };
    use crate::view::{View, ViewProperty};
    use obscur_identity::ComponentId;

    /// A trivial factory whose contract is the empty-but-valid contract for its
    /// id, used to prove the trait is object-safe and honest.
    #[derive(Debug)]
    struct StubFactory {
        id: ComponentId,
    }

    /// A minimal real component, so the stub factory can mint a renderable
    /// value rather than pretend. Its props decode from anything.
    struct StubProps;
    impl FromProps for StubProps {
        fn from_props(_: &[ViewProperty]) -> Result<Self, PropsError> {
            Ok(Self)
        }
    }
    struct StubComponent;
    impl Component for StubComponent {
        type Props = StubProps;
        fn render(&mut self, _: &mut ComponentContext<'_>, _: &Self::Props) -> View {
            View::empty()
        }
    }

    impl ComponentFactory for StubFactory {
        fn component_id(&self) -> ComponentId {
            self.id
        }

        fn contract(&self) -> ComponentContract {
            ComponentContract::new(self.id)
        }

        fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
            ComponentInstance::new(instance, self.id)
        }

        fn create_component(&self) -> Box<dyn ErasedComponent> {
            Box::new(StubComponent)
        }
    }

    #[test]
    fn is_object_safe_and_mints_created_instances() {
        let factory: Box<dyn ComponentFactory> = Box::new(StubFactory {
            id: ComponentId::from_raw(3),
        });

        assert_eq!(factory.component_id(), ComponentId::from_raw(3));
        assert_eq!(factory.contract().id, ComponentId::from_raw(3));

        let instance = factory.create_instance(InstanceId::from_raw(1));
        assert_eq!(instance.id(), InstanceId::from_raw(1));
        assert_eq!(instance.component_id(), ComponentId::from_raw(3));
        assert_eq!(instance.state(), crate::lifecycle::LifecycleState::Created);
    }
}
