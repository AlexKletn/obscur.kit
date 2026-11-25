//! Narrows "which rules could possibly match this element" from *all of them* to
//! a handful, without changing which ones do: [`RuleIndex`].

use crate::matching::Stylesheet;
use crate::selector::{ComplexSelector, MatchElement, SimpleSelector};
use obscur_identity::FastMap;
use std::collections::HashMap;

use super::RuleCandidate;
use super::subject_key::SubjectKey;

/// How many distinct buckets one element may draw candidates from before the
/// index gives up and asks the caller to walk everything.
///
/// The merge below keeps its cursors on the stack, so the bound is what keeps
/// this allocation-free. An element with more than fourteen classes that each
/// name a bucket is not styled more slowly than it is today — it is styled by
/// the unindexed walk, exactly as before.
const MAX_SOURCES: usize = 16;

#[doc = include_str!("../../docs/api/matching/rule_index.doc/RuleIndex.md")]
#[derive(Debug, Clone, Default)]
pub struct RuleIndex {
    /// Rules per sheet, in order: the shape [`RuleIndex::describes`] checks.
    shape: Vec<u32>,
    /// Every rule of every sheet, in document order, addressed by global index.
    entries: Vec<RuleCandidate>,
    /// Global rule indices whose subject compound requires this class.
    by_class: FastMap<Box<str>, Vec<u32>>,
    /// Global rule indices whose subject compound requires this id.
    by_id: FastMap<Box<str>, Vec<u32>>,
    /// Global rule indices whose subject compound requires this local name,
    /// keyed ASCII-lowercased because type matching ignores ASCII case.
    by_type: FastMap<Box<str>, Vec<u32>>,
    /// Global rule indices that at least one selector could not be keyed for.
    unkeyed: Vec<u32>,
    #[doc = include_str!("../../docs/api/matching/rule_index.doc/any_custom.md")]
    any_custom: bool,
}

impl RuleIndex {
    #[doc = include_str!("../../docs/api/matching/rule_index.doc/build.md")]
    #[must_use]
    pub fn build(sheets: &[Stylesheet]) -> Self {
        let mut index = Self {
            shape: Vec::with_capacity(sheets.len()),
            entries: Vec::new(),
            by_class: HashMap::default(),
            by_id: HashMap::default(),
            by_type: HashMap::default(),
            unkeyed: Vec::new(),
            any_custom: false,
        };
        // One monotonic counter over every declaration of every rule, matched or
        // not: the same counter `match_element_in` advances as it walks. Frozen
        // per rule, it is what lets a skipped rule keep its place in the cascade.
        let mut declarations: u32 = 0;
        for (sheet_position, sheet) in sheets.iter().enumerate() {
            index
                .shape
                .push(u32::try_from(sheet.rules().len()).unwrap_or(u32::MAX));
            for (rule_position, rule) in sheet.rules().iter().enumerate() {
                let global = u32::try_from(index.entries.len()).unwrap_or(u32::MAX);
                let has_custom = !rule.custom_properties().is_empty();
                index.any_custom |= has_custom;
                index.entries.push(RuleCandidate::new(
                    u32::try_from(sheet_position).unwrap_or(u32::MAX),
                    u32::try_from(rule_position).unwrap_or(u32::MAX),
                    declarations,
                    has_custom,
                ));
                declarations = declarations
                    .wrapping_add(u32::try_from(rule.block().len()).unwrap_or(u32::MAX));
                index.file(global, rule.selectors().selectors());
            }
        }
        index
    }

    #[doc = include_str!("../../docs/api/matching/rule_index.doc/describes.md")]
    #[must_use]
    pub fn describes(&self, sheets: &[Stylesheet]) -> bool {
        self.shape.len() == sheets.len()
            && self
                .shape
                .iter()
                .zip(sheets)
                .all(|(count, sheet)| *count as usize == sheet.rules().len())
    }

    #[doc = include_str!("../../docs/api/matching/rule_index.doc/any_custom.md")]
    #[must_use]
    pub const fn any_custom(&self) -> bool {
        self.any_custom
    }

    #[doc = include_str!("../../docs/api/matching/rule_index.doc/candidates.md")]
    pub fn candidates<E: MatchElement>(
        &self,
        element: &E,
        mut visit: impl FnMut(RuleCandidate),
    ) -> bool {
        let mut sources: [&[u32]; MAX_SOURCES] = [&[]; MAX_SOURCES];
        let mut count = 0usize;
        let mut overflowed = false;

        if !self.unkeyed.is_empty() {
            sources[count] = &self.unkeyed;
            count += 1;
        }
        if let Some(id) = element.id()
            && let Some(bucket) = self.by_id.get(id)
        {
            sources[count] = bucket;
            count += 1;
        }
        if let Some(local) = element.local_name()
            && let Some(bucket) = self.type_bucket(local)
        {
            sources[count] = bucket;
            count += 1;
        }
        if !self.by_class.is_empty() {
            let by_class = &self.by_class;
            let enumerated = element.enumerate_classes(&mut |class| {
                if let Some(bucket) = by_class.get(class) {
                    if count < MAX_SOURCES {
                        sources[count] = bucket;
                        count += 1;
                    } else {
                        overflowed = true;
                    }
                }
            });
            if !enumerated {
                return false;
            }
        }
        if overflowed {
            return false;
        }

        // A k-way merge over buckets that are each already ascending, so the
        // candidates arrive in the same order the unindexed walk would reach
        // them in, and a rule filed under two of the element's classes is
        // visited once.
        let mut cursors = [0usize; MAX_SOURCES];
        loop {
            let mut next: Option<u32> = None;
            for source in 0..count {
                if let Some(&global) = sources[source].get(cursors[source]) {
                    next = Some(next.map_or(global, |best: u32| best.min(global)));
                }
            }
            let Some(global) = next else { break };
            for source in 0..count {
                while sources[source].get(cursors[source]) == Some(&global) {
                    cursors[source] += 1;
                }
            }
            visit(self.entries[global as usize]);
        }
        true
    }

    /// The bucket for a local name, tolerating an element that spells its name
    /// with capitals where the sheets spelled it without.
    fn type_bucket(&self, local: &str) -> Option<&Vec<u32>> {
        if let Some(bucket) = self.by_type.get(local) {
            return Some(bucket);
        }
        if local.bytes().any(|byte| byte.is_ascii_uppercase()) {
            return self.by_type.get(local.to_ascii_lowercase().as_str());
        }
        None
    }

    /// Files one rule under every key its selectors state, or under nothing at
    /// all if one of them states none.
    fn file(&mut self, global: u32, selectors: &[ComplexSelector]) {
        // The rule matches if ANY of its selectors does, so it may only be
        // skipped when EVERY one of them is keyed and none of those keys is the
        // element's. One unkeyable selector puts the whole rule in the bucket
        // every element reads.
        if selectors
            .iter()
            .any(|selector| subject_key(selector).is_none())
        {
            self.unkeyed.push(global);
            return;
        }
        for selector in selectors {
            match subject_key(selector) {
                Some(SubjectKey::Id(name)) => file_under(&mut self.by_id, name, global),
                Some(SubjectKey::Class(name)) => file_under(&mut self.by_class, name, global),
                Some(SubjectKey::Type(name)) => {
                    file_under(&mut self.by_type, &name.to_ascii_lowercase(), global);
                }
                None => unreachable!("the unkeyable case returned above"),
            }
        }
    }
}

/// Appends `global` to `name`'s bucket unless it is already its last entry.
///
/// Rules are filed in ascending order, so "already present" can only mean
/// "already last" — a rule two of whose selectors share a key is filed once,
/// and every bucket stays sorted, which is what the merge assumes.
fn file_under(buckets: &mut FastMap<Box<str>, Vec<u32>>, name: &str, global: u32) {
    let bucket = buckets.entry(name.into()).or_default();
    if bucket.last() != Some(&global) {
        bucket.push(global);
    }
}

/// The necessary condition this selector's subject compound states, preferring
/// the most selective of the three.
///
/// Ordering id before class before type is a cost choice, not a correctness
/// one: any single one of them is a necessary condition on its own, and the
/// rarest key makes the smallest bucket.
fn subject_key(selector: &ComplexSelector) -> Option<SubjectKey<'_>> {
    let mut class: Option<&str> = None;
    let mut local: Option<&str> = None;
    for simple in selector.subject().simples() {
        match simple {
            SimpleSelector::Id(name) => return Some(SubjectKey::Id(name)),
            SimpleSelector::Class(name) if class.is_none() => class = Some(name),
            SimpleSelector::Type(name) if local.is_none() => local = Some(name),
            _ => {}
        }
    }
    class
        .map(SubjectKey::Class)
        .or_else(|| local.map(SubjectKey::Type))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cascade::CascadeOrigin;
    use crate::matching::{BlockDeclaration, StyleRule};
    use crate::property::{PropertyId, PropertyValue};
    use crate::selector::{
        Combinator, CompoundSelector, PseudoClassSelector, PseudoState, SelectorList,
    };

    struct Fake {
        local_name: Option<&'static str>,
        id: Option<&'static str>,
        classes: Vec<&'static str>,
        enumerable: bool,
    }

    impl Fake {
        fn new() -> Self {
            Self {
                local_name: Some("div"),
                id: None,
                classes: Vec::new(),
                enumerable: true,
            }
        }

        fn with_class(mut self, class: &'static str) -> Self {
            self.classes.push(class);
            self
        }

        fn with_id(mut self, id: &'static str) -> Self {
            self.id = Some(id);
            self
        }
    }

    impl MatchElement for Fake {
        fn local_name(&self) -> Option<&str> {
            self.local_name
        }
        fn id(&self) -> Option<&str> {
            self.id
        }
        fn has_class(&self, class: &str) -> bool {
            self.classes.contains(&class)
        }
        fn enumerate_classes(&self, visit: &mut dyn FnMut(&str)) -> bool {
            if !self.enumerable {
                return false;
            }
            for class in &self.classes {
                visit(class);
            }
            true
        }
        fn attribute(&self, _name: &str) -> Option<&str> {
            None
        }
        fn pseudo_state(&self) -> PseudoState {
            PseudoState::empty()
        }
        fn parent(&self) -> Option<Self> {
            None
        }
        fn previous_sibling_element(&self) -> Option<Self> {
            None
        }
        fn index_among_siblings(&self) -> usize {
            0
        }
        fn sibling_count(&self) -> usize {
            1
        }
        fn is_root(&self) -> bool {
            true
        }
        fn has_children(&self) -> bool {
            false
        }
    }

    fn one(simples: Vec<SimpleSelector>) -> SelectorList {
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(simples),
            Vec::new(),
        )])
    }

    fn decl() -> BlockDeclaration {
        BlockDeclaration::normal(PropertyId::Opacity, PropertyValue::Number(1.0))
    }

    fn rule(selectors: SelectorList, block: usize) -> StyleRule {
        StyleRule::new(selectors, (0..block).map(|_| decl()).collect())
    }

    fn sheet(rules: Vec<StyleRule>) -> Stylesheet {
        Stylesheet::new(CascadeOrigin::GlobalApp, rules)
    }

    fn seen<E: MatchElement>(index: &RuleIndex, element: &E) -> Option<Vec<u32>> {
        let mut out = Vec::new();
        let decided = index.candidates(element, |candidate| {
            out.push(candidate.decl_start());
        });
        decided.then_some(out)
    }

    #[test]
    fn a_class_rule_reaches_only_the_elements_carrying_that_class() {
        let sheets = vec![sheet(vec![
            rule(one(vec![SimpleSelector::Class("key".into())]), 1),
            rule(one(vec![SimpleSelector::Class("tool".into())]), 1),
        ])];
        let index = RuleIndex::build(&sheets);
        // decl_start doubles as the rule's identity here: rule 0 starts at 0,
        // rule 1 at 1.
        assert_eq!(seen(&index, &Fake::new().with_class("key")), Some(vec![0]));
        assert_eq!(seen(&index, &Fake::new().with_class("tool")), Some(vec![1]));
        assert_eq!(seen(&index, &Fake::new()), Some(vec![]));
    }

    #[test]
    fn the_bucketless_rules_are_offered_to_every_element() {
        // The failure this index is most exposed to: a rule with no class, id or
        // type on its subject states no necessary condition, so no bucket can
        // hold it and every element must still be offered it.
        let bucketless = vec![
            // `*`
            one(vec![SimpleSelector::Universal]),
            // `:hover`
            one(vec![SimpleSelector::PseudoClass(
                PseudoClassSelector::Hover,
            )]),
            // `div > *`
            SelectorList::new(vec![ComplexSelector::new(
                CompoundSelector::new(vec![SimpleSelector::Type("div".into())]),
                vec![(
                    Combinator::Child,
                    CompoundSelector::new(vec![SimpleSelector::Universal]),
                )],
            )]),
            // `[data-x]`
            one(vec![SimpleSelector::Attribute(
                crate::selector::AttributeSelector {
                    name: "data-x".into(),
                    matcher: None,
                },
            )]),
        ];
        for selectors in bucketless {
            let sheets = vec![sheet(vec![rule(selectors, 1)])];
            let index = RuleIndex::build(&sheets);
            for element in [
                Fake::new(),
                Fake::new().with_class("key"),
                Fake::new().with_id("main"),
                Fake {
                    local_name: None,
                    ..Fake::new()
                },
            ] {
                assert_eq!(
                    seen(&index, &element),
                    Some(vec![0]),
                    "a rule no bucket can hold must reach every element"
                );
            }
        }
    }

    #[test]
    fn a_rule_whose_selector_list_mixes_keyed_and_unkeyed_is_never_filed_away() {
        // `.key, *` — one selector is keyable and one is not. Filing the rule
        // under `key` alone would lose it for every element without that class,
        // which is exactly what `*` promises not to do.
        let selectors = SelectorList::new(vec![
            ComplexSelector::new(
                CompoundSelector::new(vec![SimpleSelector::Class("key".into())]),
                Vec::new(),
            ),
            ComplexSelector::new(
                CompoundSelector::new(vec![SimpleSelector::Universal]),
                Vec::new(),
            ),
        ]);
        let sheets = vec![sheet(vec![rule(selectors, 1)])];
        let index = RuleIndex::build(&sheets);
        assert_eq!(seen(&index, &Fake::new()), Some(vec![0]));
        assert_eq!(seen(&index, &Fake::new().with_class("key")), Some(vec![0]));
    }

    #[test]
    fn a_rule_filed_under_two_of_the_elements_own_classes_arrives_once() {
        // `.a.b { }` on an element carrying both: the rule sits in one bucket
        // (the first class keys it), but `.a, .b` sits in two, and a duplicate
        // would be cascaded twice.
        let selectors = SelectorList::new(vec![
            ComplexSelector::new(
                CompoundSelector::new(vec![SimpleSelector::Class("a".into())]),
                Vec::new(),
            ),
            ComplexSelector::new(
                CompoundSelector::new(vec![SimpleSelector::Class("b".into())]),
                Vec::new(),
            ),
        ]);
        let sheets = vec![sheet(vec![rule(selectors, 1)])];
        let index = RuleIndex::build(&sheets);
        assert_eq!(
            seen(&index, &Fake::new().with_class("a").with_class("b")),
            Some(vec![0])
        );
    }

    #[test]
    fn candidates_arrive_in_document_order_across_sheets_and_buckets() {
        // Three sheets, keys interleaved so that bucket order and document order
        // disagree: `#main` (sheet 0), `.key` (sheet 1), `div` (sheet 2).
        let sheets = vec![
            sheet(vec![rule(one(vec![SimpleSelector::Id("main".into())]), 2)]),
            sheet(vec![rule(
                one(vec![SimpleSelector::Class("key".into())]),
                3,
            )]),
            sheet(vec![rule(one(vec![SimpleSelector::Type("div".into())]), 1)]),
        ];
        let index = RuleIndex::build(&sheets);
        let element = Fake::new().with_id("main").with_class("key");
        // decl_start: 0, then 0+2, then 0+2+3.
        assert_eq!(seen(&index, &element), Some(vec![0, 2, 5]));
    }

    #[test]
    fn decl_start_counts_the_declarations_of_rules_that_were_skipped() {
        // The number that keeps the cascade honest: rule 2 must be told its
        // block begins at 7, even though the walk never touched rules 0 and 1.
        let sheets = vec![sheet(vec![
            rule(one(vec![SimpleSelector::Class("nope".into())]), 4),
            rule(one(vec![SimpleSelector::Class("also-nope".into())]), 3),
            rule(one(vec![SimpleSelector::Class("key".into())]), 1),
        ])];
        let index = RuleIndex::build(&sheets);
        assert_eq!(seen(&index, &Fake::new().with_class("key")), Some(vec![7]));
    }

    #[test]
    fn a_type_bucket_ignores_ascii_case_the_way_type_matching_does() {
        let sheets = vec![sheet(vec![rule(
            one(vec![SimpleSelector::Type("Button".into())]),
            1,
        )])];
        let index = RuleIndex::build(&sheets);
        let mut element = Fake::new();
        element.local_name = Some("button");
        assert_eq!(seen(&index, &element), Some(vec![0]));
        let mut shouty = Fake::new();
        shouty.local_name = Some("BUTTON");
        assert_eq!(seen(&index, &shouty), Some(vec![0]));
    }

    #[test]
    fn an_element_that_cannot_list_its_classes_is_refused_rather_than_guessed_at() {
        let sheets = vec![sheet(vec![rule(
            one(vec![SimpleSelector::Class("key".into())]),
            1,
        )])];
        let index = RuleIndex::build(&sheets);
        let opaque = Fake {
            enumerable: false,
            ..Fake::new().with_class("key")
        };
        assert_eq!(seen(&index, &opaque), None, "the caller must walk it all");
    }

    #[test]
    fn describes_rejects_a_set_of_a_different_shape() {
        let sheets = vec![sheet(vec![rule(
            one(vec![SimpleSelector::Class("key".into())]),
            1,
        )])];
        let index = RuleIndex::build(&sheets);
        assert!(index.describes(&sheets));
        assert!(!index.describes(&[]));
        assert!(!index.describes(&[sheet(Vec::new())]));
        assert!(!index.describes(&[sheets[0].clone(), sheet(Vec::new())]));
    }

    #[test]
    fn any_custom_reports_whether_the_set_declares_one_at_all() {
        use crate::property::CustomProperty;
        use obscur_identity::InternedSymbol;

        let plain = vec![sheet(vec![rule(
            one(vec![SimpleSelector::Class("key".into())]),
            1,
        )])];
        assert!(!RuleIndex::build(&plain).any_custom());

        let branded = vec![sheet(vec![
            rule(one(vec![SimpleSelector::Class("key".into())]), 1).with_custom_properties(vec![
                CustomProperty::new(InternedSymbol::named("--brand"), PropertyValue::Number(1.0)),
            ]),
        ])];
        let index = RuleIndex::build(&branded);
        assert!(index.any_custom());
        let mut flags = Vec::new();
        let _ = index.candidates(&Fake::new().with_class("key"), |candidate| {
            flags.push(candidate.has_custom());
        });
        assert_eq!(flags, vec![true]);
    }

    #[test]
    fn an_empty_selector_list_is_filed_nowhere_because_it_matches_nothing() {
        // `matches_list` over no selectors is `None` for every element, so a rule
        // reached by no bucket loses nothing.
        let sheets = vec![sheet(vec![rule(SelectorList::new(Vec::new()), 1)])];
        let index = RuleIndex::build(&sheets);
        assert_eq!(seen(&index, &Fake::new().with_class("key")), Some(vec![]));
    }
}
