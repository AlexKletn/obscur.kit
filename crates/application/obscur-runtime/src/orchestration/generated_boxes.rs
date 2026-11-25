//! Layout and paint projection for generated decorations.

use obscur_identity::{FastMap, InternedSymbol, NodeId};
use obscur_layout::model::{AlignItems, LayoutDisplay};
use obscur_style::cascade::ResolvedDeclarations;
use obscur_style::computed::{ResolutionContext, StyleResolver};
use obscur_style::matching::ContainerQueryEnvironment;
use obscur_style::property::PropertyId;
use obscur_tree::node::{LogicalParent, NodeKind};
use obscur_tree::tree::RetainedTree;

use super::{ComputedStyles, GeneratedContent, GeneratedRun, LayoutBoxes, LayoutStyles};

/// Internal tag for CSS-generated decoration nodes.
pub(super) const DECORATION_TAG: InternedSymbol = InternedSymbol::from_raw(0);

/// Private tree and geometry for the decorative boxes of one arranged frame.
pub(crate) struct GeneratedBoxes {
    pub(crate) tree: RetainedTree,
    pub(crate) styles: ComputedStyles,
    pub(crate) boxes: LayoutBoxes,
    text_rows: Vec<NodeId>,
}

impl GeneratedBoxes {
    /// Project generated boxes as visual children of their owners.
    pub(crate) fn project(
        tree: &RetainedTree,
        computed: &ComputedStyles,
        generated: &FastMap<NodeId, GeneratedContent>,
        context: ResolutionContext<'_>,
        query_containers: &ContainerQueryEnvironment,
    ) -> Option<Self> {
        if !generated.values().any(GeneratedContent::has_boxes) {
            return None;
        }
        let mut projection = Self {
            tree: tree.clone(),
            styles: ComputedStyles::default(),
            boxes: LayoutBoxes::default(),
            text_rows: Vec::new(),
        };
        for (owner, _) in tree.iter() {
            let Some(content) = generated.get(&owner) else {
                continue;
            };
            let runs = [(true, &content.before), (false, &content.after)];
            if runs.iter().any(|(_, run)| run.is_some())
                && runs.iter().filter_map(|(_, run)| run.as_ref()).all(|run| {
                    !run.text.is_empty()
                        && run.declarations.iter().all(|(property, _)| {
                            *property == PropertyId::Content
                                || property.default_inheritance().is_inherited()
                        })
                })
                && tree
                    .visual_children(owner)
                    .all(|child| tree.get(child).is_some_and(|node| node.kind().is_text()))
            {
                projection.text_rows.push(owner);
            }
            let context = context
                .with_query_unit_dimensions(query_containers.descendant_unit_dimensions(owner));
            for (before, run) in runs {
                if let Some(run) = run {
                    projection.add_box(computed, owner, run, before, context);
                }
            }
        }
        Some(projection)
    }

    /// Arrange text-only flow content in one row when decorators supply only text styling.
    pub(crate) fn lay_out_text_rows(&self, styles: &mut LayoutStyles) {
        for node in &self.text_rows {
            if let Some(style) = styles.get_mut(node)
                && style.display == LayoutDisplay::Block
                && !style.writing_mode.is_vertical()
            {
                style.display = LayoutDisplay::Flex;
                style.align_items = AlignItems::Start;
            }
        }
    }

    fn add_box(
        &mut self,
        computed: &ComputedStyles,
        owner: NodeId,
        run: &GeneratedRun,
        before: bool,
        context: ResolutionContext<'_>,
    ) {
        let node = self.tree.alloc(NodeKind::Synthetic {
            tag: DECORATION_TAG,
        });
        let parent = self
            .tree
            .get_mut(owner)
            .expect("owner is in the projection");
        if before {
            parent.children.insert(0, node);
        } else {
            parent.push_child(node);
        }
        self.tree
            .get_mut(node)
            .expect("generated box is live")
            .set_logical_parent(LogicalParent::of(owner));
        let resolver = StyleResolver::new();
        let custom = FastMap::default();
        let style = resolver.resolve(&run.declarations, &custom, computed.get(&owner), context);
        if !run.text.is_empty() {
            let text = self.tree.alloc(NodeKind::Text {
                content: run.text.clone(),
            });
            self.tree
                .get_mut(node)
                .expect("generated box is live")
                .push_child(text);
            self.tree
                .get_mut(text)
                .expect("generated text is live")
                .set_logical_parent(LogicalParent::of(node));
            let text_style = resolver.resolve(
                &ResolvedDeclarations::default(),
                &custom,
                Some(&style),
                context,
            );
            self.styles.insert(text, text_style);
        }
        self.styles.insert(node, style);
    }
}
