//! Scope boundaries and `::part()` matching for authored CSS against the
//! retained tree (§2.12 exported style parts, §4.15 scoped-by-default).
//!
//! A component's rendered output is *scoped by default*: an outside stylesheet's
//! ordinary selectors do not reach a component-internal element. The one door
//! through the boundary is the exported style part — `Component::part(name)` —
//! which matches an internal element the component exports under `name` while
//! the rest of the compound matches the component **host**.
//!
//! §4.15 has a second half, and it lives here too: "*explicitly open/global
//! scopes are possible*". A component whose contract declares
//! [`StyleScope::Open`](obscur_view::component::StyleScope::Open) is mounted with
//! [`NodeFlags::OPEN_STYLE_SCOPE`] and closes no boundary at all, so everything
//! it renders is matched exactly as authored content is. Until that existed a
//! component's output could be styled by nothing whatsoever — there is no
//! producer for the `ComponentModule` cascade origin yet, so `::part()` was the
//! only route to any declaration reaching an internal element, and the external
//! consumer audit's finding 2-1 recorded the consequence: "a component-rendered
//! subtree is matched by no selector".
//!
//! The pure `obscur-style` matcher cannot decide this: it has no view of the
//! component-host boundary, which lives on the retained tree
//! ([`NodeFlags::COMPONENT_HOST`]). These helpers supply that view so the
//! runtime's style-matching coordinator (and the generated-content resolver) can
//! anchor a `::part()` rule on the host while gating the crossing on the export.

use obscur_style::selector::{
    ComplexSelector, CompoundSelector, MatchElement, SelectorMatcher, SelectorSpecificity,
    SimpleSelector,
};
use obscur_tree::node::NodeFlags;
use obscur_tree::tree::RetainedTree;

use obscur_identity::NodeId;

use super::TreeElement;

/// Returns the nearest **strict** ancestor of `node` that is a *scoping*
/// component host (`NodeFlags::COMPONENT_HOST` without
/// `NodeFlags::OPEN_STYLE_SCOPE`), or [`None`] when nothing between `node` and
/// the root closes a scope over it.
///
/// The host itself is not its own ancestor, so a bare component host reports
/// [`None`] (it is authored content, not internal to a component). Its rendered
/// children — and their descendants — report the host: they are the scoped
/// output the boundary contains.
///
/// # An open host is transparent, not terminal
///
/// A component that declared [`StyleScope::Open`](obscur_view::component::StyleScope::Open)
/// (§4.15 "explicitly open/global scopes are possible") does not close a
/// boundary, so the walk **continues past it** rather than stopping and
/// reporting `None`. That distinction is the whole composition rule: an open
/// component used inside a scoped one is still inside the scoped one — opening
/// yourself lets the document reach you, it does not let you punch a hole
/// through somebody else's encapsulation.
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

/// Returns `true` when the non-part simples of `selector`'s subject compound —
/// plus any ancestor/sibling combinator prefix — match `host`.
///
/// The `::part()` simple itself is a target, not a host constraint, so it is
/// skipped. A **type** simple is enforced against the host's CSS type when the
/// host exposes one; a mounted component host that carries no CSS type name (a
/// user component with no `type=` on its element) does not have the host-type
/// filter enforced — the component-host boundary and the export name already
/// gate the crossing, so this stays permissive rather than failing closed on a
/// host that simply never named a type. Class / id / attribute / pseudo-class
/// simples are matched exactly as the ordinary matcher would.
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

/// Returns the specificity of the highest-ranked `::part(want)` selector in
/// `selector`'s owning list that both targets `want` and whose host constraints
/// match `host`, or [`None`] when none do.
///
/// The caller has already decided the subject element *exports* `want`; this
/// only decides whether the `Component` half of `Component::part(want)` matches
/// the host.
#[must_use]
pub(crate) fn part_specificity_for(
    selector: &ComplexSelector,
    want: &str,
    host: &TreeElement,
) -> Option<SelectorSpecificity> {
    (subject_part_name(selector) == Some(want) && part_host_matches(selector, host))
        .then(|| selector.specificity())
}
