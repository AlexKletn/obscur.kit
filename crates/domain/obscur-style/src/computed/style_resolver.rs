//! Turns cascaded values into computed values: [`StyleResolver`] (§4.13–§4.14).
//!
//! The resolver implements the computed-value algorithm for a single node:
//!
//! 1. **Effective custom properties** — custom properties always inherit, so the
//!    node's effective set is the parent's set overridden by the node's own
//!    cascaded custom properties. This set is resolved *before* any computed
//!    value, and is stored on the resulting [`ComputedStyle`] so descendants
//!    inherit it (§4.14).
//! 2. **`var()` substitution** — a [`PropertyValue::CustomRef`] is replaced by the
//!    referenced effective custom property, following chains with a visited-set
//!    guard so a cycle terminates. An unresolved reference (missing name or a
//!    cycle) resolves as if the declaration were [`PropertyValue::Unset`].
//! 3. **Per-property computed value** — the (substituted) cascaded value is
//!    resolved against the CSS-wide keywords and default inheritance to produce a
//!    concrete [`ComputedValue`].
//!
//! # Deliberate limitations
//!
//! * **Keyword initial values are deferred.** The initial values of keyword-
//!   valued properties (`display: inline`, `visibility: visible`) are interned
//!   symbols, and this crate has no interner in Phase 1. Rather than fake one,
//!   [`StyleResolver::initial_value`] returns [`None`] for those properties, and a
//!   property that would need its (unavailable) initial value is simply left
//!   unresolved on the [`ComputedStyle`]. A later phase supplies the interner and
//!   fills these in.
//! * **Single starter property set.** The resolver computes only the Phase-1
//!   longhands listed in [`StyleResolver::computed_properties`]; the set grows as
//!   more longhands are modelled.

use crate::cascade::ResolvedDeclarations;
use crate::computed::{ComputedStyle, ComputedValue};
use crate::property::{InheritedProperty, Length, PropertyId, PropertyValue, StyleColor};
use obscur_identity::InternedSymbol;
use std::collections::{HashMap, HashSet};

/// The Phase-1 longhands the resolver computes.
///
/// This is a local `const` list because the authoritative [`PropertyId`] enum
/// lives in another module that this sub-step must not edit.
const COMPUTED_PROPERTIES: &[PropertyId] = &[
    PropertyId::Color,
    PropertyId::BackgroundColor,
    PropertyId::Opacity,
    PropertyId::Width,
    PropertyId::Height,
    PropertyId::Display,
    PropertyId::Visibility,
];

/// Stateless resolver that computes a node's [`ComputedStyle`] from its cascaded
/// values, its cascaded custom properties, and its parent's computed style.
///
/// It carries no state; construct one with [`StyleResolver::new`] (or
/// [`Default`]) and call [`StyleResolver::resolve`]. Resolution is deterministic,
/// panic-free, and cycle-safe on `var()`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StyleResolver;

impl StyleResolver {
    /// Creates a resolver.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns the set of properties this resolver computes.
    #[must_use]
    pub const fn computed_properties() -> &'static [PropertyId] {
        COMPUTED_PROPERTIES
    }

    /// Returns the initial value of `property`, if it can be expressed without an
    /// interner.
    ///
    /// Value-typed properties have concrete initials: `color` is opaque black,
    /// `background-color` is transparent, `opacity` is `1.0`, and `width` /
    /// `height` are `auto`. Keyword-valued properties (`display`, `visibility`)
    /// return [`None`]: their initial keyword is an interned symbol that Phase 1
    /// cannot mint, so the property is left unresolved instead (see the module
    /// docs).
    // Several arms share a body (`Some(number(1.0))`, `None`) but are grouped by
    // CSS meaning with distinct explanatory comments; merging them would obscure
    // why each property is unresolved or shares an initial.
    #[allow(clippy::match_same_arms)]
    #[must_use]
    pub fn initial_value(self, property: PropertyId) -> Option<ComputedValue> {
        match property {
            PropertyId::Color => Some(ComputedValue::color(StyleColor::BLACK)),
            PropertyId::BackgroundColor => Some(ComputedValue::color(StyleColor::TRANSPARENT)),
            PropertyId::Opacity => Some(ComputedValue::number(1.0)),
            PropertyId::Width | PropertyId::Height => Some(ComputedValue::length(Length::Auto)),
            // Box-model spacing: initial `0`.
            PropertyId::MarginTop
            | PropertyId::MarginRight
            | PropertyId::MarginBottom
            | PropertyId::MarginLeft
            | PropertyId::PaddingTop
            | PropertyId::PaddingRight
            | PropertyId::PaddingBottom
            | PropertyId::PaddingLeft
            | PropertyId::Gap => Some(ComputedValue::length(Length::px(0.0))),
            // Sizing minimums and `flex-basis`: initial `auto`.
            PropertyId::MinWidth | PropertyId::MinHeight | PropertyId::FlexBasis => {
                Some(ComputedValue::length(Length::Auto))
            }
            // `max-width` / `max-height` initial is `none`, which `Length` cannot
            // yet represent; left unresolved until a `none` length lands.
            PropertyId::MaxWidth | PropertyId::MaxHeight => None,
            // Flex factors: `flex-grow` initial `0`, `flex-shrink` initial `1`.
            PropertyId::FlexGrow => Some(ComputedValue::number(0.0)),
            PropertyId::FlexShrink => Some(ComputedValue::number(1.0)),
            // `font-size` initial is `medium` (~16px logical).
            PropertyId::FontSize => Some(ComputedValue::length(Length::px(16.0))),
            // `font-family` initial is UA-dependent; left unresolved so an absent
            // authored value falls back to the runtime's default font chain.
            PropertyId::FontFamily => None,
            // Keyword initials need the interner; deferred to a later phase.
            PropertyId::Display
            | PropertyId::Visibility
            | PropertyId::FlexDirection
            | PropertyId::JustifyContent
            | PropertyId::AlignItems
            // `direction` (`ltr`) / `font-style` (`normal`) keyword initials are
            // likewise deferred, and `font-weight`'s initial (`normal` = 400) is
            // left unresolved so an absent value falls back to the shaping default
            // (also 400) — byte-for-byte identical to today's behaviour.
            | PropertyId::Direction
            | PropertyId::FontWeight
            | PropertyId::FontStyle
            // `text-align` initial is `start`; its keyword initial likewise needs
            // the interner, so an absent value falls back to the runtime's default
            // (`Start`) — byte-for-byte identical to today's behaviour.
            | PropertyId::TextAlign
            // `text-overflow` initial is `clip`; its keyword initial likewise needs
            // the interner, so an absent value falls back to the runtime's default
            // (`Clip`) — byte-for-byte identical to today's behaviour.
            | PropertyId::TextOverflow => None,
        }
    }

    /// Computes the node's [`ComputedStyle`] from its cascaded declarations, its
    /// cascaded custom properties, and its parent's computed style (§4.14).
    ///
    /// `parent` is [`None`] for the root; every inherited property then falls
    /// back to its initial value. The returned style stores the node's effective
    /// (inheritable) custom properties for its descendants.
    #[must_use]
    pub fn resolve(
        self,
        cascaded: &ResolvedDeclarations,
        custom: &HashMap<InternedSymbol, PropertyValue>,
        parent: Option<&ComputedStyle>,
    ) -> ComputedStyle {
        let mut style = ComputedStyle::new();

        // Step 1: effective custom properties = parent's (inherited), overridden
        // by this node's cascaded custom properties. Resolved before any computed
        // value (§4.14).
        let mut effective: HashMap<InternedSymbol, PropertyValue> = HashMap::new();
        if let Some(parent) = parent {
            for (name, value) in parent.custom_properties() {
                effective.insert(*name, value.clone());
            }
        }
        for (name, value) in custom {
            effective.insert(*name, value.clone());
        }
        for (name, value) in &effective {
            style.set_custom_property(*name, value.clone());
        }

        // Steps 2–3: compute each property.
        for &property in COMPUTED_PROPERTIES {
            if let Some(value) = self.resolve_property(property, cascaded, &effective, parent) {
                style.set(property, value);
            }
        }

        style
    }

    /// Resolves one property to a computed value, or [`None`] if it is left
    /// unresolved (deferred keyword initial).
    fn resolve_property(
        self,
        property: PropertyId,
        cascaded: &ResolvedDeclarations,
        custom: &HashMap<InternedSymbol, PropertyValue>,
        parent: Option<&ComputedStyle>,
    ) -> Option<ComputedValue> {
        match cascaded.get(property) {
            Some(raw) => {
                // Step 2: substitute var() (unresolved -> Unset).
                let value = Self::substitute(raw, custom);
                self.resolve_specified(property, value, parent)
            }
            // No cascaded value: default inheritance.
            None => {
                if InheritedProperty::of(property).is_inherited() {
                    self.inherited_from(property, parent)
                } else {
                    self.initial_value(property)
                }
            }
        }
    }

    /// Resolves a substituted specified value (never a [`PropertyValue::CustomRef`])
    /// against the CSS-wide keywords.
    fn resolve_specified(
        self,
        property: PropertyId,
        value: PropertyValue,
        parent: Option<&ComputedStyle>,
    ) -> Option<ComputedValue> {
        match value {
            PropertyValue::Inherit => self.inherited_from(property, parent),
            PropertyValue::Initial => self.initial_value(property),
            PropertyValue::Unset => {
                if InheritedProperty::of(property).is_inherited() {
                    self.inherited_from(property, parent)
                } else {
                    self.initial_value(property)
                }
            }
            // Concrete leaf (var() substitution already eliminated CustomRef).
            concrete => ComputedValue::from_concrete(concrete),
        }
    }

    /// Returns the inherited computed value for `property`: the parent's computed
    /// value, or the property's initial value when there is no parent.
    ///
    /// When a parent exists but did not compute `property` (a deferred keyword
    /// initial), the result is [`None`] and the property is left unresolved.
    fn inherited_from(
        self,
        property: PropertyId,
        parent: Option<&ComputedStyle>,
    ) -> Option<ComputedValue> {
        match parent {
            Some(parent) => parent.get(property).cloned(),
            None => self.initial_value(property),
        }
    }

    /// Substitutes `var()` references, following chains with a visited guard so a
    /// cycle terminates.
    ///
    /// Returns the referenced concrete value, or [`PropertyValue::Unset`] when the
    /// reference is unresolvable — the name is missing, or a cycle was detected.
    /// A non-reference value is returned unchanged.
    fn substitute(
        value: &PropertyValue,
        custom: &HashMap<InternedSymbol, PropertyValue>,
    ) -> PropertyValue {
        let mut current = value.clone();
        let mut visited: HashSet<InternedSymbol> = HashSet::new();
        loop {
            match current {
                PropertyValue::CustomRef(name) => {
                    if !visited.insert(name) {
                        // Cycle: treat as unset.
                        return PropertyValue::Unset;
                    }
                    match custom.get(&name) {
                        Some(next) => current = next.clone(),
                        // Missing custom property: treat as unset.
                        None => return PropertyValue::Unset,
                    }
                }
                other => return other,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sym(raw: u32) -> InternedSymbol {
        InternedSymbol::from_raw(raw)
    }

    #[test]
    fn inherit_takes_parent_computed_value() {
        let resolver = StyleResolver::new();
        let mut parent = ComputedStyle::new();
        parent.set(
            PropertyId::Color,
            ComputedValue::color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
        );

        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Color, PropertyValue::Inherit);

        let computed = resolver.resolve(&cascaded, &HashMap::new(), Some(&parent));
        assert_eq!(
            computed
                .get(PropertyId::Color)
                .and_then(ComputedValue::as_color),
            Some(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
        );
    }

    #[test]
    fn inherit_without_parent_uses_initial() {
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Color, PropertyValue::Inherit);

        let computed = resolver.resolve(&cascaded, &HashMap::new(), None);
        assert_eq!(
            computed
                .get(PropertyId::Color)
                .and_then(ComputedValue::as_color),
            Some(StyleColor::BLACK)
        );
    }

    #[test]
    fn unset_on_inherited_property_inherits() {
        let resolver = StyleResolver::new();
        let mut parent = ComputedStyle::new();
        parent.set(
            PropertyId::Color,
            ComputedValue::color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)),
        );

        // `color` inherits by default, so `unset` inherits the parent's value.
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Color, PropertyValue::Unset);

        let computed = resolver.resolve(&cascaded, &HashMap::new(), Some(&parent));
        assert_eq!(
            computed
                .get(PropertyId::Color)
                .and_then(ComputedValue::as_color),
            Some(StyleColor::rgba(0.0, 1.0, 0.0, 1.0))
        );
    }

    #[test]
    fn unset_on_non_inherited_property_uses_initial() {
        let resolver = StyleResolver::new();
        let mut parent = ComputedStyle::new();
        parent.set(PropertyId::Opacity, ComputedValue::number(0.2));

        // `opacity` does not inherit, so `unset` resets to its initial (1.0).
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Opacity, PropertyValue::Unset);

        let computed = resolver.resolve(&cascaded, &HashMap::new(), Some(&parent));
        assert_eq!(computed.opacity(), Some(1.0));
    }

    #[test]
    fn var_substitution_resolves_reference() {
        let resolver = StyleResolver::new();
        let name = sym(1);
        let mut custom = HashMap::new();
        custom.insert(name, PropertyValue::Number(0.5));

        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Opacity, PropertyValue::CustomRef(name));

        let computed = resolver.resolve(&cascaded, &custom, None);
        assert_eq!(computed.opacity(), Some(0.5));
    }

    #[test]
    fn var_substitution_follows_chain() {
        let resolver = StyleResolver::new();
        let (a, b) = (sym(1), sym(2));
        let mut custom = HashMap::new();
        custom.insert(a, PropertyValue::CustomRef(b));
        custom.insert(b, PropertyValue::Number(0.75));

        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Opacity, PropertyValue::CustomRef(a));

        let computed = resolver.resolve(&cascaded, &custom, None);
        assert_eq!(computed.opacity(), Some(0.75));
    }

    #[test]
    fn unresolved_var_behaves_as_unset() {
        let resolver = StyleResolver::new();
        // Reference to a missing custom property -> unset -> initial for opacity.
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Opacity, PropertyValue::CustomRef(sym(99)));

        let computed = resolver.resolve(&cascaded, &HashMap::new(), None);
        assert_eq!(computed.opacity(), Some(1.0));
    }

    #[test]
    fn cyclic_var_terminates_and_behaves_as_unset() {
        let resolver = StyleResolver::new();
        let (a, b) = (sym(1), sym(2));
        let mut custom = HashMap::new();
        custom.insert(a, PropertyValue::CustomRef(b));
        custom.insert(b, PropertyValue::CustomRef(a));

        // `color` inherits, so cyclic -> unset -> inherit; with no parent -> initial.
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Color, PropertyValue::CustomRef(a));

        let computed = resolver.resolve(&cascaded, &custom, None);
        assert_eq!(
            computed
                .get(PropertyId::Color)
                .and_then(ComputedValue::as_color),
            Some(StyleColor::BLACK)
        );
    }

    #[test]
    fn default_inheritance_of_unspecified_property() {
        let resolver = StyleResolver::new();
        let mut parent = ComputedStyle::new();
        parent.set(
            PropertyId::Color,
            ComputedValue::color(StyleColor::rgba(0.1, 0.2, 0.3, 1.0)),
        );
        parent.set(PropertyId::Opacity, ComputedValue::number(0.4));

        // Nothing cascaded on the child.
        let cascaded = ResolvedDeclarations::new();
        let computed = resolver.resolve(&cascaded, &HashMap::new(), Some(&parent));

        // `color` inherits by default -> parent's value.
        assert_eq!(
            computed
                .get(PropertyId::Color)
                .and_then(ComputedValue::as_color),
            Some(StyleColor::rgba(0.1, 0.2, 0.3, 1.0))
        );
        // `opacity` does not inherit -> its initial value (1.0), not parent's 0.4.
        assert_eq!(computed.opacity(), Some(1.0));
    }

    #[test]
    fn effective_custom_properties_inherit_and_override() {
        let resolver = StyleResolver::new();
        let (a, b) = (sym(1), sym(2));
        let mut parent = ComputedStyle::new();
        parent.set_custom_property(a, PropertyValue::Number(1.0));
        parent.set_custom_property(b, PropertyValue::Number(2.0));

        // Child overrides `a`, leaves `b`.
        let mut custom = HashMap::new();
        custom.insert(a, PropertyValue::Number(9.0));

        let computed = resolver.resolve(&ResolvedDeclarations::new(), &custom, Some(&parent));
        assert_eq!(
            computed.custom_property(a),
            Some(&PropertyValue::Number(9.0))
        );
        assert_eq!(
            computed.custom_property(b),
            Some(&PropertyValue::Number(2.0))
        );
    }

    #[test]
    fn keyword_initial_is_left_unresolved() {
        let resolver = StyleResolver::new();
        assert_eq!(resolver.initial_value(PropertyId::Display), None);
        // With no cascaded value and no parent, `display` cannot be computed yet.
        let computed = resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), None);
        assert_eq!(computed.get(PropertyId::Display), None);
    }
}
