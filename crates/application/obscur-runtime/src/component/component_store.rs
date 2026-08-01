#![doc = include_str!("component_store.doc/_module.md")]

use obscur_identity::{ComponentId, NodeId, SignalId};
use obscur_tree::node::{NodeFlags, NodeKind};
use obscur_tree::reconciliation::ReconciliationPlan;
use obscur_tree::tree::{RetainedTree, SecondaryMap};
use obscur_view::component::{ComponentRegistration, ComponentRegistry, InstanceId, RegistryError};
use obscur_view::state::SignalGenerations;

use crate::component::MountedComponent;

#[doc = include_str!("component_store.doc/ComponentStore.md")]
#[derive(Debug, Default)]
pub struct ComponentStore {
    #[doc = include_str!("component_store.doc/registry.md")]
    registry: ComponentRegistry,
    #[doc = include_str!("component_store.doc/mounted.md")]
    mounted: SecondaryMap<MountedComponent>,
    #[doc = include_str!("component_store.doc/next_instance.md")]
    next_instance: u64,
    #[doc = include_str!("component_store.doc/signals.md")]
    signals: SignalGenerations,
}

impl ComponentStore {
    #[doc = include_str!("component_store.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("component_store.doc/register.md")]
    pub fn register(&mut self, registration: ComponentRegistration) -> Result<(), RegistryError> {
        self.registry.register(registration)
    }

    #[doc = include_str!("component_store.doc/has_registrations.md")]
    #[must_use]
    pub fn has_registrations(&self) -> bool {
        !self.registry.is_empty()
    }

    #[doc = include_str!("component_store.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.mounted.len()
    }

    #[doc = include_str!("component_store.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mounted.is_empty()
    }

    #[doc = include_str!("component_store.doc/get.md")]
    #[must_use]
    pub fn get(&self, node: NodeId) -> Option<&MountedComponent> {
        self.mounted.get(node)
    }

    #[doc = include_str!("component_store.doc/get_mut.md")]
    pub fn get_mut(&mut self, node: NodeId) -> Option<&mut MountedComponent> {
        self.mounted.get_mut(node)
    }

    #[doc = include_str!("component_store.doc/nodes.md")]
    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.mounted.iter().map(|(node, _)| node)
    }

    #[doc = include_str!("component_store.doc/stale_nodes.md")]
    #[must_use]
    pub fn stale_nodes(&self) -> Vec<NodeId> {
        self.mounted
            .iter()
            .filter(|(_, mounted)| mounted.needs_render())
            .map(|(node, _)| node)
            .collect()
    }

    #[doc = include_str!("component_store.doc/signals_2.md")]
    #[must_use]
    pub const fn signals(&self) -> &SignalGenerations {
        &self.signals
    }

    #[doc = include_str!("component_store.doc/notify.md")]
    pub fn notify(&mut self, source: SignalId) -> usize {
        let current = self.signals.bump(source);
        let stale: Vec<NodeId> = self
            .mounted
            .iter()
            .filter(|(_, mounted)| mounted.depends_on_stale(source, current))
            .map(|(node, _)| node)
            .collect();
        for node in &stale {
            if let Some(mounted) = self.mounted.get_mut(*node) {
                mounted.mark_dirty();
            }
        }
        stale.len()
    }

    #[doc = include_str!("component_store.doc/has_stale.md")]
    #[must_use]
    pub fn has_stale(&self) -> bool {
        self.mounted.iter().any(|(_, m)| m.needs_render())
    }

    #[doc = include_str!("component_store.doc/sync.md")]
    pub fn sync(&mut self, tree: &mut RetainedTree, plan: &ReconciliationPlan) {
        for op in plan {
            if op.is_insert() {
                self.mount(tree, op.id());
            } else if op.is_remove() {
                self.unmount(tree, op.id());
            } else if op.is_update()
                && let Some(mounted) = self.mounted.get_mut(op.id())
            {
                mounted.mark_dirty();
            }
        }
    }

    #[doc = include_str!("component_store.doc/mount.md")]
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
        mounted.mount(&self.signals);
        self.mounted.insert(node, mounted);
        // Tell reconciliation that this node's children now belong to the
        // component: without the flag, the next root reconcile would see the
        // desired node's (empty) children and delete the component's output.
        //
        // The same place records the contract's §4.15 style scope, because the
        // style pass decides scoping by walking the tree and the registry is not
        // reachable from there. Scoped is the default, so the flag is only ever
        // *added* by a component that explicitly opened itself.
        if let Some(retained) = tree.get_mut(node) {
            retained.flags.insert(NodeFlags::COMPONENT_HOST);
            if registration.contract().style_scope.is_open() {
                retained.flags.insert(NodeFlags::OPEN_STYLE_SCOPE);
            }
        }
    }

    #[doc = include_str!("component_store.doc/unmount.md")]
    fn unmount(&mut self, tree: &mut RetainedTree, node: NodeId) {
        if let Some(mut mounted) = self.mounted.remove(node) {
            mounted.unmount(&self.signals);
        }
        // The node is usually already gone (a Remove op means the reconciler
        // dropped it), but clear the flag defensively for any caller that
        // unmounts a still-live node.
        if let Some(retained) = tree.get_mut(node) {
            retained
                .flags
                .remove(NodeFlags::COMPONENT_HOST.union(NodeFlags::OPEN_STYLE_SCOPE));
        }
    }
}

#[doc = include_str!("component_store.doc/component_id_of.md")]
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
        assert_eq!(
            &*log.borrow(),
            &["mount"],
            "no remount on an unchanged view"
        );
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
