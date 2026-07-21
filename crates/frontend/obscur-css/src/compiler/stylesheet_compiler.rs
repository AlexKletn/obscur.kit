//! The compiler entry point: a parsed stylesheet into native structures (§4.12).
//!
//! [`StylesheetCompiler`] walks a [`ParsedStylesheet`] and lowers it into a
//! [`CompiledStylesheet`] of `obscur-style` cascade declarations and custom
//! properties, `obscur-motion` keyframe tracks and transition templates, and the
//! conditional (`@media` / `@container`) groups that gate them — carrying
//! forward the parser's own errors and unsupported-feature diagnostics (§4.13).
//!
//! Because `obscur_identity::InternedSymbol` is an opaque handle with no shared
//! global interner, the compiler owns its own [`CssSymbolInterner`]: every
//! keyword, custom-property name and animation name is interned here, and the
//! resulting table travels inside the [`CompiledStylesheet`] so consumers can
//! resolve a symbol back to its text.

use std::collections::HashMap;

use crate::error::{CssParseError, UnsupportedCssFeature};
use crate::parser::{
    ParsedContainerQuery, ParsedMediaQueryList, ParsedRule, ParsedStyleRule, ParsedStylesheet,
};

use obscur_identity::InternedSymbol;
use obscur_style::cascade::{CascadeOrigin, Declaration};
use obscur_style::matching::{BlockDeclaration, RuleCondition, StyleRule, Stylesheet};
use obscur_style::property::CustomProperty;
use obscur_style::selector::{SelectorList, SelectorSpecificity};

use super::animation_compiler::{AnimationCompiler, AnimationRef};
use super::condition_lowering::lower_condition;
use super::keyframe_compiler::{CompiledKeyframes, KeyframeCompiler};
use super::motion_rule::{CompiledAnimation, CompiledMotionRule};
use super::property_compiler::{
    CompiledDeclaration, PropertyCompiler, keyword_value_is_foreign, property_id_from_name,
};
use super::selector_compiler::SelectorCompiler;
use super::transition_compiler::{CompiledTransition, TransitionCompiler};

/// A string interner mapping CSS text to opaque [`InternedSymbol`] handles.
///
/// The kit exposes no shared global interner, so the CSS compiler owns this one.
/// Symbols are minted with sequential raw ids and deduplicated, so equal strings
/// always intern to the same symbol. The reverse table lets consumers
/// [`resolve`](CssSymbolInterner::resolve) a symbol back to its text.
#[derive(Debug, Clone, Default)]
pub struct CssSymbolInterner {
    lookup: HashMap<Box<str>, InternedSymbol>,
    reverse: Vec<Box<str>>,
}

impl CssSymbolInterner {
    /// Creates an empty interner.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Interns `text`, returning its symbol; equal strings share a symbol.
    pub fn intern(&mut self, text: &str) -> InternedSymbol {
        if let Some(&symbol) = self.lookup.get(text) {
            return symbol;
        }
        let raw = u32::try_from(self.reverse.len()).unwrap_or(u32::MAX);
        let symbol = InternedSymbol::from_raw(raw);
        let boxed: Box<str> = text.into();
        self.reverse.push(boxed.clone());
        self.lookup.insert(boxed, symbol);
        symbol
    }

    /// Resolves a previously interned symbol back to its text, or [`None`] if the
    /// symbol was not minted by this interner.
    #[must_use]
    pub fn resolve(&self, symbol: InternedSymbol) -> Option<&str> {
        usize::try_from(symbol.raw())
            .ok()
            .and_then(|index| self.reverse.get(index))
            .map(|boxed| &**boxed)
    }
}

/// The condition gating a [`CompiledConditionalGroup`].
///
/// The parser's normalized query is carried verbatim: evaluating it against a
/// live environment is a later (runtime) concern.
#[derive(Debug, Clone, PartialEq)]
pub enum CompiledCondition {
    /// An `@media` condition.
    Media(ParsedMediaQueryList),
    /// An `@container` condition.
    Container(ParsedContainerQuery),
}

/// A conditional group: declarations gated behind an `@media`/`@container`
/// condition (§4.15/§4.16).
///
/// The declarations inside the block are fully compiled; the group simply pairs
/// them with the condition under which they apply.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledConditionalGroup {
    /// The condition under which the grouped declarations apply.
    pub condition: CompiledCondition,
    /// The standard declarations inside the conditional block.
    pub declarations: Vec<Declaration>,
    /// The custom properties inside the conditional block.
    pub custom_properties: Vec<CustomProperty>,
    /// The transition templates inside the conditional block.
    pub transitions: Vec<CompiledTransition>,
}

/// The fully compiled stylesheet: the native structures the rest of the kit
/// consumes (spec: "Выход — структуры obscur-style и obscur-motion").
#[derive(Debug, Clone)]
pub struct CompiledStylesheet {
    /// Top-level standard cascade declarations, in source order.
    pub declarations: Vec<Declaration>,
    /// Top-level custom-property declarations.
    pub custom_properties: Vec<CustomProperty>,
    /// Conditional (`@media` / `@container`) groups.
    pub conditional_groups: Vec<CompiledConditionalGroup>,
    /// Compiled `@keyframes` rules.
    pub keyframes: Vec<CompiledKeyframes>,
    /// Top-level `transition` templates.
    pub transitions: Vec<CompiledTransition>,
    /// Selector-associated motion: per style rule, the `transition` templates
    /// and resolved `@keyframes` animations that rule declares, paired with the
    /// selector list the runtime matches against the retained tree (§4.17).
    ///
    /// This is the wiring input the runtime consumes to drive CSS transitions and
    /// animations: unlike the flat [`transitions`](Self::transitions) /
    /// [`keyframes`](Self::keyframes) outputs, each entry keeps the association
    /// between a selector and the motion it declares, so a transition fires only
    /// on the nodes its rule matched and a `@keyframes` plays only on elements
    /// that reference it via `animation`/`animation-name`.
    pub motion_rules: Vec<CompiledMotionRule>,
    /// The interner backing every [`InternedSymbol`] in this stylesheet.
    pub symbols: CssSymbolInterner,
    /// The runtime-matchable stylesheet: every style rule lowered into the
    /// `obscur-style` selector + [`BlockDeclaration`] model the
    /// [`StyleMatcher`](obscur_style::matching::StyleMatcher) consumes (§4.13).
    ///
    /// This is an *additional* output alongside [`declarations`](Self::declarations)
    /// (which pre-resolve the cascade context per selector): here each rule keeps
    /// its authored selector list and block, and the matcher supplies specificity,
    /// origin and source order at match time. Only standard longhand declarations
    /// enter a block; custom properties and unsupported declarations are recorded
    /// on the other fields, not here. Style rules nested inside `@media` /
    /// `@container` groups are lowered with their group's condition **attached**
    /// as an `obscur-style`
    /// [`RuleCondition`](obscur_style::matching::RuleCondition), so the matcher
    /// only lets them contribute when that condition holds against the runtime
    /// environment (§4.13); the condition is *also* modelled, alongside the
    /// group's flat declarations, in [`conditional_groups`](Self::conditional_groups).
    pub matchable: Stylesheet,
    /// Recognised-but-unsupported features (from parsing and compiling).
    pub unsupported: Vec<UnsupportedCssFeature>,
    /// Syntax errors carried forward from parsing.
    pub errors: Vec<CssParseError>,
}

/// The staging buffer a rule list compiles into before it is lifted into the
/// [`CompiledStylesheet`] (or wrapped in a [`CompiledConditionalGroup`]).
#[derive(Default)]
struct Accumulator {
    declarations: Vec<Declaration>,
    custom_properties: Vec<CustomProperty>,
    conditional_groups: Vec<CompiledConditionalGroup>,
    keyframes: Vec<CompiledKeyframes>,
    transitions: Vec<CompiledTransition>,
    matchable_rules: Vec<StyleRule>,
    motion_rules: Vec<PendingMotionRule>,
    unsupported: Vec<UnsupportedCssFeature>,
}

/// A style rule's motion before `@keyframes` names are resolved to tracks: the
/// rule's selector list, its transition templates, and its (still-by-name)
/// animation references. Resolved into a [`CompiledMotionRule`] in
/// [`StylesheetCompiler::compile`], where every `@keyframes` rule is known.
struct PendingMotionRule {
    selectors: SelectorList,
    transitions: Vec<CompiledTransition>,
    animations: Vec<AnimationRef>,
}

/// Compiles a [`ParsedStylesheet`] into a [`CompiledStylesheet`] (§4.12).
///
/// The compiler owns the string interner, the selector-id counter and the
/// document-order counter used to rank declarations (§4.14). Construct one, set
/// the [`CascadeOrigin`] with [`with_origin`](StylesheetCompiler::with_origin)
/// if the default (`GlobalApp`) is not right, then
/// [`compile`](StylesheetCompiler::compile).
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
        let mut acc = Accumulator::default();
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
    fn compile_rules(&mut self, rules: &[ParsedRule], acc: &mut Accumulator) {
        for rule in rules {
            match rule {
                ParsedRule::Style(style) => self.compile_style_rule(style, acc),
                ParsedRule::Media(media) => {
                    let mut inner = Accumulator::default();
                    self.compile_rules(&media.rules, &mut inner);
                    lift_group(CompiledCondition::Media(media.query.clone()), inner, acc);
                }
                ParsedRule::Container(container) => {
                    let mut inner = Accumulator::default();
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
    fn compile_style_rule(&mut self, style: &ParsedStyleRule, acc: &mut Accumulator) {
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
        for decl in &style.declarations {
            if is_transition_shorthand(&decl.name) {
                let mut compiled = self.transitions.compile(decl, &mut acc.unsupported);
                rule_transitions.append(&mut compiled);
            }
        }
        acc.transitions.extend(rule_transitions.iter().copied());
        let animations = self
            .animations
            .compile(&style.declarations, &mut self.interner);
        if !rule_transitions.is_empty() || !animations.is_empty() {
            acc.motion_rules.push(PendingMotionRule {
                selectors: SelectorCompiler::lower_list(&style.selectors),
                transitions: rule_transitions,
                animations,
            });
        }

        let rule = self.lower_matchable_rule(style);
        acc.matchable_rules.push(rule);
    }

    /// Builds the runtime-matchable [`StyleRule`] for a parsed style rule: the
    /// lowered selector list paired with a block of authored [`BlockDeclaration`]s.
    ///
    /// The block is built once per rule (it is selector-independent). Motion
    /// declarations are routed away as they are elsewhere, and only standard
    /// longhands become block declarations; custom properties and unsupported
    /// values are handled by the flat cascade path (their diagnostics are raised
    /// there, so they are silently skipped here to avoid double reporting).
    fn lower_matchable_rule(&mut self, style: &ParsedStyleRule) -> StyleRule {
        let mut block = Vec::new();
        // The longhands this rule authored whose value could not be parsed: they
        // never enter the block, so a matched element loses only that one
        // declaration (§4.13). Recorded on the rule so the runtime can raise a
        // per-node diagnostic attributed to the element that matched it.
        let mut invalid_values = Vec::new();
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
                if let CompiledDeclaration::Standard(declaration) = compiled {
                    block.push(BlockDeclaration::new(
                        declaration.property(),
                        declaration.value().clone(),
                        declaration.important(),
                    ));
                }
            }
            // A recognised longhand handed a bare keyword it cannot use had an
            // unparseable value: the lenient keyword path lowered it to an inert
            // interned keyword the property ignores, so it is dropped in effect —
            // an "invalid value" fault (§4.13) recorded per rule. Unknown property
            // names and custom properties are a different class (and both fail
            // `property_id_from_name`, so they are excluded here).
            if let Some(property) = property_id_from_name(&decl.name)
                && keyword_value_is_foreign(property, &decl.value)
            {
                invalid_values.push(property);
            }
        }
        StyleRule::new(SelectorCompiler::lower_list(&style.selectors), block)
            .with_invalid_values(invalid_values)
    }
}

impl Default for StylesheetCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Lifts an inner accumulator into `acc`: nested keyframes, conditional groups
/// and diagnostics rise to the enclosing scope, while the flat declarations and
/// transitions are wrapped in a [`CompiledConditionalGroup`] under `condition`.
///
/// The nested style rules also rise into the enclosing matchable set, but each
/// is now **gated** by `condition`: its `condition` is lowered into an
/// `obscur-style` [`RuleCondition`](obscur_style::matching::RuleCondition) and
/// attached to the rule, so the runtime matcher only lets it contribute when the
/// condition holds against the live environment (§4.13). A rule that already
/// carries a condition (it came from a group nested inside this one) is gated by
/// the conjunction of the outer and inner conditions.
fn lift_group(condition: CompiledCondition, mut inner: Accumulator, acc: &mut Accumulator) {
    acc.keyframes.append(&mut inner.keyframes);
    acc.conditional_groups.append(&mut inner.conditional_groups);
    // Motion rules nested inside a conditional group rise into the enclosing
    // scope. Their group condition is not yet propagated to the runtime motion
    // driver (a documented limitation): a transition/animation authored inside
    // `@media`/`@container` currently drives regardless of the environment.
    acc.motion_rules.append(&mut inner.motion_rules);
    acc.unsupported.append(&mut inner.unsupported);
    let lowered = lower_condition(&condition);
    for rule in inner.matchable_rules {
        let combined = match rule.condition() {
            Some(existing) => RuleCondition::All(vec![lowered.clone(), existing.clone()]),
            None => lowered.clone(),
        };
        acc.matchable_rules.push(rule.with_condition(combined));
    }
    acc.conditional_groups.push(CompiledConditionalGroup {
        condition,
        declarations: inner.declarations,
        custom_properties: inner.custom_properties,
        transitions: inner.transitions,
    });
}

/// Resolves a [`PendingMotionRule`]'s `animation` references against the
/// stylesheet's compiled `@keyframes`, producing a [`CompiledMotionRule`], or
/// [`None`] when the rule ends up carrying no drivable motion.
///
/// Each [`AnimationRef`] is matched by interned name to a [`CompiledKeyframes`]
/// (the names share one interner, so equal names intern equal); an unknown name
/// resolves to nothing. A rule with neither transitions nor a resolvable
/// animation is dropped.
fn resolve_motion_rule(
    pending: PendingMotionRule,
    keyframes: &[CompiledKeyframes],
) -> Option<CompiledMotionRule> {
    let animations: Vec<CompiledAnimation> = pending
        .animations
        .iter()
        .filter_map(|reference| {
            keyframes
                .iter()
                .find(|compiled| compiled.name == reference.name)
                .map(|compiled| CompiledAnimation {
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
    if pending.transitions.is_empty() && animations.is_empty() {
        return None;
    }
    Some(CompiledMotionRule {
        selectors: pending.selectors,
        transitions: pending.transitions,
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
    use crate::error::UnsupportedFeatureKind;
    use crate::source::CssSource;
    use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor};
    use obscur_style::selector::SelectorSpecificity;

    fn compile(text: &str) -> CompiledStylesheet {
        let src = CssSource::anonymous(text);
        let sheet = crate::parser::StylesheetParser::new(&src).parse();
        StylesheetCompiler::new().compile(&sheet)
    }

    #[test]
    fn interner_dedups_and_round_trips() {
        let mut interner = CssSymbolInterner::new();
        let a = interner.intern("flex");
        let b = interner.intern("flex");
        let c = interner.intern("block");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(interner.resolve(a), Some("flex"));
        assert_eq!(interner.resolve(c), Some("block"));
        assert_eq!(interner.resolve(InternedSymbol::from_raw(99)), None);
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
        // The whole point of the diagnostic: `.badge::before` compiles to a rule
        // that styles nothing (no generated box exists), so the host is told
        // rather than left to wonder why the declaration had no effect.
        let sheet = compile(".badge::before { color: red }");
        let reported: Vec<&str> = sheet
            .unsupported
            .iter()
            .filter(|f| f.kind() == UnsupportedFeatureKind::PseudoElement)
            .map(UnsupportedCssFeature::name)
            .collect();
        assert_eq!(reported, ["::before"]);
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
        // Selector `.counter` lowers to a single class simple selector.
        let selectors = first.selectors().selectors();
        assert_eq!(selectors.len(), 1);
        assert_eq!(
            selectors[0].subject().simples(),
            &[SimpleSelector::Class("counter".into())]
        );
        // Its block is one `background-color: #ff0000` declaration.
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
        // display + flex-direction + 4 margin longhands + gap + font-size = 8.
        assert_eq!(block.len(), 8);
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
        // The `--brand` custom property lowered to a red colour.
        assert_eq!(
            sheet.custom_properties[0].value(),
            &PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
        );
    }
}
