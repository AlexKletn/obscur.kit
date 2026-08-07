#![doc = include_str!("node_converter.doc/_module.md")]

use accesskit::{Invalid, Node, NodeId, Rect, Toggled};
use obscur_accessibility::semantic::{SemanticNode, SemanticValue};
use obscur_identity::NodeId as DomainNodeId;

use super::{ActionConverter, RoleConverter};

#[doc = include_str!("node_converter.doc/NodeConverter.md")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeConverter {
    role: RoleConverter,
    action: ActionConverter,
}

impl NodeConverter {
    #[doc = include_str!("node_converter.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            role: RoleConverter::new(),
            action: ActionConverter::new(),
        }
    }

    #[doc = include_str!("node_converter.doc/node_id.md")]
    #[must_use]
    pub const fn node_id(&self, id: DomainNodeId) -> NodeId {
        NodeId(id.raw())
    }

    #[doc = include_str!("node_converter.doc/convert.md")]
    #[must_use]
    pub fn convert(&self, node: &SemanticNode, children: &[DomainNodeId]) -> Node {
        let mut out = Node::new(self.role.convert(node.role()));

        if let Some(name) = node.name() {
            out.set_label(name);
        }
        if let Some(description) = node.description() {
            out.set_description(description);
        }

        let state = node.state();
        if state.disabled {
            out.set_disabled();
        }
        if node.is_ignored() {
            out.set_hidden();
        }
        if state.selected {
            out.set_selected(true);
        }
        if state.required {
            out.set_required();
        }
        if state.readonly {
            out.set_read_only();
        }
        if state.busy {
            out.set_busy();
        }
        if state.invalid {
            out.set_invalid(Invalid::True);
        }
        if let Some(checked) = state.checked {
            out.set_toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            });
        }
        if let Some(expanded) = state.expanded {
            out.set_expanded(expanded);
        }

        match node.value() {
            SemanticValue::None | SemanticValue::Bool(_) => {}
            SemanticValue::Text(text) => out.set_value(text.clone()),
            SemanticValue::Number { value, min, max } => {
                out.set_numeric_value(f64::from(*value));
                out.set_min_numeric_value(f64::from(*min));
                out.set_max_numeric_value(f64::from(*max));
            }
            SemanticValue::Range {
                low,
                high,
                min,
                max,
            } => {
                // AccessKit has one `numeric_value` per node, as every platform
                // API it fronts does, so a two-thumb control cannot state its
                // position numerically here. Filling that slot with one end
                // would announce half the control as the whole of it, so it is
                // left EMPTY and the two ends are spoken through the value
                // string instead — which is what a reader falls back to.
                // The bounds are still true of the control, so they are stated.
                out.set_value(format!("{low} to {high}"));
                out.set_min_numeric_value(f64::from(*min));
                out.set_max_numeric_value(f64::from(*max));
            }
        }

        for action in node.actions() {
            if let Some(mapped) = self.action.to_accesskit(*action) {
                out.add_action(mapped);
            }
        }

        if let Some(bounds) = node.bounds() {
            let x0 = f64::from(bounds.origin.x);
            let y0 = f64::from(bounds.origin.y);
            out.set_bounds(Rect::new(
                x0,
                y0,
                f64::from(bounds.right()),
                f64::from(bounds.bottom()),
            ));
        }

        if !children.is_empty() {
            let ids: Vec<NodeId> = children.iter().map(|id| self.node_id(*id)).collect();
            out.set_children(ids);
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::{Action, Role};
    use obscur_accessibility::semantic::{SemanticAction, SemanticRole, SemanticState};
    use obscur_geometry::{Point, Rect as DomainRect, Size};

    fn button(name: &str) -> SemanticNode {
        SemanticNode::new(DomainNodeId::from_raw(1), SemanticRole::Button).with_name(name)
    }

    #[test]
    fn button_with_name_maps_role_and_label() {
        let node = button("OK").with_action(SemanticAction::Activate);
        let out = NodeConverter::new().convert(&node, &[]);
        assert_eq!(out.role(), Role::Button);
        assert_eq!(out.label(), Some("OK"));
        assert!(out.supports_action(Action::Click));
    }

    #[test]
    fn disabled_node_sets_disabled_flag() {
        let node = button("OK").with_state(SemanticState::new().disabled(true));
        let out = NodeConverter::new().convert(&node, &[]);
        assert!(out.is_disabled());
    }

    #[test]
    fn checked_checkbox_sets_toggled() {
        let node = SemanticNode::new(DomainNodeId::from_raw(2), SemanticRole::Checkbox)
            .with_state(SemanticState::new().with_checked(Some(true)));
        let out = NodeConverter::new().convert(&node, &[]);
        assert_eq!(out.role(), Role::CheckBox);
        assert_eq!(out.toggled(), Some(Toggled::True));
    }

    #[test]
    fn unchecked_checkbox_sets_toggled_false() {
        let node = SemanticNode::new(DomainNodeId::from_raw(2), SemanticRole::Checkbox)
            .with_state(SemanticState::new().with_checked(Some(false)));
        let out = NodeConverter::new().convert(&node, &[]);
        assert_eq!(out.toggled(), Some(Toggled::False));
    }

    #[test]
    fn hidden_node_sets_hidden() {
        let node = button("x").with_state(SemanticState::new().hidden(true));
        let out = NodeConverter::new().convert(&node, &[]);
        assert!(out.is_hidden());
    }

    #[test]
    fn number_value_sets_numeric_and_bounds() {
        let node = SemanticNode::new(DomainNodeId::from_raw(3), SemanticRole::Slider)
            .with_value(SemanticValue::number(5.0, 0.0, 10.0));
        let out = NodeConverter::new().convert(&node, &[]);
        assert_eq!(out.numeric_value(), Some(5.0));
        assert_eq!(out.min_numeric_value(), Some(0.0));
        assert_eq!(out.max_numeric_value(), Some(10.0));
    }

    #[test]
    fn range_value_names_both_ends_and_claims_no_single_number() {
        // A two-thumb slider announced as "20" would be a lie about where the
        // control is; announced as "20 to 80" it is the truth AccessKit's
        // one-number slot cannot hold.
        let node = SemanticNode::new(DomainNodeId::from_raw(30), SemanticRole::Slider)
            .with_value(SemanticValue::range(20.0, 80.0, 0.0, 100.0));
        let out = NodeConverter::new().convert(&node, &[]);
        assert_eq!(out.value(), Some("20 to 80"));
        assert_eq!(out.numeric_value(), None);
        assert_eq!(out.min_numeric_value(), Some(0.0));
        assert_eq!(out.max_numeric_value(), Some(100.0));
    }

    #[test]
    fn a_valueless_progress_bar_carries_no_number() {
        // The indeterminate bar. Omitting the numeric value is exactly how ARIA
        // and AccessKit spell "running, position unknown"; a zero here would say
        // "not started", which is a different and false thing.
        let node = SemanticNode::new(DomainNodeId::from_raw(31), SemanticRole::ProgressBar);
        let out = NodeConverter::new().convert(&node, &[]);
        assert_eq!(out.numeric_value(), None);
        assert_eq!(out.min_numeric_value(), None);
        assert_eq!(out.max_numeric_value(), None);
        assert_eq!(out.value(), None);
    }

    #[test]
    fn text_value_sets_value_string() {
        let node = SemanticNode::new(DomainNodeId::from_raw(4), SemanticRole::TextInput)
            .with_value(SemanticValue::text("hi"));
        let out = NodeConverter::new().convert(&node, &[]);
        assert_eq!(out.value(), Some("hi"));
    }

    #[test]
    fn bounds_map_to_edges() {
        let node = button("b").with_bounds(DomainRect::new(
            Point { x: 1.0, y: 2.0 },
            Size {
                width: 3.0,
                height: 4.0,
            },
        ));
        let out = NodeConverter::new().convert(&node, &[]);
        let rect = out.bounds().expect("bounds set");
        assert_eq!(rect.x0, 1.0);
        assert_eq!(rect.y0, 2.0);
        assert_eq!(rect.x1, 4.0);
        assert_eq!(rect.y1, 6.0);
    }

    #[test]
    fn children_are_attached() {
        let node = SemanticNode::new(DomainNodeId::from_raw(1), SemanticRole::List);
        let out = NodeConverter::new().convert(
            &node,
            &[DomainNodeId::from_raw(2), DomainNodeId::from_raw(3)],
        );
        assert_eq!(out.children(), &[NodeId(2), NodeId(3)]);
    }

    #[test]
    fn node_id_preserves_raw() {
        let c = NodeConverter::new();
        assert_eq!(c.node_id(DomainNodeId::from_raw(42)), NodeId(42));
    }
}
