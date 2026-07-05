//! The typed value a property can hold.
//!
//! [`PropertyValue`] is the typed result the cascade works with — never CSS
//! source text. Keyword-valued properties (for example `display: block`) store
//! their keyword as an [`InternedSymbol`] handle, not a string: matching is done
//! on the interned identity, not by comparing characters.
//!
//! Two kinds of value are resolved *before* computed values are produced:
//!
//! * the CSS-wide keywords [`PropertyValue::Inherit`], [`PropertyValue::Initial`]
//!   and [`PropertyValue::Unset`], and
//! * an unresolved custom-property reference [`PropertyValue::CustomRef`]
//!   (`var(--name)`), which is substituted during custom-property resolution
//!   (§4.14) before computed-value resolution runs.
//!
//! `calc()` / `min()` / `max()` / `clamp()` and relative/viewport units are
//! modelled: a plain relative unit is a [`PropertyValue::Length`] carrying the
//! relevant [`Length`] variant, and a math function is a
//! [`PropertyValue::Calc`] holding a symbolic [`LengthExpr`]. Both resolve to
//! concrete pixels at the layout/compute step against a
//! [`LengthContext`](crate::property::LengthContext) (spec §4.5, §4.13).

use crate::property::{GridTrack, Length, LengthExpr, StyleColor};
use obscur_identity::InternedSymbol;

/// A typed CSS property value.
///
/// Because [`PropertyValue::Number`], [`PropertyValue::Length`] and
/// [`PropertyValue::Color`] carry `f32` data, this type derives [`PartialEq`]
/// but not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// A keyword value, identified by its interned symbol (for example the
    /// symbols for `block`, `flex` or `hidden`). The symbol is an identity
    /// handle, not a string to be matched character-by-character.
    Keyword(InternedSymbol),
    /// A length value (see [`Length`]).
    Length(Length),
    /// A symbolic `calc()` / `min()` / `max()` / `clamp()` length expression
    /// (see [`LengthExpr`]), resolved to concrete pixels at the layout/compute
    /// step against a [`LengthContext`](crate::property::LengthContext).
    Calc(LengthExpr),
    /// A bare number (for example the value of `opacity`).
    Number(f32),
    /// A color value (see [`StyleColor`]).
    Color(StyleColor),
    /// An ordered `font-family` list, in author order. Each entry is either a
    /// concrete family name (for example `"Helvetica Neue"`) or a generic
    /// keyword (`"serif"`, `"sans-serif"`, `"monospace"`, `"cursive"`,
    /// `"fantasy"`, `"system-ui"`). Family names are stored **as strings**, not
    /// interned symbols: the runtime matches them against its font registry by
    /// string, so this value crosses the CSS/runtime interner boundary without a
    /// shared interner.
    FontFamilies(Box<[Box<str>]>),
    /// A grid track list, in author order: the value of `grid-template-columns`
    /// or `grid-template-rows`. Each entry is a [`GridTrack`] sizing function
    /// (`100px`, `1fr`, `auto`, `minmax(…)`); `repeat(…)` is expanded to its flat
    /// track list before this value is built, so this list is already flat. The
    /// runtime converts each [`GridTrack`] to `obscur_layout`'s `TrackSize` when
    /// it bridges the computed style into a grid layout input (spec §4.1).
    GridTracks(Box<[GridTrack]>),
    /// An unresolved `var(--name)` reference to a custom property, named by its
    /// interned symbol. This is a pre-resolution placeholder: it is substituted
    /// with the referenced custom property's value before computed-value
    /// resolution (§4.14).
    CustomRef(InternedSymbol),
    /// The CSS-wide `inherit` keyword.
    Inherit,
    /// The CSS-wide `initial` keyword.
    Initial,
    /// The CSS-wide `unset` keyword (behaves as `inherit` for inherited
    /// properties and `initial` otherwise).
    Unset,
}

impl PropertyValue {
    /// Returns the color if this value is a [`PropertyValue::Color`].
    #[must_use]
    pub const fn as_color(&self) -> Option<StyleColor> {
        match self {
            Self::Color(color) => Some(*color),
            _ => None,
        }
    }

    /// Builds a [`PropertyValue::FontFamilies`] from an ordered iterator of
    /// family / generic-keyword names, preserving author order.
    #[must_use]
    pub fn font_families<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        Self::FontFamilies(names.into_iter().map(Into::into).collect())
    }

    /// Returns the ordered `font-family` list if this value is a
    /// [`PropertyValue::FontFamilies`].
    #[must_use]
    pub fn as_font_families(&self) -> Option<&[Box<str>]> {
        match self {
            Self::FontFamilies(names) => Some(names),
            _ => None,
        }
    }

    /// Builds a [`PropertyValue::GridTracks`] from an ordered iterator of
    /// [`GridTrack`]s, preserving author order.
    #[must_use]
    pub fn grid_tracks<I>(tracks: I) -> Self
    where
        I: IntoIterator<Item = GridTrack>,
    {
        Self::GridTracks(tracks.into_iter().collect())
    }

    /// Returns the ordered grid track list if this value is a
    /// [`PropertyValue::GridTracks`].
    #[must_use]
    pub fn as_grid_tracks(&self) -> Option<&[GridTrack]> {
        match self {
            Self::GridTracks(tracks) => Some(tracks),
            _ => None,
        }
    }

    /// Returns the length if this value is a [`PropertyValue::Length`].
    #[must_use]
    pub const fn as_length(&self) -> Option<Length> {
        match self {
            Self::Length(length) => Some(*length),
            _ => None,
        }
    }

    /// Returns the length expression if this value is a [`PropertyValue::Calc`].
    #[must_use]
    pub const fn as_calc(&self) -> Option<&LengthExpr> {
        match self {
            Self::Calc(expr) => Some(expr),
            _ => None,
        }
    }

    /// Returns `true` when this value is one of the CSS-wide keywords
    /// (`inherit`, `initial`, `unset`).
    #[must_use]
    pub const fn is_wide_keyword(&self) -> bool {
        matches!(self, Self::Inherit | Self::Initial | Self::Unset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_color_extracts_only_colors() {
        let value = PropertyValue::Color(StyleColor::BLACK);
        assert_eq!(value.as_color(), Some(StyleColor::BLACK));
        assert_eq!(PropertyValue::Number(1.0).as_color(), None);
    }

    #[test]
    fn as_length_extracts_only_lengths() {
        let value = PropertyValue::Length(Length::px(8.0));
        assert_eq!(value.as_length(), Some(Length::px(8.0)));
        assert_eq!(PropertyValue::Number(1.0).as_length(), None);
    }

    #[test]
    fn wide_keyword_detection() {
        assert!(PropertyValue::Inherit.is_wide_keyword());
        assert!(PropertyValue::Initial.is_wide_keyword());
        assert!(PropertyValue::Unset.is_wide_keyword());
        assert!(!PropertyValue::Number(0.5).is_wide_keyword());
        assert!(!PropertyValue::CustomRef(InternedSymbol::from_raw(1)).is_wide_keyword());
    }

    #[test]
    fn font_families_round_trips_in_order() {
        let value = PropertyValue::font_families(["Foo", "Arial", "sans-serif"]);
        let names = value.as_font_families().expect("font families");
        let collected: Vec<&str> = names.iter().map(|n| &**n).collect();
        assert_eq!(collected, ["Foo", "Arial", "sans-serif"]);
        // Other value kinds are not font families.
        assert_eq!(PropertyValue::Number(1.0).as_font_families(), None);
    }

    #[test]
    fn grid_tracks_round_trips_in_order() {
        use crate::property::GridTrack;
        let value = PropertyValue::grid_tracks([
            GridTrack::Fixed(Length::px(100.0)),
            GridTrack::Fr(1.0),
            GridTrack::Auto,
        ]);
        let tracks = value.as_grid_tracks().expect("grid tracks");
        assert_eq!(tracks.len(), 3);
        assert_eq!(tracks[0], GridTrack::Fixed(Length::px(100.0)));
        assert_eq!(tracks[1], GridTrack::Fr(1.0));
        assert_eq!(tracks[2], GridTrack::Auto);
        // Other value kinds are not grid tracks.
        assert_eq!(PropertyValue::Number(1.0).as_grid_tracks(), None);
    }

    #[test]
    fn custom_ref_carries_symbol() {
        let name = InternedSymbol::from_raw(42);
        assert_eq!(
            PropertyValue::CustomRef(name),
            PropertyValue::CustomRef(name)
        );
    }
}
