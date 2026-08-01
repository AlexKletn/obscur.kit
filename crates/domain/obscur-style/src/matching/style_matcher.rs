#![doc = include_str!("style_matcher.doc/_module.md")]

use crate::cascade::{Cascade, Declaration, ResolvedDeclarations};
use crate::matching::{MediaEnvironment, Stylesheet};
use crate::selector::{MatchElement, SelectorMatcher};

#[doc = include_str!("style_matcher.doc/StyleMatcher.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct StyleMatcher;

impl StyleMatcher {
    #[doc = include_str!("style_matcher.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("style_matcher.doc/match_element.md")]
    #[must_use]
    pub fn match_element<E: MatchElement>(
        &self,
        sheets: &[Stylesheet],
        element: &E,
    ) -> ResolvedDeclarations {
        self.match_element_in(sheets, element, &MediaEnvironment::unbounded())
    }

    #[doc = include_str!("style_matcher.doc/match_element_in.md")]
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
                let specificity = SelectorMatcher::matches_list(rule.selectors(), element);
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

    #[doc = include_str!("style_matcher.doc/match_single.md")]
    #[must_use]
    pub fn match_single<E: MatchElement>(
        &self,
        sheet: &Stylesheet,
        element: &E,
    ) -> ResolvedDeclarations {
        self.match_element(std::slice::from_ref(sheet), element)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cascade::CascadeOrigin;
    use crate::matching::{BlockDeclaration, StyleRule};
    use crate::property::{PropertyId, PropertyValue, StyleColor};
    use crate::selector::{
        ComplexSelector, CompoundSelector, PseudoState, SelectorList, SimpleSelector,
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

    // ---- selector / rule construction helpers ------------------------------

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

    // ---- tests -------------------------------------------------------------

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
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                rule(
                    vec![ty("div")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.3))],
                ),
                rule(
                    vec![class("cls")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.6))],
                ),
                rule(
                    vec![id("main")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.9))],
                ),
            ],
        );
        let element = FakeElement::new("div").with_id("main").with_class("cls");
        let resolved = StyleMatcher::new().match_single(&sheet, &element);
        assert_eq!(resolved.get(PropertyId::Color), Some(&color(0.9)));
    }

    #[test]
    fn class_beats_type() {
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                rule(
                    vec![ty("div")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.3))],
                ),
                rule(
                    vec![class("cls")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.6))],
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
        // rule in document order must win.
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
            ],
        );
        let element = FakeElement::new("div").with_class("cls");
        let resolved = StyleMatcher::new().match_single(&sheet, &element);
        assert_eq!(resolved.get(PropertyId::Color), Some(&color(0.8)));
    }

    #[test]
    fn important_beats_normal_regardless_of_specificity() {
        // Low-specificity !important beats high-specificity normal.
        let sheet = Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                rule(
                    vec![id("main")],
                    vec![BlockDeclaration::normal(PropertyId::Color, color(0.3))],
                ),
                rule(
                    vec![ty("div")],
                    vec![BlockDeclaration::important(PropertyId::Color, color(0.7))],
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
        // high-specificity normal one in an earlier origin (Theme).
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
        let resolved = StyleMatcher::new().match_element(&[theme, inline], &element);
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
}
