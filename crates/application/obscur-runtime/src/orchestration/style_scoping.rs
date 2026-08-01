#![doc = include_str!("style_scoping.doc/_module.md")]

use obscur_style::selector::{
    ComplexSelector, CompoundSelector, MatchElement, SelectorMatcher, SelectorSpecificity,
    SimpleSelector,
};
use obscur_tree::node::NodeFlags;
use obscur_tree::tree::RetainedTree;

use obscur_identity::NodeId;

use super::TreeElement;

#[doc = include_str!("style_scoping.doc/component_host.md")]
#[must_use]
pub(crate) fn component_host(tree: &RetainedTree, node: NodeId) -> Option<NodeId> {
    let mut current = tree.get(node)?.logical_parent().parent();
    while let Some(id) = current {
        let record = tree.get(id)?;
        if record.flags().contains(NodeFlags::COMPONENT_HOST)
            && !record.flags().contains(NodeFlags::OPEN_STYLE_SCOPE)
        {
            return Some(id);
        }
        current = record.logical_parent().parent();
    }
    None
}

#[doc = include_str!("style_scoping.doc/subject_part_name.md")]
#[must_use]
pub(crate) fn subject_part_name(selector: &ComplexSelector) -> Option<&str> {
    selector
        .subject()
        .simples()
        .iter()
        .find_map(SimpleSelector::part_name)
}

#[doc = include_str!("style_scoping.doc/part_host_matches.md")]
#[must_use]
pub(crate) fn part_host_matches(selector: &ComplexSelector, host: &TreeElement) -> bool {
    subject_simples_match_host(selector.subject(), host) && ancestor_prefix_matches(selector, host)
}

#[doc = include_str!("style_scoping.doc/subject_simples_match_host.md")]
fn subject_simples_match_host(subject: &CompoundSelector, host: &TreeElement) -> bool {
    subject.simples().iter().all(|simple| match simple {
        // Targets, not host constraints.
        SimpleSelector::Part(_) | SimpleSelector::PseudoElement(_) | SimpleSelector::Universal => {
            true
        }
        // A component host may expose no CSS type; only enforce when it does.
        SimpleSelector::Type(name) => host
            .local_name()
            .is_none_or(|local| local.eq_ignore_ascii_case(name)),
        SimpleSelector::Class(name) => host.has_class(name),
        SimpleSelector::Id(name) => host.id() == Some(&**name),
        // Attribute / pseudo-class simples reuse the ordinary single-simple test.
        other => {
            SelectorMatcher::matches_compound(&CompoundSelector::new(vec![other.clone()]), host)
        }
    })
}

#[doc = include_str!("style_scoping.doc/ancestor_prefix_matches.md")]
fn ancestor_prefix_matches(selector: &ComplexSelector, host: &TreeElement) -> bool {
    let mut tail = selector.tail().to_vec();
    let head = if let Some(last) = tail.last_mut() {
        last.1 = CompoundSelector::new(vec![SimpleSelector::Universal]);
        selector.head().clone()
    } else {
        CompoundSelector::new(vec![SimpleSelector::Universal])
    };
    SelectorMatcher::matches_complex(&ComplexSelector::new(head, tail), host)
}

#[doc = include_str!("style_scoping.doc/part_specificity_for.md")]
#[must_use]
pub(crate) fn part_specificity_for(
    selector: &ComplexSelector,
    want: &str,
    host: &TreeElement,
) -> Option<SelectorSpecificity> {
    (subject_part_name(selector) == Some(want) && part_host_matches(selector, host))
        .then(|| selector.specificity())
}
