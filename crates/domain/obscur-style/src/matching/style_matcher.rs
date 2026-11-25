//! Turns authored stylesheets plus one element into that element's cascaded
//! [`ResolvedDeclarations`]: [`StyleMatcher`].

use crate::cascade::{Cascade, CascadePriority, Declaration, ImportantFlag, ResolvedDeclarations};
use crate::matching::{MediaEnvironment, RuleIndex, Stylesheet};
use crate::property::PropertyValue;
use crate::selector::{MatchElement, SelectorMatcher};
use obscur_identity::{FastMap, InternedSymbol};
use std::collections::HashMap;

#[doc = include_str!("../../docs/api/matching/style_matcher.doc/StyleMatcher.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct StyleMatcher;

impl StyleMatcher {
    /// Creates a style matcher.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("../../docs/api/matching/style_matcher.doc/match_element.md")]
    #[must_use]
    pub fn match_element<E: MatchElement>(
        &self,
        sheets: &[Stylesheet],
        element: &E,
    ) -> ResolvedDeclarations {
        self.match_element_in(sheets, element, &MediaEnvironment::unbounded())
    }

    #[doc = include_str!("../../docs/api/matching/style_matcher.doc/match_element_in.md")]
    #[must_use]
    pub fn match_element_in<E: MatchElement>(
        &self,
        sheets: &[Stylesheet],
        element: &E,
        environment: &MediaEnvironment,
    ) -> ResolvedDeclarations {
        let mut collected: Vec<Declaration> = Vec::new();
        // A single monotonic counter across all sheets/rules/declarations, in
        // document order. Incremented per encountered declaration so that later
        // declarations win exact ties within their cascade band.
        let mut source_order: u32 = 0;

        for sheet in sheets {
            let origin = sheet.origin();
            for rule in sheet.rules() {
                let Some(specificity) = SelectorMatcher::matches_list(rule.selectors(), element)
                else {
                    // The block of a rule that did not match is entered for one
                    // reason only — to advance `source_order` past it — and that
                    // is arithmetic, not a walk. Skipping it leaves every later
                    // declaration with the number it had before.
                    source_order = source_order.wrapping_add(block_span(rule.block().len()));
                    continue;
                };
                let condition_holds = rule
                    .condition()
                    .is_none_or(|condition| condition.holds(environment));
                for block_decl in rule.block() {
                    let order = source_order;
                    source_order = source_order.wrapping_add(1);
                    if condition_holds {
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

    #[doc = include_str!("../../docs/api/matching/style_matcher.doc/match_element_indexed.md")]
    #[must_use]
    pub fn match_element_indexed<E: MatchElement>(
        &self,
        index: &RuleIndex,
        sheets: &[Stylesheet],
        element: &E,
        environment: &MediaEnvironment,
    ) -> ResolvedDeclarations {
        if !index.describes(sheets) {
            return self.match_element_in(sheets, element, environment);
        }
        let mut collected: Vec<Declaration> = Vec::new();
        let decided = index.candidates(element, |candidate| {
            let sheet = &sheets[candidate.sheet()];
            let rule = &sheet.rules()[candidate.rule()];
            let Some(specificity) = SelectorMatcher::matches_list(rule.selectors(), element) else {
                return;
            };
            if !rule
                .condition()
                .is_none_or(|condition| condition.holds(environment))
            {
                return;
            }
            for (offset, block_decl) in rule.block().iter().enumerate() {
                collected.push(Declaration::new(
                    block_decl.property(),
                    block_decl.value().clone(),
                    block_decl.important_flag(),
                    sheet.origin(),
                    specificity,
                    candidate.decl_start().wrapping_add(block_span(offset)),
                    None,
                ));
            }
        });
        if !decided {
            // The index declined to answer for this element and visited nothing;
            // the unindexed walk is the fallback, not a guess.
            return self.match_element_in(sheets, element, environment);
        }

        Cascade::new().resolve(&collected)
    }

    #[doc = include_str!("../../docs/api/matching/style_matcher.doc/match_custom_properties_in.md")]
    #[must_use]
    pub fn match_custom_properties_in<E: MatchElement>(
        &self,
        sheets: &[Stylesheet],
        element: &E,
        environment: &MediaEnvironment,
    ) -> FastMap<InternedSymbol, PropertyValue> {
        let mut winners: FastMap<InternedSymbol, (CascadePriority, PropertyValue)> =
            HashMap::default();
        let mut source_order: u32 = 0;

        for sheet in sheets {
            let origin = sheet.origin();
            for rule in sheet.rules() {
                // Before the selector match, not after: the branch is what keeps
                // this second walk free for the sheets that use no custom
                // property, which is most of them.
                if rule.custom_properties().is_empty() {
                    continue;
                }
                let Some(specificity) = SelectorMatcher::matches_list(rule.selectors(), element)
                else {
                    continue;
                };
                if !rule
                    .condition()
                    .is_none_or(|condition| condition.holds(environment))
                {
                    continue;
                }
                for custom in rule.custom_properties() {
                    let order = source_order;
                    source_order = source_order.wrapping_add(1);
                    // A custom property carries no `!important` of its own: the
                    // compiler keeps its value verbatim, flag and all, so there is
                    // nothing here to read one off. Every candidate therefore sits
                    // in its origin's normal band and the comparison is decided by
                    // origin, specificity and order alone.
                    let priority =
                        CascadePriority::new(origin, ImportantFlag::Normal, specificity, order);
                    match winners.get(&custom.name()) {
                        Some((winner, _)) if priority < *winner => {}
                        _ => {
                            winners.insert(custom.name(), (priority, custom.value().clone()));
                        }
                    }
                }
            }
        }

        winners
            .into_iter()
            .map(|(name, (_, value))| (name, value))
            .collect()
    }

    #[doc = include_str!("../../docs/api/matching/style_matcher.doc/match_custom_properties_indexed.md")]
    #[must_use]
    pub fn match_custom_properties_indexed<E: MatchElement>(
        &self,
        index: &RuleIndex,
        sheets: &[Stylesheet],
        element: &E,
        environment: &MediaEnvironment,
    ) -> FastMap<InternedSymbol, PropertyValue> {
        if !index.describes(sheets) {
            return self.match_custom_properties_in(sheets, element, environment);
        }
        if !index.any_custom() {
            // No sheet in the set declares one, so no element can carry one, and
            // the selector engine is never asked.
            return HashMap::default();
        }
        let mut winners: FastMap<InternedSymbol, (CascadePriority, PropertyValue)> =
            HashMap::default();
        let mut source_order: u32 = 0;
        let decided = index.candidates(element, |candidate| {
            if !candidate.has_custom() {
                return;
            }
            let sheet = &sheets[candidate.sheet()];
            let rule = &sheet.rules()[candidate.rule()];
            let Some(specificity) = SelectorMatcher::matches_list(rule.selectors(), element) else {
                return;
            };
            if !rule
                .condition()
                .is_none_or(|condition| condition.holds(environment))
            {
                return;
            }
            for custom in rule.custom_properties() {
                let order = source_order;
                source_order = source_order.wrapping_add(1);
                let priority =
                    CascadePriority::new(sheet.origin(), ImportantFlag::Normal, specificity, order);
                match winners.get(&custom.name()) {
                    Some((winner, _)) if priority < *winner => {}
                    _ => {
                        winners.insert(custom.name(), (priority, custom.value().clone()));
                    }
                }
            }
        });
        if !decided {
            return self.match_custom_properties_in(sheets, element, environment);
        }

        winners
            .into_iter()
            .map(|(name, (_, value))| (name, value))
            .collect()
    }

    /// Matches `element` against a single `sheet`, a convenience over
    /// [`match_element`](Self::match_element) for the one-origin case.
    #[must_use]
    pub fn match_single<E: MatchElement>(
        &self,
        sheet: &Stylesheet,
        element: &E,
    ) -> ResolvedDeclarations {
        self.match_element(std::slice::from_ref(sheet), element)
    }
}

/// A count of declarations as a `source_order` shift.
///
/// A block longer than `u32::MAX` cannot be authored, parsed or held; the
/// saturation is here so the conversion has one obvious answer rather than a
/// silently truncating cast.
fn block_span(declarations: usize) -> u32 {
    u32::try_from(declarations).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cascade::CascadeOrigin;
    use crate::matching::{
        BlockDeclaration, ConditionAxis, ConditionComparator, FeatureQuery, RuleCondition,
        StyleRule,
    };
    use crate::property::{PropertyId, PropertyValue, StyleColor};
    use crate::selector::{
        AttrMatch, AttributeSelector, Combinator, ComplexSelector, CompoundSelector,
        PseudoClassSelector, PseudoState, SelectorList, SimpleSelector,
    };

    /// A single standalone element for style-matching tests: no parent and no
    /// siblings, so only compound selectors on the subject are exercised.
    struct FakeElement {
        local_name: Option<String>,
        id: Option<String>,
        classes: Vec<String>,
    }

    impl FakeElement {
        fn new(local_name: &str) -> Self {
            Self {
                local_name: Some(local_name.to_owned()),
                id: None,
                classes: Vec::new(),
            }
        }

        fn with_id(mut self, id: &str) -> Self {
            self.id = Some(id.to_owned());
            self
        }

        fn with_class(mut self, class: &str) -> Self {
            self.classes.push(class.to_owned());
            self
        }
    }

    impl MatchElement for FakeElement {
        fn local_name(&self) -> Option<&str> {
            self.local_name.as_deref()
        }

        fn id(&self) -> Option<&str> {
            self.id.as_deref()
        }

        fn has_class(&self, class: &str) -> bool {
            self.classes.iter().any(|c| c == class)
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

    fn selectors(simples: Vec<SimpleSelector>) -> SelectorList {
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(simples),
            Vec::new(),
        )])
    }

    fn class(name: &str) -> SimpleSelector {
        SimpleSelector::Class(name.into())
    }

    fn id(name: &str) -> SimpleSelector {
        SimpleSelector::Id(name.into())
    }

    fn ty(name: &str) -> SimpleSelector {
        SimpleSelector::Type(name.into())
    }

    fn color(v: f32) -> PropertyValue {
        PropertyValue::Color(StyleColor::rgba(v, 0.0, 0.0, 1.0))
    }

    fn rule(sel: Vec<SimpleSelector>, block: Vec<BlockDeclaration>) -> StyleRule {
        StyleRule::new(selectors(sel), block)
    }

    #[test]
    fn single_sheet_matching_class_resolves_property() {
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![rule(
                vec![class("foo")],
                vec![BlockDeclaration::normal(
                    PropertyId::BackgroundColor,
                    color(0.5),
                )],
            )],
        );
        let element = FakeElement::new("div").with_class("foo");
        let resolved = StyleMatcher::new().match_single(&sheet, &element);
        assert_eq!(resolved.get(PropertyId::BackgroundColor), Some(&color(0.5)));
        assert_eq!(resolved.len(), 1);
    }

    #[test]
    fn non_matching_element_resolves_empty() {
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![rule(
                vec![class("foo")],
                vec![BlockDeclaration::normal(
                    PropertyId::BackgroundColor,
                    color(0.5),
                )],
            )],
        );
        let element = FakeElement::new("div"); // no `foo` class
        let resolved = StyleMatcher::new().match_single(&sheet, &element);
        assert!(resolved.is_empty());
    }

    #[test]
    fn pseudo_element_rule_does_not_style_originating_element() {
        // The audit case: `.badge::before { color: red }` must leave `.badge`
        // itself unstyled. The kit generates no `::before` box, so the rule has
        // nowhere to land; anchoring it on the subject painted the wrong element.
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![rule(
                vec![
                    class("badge"),
                    SimpleSelector::PseudoElement("before".into()),
                ],
                vec![BlockDeclaration::normal(PropertyId::Color, color(1.0))],
            )],
        );
        let element = FakeElement::new("span").with_class("badge");
        let resolved = StyleMatcher::new().match_single(&sheet, &element);
        assert_eq!(resolved.get(PropertyId::Color), None);
        assert!(resolved.is_empty());
    }

    #[test]
    fn higher_specificity_wins() {
        // #id { color: A } beats .cls { color: B }; div { color: C } loses too.
        // The rules descend in specificity as they descend the sheet, so the
        // winner is the FIRST declaration collected: a matcher that forwarded a
        // constant specificity to the cascade would hand this to `div`.
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                rule(
                    vec![id("main")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.9))],
                ),
                rule(
                    vec![class("cls")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.6))],
                ),
                rule(
                    vec![ty("div")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.3))],
                ),
            ],
        );
        let element = FakeElement::new("div").with_id("main").with_class("cls");
        let resolved = StyleMatcher::new().match_single(&sheet, &element);
        assert_eq!(resolved.get(PropertyId::Color), Some(&color(0.9)));
    }

    #[test]
    fn class_beats_type() {
        // Winner first again, so the type rule's later position cannot rescue it.
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                rule(
                    vec![class("cls")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.6))],
                ),
                rule(
                    vec![ty("div")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.3))],
                ),
            ],
        );
        let element = FakeElement::new("div").with_class("cls");
        let resolved = StyleMatcher::new().match_single(&sheet, &element);
        assert_eq!(resolved.get(PropertyId::Color), Some(&color(0.6)));
    }

    #[test]
    fn later_source_order_wins_on_equal_specificity() {
        // Two class rules of equal specificity set the same property; the later
        // rule in document order must win. A trailing *type* rule then loses, so
        // the winner sits in the middle of the sheet and "later wins" is pinned
        // to the equal-specificity pair rather than to document position alone.
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                rule(
                    vec![class("cls")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.2))],
                ),
                rule(
                    vec![class("cls")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.8))],
                ),
                rule(
                    vec![ty("div")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.4))],
                ),
            ],
        );
        let element = FakeElement::new("div").with_class("cls");
        let resolved = StyleMatcher::new().match_single(&sheet, &element);
        assert_eq!(resolved.get(PropertyId::Color), Some(&color(0.8)));
    }

    #[test]
    fn important_beats_normal_regardless_of_specificity() {
        // Low-specificity !important beats high-specificity normal — and it does
        // so from the earlier position, so only the carried `important` flag can
        // explain the result.
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                rule(
                    vec![ty("div")],
                    vec![BlockDeclaration::important(PropertyId::Color, color(0.7))],
                ),
                rule(
                    vec![id("main")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.3))],
                ),
            ],
        );
        let element = FakeElement::new("div").with_id("main");
        let resolved = StyleMatcher::new().match_single(&sheet, &element);
        assert_eq!(resolved.get(PropertyId::Color), Some(&color(0.7)));
    }

    #[test]
    fn later_origin_normal_beats_earlier_origin_normal() {
        // A low-specificity normal declaration in a later origin (Inline) beats a
        // high-specificity normal one in an earlier origin (Theme). The Inline
        // sheet is passed FIRST: precedence follows the sheet's declared origin,
        // never its position in the argument slice, and the Theme rule's higher
        // specificity would take Color if the origin were dropped on the way in.
        let theme = Stylesheet::new(
            CascadeOrigin::Theme,
            vec![rule(
                vec![id("main")],
                vec![BlockDeclaration::normal(PropertyId::Color, color(0.2))],
            )],
        );
        let inline = Stylesheet::new(
            CascadeOrigin::Inline,
            vec![rule(
                vec![ty("div")],
                vec![BlockDeclaration::normal(PropertyId::Color, color(0.9))],
            )],
        );
        let element = FakeElement::new("div").with_id("main");
        let resolved = StyleMatcher::new().match_element(&[inline, theme], &element);
        assert_eq!(resolved.get(PropertyId::Color), Some(&color(0.9)));
    }

    #[test]
    fn multiple_properties_resolve_independently() {
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![rule(
                vec![class("box")],
                vec![
                    BlockDeclaration::normal(PropertyId::Width, PropertyValue::Number(100.0)),
                    BlockDeclaration::normal(PropertyId::Height, PropertyValue::Number(50.0)),
                    BlockDeclaration::normal(PropertyId::Opacity, PropertyValue::Number(0.5)),
                ],
            )],
        );
        let element = FakeElement::new("div").with_class("box");
        let resolved = StyleMatcher::new().match_single(&sheet, &element);
        assert_eq!(resolved.len(), 3);
        assert_eq!(
            resolved.get(PropertyId::Width),
            Some(&PropertyValue::Number(100.0))
        );
        assert_eq!(
            resolved.get(PropertyId::Height),
            Some(&PropertyValue::Number(50.0))
        );
        assert_eq!(
            resolved.get(PropertyId::Opacity),
            Some(&PropertyValue::Number(0.5))
        );
    }

    #[test]
    fn source_order_counts_declarations_in_non_matching_rules() {
        // A non-matching rule between two matching ones still advances
        // source_order, but only the matching declarations are emitted; the
        // later matching value must still win.
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                rule(
                    vec![class("cls")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.1))],
                ),
                rule(
                    vec![class("nope")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.5))],
                ),
                rule(
                    vec![class("cls")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.9))],
                ),
            ],
        );
        let element = FakeElement::new("div").with_class("cls");
        let resolved = StyleMatcher::new().match_single(&sheet, &element);
        assert_eq!(resolved.get(PropertyId::Color), Some(&color(0.9)));
    }

    #[test]
    fn empty_sheets_resolve_empty() {
        let element = FakeElement::new("div");
        let resolved = StyleMatcher::new().match_element(&[], &element);
        assert!(resolved.is_empty());
    }

    #[test]
    fn conditional_rule_gates_on_environment() {
        use crate::matching::{ConditionAxis, ConditionComparator, FeatureQuery, RuleCondition};
        // `.cls { color }` gated behind `@media (max-width: 1px)`.
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                rule(
                    vec![class("cls")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.7))],
                )
                .with_condition(RuleCondition::Feature(FeatureQuery::new(
                    ConditionAxis::ViewportWidth,
                    ConditionComparator::Le,
                    1.0,
                ))),
            ],
        );
        let element = FakeElement::new("div").with_class("cls");
        let matcher = StyleMatcher::new();

        // Under a wide viewport the condition fails: the rule does not apply.
        let wide = matcher.match_element_in(
            std::slice::from_ref(&sheet),
            &element,
            &MediaEnvironment::viewport(1000.0, 1000.0),
        );
        assert!(wide.is_empty(), "max-width:1px must not match at 1000px");

        // Under a ≤1px viewport the condition holds: the rule applies.
        let narrow = matcher.match_element_in(
            std::slice::from_ref(&sheet),
            &element,
            &MediaEnvironment::viewport(1.0, 1000.0),
        );
        assert_eq!(narrow.get(PropertyId::Color), Some(&color(0.7)));
    }

    #[test]
    fn unconditional_rule_always_matches_regardless_of_environment() {
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![rule(
                vec![class("cls")],
                vec![BlockDeclaration::normal(PropertyId::Color, color(0.4))],
            )],
        );
        let element = FakeElement::new("div").with_class("cls");
        let matcher = StyleMatcher::new();
        for env in [
            MediaEnvironment::viewport(1.0, 1.0),
            MediaEnvironment::viewport(4000.0, 4000.0),
            MediaEnvironment::unbounded(),
        ] {
            let resolved = matcher.match_element_in(std::slice::from_ref(&sheet), &element, &env);
            assert_eq!(resolved.get(PropertyId::Color), Some(&color(0.4)));
        }
    }

    /// `--brand` and `--accent`, as the compiler mints them: from the name, so
    /// two sheets that never met still agree on which property is which.
    fn brand() -> InternedSymbol {
        InternedSymbol::named("--brand")
    }

    fn custom_rule(sel: Vec<SimpleSelector>, name: InternedSymbol, v: f32) -> StyleRule {
        StyleRule::new(selectors(sel), Vec::new())
            .with_custom_properties(vec![crate::property::CustomProperty::new(name, color(v))])
    }

    #[test]
    fn a_custom_property_a_matched_rule_declares_is_collected_for_the_element() {
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![custom_rule(vec![class("foo")], brand(), 0.5)],
        );
        let element = FakeElement::new("div").with_class("foo");
        let custom = StyleMatcher::new().match_custom_properties_in(
            std::slice::from_ref(&sheet),
            &element,
            &MediaEnvironment::unbounded(),
        );
        assert_eq!(custom.get(&brand()), Some(&color(0.5)));
    }

    #[test]
    fn a_custom_property_from_a_rule_that_did_not_match_is_not_collected() {
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![custom_rule(vec![class("foo")], brand(), 0.5)],
        );
        let element = FakeElement::new("div"); // no `foo` class
        let custom = StyleMatcher::new().match_custom_properties_in(
            std::slice::from_ref(&sheet),
            &element,
            &MediaEnvironment::unbounded(),
        );
        assert!(custom.is_empty());
    }

    #[test]
    fn the_more_specific_declaration_of_one_custom_property_wins() {
        // The same cascade order the longhands resolve by: a custom property is
        // a declaration, and §4.14 does not give it a cascade of its own.
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                custom_rule(vec![id("main")], brand(), 0.9),
                custom_rule(vec![class("cls")], brand(), 0.6),
                custom_rule(vec![ty("div")], brand(), 0.3),
            ],
        );
        let element = FakeElement::new("div").with_id("main").with_class("cls");
        let custom = StyleMatcher::new().match_custom_properties_in(
            std::slice::from_ref(&sheet),
            &element,
            &MediaEnvironment::unbounded(),
        );
        assert_eq!(custom.get(&brand()), Some(&color(0.9)));
    }

    #[test]
    fn two_sheets_declaring_different_custom_properties_both_survive() {
        // The names are minted from the text, so nothing about the order the
        // sheets were compiled in can make one sheet's `--brand` collide with
        // another sheet's `--accent`.
        let accent = InternedSymbol::named("--accent");
        let first = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![custom_rule(vec![class("foo")], brand(), 0.5)],
        );
        let second = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![custom_rule(vec![class("foo")], accent, 0.25)],
        );
        let element = FakeElement::new("div").with_class("foo");
        let custom = StyleMatcher::new().match_custom_properties_in(
            &[first, second],
            &element,
            &MediaEnvironment::unbounded(),
        );
        assert_eq!(custom.get(&brand()), Some(&color(0.5)));
        assert_eq!(custom.get(&accent), Some(&color(0.25)));
    }

    // ----------------------------------------------------------------------
    // The indexed path against the unindexed one.
    //
    // `ResolvedDeclarations` compares its values AND the `CascadePriority` of
    // every winner — origin, `!important`, specificity and `source_order` — so
    // `assert_eq!` between the two paths is the byte-for-byte comparison the
    // index has to survive, not a spot check on a colour.
    // ----------------------------------------------------------------------

    /// One node of a small fake tree, so combinators and structural
    /// pseudo-classes have something to walk.
    struct FakeNode {
        local: Option<&'static str>,
        id: Option<&'static str>,
        classes: Vec<&'static str>,
        control_type: Option<&'static str>,
        hover: bool,
        parent: Option<usize>,
    }

    /// A cursor into a fake tree: the `MatchElement` both paths are handed.
    #[derive(Clone, Copy)]
    struct FakeCursor<'a> {
        nodes: &'a [FakeNode],
        at: usize,
    }

    impl FakeCursor<'_> {
        fn node(&self) -> &FakeNode {
            &self.nodes[self.at]
        }

        fn siblings(&self) -> Vec<usize> {
            let parent = self.node().parent;
            (0..self.nodes.len())
                .filter(|index| self.nodes[*index].parent == parent)
                .collect()
        }
    }

    impl MatchElement for FakeCursor<'_> {
        fn local_name(&self) -> Option<&str> {
            self.node().local
        }

        fn id(&self) -> Option<&str> {
            self.node().id
        }

        fn has_class(&self, class: &str) -> bool {
            self.node().classes.contains(&class)
        }

        fn enumerate_classes(&self, visit: &mut dyn FnMut(&str)) -> bool {
            for class in &self.node().classes {
                visit(class);
            }
            true
        }

        fn attribute(&self, name: &str) -> Option<&str> {
            match name {
                "type" => self.node().control_type,
                "id" => self.node().id,
                _ => None,
            }
        }

        fn pseudo_state(&self) -> PseudoState {
            if self.node().hover {
                PseudoState::HOVER
            } else {
                PseudoState::empty()
            }
        }

        fn parent(&self) -> Option<Self> {
            self.node().parent.map(|at| Self {
                nodes: self.nodes,
                at,
            })
        }

        fn previous_sibling_element(&self) -> Option<Self> {
            let siblings = self.siblings();
            let position = siblings.iter().position(|index| *index == self.at)?;
            let previous = position.checked_sub(1)?;
            Some(Self {
                nodes: self.nodes,
                at: siblings[previous],
            })
        }

        fn index_among_siblings(&self) -> usize {
            self.siblings()
                .iter()
                .position(|index| *index == self.at)
                .unwrap_or(0)
        }

        fn sibling_count(&self) -> usize {
            self.siblings().len().max(1)
        }

        fn is_root(&self) -> bool {
            self.node().parent.is_none()
        }

        fn has_children(&self) -> bool {
            self.nodes.iter().any(|node| node.parent == Some(self.at))
        }
    }

    fn node(
        local: &'static str,
        id: Option<&'static str>,
        classes: &[&'static str],
        parent: Option<usize>,
    ) -> FakeNode {
        FakeNode {
            local: Some(local),
            id,
            classes: classes.to_vec(),
            control_type: None,
            hover: false,
            parent,
        }
    }

    /// The tree both paths are run over, node by node.
    fn fake_tree() -> Vec<FakeNode> {
        let mut nodes = vec![
            node("div", Some("root"), &["app"], None),
            node("div", None, &["row"], Some(0)),
            node("button", None, &["key", "wide"], Some(1)),
            node("button", None, &["key"], Some(1)),
            node("span", None, &[], Some(3)),
            node("div", None, &["row", "tools"], Some(0)),
            node("input", Some("field"), &["key"], Some(5)),
            node("div", None, &[], Some(5)),
            node("em", None, &["key", "tool", "wide"], Some(7)),
        ];
        nodes[2].hover = true;
        nodes[6].control_type = Some("text");
        nodes[8].hover = true;
        nodes
    }

    fn complex(
        head: Vec<SimpleSelector>,
        tail: Vec<(Combinator, Vec<SimpleSelector>)>,
    ) -> ComplexSelector {
        ComplexSelector::new(
            CompoundSelector::new(head),
            tail.into_iter()
                .map(|(combinator, simples)| (combinator, CompoundSelector::new(simples)))
                .collect(),
        )
    }

    fn pseudo(name: PseudoClassSelector) -> SimpleSelector {
        SimpleSelector::PseudoClass(name)
    }

    /// Sheets written to make the index wrong if it is wrong anywhere: equal
    /// specificities settled only by order, ties across sheets and origins,
    /// `!important`, conditions, and — the case buckets cannot hold — rules
    /// whose subject names no class, id or type.
    fn adversarial_sheets() -> Vec<Stylesheet> {
        let ua = Stylesheet::new(
            CascadeOrigin::SystemDefault,
            vec![
                rule(
                    vec![ty("button")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.01))],
                ),
                // `*` — no bucket can hold it.
                rule(
                    vec![SimpleSelector::Universal],
                    vec![BlockDeclaration::normal(
                        PropertyId::Opacity,
                        PropertyValue::Number(0.1),
                    )],
                ),
                rule(
                    vec![ty("input")],
                    vec![BlockDeclaration::important(
                        PropertyId::Width,
                        PropertyValue::Number(1.0),
                    )],
                ),
            ],
        );
        let theme = Stylesheet::new(
            CascadeOrigin::Theme,
            vec![
                // Two equal-specificity class rules on the same property: only
                // source order separates them.
                rule(
                    vec![class("key")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.2))],
                ),
                rule(
                    vec![class("wide")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.3))],
                ),
                // `:hover` alone — bucketless.
                rule(
                    vec![pseudo(PseudoClassSelector::Hover)],
                    vec![BlockDeclaration::normal(
                        PropertyId::Height,
                        PropertyValue::Number(2.0),
                    )],
                ),
                // `[type=text]` — bucketless.
                rule(
                    vec![SimpleSelector::Attribute(AttributeSelector {
                        name: "type".into(),
                        matcher: Some(AttrMatch::Exact("text".into())),
                    })],
                    vec![BlockDeclaration::normal(
                        PropertyId::Height,
                        PropertyValue::Number(3.0),
                    )],
                ),
            ],
        );
        let app = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                // A two-selector list, one keyed on `.tool` and one on `#field`.
                StyleRule::new(
                    SelectorList::new(vec![
                        complex(vec![class("tool")], Vec::new()),
                        complex(vec![id("field")], Vec::new()),
                    ]),
                    vec![BlockDeclaration::normal(
                        PropertyId::BackgroundColor,
                        color(0.4),
                    )],
                ),
                // `.row > *` — the subject is universal, so bucketless.
                StyleRule::new(
                    SelectorList::new(vec![complex(
                        vec![class("row")],
                        vec![(Combinator::Child, vec![SimpleSelector::Universal])],
                    )]),
                    vec![BlockDeclaration::normal(
                        PropertyId::BackgroundColor,
                        color(0.5),
                    )],
                ),
                // A descendant combinator whose subject IS keyed.
                StyleRule::new(
                    SelectorList::new(vec![complex(
                        vec![class("app")],
                        vec![(
                            Combinator::Descendant,
                            vec![class("key"), pseudo(PseudoClassSelector::Hover)],
                        )],
                    )]),
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.6))],
                ),
                // A sibling combinator.
                StyleRule::new(
                    SelectorList::new(vec![complex(
                        vec![class("key")],
                        vec![(Combinator::NextSibling, vec![class("key")])],
                    )]),
                    vec![BlockDeclaration::normal(
                        PropertyId::Opacity,
                        PropertyValue::Number(0.7),
                    )],
                ),
                // A structural pseudo-class with no other key: bucketless.
                rule(
                    vec![pseudo(PseudoClassSelector::FirstChild)],
                    vec![BlockDeclaration::normal(
                        PropertyId::ZIndex,
                        PropertyValue::Number(1.0),
                    )],
                ),
                // A pseudo-element that must style nothing, keyed on a class.
                rule(
                    vec![class("key"), SimpleSelector::PseudoElement("before".into())],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.9))],
                ),
                // Conditional: gated out at a wide viewport, in at a narrow one.
                rule(
                    vec![class("key")],
                    vec![BlockDeclaration::normal(
                        PropertyId::Top,
                        PropertyValue::Number(4.0),
                    )],
                )
                .with_condition(RuleCondition::Feature(FeatureQuery::new(
                    ConditionAxis::ViewportWidth,
                    ConditionComparator::Le,
                    100.0,
                ))),
                // Low-specificity `!important` against a high-specificity normal.
                rule(
                    vec![ty("em")],
                    vec![BlockDeclaration::important(
                        PropertyId::Left,
                        PropertyValue::Number(5.0),
                    )],
                ),
                rule(
                    vec![id("field"), class("key")],
                    vec![BlockDeclaration::normal(
                        PropertyId::Left,
                        PropertyValue::Number(6.0),
                    )],
                ),
                // Custom properties, keyed and bucketless.
                custom_rule(vec![class("key")], brand(), 0.11),
                custom_rule(vec![SimpleSelector::Universal], brand(), 0.22),
                custom_rule(vec![id("root")], InternedSymbol::named("--accent"), 0.33),
            ],
        );
        // A second app-origin sheet whose rules tie the first sheet's on
        // specificity and are separated only by sitting later.
        let overlay = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                rule(
                    vec![class("key")],
                    vec![BlockDeclaration::normal(
                        PropertyId::BackgroundColor,
                        color(0.8),
                    )],
                ),
                rule(
                    vec![class("tool")],
                    vec![BlockDeclaration::normal(
                        PropertyId::BackgroundColor,
                        color(0.85),
                    )],
                ),
                custom_rule(vec![class("tool")], brand(), 0.44),
            ],
        );
        vec![ua, theme, app, overlay]
    }

    #[test]
    fn the_indexed_walk_resolves_every_node_exactly_as_the_full_walk_does() {
        let sheets = adversarial_sheets();
        let index = RuleIndex::build(&sheets);
        let nodes = fake_tree();
        let matcher = StyleMatcher::new();
        let mut offered = 0usize;
        let total: usize = sheets.iter().map(|sheet| sheet.rules().len()).sum();

        for environment in [
            MediaEnvironment::unbounded(),
            MediaEnvironment::viewport(50.0, 50.0),
            MediaEnvironment::viewport(1000.0, 1000.0),
        ] {
            for at in 0..nodes.len() {
                let element = FakeCursor { nodes: &nodes, at };
                assert_eq!(
                    matcher.match_element_indexed(&index, &sheets, &element, &environment),
                    matcher.match_element_in(&sheets, &element, &environment),
                    "node {at} resolved differently through the index"
                );
                assert_eq!(
                    matcher.match_custom_properties_indexed(
                        &index,
                        &sheets,
                        &element,
                        &environment
                    ),
                    matcher.match_custom_properties_in(&sheets, &element, &environment),
                    "node {at} collected different custom properties through the index"
                );
                let mut seen = 0usize;
                assert!(index.candidates(&element, |_| seen += 1));
                offered += seen;
            }
        }

        // Without this the test would still pass if every rule landed in the
        // bucketless pile — agreement is cheap when nothing is skipped.
        let walked = total * nodes.len() * 3;
        assert!(
            offered * 2 < walked,
            "the index offered {offered} of {walked} rule visits — it is not narrowing anything"
        );
    }

    #[test]
    fn between_two_rules_of_equal_specificity_the_later_one_still_wins_through_the_index() {
        // `.key { color }` then `.wide { color }`, equal specificity, separated by
        // document order alone — and the element carries both classes, so the two
        // arrive from two DIFFERENT buckets. Nothing about which bucket a rule was
        // found in may reach the cascade: if the index handed out a `source_order`
        // of its own counting, or merged the buckets one after another instead of
        // by position, the earlier rule would win and the keyboard would wear the
        // wrong colour with every test above still green.
        let sheets = adversarial_sheets();
        let index = RuleIndex::build(&sheets);
        let nodes = fake_tree();
        // Node 2 is the `button.key.wide`.
        let element = FakeCursor {
            nodes: &nodes,
            at: 2,
        };
        let resolved = StyleMatcher::new().match_element_indexed(
            &index,
            &sheets,
            &element,
            &MediaEnvironment::unbounded(),
        );
        // `.wide` stands later in the theme sheet than `.key`, and the app sheet's
        // `.app .key:hover` (three classes' worth of specificity) outranks both.
        assert_eq!(resolved.get(PropertyId::Color), Some(&color(0.6)));

        // Same question one band down: with the app rule gone, the tie between the
        // two theme rules is all that is left, and later must still win.
        let theme_only = vec![sheets[1].clone()];
        let theme_index = RuleIndex::build(&theme_only);
        let themed = StyleMatcher::new().match_element_indexed(
            &theme_index,
            &theme_only,
            &element,
            &MediaEnvironment::unbounded(),
        );
        assert_eq!(themed.get(PropertyId::Color), Some(&color(0.3)));
    }

    #[test]
    fn a_custom_property_declared_twice_at_equal_specificity_keeps_the_later_value() {
        // The same tie, in the second pass, where `source_order` is counted per
        // ELEMENT rather than read off the rule — so it depends on the order the
        // index yields candidates in and on nothing else.
        let sheets = adversarial_sheets();
        let index = RuleIndex::build(&sheets);
        let nodes = fake_tree();
        // Node 8 is the `em.key.tool.wide`: `.key { --brand: 0.11 }` in the app
        // sheet, `.tool { --brand: 0.44 }` later in the overlay sheet.
        let element = FakeCursor {
            nodes: &nodes,
            at: 8,
        };
        let custom = StyleMatcher::new().match_custom_properties_indexed(
            &index,
            &sheets,
            &element,
            &MediaEnvironment::unbounded(),
        );
        assert_eq!(custom.get(&brand()), Some(&color(0.44)));
    }

    #[test]
    fn an_element_that_cannot_list_its_classes_still_resolves_correctly() {
        // `FakeElement` does not override `enumerate_classes`, so the indexed
        // entry points must fall back to the full walk rather than conclude the
        // element carries no classes at all.
        let sheets = adversarial_sheets();
        let index = RuleIndex::build(&sheets);
        let element = FakeElement::new("button")
            .with_class("key")
            .with_class("wide");
        let matcher = StyleMatcher::new();
        let environment = MediaEnvironment::unbounded();
        assert_eq!(
            matcher.match_element_indexed(&index, &sheets, &element, &environment),
            matcher.match_element_in(&sheets, &element, &environment),
        );
    }

    #[test]
    fn an_index_built_for_other_sheets_is_refused_rather_than_believed() {
        let sheets = adversarial_sheets();
        let stale = RuleIndex::build(&sheets[..2]);
        let nodes = fake_tree();
        let matcher = StyleMatcher::new();
        let environment = MediaEnvironment::unbounded();
        for at in 0..nodes.len() {
            let element = FakeCursor { nodes: &nodes, at };
            assert_eq!(
                matcher.match_element_indexed(&stale, &sheets, &element, &environment),
                matcher.match_element_in(&sheets, &element, &environment),
                "node {at} was resolved from an index that does not describe the sheets"
            );
        }
    }
}
