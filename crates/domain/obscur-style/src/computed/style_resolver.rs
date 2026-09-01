#![doc = include_str!("style_resolver.doc/_module.md")]

use std::rc::Rc;

use crate::cascade::ResolvedDeclarations;
use crate::computed::absolutize::{absolute_font_size, absolutize_value};
use crate::computed::computed_property_set::{COMPUTED_PROPERTIES, FACE_PROPERTIES};
use crate::computed::effective_custom_properties::{effective_custom_properties, substitute};
use crate::computed::initial_value::{INITIAL_FONT_SIZE, initial_value_of};
use crate::computed::logical_cascade::{cascaded_for, resolves_rtl, resolves_writing_mode};
use crate::computed::relative_font_weight::resolve_relative_weight;
use crate::computed::substituted_value::substituted_value;
use crate::computed::{ComputedStyle, ComputedValue, CustomValueParser, ResolutionContext};
use crate::property::{
    InheritedProperty, Length, LengthContext, PropertyId, PropertyValue, WritingMode,
};
use obscur_identity::{FastMap, InternedSymbol};

#[doc = include_str!("style_resolver.doc/StyleResolver.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StyleResolver;

impl StyleResolver {
    #[doc = include_str!("style_resolver.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("style_resolver.doc/computed_properties.md")]
    #[must_use]
    pub const fn computed_properties() -> &'static [PropertyId] {
        COMPUTED_PROPERTIES
    }

    #[doc = include_str!("style_resolver.doc/initial_value.md")]
    #[must_use]
    pub fn initial_value(self, property: PropertyId) -> Option<ComputedValue> {
        initial_value_of(property)
    }

    #[doc = include_str!("style_resolver.doc/font_size_of.md")]
    #[must_use]
    pub fn font_size_of(style: &ComputedStyle) -> f32 {
        style
            .get(PropertyId::FontSize)
            .and_then(ComputedValue::as_length)
            .and_then(Length::as_px)
            .map_or(INITIAL_FONT_SIZE, |px| px.0)
    }

    #[doc = include_str!("style_resolver.doc/resolve.md")]
    #[must_use]
    pub fn resolve(
        self,
        cascaded: &ResolvedDeclarations,
        custom: &FastMap<InternedSymbol, PropertyValue>,
        parent: Option<&ComputedStyle>,
        ctx: ResolutionContext,
    ) -> ComputedStyle {
        // Sized up front: this is the one caller that fills a style right up
        // to the size of the property set, and growing into it rehashes six
        // times per node.
        let mut style = ComputedStyle::with_capacity(COMPUTED_PROPERTIES.len());

        // Step 1: effective custom properties = parent's (inherited), overridden
        // by this node's cascaded custom properties. Resolved before any computed
        // value (§4.14).
        // A node that declares none of its own inherits the parent's set whole,
        // and shares it rather than copying it. That is the common case in any
        // themed application: variables are declared once at the root and inherit
        // everywhere, and this used to cost one clone of every value into an
        // intermediate map plus a second clone out of it into this node's style —
        // for values identical to the parent's, on every node, on every restyle.
        let effective: Rc<FastMap<InternedSymbol, PropertyValue>> = if custom.is_empty() {
            parent
                .map(ComputedStyle::shared_custom_properties)
                .unwrap_or_default()
        } else {
            Rc::new(effective_custom_properties(custom, parent))
        };
        style.set_shared_custom_properties(Rc::clone(&effective));

        // Step 1a: `direction` before anything else. A logical property maps onto
        // a physical one *through* the computed direction (spec §4.13), so it has
        // to be known before any box-model property is resolved — resolving it as
        // part of the loop below would make the answer depend on iteration order.
        let rtl = resolves_rtl(cascaded, parent);

        // Step 1a′: `writing-mode`, for the same reason as `direction` — it is the
        // second input the logical→physical box-model mapping consults (the block
        // axis rotates around it, css-writing-modes-4 §6.2), so it must be known
        // before any box-model property is resolved. It inherits, and defaults to
        // `horizontal-tb`, under which every mapping is byte-for-byte unchanged.
        let wm = resolves_writing_mode(cascaded, parent);

        // Step 1b: the face-selecting `font-*` properties, so the `ch` measurement
        // in step 1d has a face to select. None of them can carry a `<length>` (a
        // family list, a number, a keyword — all three land in `absolutize_value`'s
        // identity arm), which is exactly why they can be resolved before a length
        // context exists.
        //
        // These are resolved here IN ADDITION to the loop below, not instead of it:
        // `resolve_property` is deterministic, so the loop recomputes the identical
        // value and overwrites this one with it. That is deliberate. Skipping them
        // in the loop would save three resolutions per node and buy a trap — the
        // loop's fold is what `absolutize_value`'s exhaustive match guards, and a
        // property that silently bypasses it is the exact defect that match exists
        // to prevent. The early set feeds the port; the loop remains the authority.
        for &property in FACE_PROPERTIES {
            if let Some(value) = self.resolve_property(
                property,
                cascaded,
                &effective,
                parent,
                rtl,
                wm,
                ctx.custom_values,
            ) {
                style.set(property, value);
            }
        }

        // Step 1c: `font-size` before every remaining property: it is the base
        // every `em` on this node resolves against, so folding it as part of the
        // loop below would make the answer depend on iteration order (§4.14). Its
        // own relative units resolve against the PARENT's size — including `ch`,
        // which is why the parent's metrics are measured here and not the node's.
        let font_size = self.resolve_font_size(cascaded, &effective, parent, rtl, wm, ctx);
        style.set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(font_size)),
        );

        let lengths = LengthContext::new(
            font_size,
            ctx.root_font_size,
            ctx.viewport_width,
            ctx.viewport_height,
        )
        .with_environment(ctx.environment);

        // Steps 2–4: compute each property, then absolutize the lengths it
        // carries against the context `font-size` just completed.
        //
        // A value that contains a `ch` is set aside instead: it cannot be folded
        // until the `0` glyph is measured, and measuring costs a font-fallback
        // chain and a shaping call. `deferred` stays empty — and `Vec::new`
        // allocates nothing — for the overwhelming majority of nodes, which is the
        // point: a stylesheet that never writes `ch` must cost exactly what it did
        // before `ch` was resolvable here. This is the same discipline layout's
        // `ch_width_for` already applied; it is not a new tax on every node.
        let mut deferred: Vec<(PropertyId, ComputedValue)> = Vec::new();
        for &property in COMPUTED_PROPERTIES {
            if property == PropertyId::FontSize {
                continue;
            }
            if let Some(value) = self.resolve_property(
                property,
                cascaded,
                &effective,
                parent,
                rtl,
                wm,
                ctx.custom_values,
            ) {
                if value.value().uses_ch() {
                    deferred.push((property, value));
                } else {
                    style.set(property, value.map_value(|v| absolutize_value(v, &lengths)));
                }
            }
        }

        // Step 5: the node's OWN `ch` base, measured once and only if something
        // asked for it. `style` now carries this node's resolved `font-size` and
        // its face-selecting `font-*` properties — exactly, and only, what the port
        // is promised — so the `0` glyph is measured in the face this node will
        // actually paint in, at the size it will actually paint at. This is the
        // measurement threaded INTO computed-value time: with it `ch` folds here
        // like every other unit, and paint stops reading `0`.
        //
        // With no port, or a port that cannot measure, the deferred values are set
        // unfolded and their `ch` terms stay symbolic — the pre-port behaviour, and
        // still better than baking in `ch_base`'s `0.5em` substitute.
        if !deferred.is_empty() {
            let lengths = match ctx.metrics.and_then(|metrics| metrics.zero_advance(&style)) {
                Some(ch_width) => lengths.with_ch_width(ch_width),
                None => lengths,
            };
            for (property, value) in deferred {
                style.set(property, value.map_value(|v| absolutize_value(v, &lengths)));
            }
        }

        style
    }

    #[doc = include_str!("style_resolver.doc/resolve_font_size.md")]
    fn resolve_font_size(
        self,
        cascaded: &ResolvedDeclarations,
        custom: &FastMap<InternedSymbol, PropertyValue>,
        parent: Option<&ComputedStyle>,
        rtl: bool,
        wm: WritingMode,
        ctx: ResolutionContext,
    ) -> f32 {
        // No parent = the root, whose implicit parent `font-size` is the initial
        // `medium` — the same value `rem` means there.
        let parent_font_size = parent.map_or(INITIAL_FONT_SIZE, Self::font_size_of);
        let declared = self.resolve_property(
            PropertyId::FontSize,
            cascaded,
            custom,
            parent,
            rtl,
            wm,
            ctx.custom_values,
        );
        // A math function — `calc()`, `min()`, `max()`, `clamp()` — is a
        // `PropertyValue::Calc`, not a `Length`, so reading only the length here
        // dropped the whole declaration and left `font-size` at its initial
        // `medium`. That silently ruled out **responsive typography entirely**:
        // `clamp(16px, 5vmin, 30px)` is how type is meant to follow the window
        // between bounds, and it computed to 16px at every size.
        //
        // The expression is folded against the **parent's** size, per CSS Values 4
        // §6.1.2: a font-relative unit inside a `font-size` means the parent's,
        // not this node's — which is also why `%` resolves against it. A `ch`
        // inside the expression takes `ch_base`'s `0.5em` substitute rather than a
        // measured advance; measuring it would need the parent's glyph metrics
        // threaded through the expression walk, and `font-size: calc(… 1ch …)` is
        // vanishingly rare next to the units this unlocks.
        let specified = match declared.as_ref().map(ComputedValue::value) {
            Some(PropertyValue::Calc(expr)) => expr
                .resolve(
                    &LengthContext::new(
                        parent_font_size,
                        ctx.root_font_size,
                        ctx.viewport_width,
                        ctx.viewport_height,
                    )
                    .with_percent_basis(parent_font_size)
                    .with_environment(ctx.environment),
                )
                .map(Length::px),
            _ => declared.and_then(|value| value.as_length()),
        };
        // `font-size: 2ch` needs the PARENT's `0` advance, not this node's — see
        // `absolute_font_size`. Measured only when the declaration is actually a
        // `ch`: the port shapes a glyph, and `font-size` is on every node while
        // `font-size: <n>ch` is on almost none.
        let parent_ch_width = specified
            .filter(|length| length.is_ch())
            .and(ctx.metrics)
            .and_then(|metrics| metrics.zero_advance(&Self::font_context_of(parent)));
        absolute_font_size(
            specified,
            parent_font_size,
            ctx.root_font_size,
            ctx.viewport_width,
            ctx.viewport_height,
            parent_ch_width,
        )
    }

    #[doc = include_str!("style_resolver.doc/font_context_of.md")]
    fn font_context_of(parent: Option<&ComputedStyle>) -> ComputedStyle {
        parent.cloned().unwrap_or_else(|| {
            let mut initial = ComputedStyle::new();
            initial.set(
                PropertyId::FontSize,
                ComputedValue::length(Length::px(INITIAL_FONT_SIZE)),
            );
            initial
        })
    }

    #[doc = include_str!("style_resolver.doc/resolve_property.md")]
    // Eight arguments, one over the threshold, and the eighth is the port the
    // substitution step needs; bundling the other seven into a struct to hide
    // that would cost every caller a construction and the reader a hop.
    #[allow(clippy::too_many_arguments)]
    fn resolve_property(
        self,
        property: PropertyId,
        cascaded: &ResolvedDeclarations,
        custom: &FastMap<InternedSymbol, PropertyValue>,
        parent: Option<&ComputedStyle>,
        rtl: bool,
        wm: WritingMode,
        custom_values: Option<&dyn CustomValueParser>,
    ) -> Option<ComputedValue> {
        match cascaded_for(cascaded, property, rtl, wm) {
            Some(raw) => {
                let value = substitute(raw, custom);
                // CSS Variables 1 §3: a substitution that is invalid AT
                // COMPUTED-VALUE TIME makes the property compute to `unset` —
                // the inherited value where it inherits, the initial value
                // where it does not. Absence is a third behaviour the spec does
                // not offer, and it is what a wrong-typed value falls into by
                // accident: `font-family: var(--font-body)` holding a length
                // reaches the longhand and reads as no family at all, rather
                // than as the family the parent set. Only a value that arrived
                // through a reference is judged here; an ordinary declaration
                // was judged by the parser that lowered it.
                let value = if matches!(raw, PropertyValue::CustomRef(_)) {
                    substituted_value(property, value, custom_values)
                } else {
                    value
                };
                // A relative `font-weight` (`bolder` / `lighter`) resolves here,
                // at the computed-value step, against the parent's already-
                // computed weight (§4.14) — never as a fixed number.
                if property == PropertyId::FontWeight
                    && let Some(weight) = resolve_relative_weight(&value, parent)
                {
                    return Some(ComputedValue::number(weight));
                }
                self.resolve_specified(property, value, parent)
            }
            None => {
                if InheritedProperty::of(property).is_inherited() {
                    self.inherited_from(property, parent)
                } else {
                    self.initial_value(property)
                }
            }
        }
    }

    #[doc = include_str!("style_resolver.doc/resolve_specified.md")]
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

    #[doc = include_str!("style_resolver.doc/inherited_from.md")]
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::property::{StyleColor, layout_keyword};

    /// Root font-size 16 (the initial), viewport 800x600: the bases a test that
    /// is not about relative units resolves against.
    fn ctx() -> ResolutionContext<'static> {
        ResolutionContext::for_root(800.0, 600.0)
    }

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

        let computed = resolver.resolve(&cascaded, &HashMap::default(), Some(&parent), ctx());
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

        let computed = resolver.resolve(&cascaded, &HashMap::default(), None, ctx());
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

        let computed = resolver.resolve(&cascaded, &HashMap::default(), Some(&parent), ctx());
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

        let computed = resolver.resolve(&cascaded, &HashMap::default(), Some(&parent), ctx());
        assert_eq!(computed.opacity(), Some(1.0));
    }

    #[test]
    fn var_substitution_resolves_reference() {
        let resolver = StyleResolver::new();
        let name = sym(1);
        let mut custom = HashMap::default();
        custom.insert(name, PropertyValue::Number(0.5));

        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Opacity, PropertyValue::CustomRef(name));

        let computed = resolver.resolve(&cascaded, &custom, None, ctx());
        assert_eq!(computed.opacity(), Some(0.5));
    }

    #[test]
    fn var_substitution_follows_chain() {
        let resolver = StyleResolver::new();
        let (a, b) = (sym(1), sym(2));
        let mut custom = HashMap::default();
        custom.insert(a, PropertyValue::CustomRef(b));
        custom.insert(b, PropertyValue::Number(0.75));

        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Opacity, PropertyValue::CustomRef(a));

        let computed = resolver.resolve(&cascaded, &custom, None, ctx());
        assert_eq!(computed.opacity(), Some(0.75));
    }

    #[test]
    fn unresolved_var_behaves_as_unset() {
        let resolver = StyleResolver::new();
        // Reference to a missing custom property -> unset -> initial for opacity.
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Opacity, PropertyValue::CustomRef(sym(99)));

        let computed = resolver.resolve(&cascaded, &HashMap::default(), None, ctx());
        assert_eq!(computed.opacity(), Some(1.0));
    }

    #[test]
    fn cyclic_var_terminates_and_behaves_as_unset() {
        let resolver = StyleResolver::new();
        let (a, b) = (sym(1), sym(2));
        let mut custom = HashMap::default();
        custom.insert(a, PropertyValue::CustomRef(b));
        custom.insert(b, PropertyValue::CustomRef(a));

        // `color` inherits, so cyclic -> unset -> inherit; with no parent -> initial.
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Color, PropertyValue::CustomRef(a));

        let computed = resolver.resolve(&cascaded, &custom, None, ctx());
        assert_eq!(
            computed
                .get(PropertyId::Color)
                .and_then(ComputedValue::as_color),
            Some(StyleColor::BLACK)
        );
    }

    #[test]
    fn a_substitution_the_property_cannot_carry_computes_to_unset_and_inherits() {
        // CSS Variables 1 §3: invalid AT COMPUTED-VALUE TIME is `unset`, and
        // `font-family` inherits, so the parent's family is what this node
        // paints in. Carrying the length through instead leaves the property
        // holding a value nothing can read — absent, which is neither the
        // inherited value nor the initial one, and is not a behaviour CSS has.
        let resolver = StyleResolver::new();
        let name = sym(1);
        let mut custom = HashMap::default();
        custom.insert(name, PropertyValue::Length(Length::px(12.0)));

        let mut parent = ComputedStyle::new();
        parent.set(
            PropertyId::FontFamily,
            ComputedValue::from_concrete(PropertyValue::font_families(["Georgia"]))
                .expect("a family list is a computed value"),
        );

        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::FontFamily, PropertyValue::CustomRef(name));

        let computed = resolver.resolve(&cascaded, &custom, Some(&parent), ctx());
        assert_eq!(
            computed
                .get(PropertyId::FontFamily)
                .map(ComputedValue::value),
            Some(&PropertyValue::font_families(["Georgia"]))
        );
    }

    #[test]
    fn a_substitution_the_property_cannot_carry_takes_the_initial_when_it_does_not_inherit() {
        // The other half of `unset`. `background-color` does not inherit, so a
        // keyword that is no colour resets it to `transparent` — the parent's
        // red must not reach it, and neither must the keyword.
        let resolver = StyleResolver::new();
        let name = sym(1);
        let mut custom = HashMap::default();
        custom.insert(name, PropertyValue::Keyword(sym(7)));

        let mut parent = ComputedStyle::new();
        parent.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
        );

        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::BackgroundColor, PropertyValue::CustomRef(name));

        let computed = resolver.resolve(&cascaded, &custom, Some(&parent), ctx());
        assert_eq!(
            computed
                .get(PropertyId::BackgroundColor)
                .and_then(ComputedValue::as_color),
            Some(StyleColor::TRANSPARENT)
        );
    }

    #[test]
    fn a_substitution_the_property_can_carry_is_untouched_by_the_grammar_check() {
        // The guard against over-rejecting: the intrinsic sizing keywords are
        // the one keyword family a custom property carries intact, because they
        // lower to reserved symbols rather than to a sheet's own interner.
        let resolver = StyleResolver::new();
        let name = sym(1);
        let mut custom = HashMap::default();
        custom.insert(
            name,
            PropertyValue::Keyword(layout_keyword::SIZING_MIN_CONTENT),
        );

        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::Width, PropertyValue::CustomRef(name));

        let computed = resolver.resolve(&cascaded, &custom, None, ctx());
        assert_eq!(
            computed.get(PropertyId::Width).map(ComputedValue::value),
            Some(&PropertyValue::Keyword(layout_keyword::SIZING_MIN_CONTENT))
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

        let cascaded = ResolvedDeclarations::new();
        let computed = resolver.resolve(&cascaded, &HashMap::default(), Some(&parent), ctx());

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
        let mut custom = HashMap::default();
        custom.insert(a, PropertyValue::Number(9.0));

        let computed =
            resolver.resolve(&ResolvedDeclarations::new(), &custom, Some(&parent), ctx());
        assert_eq!(
            computed.custom_property(a),
            Some(&PropertyValue::Number(9.0))
        );
        assert_eq!(
            computed.custom_property(b),
            Some(&PropertyValue::Number(2.0))
        );
    }

    /// Reads a computed numeric `font-weight` off a resolved style.
    fn weight_of(style: &ComputedStyle) -> Option<f32> {
        match style.get(PropertyId::FontWeight).map(ComputedValue::value) {
            Some(PropertyValue::Number(n)) => Some(*n),
            _ => None,
        }
    }

    #[test]
    fn bolder_resolves_against_parent_computed_weight() {
        let resolver = StyleResolver::new();
        let mut parent = ComputedStyle::new();
        parent.set(PropertyId::FontWeight, ComputedValue::number(700.0));

        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(
            PropertyId::FontWeight,
            PropertyValue::Keyword(layout_keyword::FONT_WEIGHT_BOLDER),
        );

        let computed = resolver.resolve(&cascaded, &HashMap::default(), Some(&parent), ctx());
        assert_eq!(weight_of(&computed), Some(900.0));
    }

    #[test]
    fn lighter_without_parent_resolves_against_normal() {
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(
            PropertyId::FontWeight,
            PropertyValue::Keyword(layout_keyword::FONT_WEIGHT_LIGHTER),
        );

        // No parent -> the implicit parent weight is `normal` (400) -> lighter 100.
        let computed = resolver.resolve(&cascaded, &HashMap::default(), None, ctx());
        assert_eq!(weight_of(&computed), Some(100.0));
    }

    #[test]
    fn absolute_font_weight_is_unchanged() {
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::FontWeight, PropertyValue::Number(500.0));

        // An absolute numeric weight is stored verbatim, regardless of the parent.
        let mut parent = ComputedStyle::new();
        parent.set(PropertyId::FontWeight, ComputedValue::number(900.0));
        let computed = resolver.resolve(&cascaded, &HashMap::default(), Some(&parent), ctx());
        assert_eq!(weight_of(&computed), Some(500.0));
    }

    #[test]
    fn absent_font_weight_inherits_parent_weight() {
        let resolver = StyleResolver::new();
        let mut parent = ComputedStyle::new();
        parent.set(PropertyId::FontWeight, ComputedValue::number(700.0));

        // `font-weight` inherits: a child with none takes the parent's weight.
        let computed = resolver.resolve(
            &ResolvedDeclarations::new(),
            &HashMap::default(),
            Some(&parent),
            ctx(),
        );
        assert_eq!(weight_of(&computed), Some(700.0));
    }

    #[test]
    fn keyword_initial_is_left_unresolved() {
        let resolver = StyleResolver::new();
        assert_eq!(resolver.initial_value(PropertyId::Display), None);
        // With no cascaded value and no parent, `display` cannot be computed yet.
        let computed = resolver.resolve(
            &ResolvedDeclarations::new(),
            &HashMap::default(),
            None,
            ctx(),
        );
        assert_eq!(computed.get(PropertyId::Display), None);
    }

    #[test]
    fn authored_z_index_computes_to_a_number() {
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::ZIndex, PropertyValue::Number(5.0));

        // An authored integer `z-index` flows through to the computed style as a
        // number the scene reads for stacking.
        let computed = resolver.resolve(&cascaded, &HashMap::default(), None, ctx());
        assert_eq!(
            computed.get(PropertyId::ZIndex).map(ComputedValue::value),
            Some(&PropertyValue::Number(5.0))
        );

        // `z-index` does not inherit: a child with no authored value takes the
        // initial (`auto`, left unresolved), NOT the parent's numeric level.
        let child = resolver.resolve(
            &ResolvedDeclarations::new(),
            &HashMap::default(),
            Some(&computed),
            ctx(),
        );
        assert_eq!(child.get(PropertyId::ZIndex), None);
    }

    #[test]
    fn z_index_initial_is_auto_left_unresolved() {
        let resolver = StyleResolver::new();
        // `z-index` initial (`auto`) is deferred, so an absent value is unresolved.
        assert_eq!(resolver.initial_value(PropertyId::ZIndex), None);
        let computed = resolver.resolve(
            &ResolvedDeclarations::new(),
            &HashMap::default(),
            None,
            ctx(),
        );
        assert_eq!(computed.get(PropertyId::ZIndex), None);
    }

    #[test]
    fn border_width_initial_is_zero_style_and_color_unresolved() {
        let resolver = StyleResolver::new();
        // `border-*-width` has a concrete `0` initial so an unauthored border
        // insets nothing (box model byte-identical to before borders existed).
        assert_eq!(
            resolver.initial_value(PropertyId::BorderTopWidth),
            Some(ComputedValue::length(Length::px(0.0)))
        );
        // `border-*-style` (`none`) and `border-*-color` (`currentColor`) initials
        // are deferred, so an absent value is left unresolved.
        assert_eq!(resolver.initial_value(PropertyId::BorderTopStyle), None);
        assert_eq!(resolver.initial_value(PropertyId::BorderTopColor), None);

        // With no authored border, width computes to 0 and style/color stay unset.
        let computed = resolver.resolve(
            &ResolvedDeclarations::new(),
            &HashMap::default(),
            None,
            ctx(),
        );
        assert_eq!(
            computed
                .get(PropertyId::BorderTopWidth)
                .and_then(ComputedValue::as_length),
            Some(Length::px(0.0))
        );
        assert_eq!(computed.get(PropertyId::BorderTopStyle), None);
        assert_eq!(computed.get(PropertyId::BorderTopColor), None);
    }

    #[test]
    fn border_radius_initial_is_zero_and_authored_flows() {
        let resolver = StyleResolver::new();
        // Each corner's initial is a concrete `0` (a square corner).
        for id in [
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
        ] {
            assert_eq!(
                resolver.initial_value(id),
                Some(ComputedValue::length(Length::px(0.0)))
            );
        }
        // An unauthored node computes every corner to 0.
        let computed = resolver.resolve(
            &ResolvedDeclarations::new(),
            &HashMap::default(),
            None,
            ctx(),
        );
        assert_eq!(
            computed
                .get(PropertyId::BorderTopLeftRadius)
                .and_then(ComputedValue::as_length),
            Some(Length::px(0.0))
        );
        // An authored corner radius flows through and does not inherit.
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(
            PropertyId::BorderTopLeftRadius,
            PropertyValue::Length(Length::px(8.0)),
        );
        let parent = resolver.resolve(&cascaded, &HashMap::default(), None, ctx());
        assert_eq!(
            parent
                .get(PropertyId::BorderTopLeftRadius)
                .and_then(ComputedValue::as_length),
            Some(Length::px(8.0))
        );
        let child = resolver.resolve(
            &ResolvedDeclarations::new(),
            &HashMap::default(),
            Some(&parent),
            ctx(),
        );
        assert_eq!(
            child
                .get(PropertyId::BorderTopLeftRadius)
                .and_then(ComputedValue::as_length),
            Some(Length::px(0.0)),
            "corner radius does not inherit"
        );
    }

    #[test]
    fn authored_border_flows_to_computed_style() {
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(
            PropertyId::BorderTopWidth,
            PropertyValue::Length(Length::px(2.0)),
        );
        cascaded.insert(
            PropertyId::BorderTopStyle,
            PropertyValue::Keyword(crate::property::layout_keyword::BORDER_STYLE_SOLID),
        );
        cascaded.insert(
            PropertyId::BorderTopColor,
            PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
        );
        let computed = resolver.resolve(&cascaded, &HashMap::default(), None, ctx());
        assert_eq!(
            computed
                .get(PropertyId::BorderTopWidth)
                .and_then(ComputedValue::as_length),
            Some(Length::px(2.0))
        );
        assert_eq!(
            computed
                .get(PropertyId::BorderTopColor)
                .and_then(ComputedValue::as_color),
            Some(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
        );
        // Border does not inherit: a child with no authored border takes the
        // initial (width 0, style/color unset), NOT the parent's red border.
        let child = resolver.resolve(
            &ResolvedDeclarations::new(),
            &HashMap::default(),
            Some(&computed),
            ctx(),
        );
        assert_eq!(child.get(PropertyId::BorderTopColor), None);
        assert_eq!(
            child
                .get(PropertyId::BorderTopWidth)
                .and_then(ComputedValue::as_length),
            Some(Length::px(0.0))
        );
    }

    /// An authored typography value (previously dropped because the property was
    /// not in the computed set) now flows through the real resolver.
    #[test]
    fn authored_typography_flows_to_computed_style() {
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(
            PropertyId::FontSize,
            PropertyValue::Length(Length::px(24.0)),
        );
        cascaded.insert(
            PropertyId::FontFamily,
            PropertyValue::font_families(["Foo", "sans-serif"]),
        );
        cascaded.insert(
            PropertyId::FontStyle,
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_ITALIC),
        );
        cascaded.insert(
            PropertyId::Direction,
            PropertyValue::Keyword(layout_keyword::DIRECTION_RTL),
        );
        cascaded.insert(
            PropertyId::TextAlign,
            PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_CENTER),
        );
        cascaded.insert(
            PropertyId::TextOverflow,
            PropertyValue::Keyword(layout_keyword::TEXT_OVERFLOW_ELLIPSIS),
        );

        let computed = resolver.resolve(&cascaded, &HashMap::default(), None, ctx());

        assert_eq!(
            computed
                .get(PropertyId::FontSize)
                .and_then(ComputedValue::as_length),
            Some(Length::px(24.0))
        );
        assert_eq!(
            computed
                .get(PropertyId::FontFamily)
                .and_then(|v| v.value().as_font_families().map(<[_]>::len)),
            Some(2)
        );
        assert_eq!(
            computed
                .get(PropertyId::FontStyle)
                .map(ComputedValue::value),
            Some(&PropertyValue::Keyword(layout_keyword::FONT_STYLE_ITALIC))
        );
        assert_eq!(
            computed
                .get(PropertyId::Direction)
                .map(ComputedValue::value),
            Some(&PropertyValue::Keyword(layout_keyword::DIRECTION_RTL))
        );
        assert_eq!(
            computed
                .get(PropertyId::TextAlign)
                .map(ComputedValue::value),
            Some(&PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_CENTER))
        );
        assert_eq!(
            computed
                .get(PropertyId::TextOverflow)
                .map(ComputedValue::value),
            Some(&PropertyValue::Keyword(
                layout_keyword::TEXT_OVERFLOW_ELLIPSIS
            ))
        );
    }

    /// An authored box-model / flex value (previously dropped) now flows through.
    #[test]
    fn authored_layout_property_flows_to_computed_style() {
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(
            PropertyId::FlexDirection,
            PropertyValue::Keyword(layout_keyword::FLEX_COLUMN),
        );
        cascaded.insert(
            PropertyId::MarginTop,
            PropertyValue::Length(Length::px(8.0)),
        );
        cascaded.insert(
            PropertyId::MinWidth,
            PropertyValue::Length(Length::px(50.0)),
        );

        let computed = resolver.resolve(&cascaded, &HashMap::default(), None, ctx());

        assert_eq!(
            computed
                .get(PropertyId::FlexDirection)
                .map(ComputedValue::value),
            Some(&PropertyValue::Keyword(layout_keyword::FLEX_COLUMN))
        );
        assert_eq!(
            computed
                .get(PropertyId::MarginTop)
                .and_then(ComputedValue::as_length),
            Some(Length::px(8.0))
        );
        assert_eq!(
            computed
                .get(PropertyId::MinWidth)
                .and_then(ComputedValue::as_length),
            Some(Length::px(50.0))
        );
    }

    /// Regression guard: an unauthored node's computed style is unchanged — every
    /// deferred keyword / relative-only initial stays [`None`] (so downstream falls
    /// back to its default exactly as before), and every concrete CSS initial
    /// equals the value the downstream consumer already defaults to.
    #[test]
    fn unset_node_is_unchanged() {
        let resolver = StyleResolver::new();
        let computed = resolver.resolve(
            &ResolvedDeclarations::new(),
            &HashMap::default(),
            None,
            ctx(),
        );

        // Deferred keyword / weight initials remain unresolved, so the runtime
        // helpers keep using their hardcoded defaults (byte-for-byte as before).
        for id in [
            PropertyId::Display,
            PropertyId::FlexDirection,
            PropertyId::FlexWrap,
            PropertyId::JustifyContent,
            PropertyId::AlignContent,
            PropertyId::AlignItems,
            PropertyId::FontStyle,
            PropertyId::Direction,
            PropertyId::TextAlign,
            PropertyId::TextOverflow,
            PropertyId::FontWeight,
            PropertyId::FontFamily,
            PropertyId::MaxWidth,
            PropertyId::MaxHeight,
            PropertyId::GridTemplateColumns,
            PropertyId::GridTemplateRows,
            PropertyId::GridColumnStart,
            PropertyId::GridColumnEnd,
            PropertyId::GridRowStart,
            PropertyId::GridRowEnd,
        ] {
            assert_eq!(computed.get(id), None, "{} should stay deferred", id.name());
        }

        // Concrete CSS initials equal the downstream defaults, so no output moves.
        assert_eq!(computed.opacity(), Some(1.0));
        assert_eq!(
            computed
                .get(PropertyId::Width)
                .and_then(ComputedValue::as_length),
            Some(Length::Auto)
        );
        for id in [
            PropertyId::MarginTop,
            PropertyId::PaddingLeft,
            PropertyId::Gap,
        ] {
            assert_eq!(
                computed.get(id).and_then(ComputedValue::as_length),
                Some(Length::px(0.0)),
                "{} initial is 0",
                id.name()
            );
        }
        assert_eq!(
            computed
                .get(PropertyId::FontSize)
                .and_then(ComputedValue::as_length),
            Some(Length::px(16.0))
        );
    }

    /// An authored `grid-template-columns` track list flows through the resolver
    /// as a concrete computed value (it is not a deferred keyword initial).
    #[test]
    fn authored_grid_tracks_flow_to_computed_style() {
        use crate::property::GridTrack;
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(
            PropertyId::GridTemplateColumns,
            PropertyValue::grid_tracks([
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fr(1.0),
                GridTrack::Fr(2.0),
            ]),
        );

        let computed = resolver.resolve(&cascaded, &HashMap::default(), None, ctx());
        let tracks = computed
            .get(PropertyId::GridTemplateColumns)
            .map(ComputedValue::value)
            .and_then(PropertyValue::as_grid_tracks)
            .expect("grid tracks computed");
        assert_eq!(tracks.len(), 3);
        assert_eq!(tracks[0], GridTrack::Fixed(Length::px(100.0)));
        assert_eq!(tracks[2], GridTrack::Fr(2.0));
    }

    /// An authored `grid-auto-rows` track list flows through the resolver as a
    /// concrete computed `GridTracks` value, reusing the same value vocabulary as
    /// `grid-template-*` (it is not a deferred keyword initial).
    #[test]
    fn authored_grid_auto_rows_flow_to_computed_style() {
        use crate::property::GridTrack;
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(
            PropertyId::GridAutoRows,
            PropertyValue::grid_tracks([GridTrack::Fixed(Length::px(40.0))]),
        );
        let computed = resolver.resolve(&cascaded, &HashMap::default(), None, ctx());
        let tracks = computed
            .get(PropertyId::GridAutoRows)
            .map(ComputedValue::value)
            .and_then(PropertyValue::as_grid_tracks)
            .expect("grid-auto-rows computed");
        assert_eq!(tracks, &[GridTrack::Fixed(Length::px(40.0))]);
        // Unauthored on a child: not inherited, so it is absent (runtime defaults
        // to the `[Auto]` implicit-track sizing).
        let child = resolver.resolve(
            &ResolvedDeclarations::new(),
            &HashMap::default(),
            Some(&computed),
            ctx(),
        );
        assert!(child.get(PropertyId::GridAutoRows).is_none());
    }

    /// An authored `grid-column-start` / `-end` line edge flows through the
    /// resolver as a concrete computed value (it is not a deferred keyword initial).
    #[test]
    fn authored_grid_line_flows_to_computed_style() {
        use crate::property::GridLineValue;
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(
            PropertyId::GridColumnStart,
            PropertyValue::GridLine(GridLineValue::Line(1)),
        );
        cascaded.insert(
            PropertyId::GridColumnEnd,
            PropertyValue::GridLine(GridLineValue::Span(2)),
        );

        let computed = resolver.resolve(&cascaded, &HashMap::default(), None, ctx());
        assert_eq!(
            computed
                .get(PropertyId::GridColumnStart)
                .map(ComputedValue::value)
                .and_then(PropertyValue::as_grid_line),
            Some(&GridLineValue::Line(1))
        );
        assert_eq!(
            computed
                .get(PropertyId::GridColumnEnd)
                .map(ComputedValue::value)
                .and_then(PropertyValue::as_grid_line),
            Some(&GridLineValue::Span(2))
        );
    }

    /// Inheritance still flows for an inherited property newly in the set:
    /// `font-size` with no cascaded value takes the parent's computed size.
    #[test]
    fn inherited_font_size_flows_from_parent() {
        let resolver = StyleResolver::new();
        let mut parent = ComputedStyle::new();
        parent.set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(30.0)),
        );

        let computed = resolver.resolve(
            &ResolvedDeclarations::new(),
            &HashMap::default(),
            Some(&parent),
            ctx(),
        );
        assert_eq!(
            computed
                .get(PropertyId::FontSize)
                .and_then(ComputedValue::as_length),
            Some(Length::px(30.0))
        );
    }

    // ---- logical properties (spec §4.13) -----------------------------------

    mod logical {
        use super::super::StyleResolver;
        use std::collections::HashMap;

        use super::ctx;
        use crate::cascade::{CascadeOrigin, CascadePriority, ImportantFlag, ResolvedDeclarations};
        use crate::computed::{ComputedStyle, ComputedValue};
        use crate::property::{Length, PropertyId, PropertyValue, layout_keyword};
        use crate::selector::SelectorSpecificity;

        /// A cascade priority in the author origin at `source_order`, so tests can
        /// state "this declaration came later" the way real CSS does.
        fn at(source_order: u32) -> CascadePriority {
            CascadePriority::new(
                CascadeOrigin::GlobalApp,
                ImportantFlag::Normal,
                SelectorSpecificity::new(0, 1, 0),
                source_order,
            )
        }

        fn px(value: f32) -> PropertyValue {
            PropertyValue::Length(Length::px(value))
        }

        /// Resolves `cascaded` with no parent, under the given direction.
        fn resolve(cascaded: &ResolvedDeclarations, rtl: bool) -> ComputedStyle {
            let mut with_direction = ResolvedDeclarations::new();
            for (property, value) in cascaded.iter() {
                if let Some(priority) = cascaded.priority_of(*property) {
                    with_direction.insert_with_priority(*property, value.clone(), priority);
                } else {
                    with_direction.insert(*property, value.clone());
                }
            }
            if rtl {
                with_direction.insert(
                    PropertyId::Direction,
                    PropertyValue::Keyword(layout_keyword::DIRECTION_RTL),
                );
            }
            StyleResolver::new().resolve(&with_direction, &HashMap::default(), None, ctx())
        }

        fn margin(style: &ComputedStyle, property: PropertyId) -> Option<Length> {
            style.get(property).and_then(ComputedValue::as_length)
        }

        #[test]
        fn an_inline_logical_margin_follows_the_direction() {
            // THE reason logical properties exist: `margin-inline-start` is the
            // LEFT edge in ltr and the RIGHT edge in rtl, from one declaration.
            let mut cascaded = ResolvedDeclarations::new();
            cascaded.insert_with_priority(PropertyId::MarginInlineStart, px(10.0), at(0));

            let ltr = resolve(&cascaded, false);
            assert_eq!(margin(&ltr, PropertyId::MarginLeft), Some(Length::px(10.0)));
            assert_eq!(margin(&ltr, PropertyId::MarginRight), Some(Length::px(0.0)));

            let rtl = resolve(&cascaded, true);
            assert_eq!(
                margin(&rtl, PropertyId::MarginRight),
                Some(Length::px(10.0))
            );
            assert_eq!(margin(&rtl, PropertyId::MarginLeft), Some(Length::px(0.0)));
        }

        #[test]
        fn the_block_axis_is_unmoved_by_direction() {
            // Under the default `horizontal-tb` writing mode the block axis is
            // vertical and `direction` never touches it: `block-start` is `top` in
            // either direction. (A vertical writing mode would rotate it; that is
            // covered by the `physical_under` mapping and the flow-layout tests.)
            let mut cascaded = ResolvedDeclarations::new();
            cascaded.insert_with_priority(PropertyId::PaddingBlockStart, px(8.0), at(0));

            for rtl in [false, true] {
                let style = resolve(&cascaded, rtl);
                assert_eq!(
                    margin(&style, PropertyId::PaddingTop),
                    Some(Length::px(8.0)),
                    "rtl={rtl}"
                );
            }
        }

        #[test]
        fn a_later_logical_declaration_beats_an_earlier_physical_one() {
            // The subtle half, and why the cascade priority had to be retained:
            // `margin-left` and `margin-inline-start` are different PropertyIds that
            // cascade independently, but CSS decides between them by ordinary order.
            let mut cascaded = ResolvedDeclarations::new();
            cascaded.insert_with_priority(PropertyId::MarginLeft, px(1.0), at(0));
            cascaded.insert_with_priority(PropertyId::MarginInlineStart, px(2.0), at(1));

            let style = resolve(&cascaded, false);
            assert_eq!(
                margin(&style, PropertyId::MarginLeft),
                Some(Length::px(2.0)),
                "the later declaration wins, whichever form it took"
            );
        }

        #[test]
        fn a_later_physical_declaration_beats_an_earlier_logical_one() {
            // The same rule the other way round — a logical property is not
            // privileged, it just participates.
            let mut cascaded = ResolvedDeclarations::new();
            cascaded.insert_with_priority(PropertyId::MarginInlineStart, px(2.0), at(0));
            cascaded.insert_with_priority(PropertyId::MarginLeft, px(1.0), at(1));

            let style = resolve(&cascaded, false);
            assert_eq!(
                margin(&style, PropertyId::MarginLeft),
                Some(Length::px(1.0))
            );
        }

        #[test]
        fn an_important_logical_declaration_outranks_a_later_normal_physical_one() {
            // Priority is the whole comparison, not just source order.
            let mut cascaded = ResolvedDeclarations::new();
            cascaded.insert_with_priority(
                PropertyId::MarginInlineStart,
                px(2.0),
                CascadePriority::new(
                    CascadeOrigin::GlobalApp,
                    ImportantFlag::Important,
                    SelectorSpecificity::new(0, 1, 0),
                    0,
                ),
            );
            cascaded.insert_with_priority(PropertyId::MarginLeft, px(1.0), at(99));

            let style = resolve(&cascaded, false);
            assert_eq!(
                margin(&style, PropertyId::MarginLeft),
                Some(Length::px(2.0))
            );
        }

        #[test]
        fn opposite_inline_edges_do_not_collide() {
            let mut cascaded = ResolvedDeclarations::new();
            cascaded.insert_with_priority(PropertyId::MarginInlineStart, px(3.0), at(0));
            cascaded.insert_with_priority(PropertyId::MarginInlineEnd, px(5.0), at(1));

            let ltr = resolve(&cascaded, false);
            assert_eq!(margin(&ltr, PropertyId::MarginLeft), Some(Length::px(3.0)));
            assert_eq!(margin(&ltr, PropertyId::MarginRight), Some(Length::px(5.0)));

            // Under rtl both swap — start is right, end is left.
            let rtl = resolve(&cascaded, true);
            assert_eq!(margin(&rtl, PropertyId::MarginRight), Some(Length::px(3.0)));
            assert_eq!(margin(&rtl, PropertyId::MarginLeft), Some(Length::px(5.0)));
        }

        #[test]
        fn a_logical_property_is_never_computed_itself() {
            // One box edge, one source of truth: the computed style holds
            // `margin-left`, so no downstream phase has to ask "or was it declared
            // logically?".
            let mut cascaded = ResolvedDeclarations::new();
            cascaded.insert_with_priority(PropertyId::MarginInlineStart, px(10.0), at(0));
            let style = resolve(&cascaded, false);
            assert_eq!(style.get(PropertyId::MarginInlineStart), None);
        }
    }
}
