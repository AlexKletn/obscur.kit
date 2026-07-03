//! `ComponentFactory`: the object-safe type-erasure seam for instantiating one
//! component kind (spec §2.14).

use obscur_identity::ComponentId;

use crate::component::{ComponentContract, ComponentInstance, InstanceId};

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
/// # Deferred: erased render
///
/// This trait deliberately does **not** yet bind the author's typed
/// [`Component::render`](crate::component::Component::render) into the instance.
/// Driving an *erased render* — decoding a `ViewProperty` slice into each
/// component's typed `Props` and invoking `render` behind the `dyn` boundary —
/// depends on a prop-decoding story that does not exist yet, and is a later
/// runtime concern. A factory today only mints the plain runtime record (in
/// [`LifecycleState::Created`](crate::lifecycle::LifecycleState::Created)); wiring
/// live rendering is out of scope for this sub-step.
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
    /// [`component_id`](Self::component_id). It does not (yet) carry an erased
    /// render binding — see the trait-level note on the deferred erased render.
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance;
}

#[cfg(test)]
mod tests {
    use super::ComponentFactory;
    use crate::component::{ComponentContract, ComponentInstance, InstanceId};
    use obscur_identity::ComponentId;

    /// A trivial factory whose contract is the empty-but-valid contract for its
    /// id, used to prove the trait is object-safe and honest.
    #[derive(Debug)]
    struct StubFactory {
        id: ComponentId,
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
        assert_eq!(
            instance.state(),
            crate::lifecycle::LifecycleState::Created
        );
    }
}
