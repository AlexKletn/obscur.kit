#![doc = include_str!("../../docs/api/orchestration/style_scoping.doc/_module.md")]

use obscur_style::selector::{
    ComplexSelector, CompoundSelector, MatchElement, SelectorMatcher, SelectorSpecificity,
    SimpleSelector,
};
use obscur_tree::node::NodeFlags;
use obscur_tree::tree::RetainedTree;

use obscur_identity::NodeId;

use super::TreeElement;

#[doc = include_str!("../../docs/api/orchestration/style_scoping.doc/component_host.md")]
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

/// Returns the exported-part name a complex selector's subject compound targets
/// (`Component::part(name)` → `name`), or [`None`] when the subject names no
/// `::part()`.
#[must_use]
pub(crate) fn subject_part_name(selector: &ComplexSelector) -> Option<&str> {
    selector
        .subject()
        .simples()
        .iter()
        .find_map(SimpleSelector::part_name)
}

#[doc = include_str!("../../docs/api/orchestration/style_scoping.doc/part_host_matches.md")]
#[must_use]
pub(crate) fn part_host_matches(selector: &ComplexSelector, host: &TreeElement) -> bool {
    subject_simples_match_host(selector.subject(), host) && ancestor_prefix_matches(selector, host)
}

/// Matches the subject compound's host-constraining simples against `host`.
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

/// Matches the combinator prefix to the left of the subject against `host`'s
/// ancestors/siblings, by replacing the subject compound with the universal
/// selector (which always matches the host) and running the ordinary matcher.
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

#[doc = include_str!("../../docs/api/orchestration/style_scoping.doc/part_specificity_for.md")]
#[must_use]
pub(crate) fn part_specificity_for(
    selector: &ComplexSelector,
    want: &str,
    host: &TreeElement,
) -> Option<SelectorSpecificity> {
    (subject_part_name(selector) == Some(want) && part_host_matches(selector, host))
        .then(|| selector.specificity())
}
