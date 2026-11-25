#![doc = include_str!("../../docs/api/orchestration/generated_content.doc/_module.md")]

use super::GeneratedRun;
use super::candidate_rule::CandidateRule;
use super::pseudo_rule_index::PseudoRuleIndex;

use std::collections::HashSet;

use obscur_identity::{FastMap, FastSet, NodeId};
use obscur_style::cascade::{Cascade, Declaration, ResolvedDeclarations};
use obscur_style::invalidation::StyleInvalidation;
use obscur_style::matching::{MediaEnvironment, Stylesheet};
use obscur_style::property::{PropertyId, PropertyValue};
use obscur_style::selector::{ComplexSelector, SelectorMatcher, SelectorSpecificity};
use obscur_tree::node::RetainedNode;
use obscur_tree::tree::RetainedTree;

use super::{PseudoStates, TreeElement, part_specificity_for};

/// The reserved exported-part name a text control gives its placeholder hint, so
/// `Input::part(placeholder)` (the CSS `::placeholder` analogue) can restyle it.
pub(super) const PLACEHOLDER_PART: &str = "placeholder";

/// The `::before` pseudo-element's name, as the selector engine spells it.
pub(super) const BEFORE: &str = "before";

/// The `::after` pseudo-element's name, as the selector engine spells it.
pub(super) const AFTER: &str = "after";

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
        !self.has_boxes() && self.placeholder.is_none()
    }

    /// Whether the element generates at least one decorative layout box.
    pub(crate) fn has_boxes(&self) -> bool {
        self.before.is_some() || self.after.is_some()
    }
}

#[doc = include_str!("../../docs/api/orchestration/generated_content.doc/resolve_generated_content.md")]
pub(crate) fn resolve_generated_content(
    out: &mut FastMap<NodeId, GeneratedContent>,
    tree: &RetainedTree,
    sheets: &[Stylesheet],
    environment: &MediaEnvironment,
    scope: Option<&StyleInvalidation>,
    pseudo: &PseudoStates,
) {
    let index = PseudoRuleIndex::of(sheets);
    let Some(dirty) = scope else {
        out.clear();
        for (id, _) in tree.iter() {
            if let Some(content) = content_for(tree, &index, pseudo, environment, id) {
                out.insert(id, content);
            }
        }
        return;
    };
    // A node the reconcile removed must not keep decoration the full pass would
    // never have produced for it; node ids carry a generation, so a dead id can
    // never be mistaken for a live one here.
    out.retain(|id, _| tree.get(*id).is_some_and(|node| node.kind().is_element()));
    for id in restyled_with_ancestors(tree, dirty) {
        match content_for(tree, &index, pseudo, environment, id) {
            Some(content) => {
                out.insert(id, content);
            }
            None => {
                out.remove(&id);
            }
        }
    }
}

#[doc = include_str!("../../docs/api/orchestration/generated_content.doc/restyled_with_ancestors.md")]
fn restyled_with_ancestors(tree: &RetainedTree, dirty: &StyleInvalidation) -> Vec<NodeId> {
    let mut seen: FastSet<NodeId> = HashSet::default();
    let mut scoped = Vec::new();
    for (&node, _) in dirty {
        let mut current = Some(node);
        // The restyle's reach rule is the subtree plus following siblings —
        // it never marks an ancestor, because no selector reads downwards. The
        // placeholder does: `:placeholder-shown` asks whether the control's own
        // descendants render text, so an edited text child changes the answer for
        // a node the dirty set does not contain. Walking up closes exactly that
        // gap, and stops at the first node already scoped because its ancestors
        // were scoped with it.
        while let Some(id) = current {
            if !seen.insert(id) {
                break;
            }
            scoped.push(id);
            current = tree.get(id).and_then(|node| node.logical_parent().parent());
        }
    }
    scoped
}

/// The decoration `id` carries under `index`, or [`None`] when it carries none
/// (including when `id` is not an element, or no longer in the tree).
fn content_for(
    tree: &RetainedTree,
    index: &PseudoRuleIndex<'_>,
    pseudo: &PseudoStates,
    environment: &MediaEnvironment,
    id: NodeId,
) -> Option<GeneratedContent> {
    let node = tree.get(id)?;
    if !node.kind().is_element() {
        return None;
    }
    let environment = &environment.for_subject(id);
    let element = TreeElement::with_pseudo(tree, id, pseudo);
    let content = GeneratedContent {
        before: resolve_pseudo(&index.before, &element, BEFORE, environment),
        after: resolve_pseudo(&index.after, &element, AFTER, environment),
        placeholder: resolve_placeholder(tree, &index.placeholder, &element, node, environment),
    };
    (!content.is_empty()).then_some(content)
}

#[doc = include_str!("../../docs/api/orchestration/generated_content.doc/resolve_pseudo.md")]
fn resolve_pseudo(
    candidates: &[CandidateRule<'_>],
    element: &TreeElement<'_>,
    pseudo: &str,
    environment: &MediaEnvironment,
) -> Option<GeneratedRun> {
    // No rule in the sheet set names this pseudo-element, so no element in the
    // tree can match one. This is the answer for almost every sheet ever
    // written, and it is reached without touching the element.
    if candidates.is_empty() {
        return None;
    }
    let resolved = cascade_candidates(candidates, environment, |selector| {
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
        declarations: resolved,
    })
}

#[doc = include_str!("../../docs/api/orchestration/generated_content.doc/resolve_placeholder.md")]
fn resolve_placeholder(
    tree: &RetainedTree,
    candidates: &[CandidateRule<'_>],
    element: &TreeElement<'_>,
    node: &RetainedNode,
    environment: &MediaEnvironment,
) -> Option<GeneratedRun> {
    let attributes = node.attributes();
    let placeholder = attributes.placeholder().filter(|hint| !hint.is_empty())?;
    // `:placeholder-shown` — the hint is shown only while the field's value is
    // empty. Which attribute states that value depends on which side wrote the
    // node. An application that declares a CONTROLLED value writes the reserved
    // `VALUE` attribute (`Input::view` always does), and then that value is the
    // answer — including when it is empty. Authored markup writes no such
    // attribute on purpose: `<input value="abc"/>` is HTML's *default* value,
    // which the user then owns, so it lowers to an ordinary attribute and reaches
    // the tree only as the text the mounted control renders. For that node the
    // rendered text IS the value, and reading it is what keeps the hint from
    // being painted on top of the field's own "abc".
    let is_empty = match attributes.control_value() {
        Some(declared) => declared.is_empty(),
        None => renders_no_text(tree, node),
    };
    if !is_empty {
        return None;
    }
    // The hint paints whether or not any rule styles it, so — unlike a
    // pseudo-element — an empty candidate list still yields a run, in the
    // colour the text painter defaults to.
    let resolved = cascade_candidates(candidates, environment, |selector| {
        part_specificity_for(selector, PLACEHOLDER_PART, element)
    });
    Some(GeneratedRun {
        text: placeholder.into(),
        color: resolved
            .get(PropertyId::Color)
            .and_then(PropertyValue::as_color),
        declarations: ResolvedDeclarations::default(),
    })
}

#[doc = include_str!("../../docs/api/orchestration/generated_content.doc/renders_no_text.md")]
fn renders_no_text(tree: &RetainedTree, node: &RetainedNode) -> bool {
    node.child_nodes().iter().all(|&child| {
        tree.get(child).is_none_or(|child| {
            child.kind().text().is_none_or(str::is_empty) && renders_no_text(tree, child)
        })
    })
}

#[doc = include_str!("../../docs/api/orchestration/generated_content.doc/cascade_candidates.md")]
fn cascade_candidates(
    candidates: &[CandidateRule<'_>],
    environment: &MediaEnvironment,
    mut select: impl FnMut(&ComplexSelector) -> Option<SelectorSpecificity>,
) -> ResolvedDeclarations {
    let mut collected: Vec<Declaration> = Vec::new();
    for candidate in candidates {
        let specificity = candidate
            .rule
            .selectors()
            .selectors()
            .iter()
            .filter_map(|selector| {
                probe();
                select(selector)
            })
            .max();
        let condition_holds = candidate
            .rule
            .condition()
            .is_none_or(|condition| condition.holds(environment));
        let Some(specificity) = specificity.filter(|_| condition_holds) else {
            continue;
        };
        for (offset, declaration) in candidate.rule.block().iter().enumerate() {
            collected.push(Declaration::new(
                declaration.property(),
                declaration.value().clone(),
                declaration.important_flag(),
                candidate.origin,
                specificity,
                candidate
                    .order
                    .wrapping_add(u32::try_from(offset).unwrap_or(u32::MAX)),
                None,
            ));
        }
    }
    Cascade::new().resolve(&collected)
}

#[cfg(test)]
thread_local! {
    /// How many selectors this thread has taken to the engine since the counter was
    /// last reset — the observable form of the skip. Per-thread, so tests that run
    /// concurrently do not count each other's work. Test builds only.
    static SELECTOR_PROBES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Records that one selector is about to be matched. Compiles to nothing outside
/// test builds, where `SELECTOR_PROBES` counts them and makes "this pass never
/// asked the engine anything" an assertion rather than a hope.
fn probe() {
    #[cfg(test)]
    SELECTOR_PROBES.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
mod tests {
    use obscur_style::property::StyleColor;
    use std::collections::HashMap;

    use super::*;
    use obscur_identity::ComponentId;
    use obscur_style::cascade::CascadeOrigin;
    use obscur_style::matching::{BlockDeclaration, StyleRule};
    use obscur_style::selector::{CompoundSelector, SelectorList, SimpleSelector};
    use obscur_tree::reconciliation::Reconciler;
    use obscur_view::view::{
        CLASS, PLACEHOLDER, View, ViewFragment, ViewNode, ViewProperty, ViewValue,
    };

    /// The number of times the selector engine has been consulted on this
    /// thread since [`reset_probes`]; each test thread counts its own.
    fn probes() -> usize {
        SELECTOR_PROBES.with(std::cell::Cell::get)
    }

    fn reset_probes() {
        SELECTOR_PROBES.with(|count| count.set(0));
    }

    fn red() -> PropertyValue {
        PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
    }

    fn environment() -> MediaEnvironment<'static> {
        MediaEnvironment::viewport(400.0, 300.0)
    }

    /// `.<class>` — an ordinary element selector, nothing generated about it.
    fn class_selectors(class: &str) -> SelectorList {
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Class(class.into())]),
            Vec::new(),
        )])
    }

    /// `.<class>::before` — the subject compound carries the pseudo-element.
    fn before_selectors(class: &str) -> SelectorList {
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![
                SimpleSelector::Class(class.into()),
                SimpleSelector::PseudoElement(BEFORE.into()),
            ]),
            Vec::new(),
        )])
    }

    fn sheet(rules: Vec<StyleRule>) -> Vec<Stylesheet> {
        vec![Stylesheet::new(CascadeOrigin::GlobalApp, rules)]
    }

    fn content(text: &str) -> BlockDeclaration {
        BlockDeclaration::normal(PropertyId::Content, PropertyValue::String(text.into()))
    }

    fn reconcile(view: &View) -> RetainedTree {
        let mut tree = RetainedTree::new();
        let _ = Reconciler::new().reconcile_root(&mut tree, view);
        tree
    }

    fn children(tree: &RetainedTree) -> Vec<NodeId> {
        tree.get(tree.root().unwrap())
            .unwrap()
            .child_nodes()
            .to_vec()
    }

    /// Two sibling elements, `.a` and `.b`, under the tree's root.
    fn two_elements() -> (RetainedTree, NodeId, NodeId) {
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into()))),
            ViewNode::element(ComponentId::from_raw(11))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into()))),
        ]));
        let tree = reconcile(&view);
        let kids = children(&tree);
        (tree, kids[0], kids[1])
    }

    fn dirty_with(nodes: &[NodeId]) -> StyleInvalidation {
        use obscur_style::invalidation::StyleInvalidationReason;
        let mut dirty = StyleInvalidation::new();
        for &node in nodes {
            dirty.invalidate(node, StyleInvalidationReason::PseudoStateChanged);
        }
        dirty
    }

    #[test]
    fn a_sheet_that_targets_no_pseudo_element_is_never_taken_to_the_selector_engine() {
        let (tree, _, _) = two_elements();
        let sheets = sheet(vec![StyleRule::new(
            class_selectors("a"),
            vec![BlockDeclaration::normal(PropertyId::Color, red())],
        )]);

        // The pre-computed answer: no rule can produce generated content, and the
        // pass has no reason to ask the engine about any element.
        let index = PseudoRuleIndex::of(&sheets);
        assert!(index.before.is_empty(), "no rule authors ::before");
        assert!(index.after.is_empty(), "no rule authors ::after");
        assert!(
            index.placeholder.is_empty(),
            "no rule authors ::part(placeholder)"
        );

        reset_probes();
        let mut out = HashMap::default();
        resolve_generated_content(
            &mut out,
            &tree,
            &sheets,
            &environment(),
            None,
            &PseudoStates::new(),
        );

        assert!(out.is_empty(), "an ordinary sheet decorates nothing");
        assert_eq!(
            probes(),
            0,
            "a sheet set with no pseudo-element rule must not reach the selector engine at all"
        );
    }

    #[test]
    fn a_before_rule_still_resolves_its_content_and_colour() {
        let (tree, a, b) = two_elements();
        let sheets = sheet(vec![StyleRule::new(
            before_selectors("a"),
            vec![
                content("!"),
                BlockDeclaration::normal(PropertyId::Color, red()),
            ],
        )]);

        reset_probes();
        let mut out = HashMap::default();
        resolve_generated_content(
            &mut out,
            &tree,
            &sheets,
            &environment(),
            None,
            &PseudoStates::new(),
        );

        let run = out[&a]
            .before
            .as_ref()
            .expect("`.a::before` decorates its element");
        assert_eq!(run.text.as_ref(), "!");
        assert_eq!(run.color, Some(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        assert!(!out.contains_key(&b), "`.b` matches no pseudo-element rule");
        assert!(
            probes() > 0,
            "a sheet that DOES author a pseudo-element must consult the engine"
        );
    }

    #[test]
    fn a_scoped_pass_recomputes_the_dirty_node_and_leaves_every_other_entry_standing() {
        let (tree, a, b) = two_elements();
        let sheets = sheet(vec![
            StyleRule::new(before_selectors("a"), vec![content("A")]),
            StyleRule::new(before_selectors("b"), vec![content("B")]),
        ]);
        let mut out = HashMap::default();
        resolve_generated_content(
            &mut out,
            &tree,
            &sheets,
            &environment(),
            None,
            &PseudoStates::new(),
        );

        // A sentinel no rule could ever produce: if the scoped pass recomputes
        // `a`, or drops what it did not recompute, the sentinel disappears.
        let sentinel = GeneratedContent {
            before: Some(GeneratedRun {
                text: "sentinel".into(),
                color: None,
                declarations: Default::default(),
            }),
            ..GeneratedContent::default()
        };
        out.insert(a, sentinel.clone());

        resolve_generated_content(
            &mut out,
            &tree,
            &sheets,
            &environment(),
            Some(&dirty_with(&[b])),
            &PseudoStates::new(),
        );

        assert_eq!(
            out.get(&a),
            Some(&sentinel),
            "a node outside the dirty set keeps the previous frame's content, untouched"
        );
        assert_eq!(
            out.get(&b)
                .and_then(|c| c.before.as_ref())
                .map(|r| &*r.text),
            Some("B"),
            "the dirty node is resolved again"
        );
    }

    #[test]
    fn a_dirty_text_child_re_answers_its_control_placeholder_question() {
        // The control renders "abc", so the hint must NOT be shown — but only a
        // pass that re-examines the input itself can find that out, and the
        // restyle marks the edited text node, never its parent.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(12))
                .with_prop(ViewProperty::new(
                    PLACEHOLDER,
                    ViewValue::Text("Search".into()),
                ))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("abc")])),
        ]));
        let tree = reconcile(&view);
        let input = children(&tree)[0];
        let text = tree.get(input).unwrap().child_nodes()[0];

        // The previous frame's answer, from when the field was still empty.
        let mut out = HashMap::default();
        out.insert(
            input,
            GeneratedContent {
                placeholder: Some(GeneratedRun {
                    text: "Search".into(),
                    color: None,
                    declarations: Default::default(),
                }),
                ..GeneratedContent::default()
            },
        );

        resolve_generated_content(
            &mut out,
            &tree,
            &sheet(Vec::new()),
            &environment(),
            Some(&dirty_with(&[text])),
            &PseudoStates::new(),
        );

        assert!(
            !out.contains_key(&input),
            "the hint must not survive on a control that now renders its own text"
        );
    }

    #[test]
    fn a_removed_node_does_not_keep_its_decoration_through_a_scoped_pass() {
        let (tree, a, b) = two_elements();
        let sheets = sheet(vec![StyleRule::new(
            before_selectors("a"),
            vec![content("A")],
        )]);
        let mut out = HashMap::default();
        resolve_generated_content(
            &mut out,
            &tree,
            &sheets,
            &environment(),
            None,
            &PseudoStates::new(),
        );
        assert!(out.contains_key(&a));

        // `a` is gone from this tree; only `b` is dirty.
        let mut pruned = RetainedTree::new();
        let _ = Reconciler::new().reconcile_root(
            &mut pruned,
            &View::from_fragment(ViewFragment::from_nodes(Vec::new())),
        );
        resolve_generated_content(
            &mut out,
            &pruned,
            &sheets,
            &environment(),
            Some(&dirty_with(&[b])),
            &PseudoStates::new(),
        );

        assert!(
            out.is_empty(),
            "a scoped pass must not carry decoration for a node the tree no longer has"
        );
    }
}
