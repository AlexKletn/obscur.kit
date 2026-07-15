//! `ComponentStore`: the runtime's per-node component instances.

use obscur_identity::{ComponentId, NodeId};
use obscur_tree::node::{NodeFlags, NodeKind};
use obscur_tree::reconciliation::ReconciliationPlan;
use obscur_tree::tree::{RetainedTree, SecondaryMap};
use obscur_view::component::{
    ComponentRegistration, ComponentRegistry, InstanceId, RegistryError,
};

use crate::component::MountedComponent;

/// The registered component kinds plus one live [`MountedComponent`] per node
/// that instantiates one (spec §2.1, §2.14).
///
/// # Why keyed by `NodeId`
///
/// A [`SecondaryMap`] *augments* the retained tree's identity rather than
/// competing with it (spec §1.5): the tree owns and mints the `NodeId`, and this
/// map attaches the instance to it. That is exactly the relationship wanted here,
/// and it means [`RetainedNode`](obscur_tree::node::RetainedNode) needs no
/// component-instance field.
///
/// Keying on `NodeId` is also safe against slot reuse: a `NodeId` packs the
/// arena slot's *generation*, which advances both when a slot is vacated and when
/// it is reused, so a recycled slot never collides with a stale entry.
///
/// The reconciler already resolves `(ComponentId, Option<ViewKey>)`-in-position
/// into a stable `NodeId` via keyed matching, so the node id *is* the instance
/// identity — there is no second identity to invent, and an
/// [`InstanceId`] remains only a diagnostic handle.
#[derive(Debug, Default)]
pub struct ComponentStore {
    /// The kinds a host has registered; nothing mounts without one.
    registry: ComponentRegistry,
    /// One live instance per component-backed node.
    mounted: SecondaryMap<MountedComponent>,
    /// Monotonic source of [`InstanceId`]s. Never reused, so an id in a
    /// diagnostic always names one historical instance.
    next_instance: u64,
}

impl ComponentStore {
    /// Creates an empty store with no registered kinds.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a component kind.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] when a kind is already registered under the same
    /// [`ComponentId`].
    pub fn register(&mut self, registration: ComponentRegistration) -> Result<(), RegistryError> {
        self.registry.register(registration)
    }

    /// Whether any component kind is registered — the cheap guard that lets a
    /// host which uses no components skip every code path here.
    #[must_use]
    pub fn has_registrations(&self) -> bool {
        !self.registry.is_empty()
    }

    /// The number of currently mounted instances.
    #[must_use]
    pub fn len(&self) -> usize {
        self.mounted.len()
    }

    /// Whether nothing is mounted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mounted.is_empty()
    }

    /// The instance mounted at `node`, if any.
    #[must_use]
    pub fn get(&self, node: NodeId) -> Option<&MountedComponent> {
        self.mounted.get(node)
    }

    /// Mutable access to the instance mounted at `node`, if any.
    pub fn get_mut(&mut self, node: NodeId) -> Option<&mut MountedComponent> {
        self.mounted.get_mut(node)
    }

    /// Every mounted node, in unspecified order.
    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.mounted.iter().map(|(node, _)| node)
    }

    /// Brings the store in line with a reconcile that just ran: mounts an
    /// instance for every inserted node whose component kind is registered, and
    /// unmounts every removed node's instance.
    ///
    /// Driven by the [`ReconciliationPlan`] rather than by a tree walk, because
    /// the plan already states the exact structural deltas — and driven from
    /// *here* rather than from inside the reconciler, which lives in the domain
    /// tier and can neither see a registry nor accumulate state.
    ///
    /// `Update` and `Move` ops are intentionally ignored: they change a node's
    /// content or position, not its existence, so the instance (and its local
    /// state) rides along untouched. That is the whole point of keyed identity.
    pub fn sync(&mut self, tree: &mut RetainedTree, plan: &ReconciliationPlan) {
        for op in plan {
            if op.is_insert() {
                self.mount(tree, op.id());
            } else if op.is_remove() {
                self.unmount(tree, op.id());
            }
        }
    }

    /// Mounts an instance for `node` when it is an element of a registered kind.
    ///
    /// A node whose kind is unregistered — a plain container, a widget tag, a
    /// text leaf — simply gets no instance, which is what leaves a
    /// component-free host paying nothing.
    fn mount(&mut self, tree: &mut RetainedTree, node: NodeId) {
        let Some(component) = tree.get(node).and_then(|n| component_id_of(n.kind())) else {
            return;
        };
        let Some(registration) = self.registry.get(component) else {
            return;
        };
        let instance = InstanceId::from_raw(self.next_instance);
        self.next_instance += 1;
        let mut mounted = MountedComponent::new(
            registration.create_instance(instance),
            registration.create_component(),
        );
        mounted.mount();
        self.mounted.insert(node, mounted);
        // Tell reconciliation that this node's children now belong to the
        // component: without the flag, the next root reconcile would see the
        // desired node's (empty) children and delete the component's output.
        if let Some(retained) = tree.get_mut(node) {
            retained.flags.insert(NodeFlags::COMPONENT_HOST);
        }
    }

    /// Unmounts and drops the instance at `node`, if one is mounted.
    ///
    /// The hook runs while the component's state is still readable; dropping the
    /// value afterwards releases that state and the scoped subscriptions.
    fn unmount(&mut self, tree: &mut RetainedTree, node: NodeId) {
        if let Some(mut mounted) = self.mounted.remove(node) {
            mounted.unmount();
        }
        // The node is usually already gone (a Remove op means the reconciler
        // dropped it), but clear the flag defensively for any caller that
        // unmounts a still-live node.
        if let Some(retained) = tree.get_mut(node) {
            retained.flags.remove(NodeFlags::COMPONENT_HOST);
        }
    }
}

/// The [`ComponentId`] a node instantiates, or [`None`] for a kind that names no
/// component (root, text, synthetic).
fn component_id_of(kind: &NodeKind) -> Option<ComponentId> {
    kind.component_id()
}

#[cfg(test)]
mod tests {
    use super::ComponentStore;
    use obscur_identity::ComponentId;
    use obscur_tree::reconciliation::Reconciler;
    use obscur_tree::tree::RetainedTree;
    use obscur_view::component::{
        Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
        ComponentRegistration, ComponentVersion, ErasedComponent, FromProps, InstanceId,
        PropsError,
    };
    use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty};
    use std::cell::RefCell;
    use std::rc::Rc;

    const KIND: ComponentId = ComponentId::from_raw(0xA1);
    const OTHER: ComponentId = ComponentId::from_raw(0xA2);

    /// A shared log of lifecycle events, so mount/unmount is observable.
    type Log = Rc<RefCell<Vec<&'static str>>>;

    struct ProbeProps;
    impl FromProps for ProbeProps {
        fn from_props(_: &[ViewProperty]) -> Result<Self, PropsError> {
            Ok(Self)
        }
    }

    struct Probe {
        log: Log,
    }

    impl Component for Probe {
        type Props = ProbeProps;
        fn render(&mut self, _: &mut ComponentContext<'_>, _: &Self::Props) -> View {
            self.log.borrow_mut().push("render");
            View::empty()
        }
        fn on_mount(&mut self, _: &mut obscur_view::lifecycle::MountContext<'_, '_>) {
            self.log.borrow_mut().push("mount");
        }
        fn on_unmount(&mut self, _: &mut obscur_view::lifecycle::UnmountContext<'_, '_>) {
            self.log.borrow_mut().push("unmount");
        }
    }

    #[derive(Debug)]
    struct ProbeFactory {
        id: ComponentId,
        log: Log,
    }

    impl ComponentFactory for ProbeFactory {
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
            Box::new(Probe {
                log: self.log.clone(),
            })
        }
    }

    fn store_with(id: ComponentId, log: &Log) -> ComponentStore {
        let mut store = ComponentStore::new();
        store
            .register(ComponentRegistration::new(
                Box::new(ProbeFactory {
                    id,
                    log: log.clone(),
                }),
                ComponentVersion::INITIAL,
            ))
            .expect("first registration");
        store
    }

    /// Reconciles `view` into `tree` and syncs `store` against the resulting plan.
    fn reconcile(store: &mut ComponentStore, tree: &mut RetainedTree, view: &View) {
        let plan = Reconciler::new().reconcile_root(tree, view);
        store.sync(tree, &plan);
    }

    fn view_of(components: &[ComponentId]) -> View {
        View::from_fragment(ViewFragment::from_nodes(
            components.iter().copied().map(ViewNode::element).collect(),
        ))
    }

    #[test]
    fn an_inserted_node_of_a_registered_kind_mounts_once() {
        let log: Log = Rc::new(RefCell::new(Vec::new()));
        let mut store = store_with(KIND, &log);
        let mut tree = RetainedTree::new();

        reconcile(&mut store, &mut tree, &view_of(&[KIND]));
        assert_eq!(store.len(), 1);
        assert_eq!(&*log.borrow(), &["mount"]);

        // An identical re-render reuses the node: no Insert op, so no second mount
        // and — critically — the SAME instance (and its local state) survives.
        let instance = store
            .get(store.nodes().next().expect("a mounted node"))
            .expect("mounted")
            .instance_id();
        reconcile(&mut store, &mut tree, &view_of(&[KIND]));
        assert_eq!(store.len(), 1);
        assert_eq!(&*log.borrow(), &["mount"], "no remount on an unchanged view");
        assert_eq!(
            store
                .get(store.nodes().next().expect("a mounted node"))
                .expect("mounted")
                .instance_id(),
            instance
        );
    }

    #[test]
    fn a_removed_node_unmounts_and_drops_its_instance() {
        let log: Log = Rc::new(RefCell::new(Vec::new()));
        let mut store = store_with(KIND, &log);
        let mut tree = RetainedTree::new();

        reconcile(&mut store, &mut tree, &view_of(&[KIND]));
        assert_eq!(store.len(), 1);

        reconcile(&mut store, &mut tree, &View::empty());
        assert_eq!(store.len(), 0, "the instance is gone with its node");
        assert_eq!(&*log.borrow(), &["mount", "unmount"]);
    }

    #[test]
    fn an_unregistered_kind_never_mounts() {
        // A plain container, a widget tag, or any kind the host did not register
        // costs nothing: no instance, no lifecycle.
        let log: Log = Rc::new(RefCell::new(Vec::new()));
        let mut store = store_with(KIND, &log);
        let mut tree = RetainedTree::new();

        reconcile(&mut store, &mut tree, &view_of(&[OTHER]));
        assert!(store.is_empty());
        assert!(log.borrow().is_empty());
    }

    #[test]
    fn instance_ids_are_unique_per_mount_and_never_reused() {
        let log: Log = Rc::new(RefCell::new(Vec::new()));
        let mut store = store_with(KIND, &log);
        let mut tree = RetainedTree::new();

        reconcile(&mut store, &mut tree, &view_of(&[KIND, KIND]));
        let ids: Vec<_> = store
            .nodes()
            .map(|node| store.get(node).expect("mounted").instance_id().raw())
            .collect();
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1], "two instances of one kind are distinct");

        // Unmount everything and mount again: the new instance is NOT id 0.
        reconcile(&mut store, &mut tree, &View::empty());
        reconcile(&mut store, &mut tree, &view_of(&[KIND]));
        let fresh = store
            .get(store.nodes().next().expect("a mounted node"))
            .expect("mounted")
            .instance_id()
            .raw();
        assert!(fresh >= 2, "instance ids are never recycled, got {fresh}");
    }

    #[test]
    fn a_store_with_no_registrations_reports_it() {
        let store = ComponentStore::new();
        assert!(!store.has_registrations());
        let log: Log = Rc::new(RefCell::new(Vec::new()));
        assert!(store_with(KIND, &log).has_registrations());
    }
}
