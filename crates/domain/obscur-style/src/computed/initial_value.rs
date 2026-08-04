#![doc = include_str!("initial_value.doc/_module.md")]

use crate::computed::ComputedValue;
use crate::property::{Length, PropertyId, PropertyValue, StyleColor};

#[doc = include_str!("initial_value.doc/INITIAL_FONT_SIZE.md")]
pub(super) const INITIAL_FONT_SIZE: f32 = 16.0;

#[doc = include_str!("initial_value.doc/initial_value_of.md")]
//
// Several arms share a body (`Some(number(1.0))`, `None`) but are grouped by
// CSS meaning with distinct explanatory comments; merging them would obscure
// why each property is unresolved or shares an initial.
#[allow(clippy::match_same_arms)]
pub(super) fn initial_value_of(property: PropertyId) -> Option<ComputedValue> {
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
        | PropertyId::InsetBlockEnd
        | PropertyId::BorderInlineStartWidth
        | PropertyId::BorderInlineEndWidth
        | PropertyId::BorderBlockStartWidth
        | PropertyId::BorderBlockEndWidth
        | PropertyId::BorderInlineStartStyle
        | PropertyId::BorderInlineEndStyle
        | PropertyId::BorderBlockStartStyle
        | PropertyId::BorderBlockEndStyle => None,
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
        // `object-position` initial is `50% 50%` (centered) — NOT
        // `background-position`'s `0% 0%`. Resolved explicitly so an absent
        // property still centers the replaced content (CSS Images 3 §5.2).
        PropertyId::ObjectPosition => Some(
            ComputedValue::from_concrete(PropertyValue::BackgroundPosition(
                crate::property::BackgroundPosition::new(
                    crate::property::Length::Percent(50.0),
                    crate::property::Length::Percent(50.0),
                ),
            ))
            .expect("ObjectPosition is a concrete computed value"),
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
        // `mix-blend-mode` initial is the `normal` keyword, an interned symbol
        // Phase 1 cannot mint, so it is left unresolved (the coordinator treats
        // an absent value as `normal` and brackets nothing) — like `display`.
        PropertyId::MixBlendMode => None,
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
        // `font-size` initial is `medium` (~16px logical) — the same value the
        // root's implicit parent size takes, spelled once as `INITIAL_FONT_SIZE`.
        PropertyId::FontSize => Some(ComputedValue::length(Length::px(INITIAL_FONT_SIZE))),
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
        // `direction` (`ltr`) / `writing-mode` (`horizontal-tb`) / `font-style`
        // (`normal`) keyword initials are likewise deferred, and `font-weight`'s
        // initial (`normal` = 400) is left unresolved so an absent value falls
        // back to the shaping default (also 400) — byte-for-byte identical to
        // today's behaviour. `writing-mode`'s absent value maps to
        // `horizontal-tb` in `resolves_writing_mode` / the layout bridge.
        | PropertyId::Direction
        | PropertyId::WritingMode
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
        | PropertyId::Overflow
        // `content` is never a real element's computed value (it decorates a
        // generated pseudo-element box only), so it has no element initial.
        | PropertyId::Content
        // No painter. The initial value is the keyword `none`, which needs the
        // interner, so an absent value means the same thing: nothing to draw.
        | PropertyId::Painter => None,
    }
}
