//! `SelectorResolver`: turning a [`SelectorTarget`] into a concrete node.

use obscur_geometry::Point;
use obscur_identity::{InternedSymbol, NodeId};
use obscur_runtime::UiRuntime;
use obscur_tree::node::RetainedNode;
use obscur_view::view::{AUTOMATION_ID, TEST_ID, ViewValue};

use crate::input::SelectorTarget;
use crate::snapshot::PipelineSnapshot;

#[doc = include_str!("../../docs/api/input/selector_resolver.doc/SelectorResolver.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectorResolver;

impl SelectorResolver {
    /// Creates a resolver.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("../../docs/api/input/selector_resolver.doc/resolve.md")]
    #[must_use]
    pub fn resolve(self, runtime: &UiRuntime, target: &SelectorTarget) -> Option<NodeId> {
        match target {
            SelectorTarget::Node(id) => runtime.tree().contains(*id).then_some(*id),
            SelectorTarget::Semantic { role, name } => {
                let tree = runtime.semantic_tree();
                unique_node(tree.by_role(*role, name.as_deref()).map(|(id, _)| id))
            }
            _ => Self::resolve_retained(runtime.tree().iter(), target),
        }
    }

    /// Finds a unique node using the retained tree and accessibility data
    /// captured in the snapshot.
    #[must_use]
    pub fn resolve_in_snapshot(
        self,
        snapshot: &PipelineSnapshot,
        target: &SelectorTarget,
    ) -> Option<NodeId> {
        match target {
            SelectorTarget::Node(id) => snapshot.tree().contains(*id).then_some(*id),
            SelectorTarget::Semantic { role, name } => {
                unique_node(snapshot.accessibility().iter().filter_map(|(id, node)| {
                    (!node.state().hidden
                        && node.role() == *role
                        && name.as_deref().is_none_or(|name| node.name() == Some(name)))
                    .then_some(id)
                }))
            }
            _ => Self::resolve_retained(snapshot.tree().iter(), target),
        }
    }

    fn resolve_retained<'a>(
        nodes: impl Iterator<Item = (NodeId, &'a RetainedNode)>,
        target: &SelectorTarget,
    ) -> Option<NodeId> {
        match target {
            SelectorTarget::Component(component) => unique_node(nodes.filter_map(|(id, node)| {
                (node.kind().component_id() == Some(*component)).then_some(id)
            })),
            SelectorTarget::TestId(id) => Self::by_stable_id(nodes, TEST_ID, id),
            SelectorTarget::AutomationId(id) => Self::by_stable_id(nodes, AUTOMATION_ID, id),
            _ => None,
        }
    }

    #[doc = include_str!("../../docs/api/input/selector_resolver.doc/point_for.md")]
    #[must_use]
    pub fn point_for(self, runtime: &UiRuntime, target: &SelectorTarget) -> Option<Point> {
        if let SelectorTarget::Coordinate(point) = target {
            return Some(*point);
        }
        let node = self.resolve(runtime, target)?;
        let bounds = runtime.presented_physical_bounds(node)?;
        if bounds.size.width <= 0.0 || bounds.size.height <= 0.0 {
            return None;
        }
        let point = Point::new(
            bounds.origin.x + bounds.size.width / 2.0,
            bounds.origin.y + bounds.size.height / 2.0,
        );
        runtime.hit_test(point).contains_node(node).then_some(point)
    }

    #[doc = include_str!("../../docs/api/input/selector_resolver.doc/by_stable_id.md")]
    fn by_stable_id<'a>(
        nodes: impl Iterator<Item = (NodeId, &'a RetainedNode)>,
        attribute: InternedSymbol,
        id: &str,
    ) -> Option<NodeId> {
        unique_node(nodes.filter_map(|(node_id, node)| {
            node.props()
                .iter()
                .any(|prop| {
                    prop.name() == attribute
                        && matches!(prop.value(), ViewValue::Text(text) if &**text == id)
                })
                .then_some(node_id)
        }))
    }
}

fn unique_node(mut matches: impl Iterator<Item = NodeId>) -> Option<NodeId> {
    let id = matches.next()?;
    matches.next().is_none().then_some(id)
}

#[cfg(test)]
mod tests {
    use super::{PipelineSnapshot, SelectorResolver};
    use crate::input::SelectorTarget;
    use obscur_geometry::{Point, Size};
    use obscur_identity::{ComponentId, NodeId, RuntimeId};
    use obscur_runtime::UiRuntime;
    use obscur_runtime::orchestration::StyleInputs;
    use obscur_style::cascade::CascadeOrigin;
    use obscur_style::matching::{BlockDeclaration, StyleRule, Stylesheet};
    use obscur_style::property::{Length, PropertyId, PropertyValue};
    use obscur_style::selector::{ComplexSelector, CompoundSelector, SelectorList, SimpleSelector};
    use obscur_view::view::{
        AUTOMATION_ID, CLASS, TEST_ID, View, ViewFragment, ViewNode, ViewProperty, ViewValue,
    };

    fn runtime_with(view: &View) -> UiRuntime {
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
        runtime.render_now(view, &[sheet()]);
        runtime
    }

    /// `.box { width: 100px; height: 50px }` — without a real size every element
    /// lays out 0x0, and "the point is inside the box" would hold vacuously.
    fn sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![StyleRule::new(
                SelectorList::new(vec![ComplexSelector::new(
                    CompoundSelector::new(vec![SimpleSelector::Class("box".into())]),
                    Vec::new(),
                )]),
                vec![
                    BlockDeclaration::normal(
                        PropertyId::Width,
                        PropertyValue::Length(Length::px(100.0)),
                    ),
                    BlockDeclaration::normal(
                        PropertyId::Height,
                        PropertyValue::Length(Length::px(50.0)),
                    ),
                ],
            )],
        )
    }

    /// Three sized elements: the second tagged `test-id="submit"`, the third
    /// tagged `automation-id="submit"` — the SAME string under a different
    /// identifier, so a resolver that confused the two would be caught.
    fn tagged_view() -> View {
        let boxed = |id: u64| {
            ViewNode::element(ComponentId::from_raw(id))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
        };
        View::from_fragment(ViewFragment::from_nodes(vec![
            boxed(1),
            boxed(2).with_prop(ViewProperty::new(TEST_ID, ViewValue::Text("submit".into()))),
            boxed(3).with_prop(ViewProperty::new(
                AUTOMATION_ID,
                ViewValue::Text("submit".into()),
            )),
        ]))
    }

    #[test]
    fn duplicate_stable_ids_do_not_resolve() {
        let view = View::from_fragment(ViewFragment::from_nodes(
            (1..=2)
                .map(|id| {
                    ViewNode::element(ComponentId::from_raw(id))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
                        .with_prop(ViewProperty::new(TEST_ID, ViewValue::Text("save".into())))
                        .with_prop(ViewProperty::new(
                            AUTOMATION_ID,
                            ViewValue::Text("save".into()),
                        ))
                })
                .collect(),
        ));
        let runtime = runtime_with(&view);
        let resolver = SelectorResolver::new();
        for target in [
            SelectorTarget::test_id("save"),
            SelectorTarget::automation_id("save"),
        ] {
            assert_eq!(resolver.resolve(&runtime, &target), None);
            assert_eq!(resolver.point_for(&runtime, &target), None);
        }
    }

    #[test]
    fn a_test_id_resolves_to_the_tagged_node() {
        let runtime = runtime_with(&tagged_view());
        let resolved = SelectorResolver::new()
            .resolve(&runtime, &SelectorTarget::test_id("submit"))
            .expect("the tagged node resolves");

        // It is the SECOND child — proving the match is by test-id, not position.
        let root = runtime.tree().root().expect("a root");
        assert_eq!(
            runtime.tree().get(root).expect("live").child_nodes()[1],
            resolved
        );
    }

    #[test]
    fn a_test_id_and_an_automation_id_never_match_each_other() {
        // §7.5 lists them as distinct identifiers, and they answer to different
        // owners — so the same STRING under each must reach different nodes. A
        // resolver that read one symbol for both would silently cross the wires.
        let runtime = runtime_with(&tagged_view());
        let root = runtime.tree().root().expect("a root");
        let children = runtime
            .tree()
            .get(root)
            .expect("live")
            .child_nodes()
            .to_vec();

        assert_eq!(
            SelectorResolver::new().resolve(&runtime, &SelectorTarget::test_id("submit")),
            Some(children[1])
        );
        assert_eq!(
            SelectorResolver::new().resolve(&runtime, &SelectorTarget::automation_id("submit")),
            Some(children[2]),
            "the automation-id node, not the test-id one"
        );
    }

    #[test]
    fn an_unknown_automation_id_resolves_to_nothing() {
        let runtime = runtime_with(&tagged_view());
        assert_eq!(
            SelectorResolver::new().resolve(&runtime, &SelectorTarget::automation_id("missing")),
            None
        );
    }

    #[test]
    fn an_unknown_test_id_resolves_to_nothing() {
        let runtime = runtime_with(&tagged_view());
        assert_eq!(
            SelectorResolver::new().resolve(&runtime, &SelectorTarget::test_id("missing")),
            None
        );
    }

    #[test]
    fn a_node_target_resolves_only_while_it_is_live() {
        let runtime = runtime_with(&tagged_view());
        let root = runtime.tree().root().expect("a root");
        let live = runtime.tree().get(root).expect("live").child_nodes()[0];
        assert_eq!(
            SelectorResolver::new().resolve(&runtime, &SelectorTarget::node(live)),
            Some(live)
        );
        // A stale id must answer None rather than a recycled node.
        assert_eq!(
            SelectorResolver::new()
                .resolve(&runtime, &SelectorTarget::node(NodeId::from_raw(9999))),
            None
        );
    }

    #[test]
    fn a_coordinate_is_its_own_point_and_resolves_no_node() {
        // Hit-testing belongs to the runtime's real input path; resolving it here
        // would be a second, divergent implementation.
        let runtime = runtime_with(&tagged_view());
        let point = Point::new(12.0, 34.0);
        let target = SelectorTarget::coordinate(point);
        assert_eq!(SelectorResolver::new().resolve(&runtime, &target), None);
        assert_eq!(
            SelectorResolver::new().point_for(&runtime, &target),
            Some(point)
        );
    }

    #[test]
    fn component_target_resolves_only_one_live_element() {
        let component = ComponentId::from_raw(2);
        let target = SelectorTarget::component(component);
        let resolver = SelectorResolver::new();
        let mut runtime = runtime_with(&tagged_view());
        let id = resolver.resolve(&runtime, &target).unwrap();
        assert_eq!(
            runtime.tree().get(id).unwrap().kind().component_id(),
            Some(component)
        );
        let repeated = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(component),
            ViewNode::element(component),
        ]));
        runtime.render_now(&repeated, &[]);
        assert_eq!(resolver.resolve(&runtime, &target), None);
    }

    #[test]
    fn snapshot_selectors_still_resolve_after_the_live_tree_changes() {
        let mut runtime = runtime_with(&tagged_view());
        let snapshot =
            PipelineSnapshot::capture(runtime.tree(), &StyleInputs::new(), runtime.viewport());
        let resolver = SelectorResolver::new();
        let target = SelectorTarget::automation_id("submit");
        let id = resolver.resolve(&runtime, &target).unwrap();
        runtime.render_now(
            &View::from_fragment(ViewFragment::from_nodes(Vec::new())),
            &[],
        );
        assert_eq!(resolver.resolve(&runtime, &target), None);
        assert_eq!(resolver.resolve_in_snapshot(&snapshot, &target), Some(id));
    }

    #[test]
    fn semantic_targets_resolve_only_unique_visible_matches() {
        use obscur_accessibility::semantic::SemanticRole;
        use obscur_view::view::LOCAL_NAME;

        let button = |id, label| {
            ViewNode::element(ComponentId::from_raw(id))
                .with_prop(ViewProperty::new(
                    LOCAL_NAME,
                    ViewValue::Text("button".into()),
                ))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text(label)]))
        };
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            button(1, "Save"),
            button(2, "Cancel"),
        ]));
        let runtime = runtime_with(&view);
        let resolver = SelectorResolver::new();
        let save = SelectorTarget::semantic(SemanticRole::Button, Some("Save"));
        let id = resolver.resolve(&runtime, &save).unwrap();
        let snapshot =
            PipelineSnapshot::capture(runtime.tree(), &StyleInputs::new(), runtime.viewport());
        assert_eq!(resolver.resolve_in_snapshot(&snapshot, &save), Some(id));
        assert_eq!(
            runtime.semantic_tree().get(id).unwrap().name(),
            Some("Save")
        );
        assert!(resolver.point_for(&runtime, &save).is_some());
        assert_eq!(
            resolver.resolve(
                &runtime,
                &SelectorTarget::semantic(SemanticRole::Button, None)
            ),
            None
        );
    }

    #[test]
    fn semantic_input_uses_physical_coordinates_at_device_scale() {
        use crate::input::{SyntheticDispatcher, SyntheticInput};

        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
        runtime.set_surface(Size::new(400.0, 400.0), 2.0);
        runtime.render_now(&tagged_view(), &[sheet()]);
        let resolver = SelectorResolver::new();
        let target = SelectorTarget::test_id("submit");
        let node = resolver.resolve(&runtime, &target).unwrap();
        let bounds = runtime.layout_boxes().get(&node).unwrap().bounds;
        assert_eq!(
            resolver.point_for(&runtime, &target),
            Some(Point::new(
                bounds.origin.x * 2.0 + bounds.size.width,
                bounds.origin.y * 2.0 + bounds.size.height,
            ))
        );
        let outcome = SyntheticDispatcher::new()
            .dispatch(&mut runtime, &SyntheticInput::pointer_move(target));
        assert_eq!(outcome.target, Some(node));
    }

    #[test]
    fn point_for_a_test_id_lands_inside_that_nodes_box() {
        let runtime = runtime_with(&tagged_view());
        let node = SelectorResolver::new()
            .resolve(&runtime, &SelectorTarget::test_id("submit"))
            .expect("resolves");
        let point = SelectorResolver::new()
            .point_for(&runtime, &SelectorTarget::test_id("submit"))
            .expect("has a point");

        let layout = runtime.layout_boxes().get(&node).expect("laid out");
        let origin = layout.origin();
        let size = layout.size();
        assert!(
            size.width > 0.0 && size.height > 0.0,
            "a zero-sized box would make the containment check vacuous"
        );
        assert!(
            point.x >= origin.x
                && point.x <= origin.x + size.width
                && point.y >= origin.y
                && point.y <= origin.y + size.height,
            "{point:?} is not inside {origin:?}+{size:?}"
        );
    }
}
