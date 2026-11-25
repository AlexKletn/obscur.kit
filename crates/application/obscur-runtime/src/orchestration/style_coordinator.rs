//! Resolves computed styles across the retained tree: [`StyleCoordinator`].

use super::IncrementalRestyle;

use std::collections::HashSet;
use std::rc::Rc;

use obscur_geometry::Size;
use obscur_identity::{FastBuildHasher, FastMap, FastSet, NodeId};
use obscur_style::computed::{
    ChMetrics, ComputedStyle, ComputedValue, CustomValueParser, ResolutionContext, StyleResolver,
};
use obscur_style::invalidation::StyleInvalidation;
use obscur_style::matching::MediaEnvironment;
use obscur_style::property::{EnvironmentValues, PropertyId, layout_keyword};
use obscur_tree::tree::RetainedTree;

use super::StyleInputs;

/// The computed style for every node the coordinator resolved, keyed by
/// [`NodeId`].
pub type ComputedStyles = FastMap<NodeId, ComputedStyle>;

/// The installed grammar for a substituted custom property, shared by reference
/// so a rebuilt resolution context can carry it without cloning it.
pub type CustomValues = Rc<dyn CustomValueParser>;

#[doc = include_str!("../../docs/api/orchestration/style_coordinator.doc/StyleCoordinator.md")]
#[derive(Clone, Default)]
pub struct StyleCoordinator {
    /// The stateless resolver that computes each node's style.
    resolver: StyleResolver,
    #[doc = include_str!("../../docs/api/orchestration/style_coordinator.doc/custom_values.md")]
    custom_values: Option<CustomValues>,
    locale_parent_style: Option<ComputedStyle>,
}

impl std::fmt::Debug for StyleCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StyleCoordinator")
            .field("resolver", &self.resolver)
            .field("custom_values", &self.custom_values.is_some())
            .finish()
    }
}

impl StyleCoordinator {
    /// Creates a coordinator with a fresh resolver.
    #[must_use]
    pub fn new() -> Self {
        Self {
            resolver: StyleResolver::new(),
            custom_values: None,
            locale_parent_style: None,
        }
    }

    pub(crate) fn set_locale_direction(&mut self, direction: obscur_text::shaping::TextDirection) {
        self.locale_parent_style = (direction == obscur_text::shaping::TextDirection::RightToLeft)
            .then(|| {
                let mut style = ComputedStyle::new();
                style.set(
                    PropertyId::Direction,
                    ComputedValue::keyword(layout_keyword::DIRECTION_RTL),
                );
                style
            });
    }

    #[doc = include_str!("../../docs/api/orchestration/style_coordinator.doc/set_custom_values.md")]
    pub fn set_custom_values(&mut self, parser: CustomValues) {
        self.custom_values = Some(parser);
    }

    pub(crate) fn custom_value_parser(&self) -> Option<&dyn CustomValueParser> {
        self.custom_values.as_deref()
    }

    /// Whether a grammar has been installed. A host that installs one on every
    /// stylesheet replacement asks this first, so re-installing does not become a
    /// per-frame allocation.
    #[must_use]
    pub fn has_custom_values(&self) -> bool {
        self.custom_values.is_some()
    }

    #[doc = include_str!("../../docs/api/orchestration/style_coordinator.doc/attach.md")]
    fn attach<'a>(
        &'a self,
        ctx: ResolutionContext<'a>,
        metrics: Option<&'a dyn ChMetrics>,
        environment: EnvironmentValues,
    ) -> ResolutionContext<'a> {
        let ctx = ctx.with_environment(environment);
        let ctx = match self.custom_values.as_deref() {
            Some(parser) => ctx.with_custom_values(parser),
            None => ctx,
        };
        match metrics {
            Some(metrics) => ctx.with_metrics(metrics),
            None => ctx,
        }
    }

    #[doc = include_str!("../../docs/api/orchestration/style_coordinator.doc/resolve.md")]
    #[must_use]
    pub fn resolve(
        &self,
        tree: &RetainedTree,
        inputs: &StyleInputs,
        viewport: Size,
        environment: EnvironmentValues,
        metrics: Option<&dyn ChMetrics>,
    ) -> ComputedStyles {
        self.resolve_with_media(
            tree,
            inputs,
            viewport,
            environment,
            metrics,
            &MediaEnvironment::unbounded(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_with_media(
        &self,
        tree: &RetainedTree,
        inputs: &StyleInputs,
        viewport: Size,
        environment: EnvironmentValues,
        metrics: Option<&dyn ChMetrics>,
        media: &MediaEnvironment,
    ) -> ComputedStyles {
        let Some(root) = tree.root() else {
            return ComputedStyles::default();
        };
        let order = Self::preorder(root, |id| {
            tree.get(id)
                .map_or_else(Vec::new, |node| node.child_nodes().to_vec())
        });
        let root_element = order
            .iter()
            .map(|&(node, _)| node)
            .find(|&node| tree.get(node).is_some_and(|n| n.kind().is_element()));
        self.resolve_in_order(
            &order,
            inputs,
            viewport,
            environment,
            root_element,
            metrics,
            media,
        )
    }

    #[doc = include_str!("../../docs/api/orchestration/style_coordinator.doc/resolve_incremental.md")]
    #[must_use]
    pub fn resolve_incremental(
        &self,
        tree: &RetainedTree,
        inputs: &StyleInputs,
        viewport: Size,
        environment: EnvironmentValues,
        dirty: &StyleInvalidation,
        styles: &mut ComputedStyles,
        metrics: Option<&dyn ChMetrics>,
    ) -> IncrementalRestyle {
        self.resolve_incremental_with_media(
            tree,
            inputs,
            viewport,
            environment,
            dirty,
            styles,
            metrics,
            &MediaEnvironment::unbounded(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_incremental_with_media(
        &self,
        tree: &RetainedTree,
        inputs: &StyleInputs,
        viewport: Size,
        environment: EnvironmentValues,
        dirty: &StyleInvalidation,
        styles: &mut ComputedStyles,
        metrics: Option<&dyn ChMetrics>,
        media: &MediaEnvironment,
    ) -> IncrementalRestyle {
        let mut out = IncrementalRestyle::default();
        let Some(root) = tree.root() else {
            return out;
        };
        let order = Self::preorder(root, |id| {
            tree.get(id)
                .map_or_else(Vec::new, |node| node.child_nodes().to_vec())
        });
        let root_element = order
            .iter()
            .map(|&(node, _)| node)
            .find(|&node| tree.get(node).is_some_and(|n| n.kind().is_element()));

        let mut ctx = self.attach(
            ResolutionContext::for_root(viewport.width, viewport.height),
            metrics,
            environment,
        );
        for &(node, parent) in &order {
            // The `contains_key` arm is a hole guard, not part of the model: the
            // caller's precondition says the retained map covers this tree, and if it
            // somehow does not, resolving the node is better than leaving it styleless
            // — but the caller must then not treat unlisted nodes as unchanged, which
            // is what `introduced` tells it.
            if dirty.is_invalidated(node) || !styles.contains_key(&node) {
                let parent_style = parent
                    .and_then(|p| styles.get(&p))
                    .or(self.locale_parent_style.as_ref());
                let computed = self.resolver.resolve(
                    inputs.cascaded_for(node),
                    inputs.custom_for(node),
                    parent_style,
                    ctx.with_query_unit_dimensions(media.query_unit_dimensions(node)),
                );
                out.resolved += 1;
                match styles.insert(node, computed) {
                    Some(previous) => {
                        out.previous.insert(node, previous);
                    }
                    None => out.introduced = true,
                }
            }
            // Read the `rem` base back off whatever style the root element ended
            // with — freshly resolved or retained — so it is the font-size the root
            // actually paints at either way, exactly as in `resolve_in_order`.
            if Some(node) == root_element
                && let Some(style) = styles.get(&node)
            {
                ctx = self.attach(
                    ResolutionContext::new(
                        StyleResolver::font_size_of(style),
                        viewport.width,
                        viewport.height,
                    ),
                    metrics,
                    environment,
                );
            }
        }
        out
    }

    #[doc = include_str!("../../docs/api/orchestration/style_coordinator.doc/preorder.md")]
    fn preorder<F>(root: NodeId, children_of: F) -> Vec<(NodeId, Option<NodeId>)>
    where
        F: Fn(NodeId) -> Vec<NodeId>,
    {
        let mut order: Vec<(NodeId, Option<NodeId>)> = Vec::new();
        let mut visited: FastSet<NodeId> = HashSet::default();
        let mut stack: Vec<(NodeId, Option<NodeId>)> = vec![(root, None)];

        while let Some((node, parent)) = stack.pop() {
            if !visited.insert(node) {
                // Already emitted (cycle or shared child): do not revisit.
                continue;
            }
            order.push((node, parent));
            // Push children reversed so the leftmost child is popped first,
            // preserving left-to-right pre-order.
            for &child in children_of(node).iter().rev() {
                if !visited.contains(&child) {
                    stack.push((child, Some(node)));
                }
            }
        }

        order
    }

    #[doc = include_str!("../../docs/api/orchestration/style_coordinator.doc/resolve_in_order.md")]
    #[allow(clippy::too_many_arguments)]
    fn resolve_in_order(
        &self,
        order: &[(NodeId, Option<NodeId>)],
        inputs: &StyleInputs,
        viewport: Size,
        environment: EnvironmentValues,
        root_element: Option<NodeId>,
        metrics: Option<&dyn ChMetrics>,
        media: &MediaEnvironment,
    ) -> ComputedStyles {
        let mut out: ComputedStyles =
            ComputedStyles::with_capacity_and_hasher(order.len(), FastBuildHasher::default());
        let mut ctx = self.attach(
            ResolutionContext::for_root(viewport.width, viewport.height),
            metrics,
            environment,
        );
        for &(node, parent) in order {
            let parent_style = parent
                .and_then(|p| out.get(&p))
                .or(self.locale_parent_style.as_ref());
            let computed = self.resolver.resolve(
                inputs.cascaded_for(node),
                inputs.custom_for(node),
                parent_style,
                ctx.with_query_unit_dimensions(media.query_unit_dimensions(node)),
            );
            if Some(node) == root_element {
                ctx = self.attach(
                    ResolutionContext::new(
                        StyleResolver::font_size_of(&computed),
                        viewport.width,
                        viewport.height,
                    ),
                    metrics,
                    environment,
                );
            }
            out.insert(node, computed);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// A viewport for tests that are about the walk, not about `vw`/`vh`.
    fn viewport() -> Size {
        Size::new(800.0, 600.0)
    }
    use obscur_style::cascade::ResolvedDeclarations;
    use obscur_style::computed::ComputedValue;
    use obscur_style::property::{PropertyId, PropertyValue, StyleColor, layout_keyword};

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    fn color_of(styles: &ComputedStyles, id: NodeId) -> Option<StyleColor> {
        styles
            .get(&id)
            .and_then(|style| style.get(PropertyId::Color))
            .and_then(ComputedValue::as_color)
    }

    #[test]
    fn empty_tree_resolves_to_empty_map() {
        let coordinator = StyleCoordinator::new();
        let tree = RetainedTree::new();
        let inputs = StyleInputs::new();
        assert!(
            coordinator
                .resolve(&tree, &inputs, viewport(), EnvironmentValues::none(), None)
                .is_empty()
        );
    }

    #[test]
    fn preorder_is_parent_first() {
        // root -> child -> grandchild (a chain).
        let (root, child, grandchild) = (node(1), node(2), node(3));
        let mut children: FastMap<NodeId, Vec<NodeId>> = HashMap::default();
        children.insert(root, vec![child]);
        children.insert(child, vec![grandchild]);

        let order =
            StyleCoordinator::preorder(root, |id| children.get(&id).cloned().unwrap_or_default());

        assert_eq!(
            order,
            vec![(root, None), (child, Some(root)), (grandchild, Some(child)),]
        );
    }

    #[test]
    fn preorder_visits_siblings_left_to_right_parent_before_children() {
        // root -> [a, b]; a -> [a1].
        let (root, a, b, a1) = (node(1), node(2), node(3), node(4));
        let mut children: FastMap<NodeId, Vec<NodeId>> = HashMap::default();
        children.insert(root, vec![a, b]);
        children.insert(a, vec![a1]);

        let order =
            StyleCoordinator::preorder(root, |id| children.get(&id).cloned().unwrap_or_default());

        // Every parent appears before each of its children.
        let position = |target: NodeId| order.iter().position(|&(n, _)| n == target);
        assert!(position(root) < position(a));
        assert!(position(a) < position(a1));
        assert!(position(root) < position(b));
        // Left-to-right: a (and its subtree) before b.
        assert!(position(a) < position(b));
        assert_eq!(order.len(), 4);
    }

    #[test]
    fn preorder_is_cycle_safe() {
        // root -> child -> root (a cycle) and child -> root again via diamond.
        let (root, child) = (node(1), node(2));
        let mut children: FastMap<NodeId, Vec<NodeId>> = HashMap::default();
        children.insert(root, vec![child]);
        children.insert(child, vec![root]);

        let order =
            StyleCoordinator::preorder(root, |id| children.get(&id).cloned().unwrap_or_default());

        // Terminates and emits each node exactly once, parent-first.
        assert_eq!(order, vec![(root, None), (child, Some(root))]);
    }

    #[test]
    fn inheritance_flows_down_parent_first() {
        // root -> child -> grandchild; `color` inherits by default.
        let (root, child, grandchild) = (node(1), node(2), node(3));
        let order = vec![(root, None), (child, Some(root)), (grandchild, Some(child))];

        let red = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let mut root_decls = ResolvedDeclarations::new();
        root_decls.insert(PropertyId::Color, PropertyValue::Color(red));

        let mut inputs = StyleInputs::new();
        inputs.set_cascaded(root, root_decls);
        // child and grandchild have no cascaded inputs of their own.

        let coordinator = StyleCoordinator::new();
        let styles = coordinator.resolve_in_order(
            &order,
            &inputs,
            viewport(),
            EnvironmentValues::none(),
            Some(root),
            None,
            &MediaEnvironment::unbounded(),
        );

        assert_eq!(color_of(&styles, root), Some(red));
        // Proves parent-first inheritance: descendants inherit the root's color.
        assert_eq!(color_of(&styles, child), Some(red));
        assert_eq!(color_of(&styles, grandchild), Some(red));
    }

    #[test]
    fn relative_font_weight_resolves_top_down_over_the_tree() {
        // root font-weight:700; child A `bolder` -> 900; child B (none) inherits 700.
        let (root, a, b) = (node(1), node(2), node(3));
        let order = vec![(root, None), (a, Some(root)), (b, Some(root))];

        let mut root_decls = ResolvedDeclarations::new();
        root_decls.insert(PropertyId::FontWeight, PropertyValue::Number(700.0));
        let mut a_decls = ResolvedDeclarations::new();
        a_decls.insert(
            PropertyId::FontWeight,
            PropertyValue::Keyword(layout_keyword::FONT_WEIGHT_BOLDER),
        );

        let mut inputs = StyleInputs::new();
        inputs.set_cascaded(root, root_decls);
        inputs.set_cascaded(a, a_decls);
        // Child B has no cascaded font-weight of its own.

        let styles = StyleCoordinator::new().resolve_in_order(
            &order,
            &inputs,
            viewport(),
            EnvironmentValues::none(),
            Some(root),
            None,
            &MediaEnvironment::unbounded(),
        );
        let weight = |id| match styles
            .get(&id)
            .and_then(|s| s.get(PropertyId::FontWeight))
            .map(ComputedValue::value)
        {
            Some(PropertyValue::Number(n)) => Some(*n),
            _ => None,
        };
        // `bolder` resolves against the parent's computed 700 -> 900.
        assert_eq!(weight(a), Some(900.0));
        // No authored weight -> inherits the parent's 700.
        assert_eq!(weight(b), Some(700.0));
    }

    #[test]
    fn node_without_cascaded_input_still_gets_a_style() {
        let (root, child) = (node(1), node(2));
        let order = vec![(root, None), (child, Some(root))];

        // No inputs at all for any node.
        let inputs = StyleInputs::new();
        let coordinator = StyleCoordinator::new();
        let styles = coordinator.resolve_in_order(
            &order,
            &inputs,
            viewport(),
            EnvironmentValues::none(),
            Some(root),
            None,
            &MediaEnvironment::unbounded(),
        );

        // Both nodes are resolved; the child gets defaults/inheritance.
        assert!(styles.contains_key(&child));
        // `opacity` is not inherited and defaults to its initial value 1.0.
        assert_eq!(
            styles.get(&child).and_then(ComputedStyle::opacity),
            Some(1.0)
        );
    }
}
