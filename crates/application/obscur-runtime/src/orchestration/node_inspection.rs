use std::fmt;

use obscur_geometry::Rect;
use obscur_identity::{ComponentId, NodeId};

/// One node in a [`TreeInspection`](super::TreeInspection): its identity, geometry and a few key
/// computed styles, as a structure an agent can read and assert on directly
/// (spec §7.6 inspectable tree/styles/layout).
#[derive(Debug, Clone, PartialEq)]
pub struct NodeInspection {
    /// The node's stable id.
    pub node: NodeId,
    /// The node's logical parent, if any (`None` for the root).
    pub parent: Option<NodeId>,
    /// Depth from the root (root is `0`), for indented rendering.
    pub depth: u32,
    /// The node classification: `"element"`, `"text"`, or `"group"`.
    pub kind: &'static str,
    /// The backing component id, for an element node.
    pub component: Option<ComponentId>,
    /// The element's local name (CSS type), if any.
    pub local_name: Option<String>,
    /// The element's `id` attribute, if any.
    pub id: Option<String>,
    /// The element's classes, in order.
    pub classes: Vec<String>,
    /// The literal content, for a text node.
    pub text: Option<String>,
    /// The absolute border-box rectangle, if the node was laid out this frame.
    pub bounds: Option<Rect>,
    /// A few key computed styles as `(name, value)` pairs (only those present).
    pub styles: Vec<(String, String)>,
}

impl fmt::Display for NodeInspection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.depth {
            f.write_str("  ")?;
        }
        // Identity: a CSS-style selector, or a text node's quoted content.
        if let Some(text) = &self.text {
            write!(f, "‹text› {text:?}")?;
        } else {
            let mut selector = String::new();
            if let Some(name) = &self.local_name {
                selector.push_str(name);
            } else {
                selector.push_str(self.kind);
            }
            if let Some(id) = &self.id {
                selector.push('#');
                selector.push_str(id);
            }
            for class in &self.classes {
                selector.push('.');
                selector.push_str(class);
            }
            f.write_str(&selector)?;
        }
        if let Some(r) = self.bounds {
            write!(
                f,
                "  @{:.0},{:.0} {:.0}×{:.0}",
                r.origin.x, r.origin.y, r.size.width, r.size.height
            )?;
        } else {
            f.write_str("  @(no box)")?;
        }
        write!(f, "  node #{}", self.node.raw())?;
        for (name, value) in &self.styles {
            write!(f, " {name}={value}")?;
        }
        Ok(())
    }
}
