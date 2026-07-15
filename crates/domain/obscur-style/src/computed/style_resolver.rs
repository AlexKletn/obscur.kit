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
//! * **Full modelled property set.** The resolver computes *every* modelled
//!   [`PropertyId`] longhand (see [`StyleResolver::computed_properties`]), so an
//!   authored value for any of them flows through the computed-value step to the
//!   downstream layout / scene / text consumers. A property whose only value would
//!   be a deferred keyword initial is still left unresolved (see the previous
//!   bullet); an *unauthored* node is therefore byte-for-byte unchanged, because
//!   each property still resolves to exactly the value (or [`None`]) it did before
//!   — the set merely no longer drops authored values for the added longhands.

use crate::cascade::ResolvedDeclarations;
use crate::computed::{ComputedStyle, ComputedValue};
use crate::property::{
    layout_keyword, InheritedProperty, Length, PropertyId, PropertyValue, StyleColor,
};
use obscur_identity::InternedSymbol;
use std::collections::{HashMap, HashSet};

/// The CSS initial `font-weight` (`normal`), and the weight a relative
/// `bolder` / `lighter` resolves against when there is no parent weight.
const NORMAL_FONT_WEIGHT: f32 = 400.0;

/// Every modelled longhand the resolver computes.
///
/// This is the complete [`PropertyId`] catalogue: the resolver must compute each
/// property that a downstream consumer reads back off a [`ComputedStyle`]
/// (layout box-model / flex, scene paint, text typography), otherwise an authored
/// CSS value for that property would be silently dropped at the computed-value
/// step. It is spelled out as a local `const` list — rather than derived from the
/// enum — because [`PropertyId`] lives in another module this file must not edit;
/// the list is kept exhaustive by the `computed_properties_cover_every_property_id`
/// test, which fails if a new longhand is added upstream without being computed.
const COMPUTED_PROPERTIES: &[PropertyId] = &[
    // Paint.
    PropertyId::Color,
    PropertyId::BackgroundColor,
    PropertyId::BackgroundImage,
    PropertyId::BackgroundSize,
    PropertyId::BackgroundPosition,
    PropertyId::BackgroundRepeat,
    PropertyId::ObjectFit,
    PropertyId::Opacity,
    PropertyId::Visibility,
    // Positioning.
    PropertyId::Position,
    PropertyId::Top,
    PropertyId::Right,
    PropertyId::Bottom,
    PropertyId::Left,
    PropertyId::ZIndex,
    // Box model: sizing.
    PropertyId::Width,
    PropertyId::Height,
    PropertyId::BoxSizing,
    PropertyId::MinWidth,
    PropertyId::MinHeight,
    PropertyId::MaxWidth,
    PropertyId::MaxHeight,
    // Box model: margins.
    PropertyId::MarginTop,
    PropertyId::MarginRight,
    PropertyId::MarginBottom,
    PropertyId::MarginLeft,
    // Box model: padding.
    PropertyId::PaddingTop,
    PropertyId::PaddingRight,
    PropertyId::PaddingBottom,
    PropertyId::PaddingLeft,
    // Box model: border (width feeds the box model; style + color paint).
    PropertyId::BorderTopWidth,
    PropertyId::BorderRightWidth,
    PropertyId::BorderBottomWidth,
    PropertyId::BorderLeftWidth,
    PropertyId::BorderTopStyle,
    PropertyId::BorderRightStyle,
    PropertyId::BorderBottomStyle,
    PropertyId::BorderLeftStyle,
    PropertyId::BorderTopColor,
    PropertyId::BorderRightColor,
    PropertyId::BorderBottomColor,
    PropertyId::BorderLeftColor,
    // Box model: border radius (rounds the background / border / content clip).
    PropertyId::BorderTopLeftRadius,
    PropertyId::BorderTopRightRadius,
    PropertyId::BorderBottomRightRadius,
    PropertyId::BorderBottomLeftRadius,
    // Outline (painted outside the border box; no layout).
    PropertyId::OutlineWidth,
    PropertyId::OutlineStyle,
    PropertyId::OutlineColor,
    PropertyId::OutlineOffset,
    // Box effects.
    PropertyId::BoxShadow,
    PropertyId::Filter,
    PropertyId::BackdropFilter,
    // 2D transform.
    PropertyId::Transform,
    PropertyId::TransformOrigin,
    // Flex container / item.
    PropertyId::Display,
    PropertyId::FlexDirection,
    PropertyId::FlexWrap,
    PropertyId::JustifyContent,
    PropertyId::AlignContent,
    PropertyId::AlignItems,
    PropertyId::JustifyItems,
    PropertyId::AlignSelf,
    PropertyId::JustifySelf,
    PropertyId::Gap,
    PropertyId::FlexBasis,
    PropertyId::FlexGrow,
    PropertyId::FlexShrink,
    // Grid container.
    PropertyId::GridTemplateColumns,
    PropertyId::GridTemplateRows,
    PropertyId::GridTemplateAreas,
    PropertyId::GridAutoFlow,
    PropertyId::GridAutoColumns,
    PropertyId::GridAutoRows,
    // Grid item line placement.
    PropertyId::GridColumnStart,
    PropertyId::GridColumnEnd,
    PropertyId::GridRowStart,
    PropertyId::GridRowEnd,
    PropertyId::GridArea,
    // Typography.
    PropertyId::FontSize,
    PropertyId::FontFamily,
    PropertyId::FontWeight,
    PropertyId::FontStyle,
    PropertyId::LineHeight,
    PropertyId::Direction,
    PropertyId::TextAlign,
    PropertyId::TextOverflow,
    // Inherited text spacing/casing (CSS Text 3 §8.1 / §8.2 / §2.1).
    PropertyId::LetterSpacing,
    PropertyId::WordSpacing,
    PropertyId::TextTransform,
    // Inherited white-space processing + tab-size (CSS Text 3 §3 / §4.3).
    PropertyId::WhiteSpace,
    // Inherited line-break controls (CSS Text 3 §5.2).
    PropertyId::OverflowWrap,
    PropertyId::WordBreak,
    PropertyId::TabSize,
    // Overflow (pure paint/clip; no layout effect).
    PropertyId::Overflow,
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
            // A logical property has no computed value of its own: it resolves
            // INTO its physical counterpart, which carries the initial. It is not
            // in `COMPUTED_PROPERTIES`, so these arms are unreachable in practice
            // — they exist to keep the match total rather than assumed.
            PropertyId::MarginInlineStart
            | PropertyId::MarginInlineEnd
            | PropertyId::MarginBlockStart
            | PropertyId::MarginBlockEnd
            | PropertyId::PaddingInlineStart
            | PropertyId::PaddingInlineEnd
            | PropertyId::PaddingBlockStart
            | PropertyId::PaddingBlockEnd
            | PropertyId::InsetInlineStart
            | PropertyId::InsetInlineEnd
            | PropertyId::InsetBlockStart
            | PropertyId::InsetBlockEnd => None,
            PropertyId::Color => Some(ComputedValue::color(StyleColor::BLACK)),
            PropertyId::BackgroundColor => Some(ComputedValue::color(StyleColor::TRANSPARENT)),
            // `background-image` initial is `none` (no image). There is no
            // `LinearGradient` value for `none`, so an absent value is left
            // unresolved and the scene paints no gradient fill — byte-for-byte
            // identical to before background images existed.
            PropertyId::BackgroundImage => None,
            // `background-size` initial is `auto` (draw at intrinsic size),
            // `background-position` initial is `0% 0%` (top-left), and
            // `background-repeat` initial is `repeat` (tile both axes). All three
            // have no-interner concrete values, so they resolve to the CSS initial
            // directly (the scene coordinator would default to the same values for
            // an absent property, so this is purely explicit).
            PropertyId::BackgroundSize => Some(
                ComputedValue::from_concrete(PropertyValue::BackgroundSize(
                    crate::property::BackgroundSize::Auto,
                ))
                .expect("BackgroundSize is a concrete computed value"),
            ),
            PropertyId::BackgroundPosition => Some(
                ComputedValue::from_concrete(PropertyValue::BackgroundPosition(
                    crate::property::BackgroundPosition::default(),
                ))
                .expect("BackgroundPosition is a concrete computed value"),
            ),
            PropertyId::BackgroundRepeat => Some(
                ComputedValue::from_concrete(PropertyValue::BackgroundRepeat(
                    crate::property::BackgroundRepeat::Repeat,
                ))
                .expect("BackgroundRepeat is a concrete computed value"),
            ),
            // `object-fit` initial is `fill` (stretch to the box). It has a
            // no-interner concrete value, so it resolves to the CSS initial
            // directly (the scene coordinator would default to the same for an
            // absent property, so this is purely explicit).
            PropertyId::ObjectFit => Some(
                ComputedValue::from_concrete(PropertyValue::ObjectFit(
                    crate::property::ObjectFit::Fill,
                ))
                .expect("ObjectFit is a concrete computed value"),
            ),
            // `box-shadow` initial is `none` (no shadows). There is no
            // `BoxShadow` list value for `none` other than an empty one, and an
            // absent value paints nothing all the same, so it is left unresolved —
            // byte-for-byte identical to before box-shadow existed.
            PropertyId::BoxShadow => None,
            // `filter` initial is `none` (no filter). Like `box-shadow`, an absent
            // value paints nothing, so it is left unresolved — byte-for-byte
            // identical to before `filter: drop-shadow(...)` existed.
            PropertyId::Filter => None,
            // `backdrop-filter` initial is `none` (no backdrop effect). Like
            // `filter`, an absent value frosts nothing, so it is left unresolved —
            // byte-for-byte identical to before backdrop-filter existed.
            PropertyId::BackdropFilter => None,
            // `transform` initial is `none` (no transform). There is no
            // `Transform` value for `none` other than the identity, and an absent
            // value brackets nothing all the same, so it is left unresolved —
            // byte-for-byte identical to before transforms existed.
            PropertyId::Transform => None,
            // `transform-origin` initial is `50% 50%` (the box center). An absent
            // value is left unresolved and the coordinator defaults to the center,
            // so no value need be minted here.
            PropertyId::TransformOrigin => None,
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
            // `border-*-width` initial is `medium`, but it is modelled as `0` so
            // an element with no authored border insets nothing and paints
            // nothing — byte-for-byte identical to before borders existed.
            | PropertyId::BorderTopWidth
            | PropertyId::BorderRightWidth
            | PropertyId::BorderBottomWidth
            | PropertyId::BorderLeftWidth
            // `border-*-radius` initial is `0` (a square corner), so an element
            // with no authored radius rounds nothing — byte-for-byte identical to
            // before border-radius existed.
            | PropertyId::BorderTopLeftRadius
            | PropertyId::BorderTopRightRadius
            | PropertyId::BorderBottomRightRadius
            | PropertyId::BorderBottomLeftRadius
            // `outline-width` initial is `medium`, modelled as `0` (like border
            // width) so no outline paints until authored; `outline-offset` initial
            // is `0` (the outline hugs the border box).
            | PropertyId::OutlineWidth
            | PropertyId::OutlineOffset
            | PropertyId::Gap => Some(ComputedValue::length(Length::px(0.0))),
            // Sizing minimums and `flex-basis`: initial `auto`.
            PropertyId::MinWidth | PropertyId::MinHeight | PropertyId::FlexBasis => {
                Some(ComputedValue::length(Length::Auto))
            }
            // Positioning insets: initial `auto` (no offset).
            PropertyId::Top | PropertyId::Right | PropertyId::Bottom | PropertyId::Left => {
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
            // `z-index` initial is `auto`. Although `auto` has a reserved
            // cross-crate handle, an absent value is left unresolved so the scene's
            // stacking treats the node as `auto` (effective z of 0, inheriting its
            // parent's stacking level) — byte-for-byte identical to the pre-stacking
            // paint order.
            PropertyId::ZIndex => None,
            // `border-*-style` initial is `none`; its keyword initial needs the
            // interner, so an absent value is left unresolved and the scene treats
            // the side as `none` (no border painted) — the pre-border behaviour.
            PropertyId::BorderTopStyle
            | PropertyId::BorderRightStyle
            | PropertyId::BorderBottomStyle
            | PropertyId::BorderLeftStyle
            // `border-*-color` initial is `currentColor`. There is no computed
            // value for `currentColor`, so an absent color is left unresolved and
            // the scene falls back to the computed `color` when a style + width
            // are present (the CSS `currentColor` rule).
            | PropertyId::BorderTopColor
            | PropertyId::BorderRightColor
            | PropertyId::BorderBottomColor
            | PropertyId::BorderLeftColor
            // `outline-style` initial is `none` (its keyword needs the interner, so
            // an absent value is treated as `none` — no outline painted), and
            // `outline-color` initial is `currentColor` (an absent color falls back
            // to the computed `color`, like a border color).
            | PropertyId::OutlineStyle
            | PropertyId::OutlineColor
            // Keyword initials need the interner; deferred to a later phase.
            | PropertyId::Display
            | PropertyId::Visibility
            // `box-sizing` initial is `content-box`; its keyword initial likewise
            // falls back to the runtime's default (`BoxSizing::ContentBox`) when
            // absent — byte-for-byte identical to today's behaviour.
            | PropertyId::BoxSizing
            // `position` initial is `static`; its keyword initial likewise needs
            // the interner, so an absent value falls back to the runtime's default
            // (`Position::Static`) — byte-for-byte identical to today.
            | PropertyId::Position
            | PropertyId::FlexDirection
            // `flex-wrap` initial is `nowrap`; its keyword initial needs the
            // interner, so an absent value falls back to the layout default
            // (`FlexWrap::NoWrap`) — byte-for-byte identical to today.
            | PropertyId::FlexWrap
            | PropertyId::JustifyContent
            | PropertyId::AlignContent
            | PropertyId::AlignItems
            // The grid alignment keyword initials (`justify-items` / `align-items`
            // stretch, `justify-self` / `align-self` auto) likewise need the
            // interner, so an absent value falls back to the layout defaults
            // (`Stretch` / `Auto`) — byte-for-byte identical to today.
            | PropertyId::JustifyItems
            | PropertyId::AlignSelf
            | PropertyId::JustifySelf
            // `direction` (`ltr`) / `font-style` (`normal`) keyword initials are
            // likewise deferred, and `font-weight`'s initial (`normal` = 400) is
            // left unresolved so an absent value falls back to the shaping default
            // (also 400) — byte-for-byte identical to today's behaviour.
            | PropertyId::Direction
            | PropertyId::FontWeight
            | PropertyId::FontStyle
            // `grid-template-columns` / `grid-template-rows` initial is `none`
            // (no explicit tracks). There is no `GridTracks` value for `none`, so
            // an absent template is left unresolved and the runtime falls back to
            // the default single implicit column (`GridLayout::new(1)`) — exactly
            // the behaviour before authored templates were wired.
            | PropertyId::GridTemplateColumns
            | PropertyId::GridTemplateRows
            // `grid-auto-flow` initial is `row`; its keyword initial likewise
            // needs the interner, so an absent value falls back to the runtime's
            // default (`GridAutoFlow::Row`) — byte-for-byte identical to today.
            | PropertyId::GridAutoFlow
            // `grid-auto-columns` / `grid-auto-rows` initial is `auto`. There is no
            // no-interner `GridTracks` value for a single `auto` track, so an absent
            // value is left unresolved and the runtime falls back to the default
            // implicit-track sizing (`[Auto]`) — byte-for-byte identical to today.
            | PropertyId::GridAutoColumns
            | PropertyId::GridAutoRows
            // `grid-column-start` / `-end` / `grid-row-start` / `-end` initial is
            // `auto`. There is no `GridLine` value for `auto` (it is the enum's
            // implicit default), so an absent edge is left unresolved and the
            // runtime falls back to the all-`Auto` `GridPlacement` default — exactly
            // the behaviour before authored line placement was wired.
            | PropertyId::GridColumnStart
            | PropertyId::GridColumnEnd
            | PropertyId::GridRowStart
            | PropertyId::GridRowEnd
            // `grid-template-areas` initial is `none` (no named areas), and
            // `grid-area`'s named form is absent by default. Neither has a
            // no-interner initial value, so an absent value is left unresolved and
            // the runtime falls back to no named-area placement — byte-for-byte
            // identical to today's behaviour.
            | PropertyId::GridTemplateAreas
            | PropertyId::GridArea
            // `text-align` initial is `start`; its keyword initial likewise needs
            // the interner, so an absent value falls back to the runtime's default
            // (`Start`) — byte-for-byte identical to today's behaviour.
            | PropertyId::TextAlign
            // `line-height` initial is the keyword `normal`, which cannot be
            // expressed without the interner; left unresolved so an absent value
            // falls back to the font's natural (metric) line height in the
            // coordinator — byte-for-byte identical to pre-`line-height` behaviour.
            | PropertyId::LineHeight
            // `text-overflow` initial is `clip`; its keyword initial likewise needs
            // the interner, so an absent value falls back to the runtime's default
            // (`Clip`) — byte-for-byte identical to today's behaviour.
            | PropertyId::TextOverflow
            // `letter-spacing` / `word-spacing` initial is `normal` (≡ 0). An
            // absent value is left unresolved so the runtime's spacing helpers
            // default to `0.0` — no tracking / no word spacing, byte-for-byte
            // identical to before these properties existed (CSS Text 3 §8.1/§8.2).
            | PropertyId::LetterSpacing
            | PropertyId::WordSpacing
            // `text-transform` initial is `none`; its keyword initial needs the
            // interner, so an absent value falls back to the runtime's default
            // (`TextTransform::None`, the string shaped unchanged) — byte-for-byte
            // identical to before it existed (CSS Text 3 §2.1).
            | PropertyId::TextTransform
            // `white-space` initial is `normal`; its keyword initial needs the
            // interner, so an absent value falls back to the runtime's default
            // (`WhiteSpace::Normal`) — byte-for-byte identical to before it existed
            // (CSS Text 3 §3). `tab-size` initial is `8`, left unresolved so an
            // absent value falls back to the runtime's default tab stop (§4.3).
            | PropertyId::WhiteSpace
            | PropertyId::TabSize
            // `overflow-wrap` / `word-break` initial is `normal`; their keyword
            // initials need the interner, so an absent value falls back to the
            // runtime's default (`OverflowWrap::Normal` / `WordBreak::Normal`) —
            // byte-for-byte identical to before they existed (CSS Text 3 §5.2).
            | PropertyId::OverflowWrap
            | PropertyId::WordBreak
            // `overflow` initial is `visible` (no clip); its keyword initial
            // likewise needs the interner, so an absent value falls back to the
            // coordinator's default (no added clip) — byte-for-byte identical.
            | PropertyId::Overflow => None,
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

        // Step 1a: `direction` before anything else. A logical property maps onto
        // a physical one *through* the computed direction (spec §4.13), so it has
        // to be known before any box-model property is resolved — resolving it as
        // part of the loop below would make the answer depend on iteration order.
        let rtl = Self::resolves_rtl(cascaded, parent);

        // Steps 2–3: compute each property.
        for &property in COMPUTED_PROPERTIES {
            if let Some(value) = self.resolve_property(property, cascaded, &effective, parent, rtl) {
                style.set(property, value);
            }
        }

        style
    }

    /// Whether this element's computed `direction` is `rtl`.
    ///
    /// Cascaded value first, then the parent's computed one (`direction` inherits),
    /// then the CSS initial `ltr`. This is the same resolution the `Direction`
    /// property gets in the loop; it is done early because the logical-property
    /// mapping below depends on it.
    fn resolves_rtl(cascaded: &ResolvedDeclarations, parent: Option<&ComputedStyle>) -> bool {
        if let Some(PropertyValue::Keyword(symbol)) = cascaded.get(PropertyId::Direction) {
            return *symbol == layout_keyword::DIRECTION_RTL;
        }
        parent
            .and_then(|style| style.get(PropertyId::Direction))
            .and_then(|value| match value.value() {
                PropertyValue::Keyword(symbol) => Some(*symbol),
                _ => None,
            })
            .is_some_and(|symbol| symbol == layout_keyword::DIRECTION_RTL)
    }

    /// The cascaded value for `property`, honouring any **logical** declaration
    /// that maps onto it (spec §4.13).
    ///
    /// A logical property and its physical counterpart are separate `PropertyId`s
    /// that cascade independently, but CSS decides between them by ordinary cascade
    /// order. Both winners are therefore compared here, by the priority the cascade
    /// recorded — the later/stronger declaration wins, exactly as it would between
    /// two `margin-left` rules.
    ///
    /// Ties and missing priorities prefer the physical declaration: a value with no
    /// recorded priority did not come from a cascade (a test, a runtime override),
    /// and "the physical one" is the only honest answer when nothing states which
    /// was declared later.
    fn cascaded_for<'a>(
        cascaded: &'a ResolvedDeclarations,
        property: PropertyId,
        rtl: bool,
    ) -> Option<&'a PropertyValue> {
        let physical = cascaded.get(property);
        // Only a physical box-model property can have a logical counterpart, so
        // this scan is skipped entirely for the other ~80 properties.
        let logical = PropertyId::LOGICAL
            .iter()
            .copied()
            .filter(|candidate| candidate.physical_under(rtl) == Some(property))
            .filter_map(|candidate| Some((candidate, cascaded.get(candidate)?)))
            .max_by_key(|(candidate, _)| cascaded.priority_of(*candidate));
        let Some((candidate, logical_value)) = logical else {
            return physical;
        };
        if physical.is_none() {
            return Some(logical_value);
        }
        match (
            cascaded.priority_of(candidate),
            cascaded.priority_of(property),
        ) {
            (Some(logical_priority), Some(physical_priority))
                if logical_priority > physical_priority =>
            {
                Some(logical_value)
            }
            _ => physical,
        }
    }

    /// Resolves one property to a computed value, or [`None`] if it is left
    /// unresolved (deferred keyword initial).
    fn resolve_property(
        self,
        property: PropertyId,
        cascaded: &ResolvedDeclarations,
        custom: &HashMap<InternedSymbol, PropertyValue>,
        parent: Option<&ComputedStyle>,
        rtl: bool,
    ) -> Option<ComputedValue> {
        match Self::cascaded_for(cascaded, property, rtl) {
            Some(raw) => {
                // Step 2: substitute var() (unresolved -> Unset).
                let value = Self::substitute(raw, custom);
                // A relative `font-weight` (`bolder` / `lighter`) resolves here,
                // at the computed-value step, against the parent's already-
                // computed weight (§4.14) — never as a fixed number.
                if property == PropertyId::FontWeight
                    && let PropertyValue::Keyword(symbol) = value
                    && let Some(bolder) = relative_weight_is_bolder(symbol)
                {
                    let resolved =
                        resolve_relative_font_weight(parent_font_weight(parent), bolder);
                    return Some(ComputedValue::number(resolved));
                }
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

/// Classifies a reserved relative-`font-weight` handle: `Some(true)` for
/// `bolder`, `Some(false)` for `lighter`, and [`None`] for any other symbol.
fn relative_weight_is_bolder(symbol: InternedSymbol) -> Option<bool> {
    if symbol == layout_keyword::FONT_WEIGHT_BOLDER {
        Some(true)
    } else if symbol == layout_keyword::FONT_WEIGHT_LIGHTER {
        Some(false)
    } else {
        None
    }
}

/// Reads the parent's computed numeric `font-weight`, defaulting to
/// [`NORMAL_FONT_WEIGHT`] (400) when there is no parent or the parent did not
/// compute a numeric weight (the root's implicit parent is `normal`).
fn parent_font_weight(parent: Option<&ComputedStyle>) -> f32 {
    parent
        .and_then(|p| p.get(PropertyId::FontWeight))
        .and_then(|value| match value.value() {
            PropertyValue::Number(weight) => Some(*weight),
            _ => None,
        })
        .unwrap_or(NORMAL_FONT_WEIGHT)
}

/// Resolves a relative `font-weight` (`bolder` / `lighter`) against the parent's
/// computed weight, per the CSS fixed table (§4.14).
///
/// The parent weight is rounded to the nearest hundred and clamped to the
/// `100..=900` buckets; each bucket maps to a fixed `(bolder, lighter)` pair:
/// 100/200/300 → (400, 100), 400/500 → (700, 100), 600/700 → (900, 400),
/// 800/900 → (900, 700).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn resolve_relative_font_weight(parent_weight: f32, bolder: bool) -> f32 {
    let bucket = (parent_weight / 100.0).round().clamp(1.0, 9.0) as u16;
    let (heavier, lighter) = match bucket {
        1..=3 => (400u16, 100u16),
        4 | 5 => (700, 100),
        6 | 7 => (900, 400),
        // 8..=9 (the clamp guarantees the bucket is within `1..=9`).
        _ => (900, 700),
    };
    f32::from(if bolder { heavier } else { lighter })
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

    /// Reads a computed numeric `font-weight` off a resolved style.
    fn weight_of(style: &ComputedStyle) -> Option<f32> {
        match style.get(PropertyId::FontWeight).map(ComputedValue::value) {
            Some(PropertyValue::Number(n)) => Some(*n),
            _ => None,
        }
    }

    #[test]
    fn relative_font_weight_table() {
        // The nine CSS buckets, keyed on the parent's computed weight.
        assert_eq!(resolve_relative_font_weight(100.0, true), 400.0);
        assert_eq!(resolve_relative_font_weight(100.0, false), 100.0);
        assert_eq!(resolve_relative_font_weight(400.0, true), 700.0);
        assert_eq!(resolve_relative_font_weight(400.0, false), 100.0);
        assert_eq!(resolve_relative_font_weight(500.0, true), 700.0);
        assert_eq!(resolve_relative_font_weight(600.0, true), 900.0);
        assert_eq!(resolve_relative_font_weight(600.0, false), 400.0);
        assert_eq!(resolve_relative_font_weight(700.0, true), 900.0);
        assert_eq!(resolve_relative_font_weight(700.0, false), 400.0);
        assert_eq!(resolve_relative_font_weight(800.0, false), 700.0);
        assert_eq!(resolve_relative_font_weight(900.0, false), 700.0);
        // In-between values round to the nearest hundred bucket.
        assert_eq!(resolve_relative_font_weight(430.0, true), 700.0);
        // Out-of-range weights clamp into `100..=900`.
        assert_eq!(resolve_relative_font_weight(20.0, true), 400.0);
        assert_eq!(resolve_relative_font_weight(1500.0, false), 700.0);
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

        let computed = resolver.resolve(&cascaded, &HashMap::new(), Some(&parent));
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
        let computed = resolver.resolve(&cascaded, &HashMap::new(), None);
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
        let computed = resolver.resolve(&cascaded, &HashMap::new(), Some(&parent));
        assert_eq!(weight_of(&computed), Some(500.0));
    }

    #[test]
    fn absent_font_weight_inherits_parent_weight() {
        let resolver = StyleResolver::new();
        let mut parent = ComputedStyle::new();
        parent.set(PropertyId::FontWeight, ComputedValue::number(700.0));

        // `font-weight` inherits: a child with none takes the parent's weight.
        let computed = resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), Some(&parent));
        assert_eq!(weight_of(&computed), Some(700.0));
    }

    #[test]
    fn keyword_initial_is_left_unresolved() {
        let resolver = StyleResolver::new();
        assert_eq!(resolver.initial_value(PropertyId::Display), None);
        // With no cascaded value and no parent, `display` cannot be computed yet.
        let computed = resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), None);
        assert_eq!(computed.get(PropertyId::Display), None);
    }

    #[test]
    fn authored_z_index_computes_to_a_number() {
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::ZIndex, PropertyValue::Number(5.0));

        // An authored integer `z-index` flows through to the computed style as a
        // number the scene reads for stacking.
        let computed = resolver.resolve(&cascaded, &HashMap::new(), None);
        assert_eq!(
            computed.get(PropertyId::ZIndex).map(ComputedValue::value),
            Some(&PropertyValue::Number(5.0))
        );

        // `z-index` does not inherit: a child with no authored value takes the
        // initial (`auto`, left unresolved), NOT the parent's numeric level.
        let child = resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), Some(&computed));
        assert_eq!(child.get(PropertyId::ZIndex), None);
    }

    #[test]
    fn z_index_initial_is_auto_left_unresolved() {
        let resolver = StyleResolver::new();
        // `z-index` initial (`auto`) is deferred, so an absent value is unresolved.
        assert_eq!(resolver.initial_value(PropertyId::ZIndex), None);
        let computed = resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), None);
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
        let computed = resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), None);
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
        let computed = resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), None);
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
        let parent = resolver.resolve(&cascaded, &HashMap::new(), None);
        assert_eq!(
            parent
                .get(PropertyId::BorderTopLeftRadius)
                .and_then(ComputedValue::as_length),
            Some(Length::px(8.0))
        );
        let child = resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), Some(&parent));
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
        let computed = resolver.resolve(&cascaded, &HashMap::new(), None);
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
        let child =
            resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), Some(&computed));
        assert_eq!(child.get(PropertyId::BorderTopColor), None);
        assert_eq!(
            child
                .get(PropertyId::BorderTopWidth)
                .and_then(ComputedValue::as_length),
            Some(Length::px(0.0))
        );
    }

    /// Every modelled [`PropertyId`] must be in the computed set, or an authored
    /// value for it would be dropped at the computed-value step (the bug this
    /// closes). The `match` is exhaustive, so a new upstream variant fails to
    /// compile here until it is classified; the assertion then fails until it is
    /// also added to `COMPUTED_PROPERTIES`.
    #[test]
    fn computed_properties_cover_every_property_id() {
        // Exhaustive by construction: adding a `PropertyId` variant upstream
        // breaks this `match` until the new variant is listed below.
        const fn assert_known(id: PropertyId) {
            match id {
                PropertyId::Color
                | PropertyId::BackgroundColor
                | PropertyId::BackgroundImage
                | PropertyId::BackgroundSize
                | PropertyId::BackgroundPosition
                | PropertyId::BackgroundRepeat
                | PropertyId::ObjectFit
                | PropertyId::Opacity
                | PropertyId::Width
                | PropertyId::Height
                | PropertyId::BoxSizing
                | PropertyId::Display
                | PropertyId::Visibility
                | PropertyId::Position
                | PropertyId::Top
                | PropertyId::Right
                | PropertyId::Bottom
                | PropertyId::Left
                | PropertyId::ZIndex
                | PropertyId::MarginTop
                | PropertyId::MarginRight
                | PropertyId::MarginBottom
                | PropertyId::MarginLeft
                | PropertyId::PaddingTop
                | PropertyId::PaddingRight
                | PropertyId::PaddingBottom
                | PropertyId::PaddingLeft
                | PropertyId::BorderTopWidth
                | PropertyId::BorderRightWidth
                | PropertyId::BorderBottomWidth
                | PropertyId::BorderLeftWidth
                | PropertyId::BorderTopStyle
                | PropertyId::BorderRightStyle
                | PropertyId::BorderBottomStyle
                | PropertyId::BorderLeftStyle
                | PropertyId::BorderTopColor
                | PropertyId::BorderRightColor
                | PropertyId::BorderBottomColor
                | PropertyId::BorderLeftColor
                | PropertyId::BorderTopLeftRadius
                | PropertyId::BorderTopRightRadius
                | PropertyId::BorderBottomRightRadius
                | PropertyId::BorderBottomLeftRadius
                | PropertyId::OutlineWidth
                | PropertyId::OutlineStyle
                | PropertyId::OutlineColor
                | PropertyId::OutlineOffset
                | PropertyId::BoxShadow
                | PropertyId::Filter
                | PropertyId::BackdropFilter
                | PropertyId::Transform
                | PropertyId::TransformOrigin
                | PropertyId::MinWidth
                | PropertyId::MinHeight
                | PropertyId::MaxWidth
                | PropertyId::MaxHeight
                | PropertyId::Gap
                | PropertyId::FlexBasis
                | PropertyId::FlexGrow
                | PropertyId::FlexShrink
                | PropertyId::FlexDirection
                | PropertyId::FlexWrap
                | PropertyId::JustifyContent
                | PropertyId::AlignContent
                | PropertyId::AlignItems
                | PropertyId::JustifyItems
                | PropertyId::AlignSelf
                | PropertyId::JustifySelf
                | PropertyId::GridTemplateColumns
                | PropertyId::GridTemplateRows
                | PropertyId::GridTemplateAreas
                | PropertyId::GridAutoFlow
                | PropertyId::GridAutoColumns
                | PropertyId::GridAutoRows
                | PropertyId::GridColumnStart
                | PropertyId::GridColumnEnd
                | PropertyId::GridRowStart
                | PropertyId::GridRowEnd
                | PropertyId::GridArea
                | PropertyId::FontSize
                | PropertyId::FontFamily
                | PropertyId::Direction
                | PropertyId::FontWeight
                | PropertyId::FontStyle
                | PropertyId::LineHeight
                | PropertyId::TextAlign
                | PropertyId::TextOverflow
                | PropertyId::LetterSpacing
                | PropertyId::WordSpacing
                | PropertyId::TextTransform
                | PropertyId::WhiteSpace
                | PropertyId::OverflowWrap
                | PropertyId::WordBreak
                | PropertyId::TabSize
                | PropertyId::Overflow
                // Logical properties are classified but deliberately NOT computed:
                // each resolves into its physical counterpart via
                // `cascaded_for`, so the computed style holds `margin-left` and
                // every downstream phase reads that one place. The assertion below
                // exempts them for exactly this reason.
                | PropertyId::MarginInlineStart
                | PropertyId::MarginInlineEnd
                | PropertyId::MarginBlockStart
                | PropertyId::MarginBlockEnd
                | PropertyId::PaddingInlineStart
                | PropertyId::PaddingInlineEnd
                | PropertyId::PaddingBlockStart
                | PropertyId::PaddingBlockEnd
                | PropertyId::InsetInlineStart
                | PropertyId::InsetInlineEnd
                | PropertyId::InsetBlockStart
                | PropertyId::InsetBlockEnd => {}
            }
        }
        let all = [
            PropertyId::Color,
            PropertyId::BackgroundColor,
            PropertyId::BackgroundImage,
            PropertyId::BackgroundSize,
            PropertyId::BackgroundPosition,
            PropertyId::BackgroundRepeat,
            PropertyId::ObjectFit,
            PropertyId::Opacity,
            PropertyId::Width,
            PropertyId::Height,
            PropertyId::BoxSizing,
            PropertyId::Display,
            PropertyId::Visibility,
            PropertyId::Position,
            PropertyId::Top,
            PropertyId::Right,
            PropertyId::Bottom,
            PropertyId::Left,
            PropertyId::ZIndex,
            PropertyId::MarginTop,
            PropertyId::MarginRight,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
            PropertyId::PaddingTop,
            PropertyId::PaddingRight,
            PropertyId::PaddingBottom,
            PropertyId::PaddingLeft,
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
            PropertyId::BorderTopStyle,
            PropertyId::BorderRightStyle,
            PropertyId::BorderBottomStyle,
            PropertyId::BorderLeftStyle,
            PropertyId::BorderTopColor,
            PropertyId::BorderRightColor,
            PropertyId::BorderBottomColor,
            PropertyId::BorderLeftColor,
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
            PropertyId::OutlineWidth,
            PropertyId::OutlineStyle,
            PropertyId::OutlineColor,
            PropertyId::OutlineOffset,
            PropertyId::BoxShadow,
            PropertyId::Filter,
            PropertyId::BackdropFilter,
            PropertyId::Transform,
            PropertyId::TransformOrigin,
            PropertyId::MinWidth,
            PropertyId::MinHeight,
            PropertyId::MaxWidth,
            PropertyId::MaxHeight,
            PropertyId::Gap,
            PropertyId::FlexBasis,
            PropertyId::FlexGrow,
            PropertyId::FlexShrink,
            PropertyId::FlexDirection,
            PropertyId::FlexWrap,
            PropertyId::JustifyContent,
            PropertyId::AlignContent,
            PropertyId::AlignItems,
            PropertyId::JustifyItems,
            PropertyId::AlignSelf,
            PropertyId::JustifySelf,
            PropertyId::GridTemplateColumns,
            PropertyId::GridTemplateRows,
            PropertyId::GridTemplateAreas,
            PropertyId::GridAutoFlow,
            PropertyId::GridAutoColumns,
            PropertyId::GridAutoRows,
            PropertyId::GridColumnStart,
            PropertyId::GridColumnEnd,
            PropertyId::GridRowStart,
            PropertyId::GridRowEnd,
            PropertyId::GridArea,
            PropertyId::FontSize,
            PropertyId::FontFamily,
            PropertyId::Direction,
            PropertyId::FontWeight,
            PropertyId::FontStyle,
            PropertyId::LineHeight,
            PropertyId::TextAlign,
            PropertyId::TextOverflow,
            PropertyId::LetterSpacing,
            PropertyId::WordSpacing,
            PropertyId::TextTransform,
            PropertyId::WhiteSpace,
            PropertyId::OverflowWrap,
            PropertyId::WordBreak,
            PropertyId::TabSize,
            PropertyId::Overflow,
        ];
        for id in all {
            assert_known(id);
            assert!(
                COMPUTED_PROPERTIES.contains(&id),
                "{} is modelled but not computed — authored values would be dropped",
                id.name()
            );
            assert!(
                !id.is_logical(),
                "{} is logical: it must resolve INTO a physical property, not be \
                 computed alongside one — two sources of truth for one box edge",
                id.name()
            );
        }
        assert_eq!(COMPUTED_PROPERTIES.len(), all.len());
    }

    /// An authored typography value (previously dropped because the property was
    /// not in the computed set) now flows through the real resolver.
    #[test]
    fn authored_typography_flows_to_computed_style() {
        let resolver = StyleResolver::new();
        let mut cascaded = ResolvedDeclarations::new();
        cascaded.insert(PropertyId::FontSize, PropertyValue::Length(Length::px(24.0)));
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

        let computed = resolver.resolve(&cascaded, &HashMap::new(), None);

        assert_eq!(
            computed.get(PropertyId::FontSize).and_then(ComputedValue::as_length),
            Some(Length::px(24.0))
        );
        assert_eq!(
            computed
                .get(PropertyId::FontFamily)
                .and_then(|v| v.value().as_font_families().map(<[_]>::len)),
            Some(2)
        );
        assert_eq!(
            computed.get(PropertyId::FontStyle).map(ComputedValue::value),
            Some(&PropertyValue::Keyword(layout_keyword::FONT_STYLE_ITALIC))
        );
        assert_eq!(
            computed.get(PropertyId::Direction).map(ComputedValue::value),
            Some(&PropertyValue::Keyword(layout_keyword::DIRECTION_RTL))
        );
        assert_eq!(
            computed.get(PropertyId::TextAlign).map(ComputedValue::value),
            Some(&PropertyValue::Keyword(layout_keyword::TEXT_ALIGN_CENTER))
        );
        assert_eq!(
            computed.get(PropertyId::TextOverflow).map(ComputedValue::value),
            Some(&PropertyValue::Keyword(layout_keyword::TEXT_OVERFLOW_ELLIPSIS))
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
        cascaded.insert(PropertyId::MarginTop, PropertyValue::Length(Length::px(8.0)));
        cascaded.insert(PropertyId::MinWidth, PropertyValue::Length(Length::px(50.0)));

        let computed = resolver.resolve(&cascaded, &HashMap::new(), None);

        assert_eq!(
            computed.get(PropertyId::FlexDirection).map(ComputedValue::value),
            Some(&PropertyValue::Keyword(layout_keyword::FLEX_COLUMN))
        );
        assert_eq!(
            computed.get(PropertyId::MarginTop).and_then(ComputedValue::as_length),
            Some(Length::px(8.0))
        );
        assert_eq!(
            computed.get(PropertyId::MinWidth).and_then(ComputedValue::as_length),
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
        let computed = resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), None);

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
            computed.get(PropertyId::Width).and_then(ComputedValue::as_length),
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
            computed.get(PropertyId::FontSize).and_then(ComputedValue::as_length),
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

        let computed = resolver.resolve(&cascaded, &HashMap::new(), None);
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
        let computed = resolver.resolve(&cascaded, &HashMap::new(), None);
        let tracks = computed
            .get(PropertyId::GridAutoRows)
            .map(ComputedValue::value)
            .and_then(PropertyValue::as_grid_tracks)
            .expect("grid-auto-rows computed");
        assert_eq!(tracks, &[GridTrack::Fixed(Length::px(40.0))]);
        // Unauthored on a child: not inherited, so it is absent (runtime defaults
        // to the `[Auto]` implicit-track sizing).
        let child = resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), Some(&computed));
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

        let computed = resolver.resolve(&cascaded, &HashMap::new(), None);
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
        parent.set(PropertyId::FontSize, ComputedValue::length(Length::px(30.0)));

        let computed =
            resolver.resolve(&ResolvedDeclarations::new(), &HashMap::new(), Some(&parent));
        assert_eq!(
            computed.get(PropertyId::FontSize).and_then(ComputedValue::as_length),
            Some(Length::px(30.0))
        );
    }

    // ---- logical properties (spec §4.13) -----------------------------------

    mod logical {
        use super::super::StyleResolver;
        use crate::cascade::{CascadeOrigin, CascadePriority, ImportantFlag, ResolvedDeclarations};
        use crate::computed::{ComputedStyle, ComputedValue};
        use crate::property::{Length, PropertyId, PropertyValue, layout_keyword};
        use crate::selector::SelectorSpecificity;
        use std::collections::HashMap;

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
            StyleResolver::new().resolve(&with_direction, &HashMap::new(), None)
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
            assert_eq!(margin(&rtl, PropertyId::MarginRight), Some(Length::px(10.0)));
            assert_eq!(margin(&rtl, PropertyId::MarginLeft), Some(Length::px(0.0)));
        }

        #[test]
        fn the_block_axis_is_unmoved_by_direction() {
            // No `writing-mode` is modelled, so the block axis is always vertical:
            // `block-start` is `top` in either direction.
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
            assert_eq!(margin(&style, PropertyId::MarginLeft), Some(Length::px(1.0)));
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
            assert_eq!(margin(&style, PropertyId::MarginLeft), Some(Length::px(2.0)));
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
