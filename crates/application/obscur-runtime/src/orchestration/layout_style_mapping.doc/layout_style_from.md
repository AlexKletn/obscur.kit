Maps a single [`ComputedStyle`] into layout's own [`LayoutStyle`].

This is the full computed → layout bridge (spec §3.10, §4.2): every field of
[`LayoutStyle`] is populated from the corresponding computed property so that
flex/grid formatting, the box model, and the flex item/container knobs all
engage directly from CSS. A property that is absent (or has the wrong value
kind) falls back to the matching [`LayoutStyle::default`] value.

The mapping, field by field:

* `display` ← [`PropertyId::Display`] keyword. The computed value is a
  [`PropertyValue::Keyword`] backed by an [`InternedSymbol`]; it is compared
  against the reserved [`layout_keyword`] handles — `DISPLAY_FLEX` → `Flex`,
  `DISPLAY_GRID` → `Grid`, `DISPLAY_NONE` → `None`, everything else (including
  `DISPLAY_BLOCK` and an absent property) → `Block`. Because both the CSS
  compiler and this bridge use those reserved symbol values, the identities
  compare equal across crates without a shared interner.
* `width` / `height` / `min_width` / `min_height` / `max_width` /
  `max_height` ← the matching `PropertyId::{Width, Height, MinWidth,
  MinHeight, MaxWidth, MaxHeight}` length, via [`dimension_of`]
  ([`Length::Px`] → [`Dimension::Px`], [`Length::Percent`] →
  [`Dimension::Percent`], [`Length::Auto`]/absent → [`Dimension::Auto`]).
* `margin` / `padding` / `border` ← the four `*-top/right/bottom/left`
  longhands (the `border-*-width` longhands for `border`), resolved to pixel
  [`Insets`] via [`insets_of`]. Insets are pixel-only, so a `Percent`/`Auto`
  length contributes `0.0` on that edge. An unauthored `border-*-width`
  computes to its `0` initial, so `border` stays zero by default.
* `gap` ← [`PropertyId::Gap`] length in pixels (`Percent`/`Auto` → `0.0`).
* `flex_grow` / `flex_shrink` ← `PropertyId::{FlexGrow, FlexShrink}` numbers,
  defaulting to the [`LayoutStyle::default`] values (`0.0` / `1.0`).
* `flex_basis` ← [`PropertyId::FlexBasis`] via [`dimension_of`].
* `flex_direction` ← [`PropertyId::FlexDirection`] keyword against the
  reserved `FLEX_*` handles (default `Row`).
* `flex_wrap` ← [`PropertyId::FlexWrap`] keyword against the reserved
  `FLEX_WRAP_*` handles (default `NoWrap`, i.e. single-line).
* `justify_content` ← [`PropertyId::JustifyContent`] keyword against the
  reserved `JUSTIFY_*` handles (default `FlexStart`).
* `align_content` ← [`PropertyId::AlignContent`] keyword against the same
  reserved `JUSTIFY_*` handles (grid row-track distribution; default
  `FlexStart`).
* `align_items` ← [`PropertyId::AlignItems`] keyword against the reserved
  `ALIGN_*` handles (default `Stretch`).

[`Dimension::Px`]: obscur_layout::model::Dimension::Px
[`Dimension::Percent`]: obscur_layout::model::Dimension::Percent
[`Dimension::Auto`]: obscur_layout::model::Dimension::Auto
[`Insets`]: obscur_geometry::Insets
[`Length::Px`]: obscur_style::property::Length::Px
[`Length::Percent`]: obscur_style::property::Length::Percent
[`Length::Auto`]: obscur_style::property::Length::Auto
