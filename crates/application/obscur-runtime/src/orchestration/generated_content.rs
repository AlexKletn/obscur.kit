#![doc = include_str!("generated_content.doc/_module.md")]

use std::collections::HashMap;

use obscur_identity::NodeId;
use obscur_style::cascade::{Cascade, Declaration, ResolvedDeclarations};
use obscur_style::matching::{MediaEnvironment, Stylesheet};
use obscur_style::property::{PropertyId, PropertyValue, StyleColor};
use obscur_style::selector::SelectorMatcher;
use obscur_tree::node::RetainedNode;
use obscur_tree::tree::RetainedTree;

use super::{PseudoStates, TreeElement, part_specificity_for};

#[doc = include_str!("generated_content.doc/PLACEHOLDER_PART.md")]
const PLACEHOLDER_PART: &str = "placeholder";

#[doc = include_str!("generated_content.doc/GeneratedRun.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedRun {
    #[doc = include_str!("generated_content.doc/text.md")]
    pub text: Box<str>,
    #[doc = include_str!("generated_content.doc/color.md")]
    pub color: Option<StyleColor>,
}

#[doc = include_str!("generated_content.doc/GeneratedContent.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GeneratedContent {
    #[doc = include_str!("generated_content.doc/before.md")]
    pub before: Option<GeneratedRun>,
    #[doc = include_str!("generated_content.doc/after.md")]
    pub after: Option<GeneratedRun>,
    #[doc = include_str!("generated_content.doc/placeholder.md")]
    pub placeholder: Option<GeneratedRun>,
}

impl GeneratedContent {
    fn is_empty(&self) -> bool {
        self.before.is_none() && self.after.is_none() && self.placeholder.is_none()
    }
}

#[doc = include_str!("generated_content.doc/resolve_generated_content.md")]
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

#[doc = include_str!("generated_content.doc/resolve_pseudo.md")]
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

#[doc = include_str!("generated_content.doc/resolve_placeholder.md")]
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

#[doc = include_str!("generated_content.doc/cascade_where.md")]
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
