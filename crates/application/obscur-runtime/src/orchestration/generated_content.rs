//! Resolving §4.16 `::before` / `::after` generated content into paint-only runs.
//!
//! A matched pseudo-element rule (`.badge::before { content: "…" }`) paints a
//! string before / after the originating element's own content, without the
//! generated box ever becoming a tree node, an accessibility node, or a hit-test
//! target.
//!
//! [`resolve_generated_content`] returns, per node, the runs the scene
//! coordinator paints inside that node's own paint unit — so a `::before` run
//! sorts before the element's real text child in paint order.

use std::collections::HashMap;

use obscur_identity::NodeId;
use obscur_style::cascade::{Cascade, Declaration, ResolvedDeclarations};
use obscur_style::matching::{MediaEnvironment, Stylesheet};
use obscur_style::property::{PropertyId, PropertyValue, StyleColor};
use obscur_style::selector::SelectorMatcher;
use obscur_tree::node::RetainedNode;
use obscur_tree::tree::RetainedTree;

use super::{PseudoStates, TreeElement, part_specificity_for};

/// The reserved exported-part name a text control gives its placeholder hint, so
/// `Input::part(placeholder)` (the CSS `::placeholder` analogue) can restyle it.
const PLACEHOLDER_PART: &str = "placeholder";

/// One paint-only generated run: its text and an optional resolved colour.
///
/// A `None` colour means "no `color` was resolved for this decoration" — the
/// scene coordinator falls back to the owning node's own ink.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedRun {
    /// The literal text to paint.
    pub text: Box<str>,
    /// The resolved `color`, or [`None`] to inherit the node's ink.
    pub color: Option<StyleColor>,
}

/// The generated decoration attached to one node (§4.16): `::before` / `::after`
/// content, plus the text control's synthetic **placeholder** part.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GeneratedContent {
    /// `::before` content, painted before the node's own content.
    pub before: Option<GeneratedRun>,
    /// `::after` content, painted after the node's own content.
    pub after: Option<GeneratedRun>,
    /// The text control's **placeholder** hint (the CSS `::placeholder` analogue),
    /// present only while the field is empty (`:placeholder-shown`). Its colour is
    /// resolved through the exported-part selector `Component::part(placeholder)`.
    /// This is a §4.16 synthetic part: paint-only, DevTools-visible, hit-transparent,
    /// and never a component-tree child.
    pub placeholder: Option<GeneratedRun>,
}

impl GeneratedContent {
    fn is_empty(&self) -> bool {
        self.before.is_none() && self.after.is_none() && self.placeholder.is_none()
    }
}

/// Resolves the generated decoration for every element node in `tree` under
/// `sheets` (in `environment`), keyed by node id. Nodes with no decoration are
/// omitted.
#[must_use]
pub(crate) fn resolve_generated_content(
    tree: &RetainedTree,
    sheets: &[Stylesheet],
    environment: &MediaEnvironment,
) -> HashMap<NodeId, GeneratedContent> {
    let pseudo = PseudoStates::new();
    let mut out = HashMap::new();
    for (id, node) in tree.iter() {
        if !node.kind().is_element() {
            continue;
        }
        let element = TreeElement::with_pseudo(tree, id, &pseudo);
        let content = GeneratedContent {
            before: resolve_pseudo(sheets, &element, "before", environment),
            after: resolve_pseudo(sheets, &element, "after", environment),
            placeholder: resolve_placeholder(sheets, &element, node, environment),
        };
        if !content.is_empty() {
            out.insert(id, content);
        }
    }
    out
}

/// Resolves the `content` (and its `color`) of `element`'s `pseudo`
/// (`before` / `after`) generated box, or [`None`] when no matched rule sets a
/// `content` string.
fn resolve_pseudo(
    sheets: &[Stylesheet],
    element: &TreeElement<'_>,
    pseudo: &str,
    environment: &MediaEnvironment,
) -> Option<GeneratedRun> {
    let resolved = cascade_where(sheets, environment, |selector| {
        SelectorMatcher::matches_complex_for_pseudo(selector, element, pseudo)
            .then(|| selector.specificity())
    });
    let PropertyValue::String(text) = resolved.get(PropertyId::Content)? else {
        return None;
    };
    Some(GeneratedRun {
        text: text.clone(),
        color: resolved
            .get(PropertyId::Color)
            .and_then(PropertyValue::as_color),
    })
}

/// Resolves the synthetic **placeholder** run for a text control (§4.16), or
/// [`None`] when the node carries no non-empty placeholder or its value is not
/// empty (the CSS `:placeholder-shown` rule — the hint shows only while the field
/// is empty).
///
/// Unlike an ordinary component-internal part (which crosses the scope boundary
/// only from inside a mounted component host), the placeholder's host is the field
/// element **itself**: the exported-part selector `Component::part(placeholder)`
/// is matched with the field as the host, so `Input::part(placeholder) { color }`
/// recolours the hint without the field having to be a component host. Its text is
/// the reserved `PLACEHOLDER` attribute the widget emits.
fn resolve_placeholder(
    sheets: &[Stylesheet],
    element: &TreeElement<'_>,
    node: &RetainedNode,
    environment: &MediaEnvironment,
) -> Option<GeneratedRun> {
    let attributes = node.attributes();
    // `:placeholder-shown` — the hint is shown only while the field's value is
    // empty. A control with no declared value (a bare editable) is treated as empty.
    if attributes
        .control_value()
        .is_some_and(|value| !value.is_empty())
    {
        return None;
    }
    let placeholder = attributes.placeholder().filter(|hint| !hint.is_empty())?;
    let resolved = cascade_where(sheets, environment, |selector| {
        part_specificity_for(selector, PLACEHOLDER_PART, element)
    });
    Some(GeneratedRun {
        text: placeholder.into(),
        color: resolved
            .get(PropertyId::Color)
            .and_then(PropertyValue::as_color),
    })
}

/// Cascades every declaration whose owning selector `select` accepts (returning
/// that selector's specificity), over `sheets` in document order.
///
/// This mirrors [`StyleMatcher::match_element_in`](obscur_style::matching::StyleMatcher::match_element_in):
/// `source_order` advances over every encountered declaration so document-order
/// tie-breaks are preserved, but only accepted rules' declarations are collected.
fn cascade_where(
    sheets: &[Stylesheet],
    environment: &MediaEnvironment,
    mut select: impl FnMut(
        &obscur_style::selector::ComplexSelector,
    ) -> Option<obscur_style::selector::SelectorSpecificity>,
) -> ResolvedDeclarations {
    let mut collected: Vec<Declaration> = Vec::new();
    let mut source_order: u32 = 0;
    for sheet in sheets {
        let origin = sheet.origin();
        for rule in sheet.rules() {
            let specificity = rule
                .selectors()
                .selectors()
                .iter()
                .filter_map(&mut select)
                .max();
            let condition_holds = rule
                .condition()
                .is_none_or(|condition| condition.holds(environment));
            for block_decl in rule.block() {
                let order = source_order;
                source_order = source_order.wrapping_add(1);
                if condition_holds && let Some(specificity) = specificity {
                    collected.push(Declaration::new(
                        block_decl.property(),
                        block_decl.value().clone(),
                        block_decl.important_flag(),
                        origin,
                        specificity,
                        order,
                        None,
                    ));
                }
            }
        }
    }
    Cascade::new().resolve(&collected)
}
