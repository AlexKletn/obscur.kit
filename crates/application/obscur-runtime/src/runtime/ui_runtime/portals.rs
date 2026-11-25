use obscur_identity::{ComponentId, InternedSymbol, NodeId};
use obscur_tree::portal::{Portal, PortalHostResolver, PortalTarget};
use obscur_tree::tree::Subtree;
use obscur_view::view::ViewValue;

use crate::frame::FrameReason;

use super::UiRuntime;

const PORTAL: ComponentId = ComponentId::from_raw(0x00C0_0006);
const TARGET: InternedSymbol = InternedSymbol::from_raw(0x00C1_0601);

impl UiRuntime {
    /// Installs a portal target resolver. With no custom resolver, portal targets resolve against unique element ids in this window.
    pub fn set_portal_host_resolver(&mut self, resolver: Option<Box<dyn PortalHostResolver>>) {
        self.portal_hosts = resolver;
        self.refresh_portals();
        self.arrange_cache.forget_topology();
        self.last_layout_inputs = None;
        self.request_frame(FrameReason::UiCommit);
    }

    pub(super) fn refresh_portals(&mut self) {
        self.tree.clear_portals();
        if !self
            .tree
            .iter()
            .any(|(_, node)| node.kind().component_id() == Some(PORTAL))
        {
            return;
        }
        let Some(root) = self.tree.root() else {
            return;
        };
        let projections: Vec<_> = Subtree::new(root)
            .descendants(&self.tree)
            .into_iter()
            .filter_map(|source| {
                let node = self.tree.get(source)?;
                if node.kind().component_id() != Some(PORTAL) {
                    return None;
                }
                let target = node.props().iter().find_map(|property| {
                    match (property.name(), property.value()) {
                        (TARGET, ViewValue::Text(name)) if !name.is_empty() => Some(name.as_ref()),
                        _ => None,
                    }
                })?;
                let host = match &self.portal_hosts {
                    Some(resolver) => resolver.resolve(&self.tree, source, target),
                    None => self.unique_portal_host(target),
                }?;
                Some(Portal::new(source, PortalTarget::host(host)))
            })
            .collect();
        for projection in projections {
            projection.apply(&mut self.tree);
        }
    }

    fn unique_portal_host(&self, name: &str) -> Option<NodeId> {
        let mut matches = self
            .tree
            .iter()
            .filter_map(|(id, node)| (node.attributes().id() == Some(name)).then_some(id));
        let host = matches.next()?;
        matches.next().is_none().then_some(host)
    }
}
