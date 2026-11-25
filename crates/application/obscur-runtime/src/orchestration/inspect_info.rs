use std::fmt;

use obscur_geometry::Rect;
use obscur_identity::{ComponentId, NodeId};

#[doc = include_str!("../../docs/api/orchestration/inspect.doc/InspectInfo.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct InspectInfo {
    /// The hovered node's stable id.
    pub node: NodeId,
    /// The backing component id, for an element node (`None` for group/text).
    pub component: Option<ComponentId>,
    /// The element's local name (CSS type), if any.
    pub local_name: Option<String>,
    /// The element's `id` attribute, if any.
    pub id: Option<String>,
    /// The element's classes, in order.
    pub classes: Vec<String>,
    /// The absolute border-box rectangle, in logical pixels.
    pub bounds: Rect,
}

impl fmt::Display for InspectInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // CSS-style selector: `type#id.a.b`, falling back to `‹node›` when the
        // element carries no name/id/classes (a group or bare element).
        let mut selector = String::new();
        if let Some(name) = &self.local_name {
            selector.push_str(name);
        }
        if let Some(id) = &self.id {
            selector.push('#');
            selector.push_str(id);
        }
        for class in &self.classes {
            selector.push('.');
            selector.push_str(class);
        }
        if selector.is_empty() {
            selector.push_str("‹node›");
        }

        let r = self.bounds;
        write!(
            f,
            "{selector}  ·  {:.0},{:.0} {:.0}×{:.0}  ·  node #{}",
            r.origin.x,
            r.origin.y,
            r.size.width,
            r.size.height,
            self.node.raw(),
        )?;
        if let Some(component) = self.component {
            write!(f, " · cid {:#x}", component.raw())?;
        }
        Ok(())
    }
}
