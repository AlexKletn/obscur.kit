//! `SelectorResolver`: turning a [`SelectorTarget`] into a concrete node.

use obscur_geometry::Point;
use obscur_identity::{InternedSymbol, NodeId};
use obscur_runtime::UiRuntime;
use obscur_view::view::{AUTOMATION_ID, TEST_ID, ViewValue};

use crate::input::SelectorTarget;

/// Resolves a [`SelectorTarget`] against a live runtime (spec §7.4, §7.5).
///
/// The descriptors were typed data that nothing consumed; this is the engine that
/// consumes them. Resolution is read-only and side-effect free — it answers
/// *which node*, never *what happens to it*; dispatch is
/// [`SyntheticDispatcher`](crate::input::SyntheticDispatcher)'s job.
///
/// # Why a coordinate does not resolve to a node here
///
/// [`Coordinate`](SelectorTarget::Coordinate) deliberately has **no** resolution
/// step. Hit-testing a point is the runtime's own job on the real input path, and
/// re-implementing it here would produce a second, subtly different answer — the
/// exact class of bug a headless test exists to catch. A coordinate is passed
/// through to the dispatcher as-is, which lets the runtime hit-test it exactly as
/// it would a real pointer.
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectorResolver;

impl SelectorResolver {
    /// Creates a resolver.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Resolves `target` to a live node, or [`None`] when nothing matches.
    ///
    /// * [`Node`](SelectorTarget::Node) — returned only if the id is still live,
    ///   so a stale id from a previous frame answers `None` rather than a
    ///   recycled node.
    /// * [`TestId`](SelectorTarget::TestId) — the first node, in tree order,
    ///   carrying a matching `test-id` property.
    /// * [`Coordinate`](SelectorTarget::Coordinate) — always `None`: see the
    ///   type-level note. Use [`point_for`](Self::point_for) to aim a dispatch.
    #[must_use]
    pub fn resolve(self, runtime: &UiRuntime, target: &SelectorTarget) -> Option<NodeId> {
        match target {
            SelectorTarget::Node(id) => runtime.tree().contains(*id).then_some(*id),
            SelectorTarget::TestId(id) => Self::by_stable_id(runtime, TEST_ID, id),
            SelectorTarget::AutomationId(id) => Self::by_stable_id(runtime, AUTOMATION_ID, id),
            SelectorTarget::Coordinate(_) => None,
        }
    }

    /// The viewport point a synthetic pointer aimed at `target` should land on,
    /// or [`None`] when the target does not resolve to a laid-out node.
    ///
    /// A coordinate is its own point. A node or `test-id` resolves to the centre
    /// of that node's retained layout box, so the event lands where a user
    /// clicking that element would put it — and then travels the real input path,
    /// hit-test and all.
    #[must_use]
    pub fn point_for(self, runtime: &UiRuntime, target: &SelectorTarget) -> Option<Point> {
        if let SelectorTarget::Coordinate(point) = target {
            return Some(*point);
        }
        let node = self.resolve(runtime, target)?;
        let layout = runtime.layout_boxes().get(&node)?;
        let origin = layout.origin();
        let size = layout.size();
        Some(Point::new(
            origin.x + size.width / 2.0,
            origin.y + size.height / 2.0,
        ))
    }

    /// The first node, in tree order, whose `attribute` property equals `id`.
    ///
    /// Shared by `test-id` and `automation-id`: they differ only in which symbol
    /// they read, so one lookup keyed by the symbol keeps them from drifting — and,
    /// because the symbols are distinct, a target for one never matches the other.
    ///
    /// Both are read from the node's retained **properties** rather than from its
    /// style attributes, which is what keeps §7.5's "does not affect CSS
    /// specificity / layout / visual output" true by construction.
    fn by_stable_id(runtime: &UiRuntime, attribute: InternedSymbol, id: &str) -> Option<NodeId> {
        runtime
            .tree()
            .iter()
            .find(|(_, node)| {
                node.props().iter().any(|prop| {
                    prop.name() == attribute
                        && matches!(prop.value(), ViewValue::Text(text) if &**text == id)
                })
            })
            .map(|(node_id, _)| node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::SelectorResolver;
    use crate::input::SelectorTarget;
    use obscur_geometry::{Point, Size};
    use obscur_identity::{ComponentId, NodeId, RuntimeId};
    use obscur_runtime::UiRuntime;
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
    fn a_test_id_resolves_to_the_tagged_node() {
        let runtime = runtime_with(&tagged_view());
        let resolved = SelectorResolver::new()
            .resolve(&runtime, &SelectorTarget::test_id("submit"))
            .expect("the tagged node resolves");

        // It is the SECOND child — proving the match is by test-id, not position.
        let root = runtime.tree().root().expect("a root");
        assert_eq!(
            runtime.tree().get(root).expect("live").children()[1],
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
        let children = runtime.tree().get(root).expect("live").children().to_vec();

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
        let live = runtime.tree().get(root).expect("live").children()[0];
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
