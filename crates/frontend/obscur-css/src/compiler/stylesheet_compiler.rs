#![doc = include_str!("../../docs/api/compiler/stylesheet_compiler.doc/_module.md")]

use crate::parser::{ParsedRule, ParsedStyleRule, ParsedStylesheet};

use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::{BlockDeclaration, RuleCondition, StyleRule, Stylesheet};
use obscur_style::selector::SelectorSpecificity;

use super::animation_compiler::AnimationCompiler;
use super::condition_lowering::lower_condition;
use super::longhand_catalog::{keyword_value_is_foreign, property_id_from_name};
use super::pending_motion_rule::PendingMotionRule;
use super::selector_compiler::SelectorCompiler;
use super::stylesheet_accumulator::StylesheetAccumulator;
use super::{
    CompiledAnimation, CompiledCondition, CompiledConditionalGroup, CompiledKeyframes,
    CompiledMotionRule, CompiledStylesheet, CssSymbolInterner, KeyframeCompiler,
    TransitionCompiler,
};
use super::{CompiledDeclaration, PropertyCompiler};

#[doc = include_str!("../../docs/api/compiler/stylesheet_compiler.doc/StylesheetCompiler.md")]
#[derive(Debug)]
pub struct StylesheetCompiler {
    interner: CssSymbolInterner,
    selectors: SelectorCompiler,
    properties: PropertyCompiler,
    keyframes: KeyframeCompiler,
    transitions: TransitionCompiler,
    animations: AnimationCompiler,
    origin: CascadeOrigin,
    source_order: u32,
}

impl StylesheetCompiler {
    /// Creates a compiler that assigns declarations the `GlobalApp` origin.
    #[must_use]
    pub fn new() -> Self {
        Self {
            interner: CssSymbolInterner::new(),
            selectors: SelectorCompiler::new(),
            properties: PropertyCompiler::new(),
            keyframes: KeyframeCompiler::new(),
            transitions: TransitionCompiler::new(),
            animations: AnimationCompiler::new(),
            origin: CascadeOrigin::GlobalApp,
            source_order: 0,
        }
    }

    /// Sets the [`CascadeOrigin`] every compiled declaration is tagged with.
    #[must_use]
    pub const fn with_origin(mut self, origin: CascadeOrigin) -> Self {
        self.origin = origin;
        self
    }

    /// Compiles `sheet` into a [`CompiledStylesheet`], consuming this compiler's
    /// interner into the result.
    #[must_use]
    pub fn compile(mut self, sheet: &ParsedStylesheet) -> CompiledStylesheet {
        let mut acc = StylesheetAccumulator::default();
        self.compile_rules(&sheet.rules, &mut acc);

        let mut unsupported = sheet.unsupported.clone();
        unsupported.append(&mut acc.unsupported);

        let matchable = Stylesheet::new(self.origin, acc.matchable_rules);

        // Resolve each pending motion rule's `animation` references against the
        // now-complete set of compiled `@keyframes` (a rule may reference a
        // `@keyframes` defined later in the sheet).
        let motion_rules = acc
            .motion_rules
            .into_iter()
            .filter_map(|pending| resolve_motion_rule(pending, &acc.keyframes))
            .collect();

        CompiledStylesheet {
            declarations: acc.declarations,
            custom_properties: acc.custom_properties,
            conditional_groups: acc.conditional_groups,
            keyframes: acc.keyframes,
            transitions: acc.transitions,
            motion_rules,
            symbols: self.interner,
            matchable,
            unsupported,
            errors: sheet.errors.clone(),
        }
    }

    /// Compiles a list of rules into `acc`, recursing into conditional groups.
    fn compile_rules(&mut self, rules: &[ParsedRule], acc: &mut StylesheetAccumulator) {
        for rule in rules {
            match rule {
                ParsedRule::Style(style) => self.compile_style_rule(style, acc),
                ParsedRule::Media(media) => {
                    let mut inner = StylesheetAccumulator::default();
                    self.compile_rules(&media.rules, &mut inner);
                    lift_group(CompiledCondition::Media(media.query.clone()), inner, acc);
                }
                ParsedRule::Container(container) => {
                    let mut inner = StylesheetAccumulator::default();
                    self.compile_rules(&container.rules, &mut inner);
                    lift_group(
                        CompiledCondition::Container(container.query.clone()),
                        inner,
                        acc,
                    );
                }
                ParsedRule::Keyframes(rule) => {
                    let compiled = self.keyframes.compile(rule, &mut self.interner);
                    acc.keyframes.push(compiled);
                }
            }
        }
    }

    /// Compiles a style rule: one declaration set per matched selector, plus the
    /// rule's `transition` templates (compiled once, being selector-independent).
    fn compile_style_rule(&mut self, style: &ParsedStyleRule, acc: &mut StylesheetAccumulator) {
        let selectors = self
            .selectors
            .compile_list(&style.selectors, &mut acc.unsupported);
        for selector in &selectors {
            for decl in &style.declarations {
                if is_motion_decl(&decl.name) {
                    continue; // routed to the transition/keyframe machinery below.
                }
                let order = self.source_order;
                self.source_order += 1;
                // `compile_all` expands `margin` / `padding` shorthands into
                // their longhands; every other declaration yields one result.
                for compiled in self.properties.compile_all(
                    decl,
                    selector.specificity,
                    self.origin,
                    order,
                    Some(selector.id),
                    &mut self.interner,
                ) {
                    match compiled {
                        CompiledDeclaration::Standard(declaration) => {
                            acc.declarations.push(declaration);
                        }
                        CompiledDeclaration::Custom(custom) => acc.custom_properties.push(custom),
                        CompiledDeclaration::Unsupported(feature) => acc.unsupported.push(feature),
                    }
                }
            }
        }
        // Compile this rule's `transition` templates (once — they are
        // selector-independent) and its `animation` references. They feed both
        // the flat top-level outputs (backward-compatible) and the
        // selector-associated `PendingMotionRule` the runtime drives.
        let mut rule_transitions = Vec::new();
        let mut transition_reset = false;
        let mut transition_important = false;
        for decl in &style.declarations {
            if !is_transition_shorthand(&decl.name) || (transition_important && !decl.important) {
                continue;
            }
            let reset = matches!(decl.value.as_slice(), [crate::parser::ParsedValue::Keyword(name)]
                if name.eq_ignore_ascii_case("none"));
            let compiled = if reset {
                Vec::new()
            } else {
                self.transitions.compile(decl, &mut acc.unsupported)
            };
            if reset || !compiled.is_empty() {
                rule_transitions = compiled;
                transition_reset = reset;
                transition_important = decl.important;
            }
        }
        acc.transitions.extend(rule_transitions.iter().copied());
        let animations = self
            .animations
            .compile(&style.declarations, &mut self.interner);
        if transition_reset || !rule_transitions.is_empty() || !animations.is_empty() {
            acc.motion_rules.push(PendingMotionRule {
                condition: None,
                selectors: SelectorCompiler::lower_list(&style.selectors),
                transitions: rule_transitions,
                transition_reset,
                transition_important,
                animations,
            });
        }

        let rule = self.lower_matchable_rule(style);
        acc.matchable_rules.push(rule);
    }

    #[doc = include_str!("../../docs/api/compiler/stylesheet_compiler.doc/lower_matchable_rule.md")]
    fn lower_matchable_rule(&mut self, style: &ParsedStyleRule) -> StyleRule {
        let mut block = Vec::new();
        // The longhands this rule authored whose value could not be parsed: they
        // never enter the block, so a matched element loses only that one
        // declaration (§4.13). Recorded on the rule so the runtime can raise a
        // per-node diagnostic attributed to the element that matched it.
        let mut invalid_values = Vec::new();
        // The `--name: value` declarations this rule authored. They ride ON the
        // rule, not on the sheet, because they cascade per element like every
        // other declaration: `:root { --brand: red }` and `.dark { --brand: blue }`
        // are two answers for two elements, and a sheet-level list could only
        // hold one of them.
        let mut custom_properties = Vec::new();
        for decl in &style.declarations {
            if is_motion_decl(&decl.name) {
                continue;
            }
            // The cascade context is irrelevant to a block declaration; the
            // matcher supplies specificity/origin/order at match time.
            // `compile_all` expands `margin` / `padding` shorthands so every
            // authored longhand enters the block.
            for compiled in self.properties.compile_all(
                decl,
                SelectorSpecificity::ZERO,
                self.origin,
                0,
                None,
                &mut self.interner,
            ) {
                match compiled {
                    CompiledDeclaration::Standard(declaration) => {
                        block.push(BlockDeclaration::new(
                            declaration.property(),
                            declaration.value().clone(),
                            declaration.important(),
                        ));
                    }
                    CompiledDeclaration::Custom(custom) => custom_properties.push(custom),
                    CompiledDeclaration::Unsupported(_) => {}
                }
            }
            // Keep the per-rule diagnostic for a bare keyword the property
            // cannot interpret.
            if let Some(property) = property_id_from_name(&decl.name)
                && keyword_value_is_foreign(property, &decl.value)
            {
                invalid_values.push(property);
            }
        }
        StyleRule::new(SelectorCompiler::lower_list(&style.selectors), block)
            .with_invalid_values(invalid_values)
            .with_custom_properties(custom_properties)
    }
}

impl Default for StylesheetCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[doc = include_str!("../../docs/api/compiler/stylesheet_compiler.doc/lift_group.md")]
fn lift_group(
    condition: CompiledCondition,
    mut inner: StylesheetAccumulator,
    acc: &mut StylesheetAccumulator,
) {
    acc.conditional_groups.append(&mut inner.conditional_groups);
    acc.unsupported.append(&mut inner.unsupported);
    let lowered = lower_condition(&condition, &mut acc.unsupported);
    let combine = |existing| match existing {
        Some(existing) => RuleCondition::All(vec![lowered.clone(), existing]),
        None => lowered.clone(),
    };
    for mut definition in inner.keyframes {
        definition.condition = Some(combine(definition.condition.take()));
        acc.keyframes.push(definition);
    }
    for mut rule in inner.motion_rules {
        rule.condition = Some(combine(rule.condition.take()));
        acc.motion_rules.push(rule);
    }
    for rule in inner.matchable_rules {
        let combined = combine(rule.condition().cloned());
        acc.matchable_rules.push(rule.with_condition(combined));
    }
    acc.conditional_groups.push(CompiledConditionalGroup {
        condition,
        declarations: inner.declarations,
        custom_properties: inner.custom_properties,
        transitions: inner.transitions,
    });
}

#[doc = include_str!("../../docs/api/compiler/stylesheet_compiler.doc/resolve_motion_rule.md")]
fn resolve_motion_rule(
    pending: PendingMotionRule,
    keyframes: &[CompiledKeyframes],
) -> Option<CompiledMotionRule> {
    let animations: Vec<CompiledAnimation> = pending
        .animations
        .iter()
        .enumerate()
        .flat_map(|(reference_index, reference)| {
            keyframes
                .iter()
                .filter(move |compiled| compiled.name == reference.name)
                .map(move |compiled| CompiledAnimation {
                    reference_index,
                    condition: compiled.condition.clone(),
                    tracks: compiled.tracks.clone(),
                    duration_secs: reference.duration_secs,
                    delay_secs: reference.delay_secs,
                    repeat: reference.repeat,
                    direction: reference.direction,
                    alternates: reference.alternates,
                    fill: reference.fill,
                })
        })
        .collect();
    if !pending.transition_reset && pending.transitions.is_empty() && animations.is_empty() {
        return None;
    }
    Some(CompiledMotionRule {
        condition: pending.condition,
        selectors: pending.selectors,
        transitions: pending.transitions,
        transition_reset: pending.transition_reset,
        transition_important: pending.transition_important,
        animations,
    })
}

/// Returns whether a declaration name is any `transition-*`/`animation-*`
/// property (routed away from ordinary property compilation).
fn is_motion_decl(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.starts_with("transition") || lower.starts_with("animation")
}

/// Returns whether a declaration is exactly the `transition` shorthand.
fn is_transition_shorthand(name: &str) -> bool {
    name.eq_ignore_ascii_case("transition")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{UnsupportedCssFeature, UnsupportedFeatureKind};
    use crate::source::CssSource;
    use obscur_identity::InternedSymbol;
    use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor, layout_keyword};
    use obscur_style::selector::SelectorSpecificity;

    fn compile(text: &str) -> CompiledStylesheet {
        let src = CssSource::anonymous(text);
        let sheet = crate::parser::StylesheetParser::new(&src).parse();
        StylesheetCompiler::new().compile(&sheet)
    }

    #[test]
    fn an_unknown_keyword_at_a_reserved_ordinal_is_not_read_as_that_keyword() {
        use std::fmt::Write;

        let ordinal = layout_keyword::SIZING_FIT_CONTENT.raw();
        let mut css = String::from("a { width: 60px;");
        for i in 0..ordinal {
            let _ = write!(css, " width: kw{i};");
        }
        css.push_str(" width: sideways; }");
        let sheet = compile(&css);
        assert_eq!(sheet.declarations.len(), 1);
        assert_eq!(
            sheet.declarations[0].value(),
            &PropertyValue::Length(obscur_style::property::Length::px(60.0)),
        );
    }

    #[test]
    fn compiles_a_simple_rule() {
        let sheet = compile("a { color: black; width: 10px }");
        assert_eq!(sheet.declarations.len(), 2);
        assert!(sheet.errors.is_empty());
        assert!(sheet.unsupported.is_empty());
    }

    #[test]
    fn declarations_get_ascending_source_order() {
        let sheet = compile("a { color: black; width: 10px }");
        assert_eq!(sheet.declarations[0].source_order(), 0);
        assert_eq!(sheet.declarations[1].source_order(), 1);
    }

    #[test]
    fn declaration_carries_default_origin() {
        let sheet = compile("a { color: black }");
        assert_eq!(sheet.declarations[0].origin(), CascadeOrigin::GlobalApp);
    }

    #[test]
    fn with_origin_overrides_the_default() {
        let src = CssSource::anonymous("a { color: black }");
        let parsed = crate::parser::StylesheetParser::new(&src).parse();
        let sheet = StylesheetCompiler::new()
            .with_origin(CascadeOrigin::Theme)
            .compile(&parsed);
        assert_eq!(sheet.declarations[0].origin(), CascadeOrigin::Theme);
    }

    #[test]
    fn specificity_flows_into_declarations() {
        let sheet = compile("#main { color: black }");
        assert_eq!(
            sheet.declarations[0].specificity(),
            SelectorSpecificity::new(1, 0, 0)
        );
    }

    #[test]
    fn selector_list_compiles_declarations_per_selector() {
        let sheet = compile("a, b { color: black }");
        assert_eq!(sheet.declarations.len(), 2);
        assert_ne!(
            sheet.declarations[0].selector(),
            sheet.declarations[1].selector()
        );
    }

    #[test]
    fn custom_properties_are_separated() {
        let sheet = compile("a { --brand: #fff; color: black }");
        assert_eq!(sheet.custom_properties.len(), 1);
        assert_eq!(sheet.declarations.len(), 1);
    }

    #[test]
    fn unsupported_property_is_recorded() {
        let sheet = compile("a { float: left }");
        assert!(sheet.declarations.is_empty());
        assert_eq!(sheet.unsupported.len(), 1);
    }

    #[test]
    fn pseudo_element_rule_is_reported_as_unsupported() {
        let sheet = compile(".badge::before { content: \"!\" } li::marker { color: red }");
        let reported: Vec<&str> = sheet
            .unsupported
            .iter()
            .filter(|f| f.kind() == UnsupportedFeatureKind::PseudoElement)
            .map(UnsupportedCssFeature::name)
            .collect();
        assert_eq!(reported, ["::marker"]);
    }

    #[test]
    fn keyframes_are_compiled() {
        let sheet = compile("@keyframes fade { from { opacity: 0 } to { opacity: 1 } }");
        assert_eq!(sheet.keyframes.len(), 1);
        assert_eq!(sheet.keyframes[0].tracks.len(), 1);
    }

    #[test]
    fn transition_shorthand_is_compiled() {
        let sheet = compile("a { transition: opacity 0.3s ease-in }");
        assert_eq!(sheet.transitions.len(), 1);
        assert_eq!(sheet.transitions[0].property, PropertyId::Opacity);
        // The transition decl must not also appear as a standard declaration.
        assert!(sheet.declarations.is_empty());
    }

    #[test]
    fn transition_is_compiled_once_per_rule_not_per_selector() {
        let sheet = compile("a, b { transition: opacity 0.3s }");
        assert_eq!(sheet.transitions.len(), 1);
    }

    #[test]
    fn motion_rule_carries_transition_and_selector() {
        use obscur_style::selector::SimpleSelector;
        let sheet = compile(".box { transition: width 0.5s ease-out }");
        assert_eq!(sheet.motion_rules.len(), 1);
        let rule = &sheet.motion_rules[0];
        assert_eq!(rule.transitions.len(), 1);
        assert_eq!(rule.transitions[0].property, PropertyId::Width);
        // The selector list is preserved so the runtime can match `.box`.
        let selectors = rule.selectors.selectors();
        assert_eq!(selectors.len(), 1);
        assert_eq!(
            selectors[0].subject().simples(),
            &[SimpleSelector::Class("box".into())]
        );
        assert!(rule.animations.is_empty());
    }

    #[test]
    fn motion_rule_resolves_animation_name_to_tracks() {
        let sheet = compile(
            "@keyframes fade { from { opacity: 0 } to { opacity: 1 } }\n\
             .box { animation: fade 2s }",
        );
        assert_eq!(sheet.motion_rules.len(), 1);
        let rule = &sheet.motion_rules[0];
        assert_eq!(rule.animations.len(), 1);
        assert_eq!(rule.animations[0].duration_secs, 2.0);
        assert_eq!(rule.animations[0].tracks.len(), 1);
        assert_eq!(rule.animations[0].tracks[0].0, PropertyId::Opacity);
    }

    #[test]
    fn animation_name_resolves_even_when_keyframes_defined_after() {
        // The rule references `spin` before it is declared; resolution is a
        // post-pass over the full sheet, so the tracks still bind.
        let sheet = compile(
            ".box { animation-name: spin; animation-duration: 1s }\n\
             @keyframes spin { from { width: 0px } to { width: 10px } }",
        );
        assert_eq!(sheet.motion_rules.len(), 1);
        assert_eq!(sheet.motion_rules[0].animations.len(), 1);
        assert_eq!(sheet.motion_rules[0].animations[0].duration_secs, 1.0);
    }

    #[test]
    fn unknown_animation_name_produces_no_motion_rule() {
        // `ghost` names no `@keyframes`, and there is no transition, so the rule
        // carries no drivable motion and is dropped.
        let sheet = compile(".box { animation: ghost 1s }");
        assert!(sheet.motion_rules.is_empty());
    }

    #[test]
    fn plain_rule_produces_no_motion_rule() {
        let sheet = compile(".box { color: black }");
        assert!(sheet.motion_rules.is_empty());
    }

    #[test]
    fn animation_longhands_are_not_reported_unsupported() {
        // `animation-name` is recognised and routed away, not flagged.
        let sheet = compile("a { animation-name: fade }");
        assert!(sheet.declarations.is_empty());
        assert!(sheet.unsupported.is_empty());
    }

    #[test]
    fn media_rule_becomes_a_conditional_group() {
        let sheet = compile("@media screen { a { color: black } }");
        assert_eq!(sheet.conditional_groups.len(), 1);
        let group = &sheet.conditional_groups[0];
        assert!(matches!(group.condition, CompiledCondition::Media(_)));
        assert_eq!(group.declarations.len(), 1);
        // Conditional declarations are not duplicated at the top level.
        assert!(sheet.declarations.is_empty());
    }

    #[test]
    fn container_rule_becomes_a_conditional_group() {
        let sheet = compile("@container (min-width: 400px) { a { width: 10px } }");
        assert_eq!(sheet.conditional_groups.len(), 1);
        assert!(matches!(
            sheet.conditional_groups[0].condition,
            CompiledCondition::Container(_)
        ));
    }

    #[test]
    fn parser_errors_and_unsupported_carry_through() {
        let sheet = compile("@import \"x.css\"; a { color: black }");
        assert_eq!(sheet.unsupported.len(), 1);
        assert_eq!(sheet.unsupported[0].name(), "@import");
        assert_eq!(sheet.declarations.len(), 1);
    }

    #[test]
    fn excluded_pseudo_class_is_reported() {
        let sheet = compile("a:visited { color: black }");
        assert!(sheet.unsupported.iter().any(|f| f.name() == ":visited"));
    }

    #[test]
    fn matchable_holds_one_rule_per_style_rule() {
        let sheet =
            compile(".counter { background-color: #ff0000 } #main .value { color: #00ff00 }");
        let rules = sheet.matchable.rules();
        assert_eq!(rules.len(), 2);
        assert_eq!(sheet.matchable.origin(), CascadeOrigin::GlobalApp);
    }

    #[test]
    fn matchable_first_rule_lowers_class_and_block() {
        use obscur_style::selector::SimpleSelector;
        let sheet =
            compile(".counter { background-color: #ff0000 } #main .value { color: #00ff00 }");
        let first = &sheet.matchable.rules()[0];
        let selectors = first.selectors().selectors();
        assert_eq!(selectors.len(), 1);
        assert_eq!(
            selectors[0].subject().simples(),
            &[SimpleSelector::Class("counter".into())]
        );
        assert_eq!(first.block().len(), 1);
        assert_eq!(first.block()[0].property(), PropertyId::BackgroundColor);
        assert_eq!(
            *first.block()[0].value(),
            PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
        );
    }

    #[test]
    fn matchable_second_rule_lowers_descendant_combinator() {
        use obscur_style::selector::{Combinator, SimpleSelector};
        let sheet = compile("#main .value { color: #00ff00 }");
        let complex = &sheet.matchable.rules()[0].selectors().selectors()[0];
        assert_eq!(
            complex.head().simples(),
            &[SimpleSelector::Id("main".into())]
        );
        assert_eq!(complex.tail().len(), 1);
        assert_eq!(complex.tail()[0].0, Combinator::Descendant);
    }

    #[test]
    fn matchable_skips_custom_and_unsupported_from_blocks() {
        // `--brand` (custom) and `float` (unsupported) must not enter the block;
        // only the standard `color` declaration does.
        let sheet = compile("a { --brand: #fff; float: left; color: black }");
        let block = sheet.matchable.rules()[0].block();
        assert_eq!(block.len(), 1);
        assert_eq!(block[0].property(), PropertyId::Color);
    }

    #[test]
    fn a_rules_custom_properties_ride_on_the_matchable_rule() {
        // The sheet-level list cannot say WHICH elements got `--brand`, and that
        // is the whole question: a custom property cascades per element like any
        // other declaration. A rule that dropped it is a `var()` that resolves
        // to nothing.
        let sheet = compile(":root { --brand: #ff0000 } a { color: var(--brand) }");
        let root = &sheet.matchable.rules()[0];
        assert_eq!(root.custom_properties().len(), 1);
        assert_eq!(
            root.custom_properties()[0].name(),
            InternedSymbol::named("--brand")
        );
        // The declared TEXT: what `#ff0000` means is `color`'s business, not
        // the declaration's (CSS Variables 1 §2).
        assert_eq!(
            root.custom_properties()[0].value(),
            &PropertyValue::RawTokens("#ff0000".into())
        );
        // The rule that only READS it declares none.
        assert!(sheet.matchable.rules()[1].custom_properties().is_empty());
        assert_eq!(
            sheet.matchable.rules()[1].block()[0].value(),
            &PropertyValue::CustomRef(InternedSymbol::named("--brand"))
        );
    }

    #[test]
    fn a_custom_property_declared_inside_a_media_group_rides_on_its_gated_rule() {
        let sheet = compile("@media screen { :root { --brand: #ff0000 } }");
        let rule = &sheet.matchable.rules()[0];
        assert!(rule.condition().is_some(), "still gated");
        assert_eq!(rule.custom_properties().len(), 1);
    }

    #[test]
    fn matchable_includes_conditional_group_rules_gated() {
        let sheet = compile("a { color: black } @media screen { p { width: 10px } }");
        // Both the top-level `a` rule and the nested `p` rule are matchable.
        assert_eq!(sheet.matchable.rules().len(), 2);
        // The conditional group output is preserved untouched.
        assert_eq!(sheet.conditional_groups.len(), 1);
        // The top-level rule is unconditional; the nested rule carries a gating
        // condition lowered from its `@media` group.
        assert!(sheet.matchable.rules()[0].condition().is_none());
        assert!(sheet.matchable.rules()[1].condition().is_some());
    }

    #[test]
    fn nested_media_rule_gates_by_viewport_width() {
        use obscur_style::matching::MediaEnvironment;
        let sheet = compile("@media (max-width: 1px) { a { color: black } }");
        let rule = &sheet.matchable.rules()[0];
        let condition = rule.condition().expect("nested rule is gated");
        // Fails at a wide viewport, holds at a ≤1px one.
        assert!(!condition.holds(&MediaEnvironment::viewport(1000.0, 1000.0)));
        assert!(condition.holds(&MediaEnvironment::viewport(1.0, 1000.0)));
    }

    #[test]
    fn a_media_feature_we_cannot_evaluate_reaches_the_sheets_reports() {
        use obscur_style::matching::MediaEnvironment;
        let sheet = compile("@media (update: fast) { a { color: black } }");
        // The rule survives compilation but is gated off, so a dark theme does
        // not paint over a light app…
        let condition = sheet.matchable.rules()[0]
            .condition()
            .expect("nested rule is gated");
        assert!(!condition.holds(&MediaEnvironment::viewport(1000.0, 1000.0)));
        // …and the author is told, instead of losing the block in silence.
        let reported: Vec<&str> = sheet
            .unsupported
            .iter()
            .map(UnsupportedCssFeature::name)
            .collect();
        assert_eq!(reported, ["@media (update)"]);
    }

    #[test]
    fn nested_groups_combine_conditions() {
        // A rule inside `@media (max-width: 800px)` inside
        // `@container (min-inline-size: 400px)` must satisfy both axes.
        use obscur_style::matching::MediaEnvironment;
        let sheet = compile(
            "@container (min-inline-size: 400px) { @media (max-width: 800px) { a { color: black } } }",
        );
        let condition = sheet.matchable.rules()[0]
            .condition()
            .expect("doubly-nested rule is gated");
        let env = MediaEnvironment::viewport(500.0, 500.0);
        // Container too small: fails.
        assert!(!condition.holds(&env.with_container(200.0, 200.0)));
        // Container large enough and viewport ≤ 800px: holds.
        assert!(condition.holds(&env.with_container(500.0, 200.0)));
        // Viewport too wide: fails even with a large container.
        assert!(
            !condition
                .holds(&MediaEnvironment::viewport(900.0, 500.0).with_container(500.0, 200.0))
        );
    }

    #[test]
    fn matchable_carries_configured_origin() {
        let src = CssSource::anonymous("a { color: black }");
        let parsed = crate::parser::StylesheetParser::new(&src).parse();
        let sheet = StylesheetCompiler::new()
            .with_origin(CascadeOrigin::Theme)
            .compile(&parsed);
        assert_eq!(sheet.matchable.origin(), CascadeOrigin::Theme);
    }

    #[test]
    fn matchable_lowers_box_and_flex_properties() {
        use obscur_style::property::layout_keyword;
        let sheet = compile(
            ".row { display: flex; flex-direction: column; margin: 8px; gap: 4px; font-size: 14px }",
        );
        let block = sheet.matchable.rules()[0].block();
        // display + flex-direction + 4 margins + 2 gaps + font-size.
        assert_eq!(block.len(), 9);
        for axis in [PropertyId::RowGap, PropertyId::ColumnGap] {
            let declaration = block.iter().find(|value| value.property() == axis).unwrap();
            assert_eq!(
                *declaration.value(),
                PropertyValue::Length(obscur_style::property::Length::px(4.0))
            );
        }
        // `display: flex` lowers to the reserved cross-crate keyword symbol.
        let display = block
            .iter()
            .find(|b| b.property() == PropertyId::Display)
            .unwrap();
        assert_eq!(
            *display.value(),
            PropertyValue::Keyword(layout_keyword::DISPLAY_FLEX)
        );
        // `flex-direction: column` likewise.
        let dir = block
            .iter()
            .find(|b| b.property() == PropertyId::FlexDirection)
            .unwrap();
        assert_eq!(
            *dir.value(),
            PropertyValue::Keyword(layout_keyword::FLEX_COLUMN)
        );
        // The `margin` shorthand expanded to four longhands in the flat cascade.
        let margins = sheet
            .declarations
            .iter()
            .filter(|d| {
                matches!(
                    d.property(),
                    PropertyId::MarginTop
                        | PropertyId::MarginRight
                        | PropertyId::MarginBottom
                        | PropertyId::MarginLeft
                )
            })
            .count();
        assert_eq!(margins, 4);
        assert!(sheet.unsupported.is_empty());
    }

    #[test]
    fn end_to_end_mixed_stylesheet() {
        let sheet = compile(
            "\
            :root { --brand: #ff0000 }\n\
            a { color: var(--brand); width: 50% }\n\
            @keyframes fade { from { opacity: 0 } to { opacity: 1 } }\n\
            button { transition: opacity 0.2s ease-out }\n\
            @media screen { p { color: black } }\n\
            ",
        );
        assert_eq!(sheet.custom_properties.len(), 1);
        assert_eq!(sheet.keyframes.len(), 1);
        assert_eq!(sheet.transitions.len(), 1);
        assert_eq!(sheet.conditional_groups.len(), 1);
        // `a { color: var(--brand); width: 50% }` -> two standard declarations.
        assert_eq!(sheet.declarations.len(), 2);
        let width = sheet
            .declarations
            .iter()
            .find(|d| d.property() == PropertyId::Width)
            .unwrap();
        assert_eq!(*width.value(), PropertyValue::Length(Length::Percent(50.0)));
        // The `--brand` custom property kept the text it was declared with.
        assert_eq!(
            sheet.custom_properties[0].value(),
            &PropertyValue::RawTokens("#ff0000".into())
        );
    }
}
