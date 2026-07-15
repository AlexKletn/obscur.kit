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

use crate::property::{
    BackgroundLayer, BackgroundPosition, BackgroundRepeat, BackgroundSize, BoxShadow,
    ConicGradient, FilterOp, GridAreaTemplate, GridLineValue, GridTrack, GridTrackList, Length,
    LengthExpr, LinearGradient, ObjectFit, RadialGradient, StyleColor, Transform, TransformOrigin,
};
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
    /// A `linear-gradient(...)` background image (see [`LinearGradient`]): the
    /// value of `background-image` when it is an authored linear gradient. The
    /// gradient carries its angle (CSS convention) and resolved color stops; the
    /// scene coordinator turns the angle plus the painted box's bounds into the
    /// concrete gradient axis when it builds the fill brush (spec §5.2).
    LinearGradient(LinearGradient),
    /// A `radial-gradient(...)` background image (see [`RadialGradient`]): the
    /// value of `background-image` when it is an authored radial gradient. The
    /// gradient carries its box-relative shape / size / center and resolved color
    /// stops; the scene coordinator turns those plus the painted box's bounds into
    /// the concrete center and radius when it builds the fill brush (spec §5.2).
    RadialGradient(RadialGradient),
    /// A `conic-gradient(...)` background image (see [`ConicGradient`]): the value
    /// of `background-image` when it is an authored conic gradient. The gradient
    /// carries its starting angle (CSS convention), box-relative center, and
    /// resolved color stops; the scene coordinator turns those plus the painted
    /// box's bounds into the concrete center and sweep start angle when it builds
    /// the fill brush (spec §5.2).
    ConicGradient(ConicGradient),
    /// A `url(...)` background image: the value of `background-image` when it is
    /// an authored image reference. The payload is the **raw** URL/path string
    /// exactly as authored (unquoted or with its quotes stripped), *not* resolved
    /// against any base — the runtime hands it to a `ResourceLoader` /
    /// `ResourceDecoder` and, once decoded, the scene coordinator paints the
    /// resulting image stretched over the node's box (spec §5.2, §5.16). A URL
    /// whose image is not yet loaded simply paints nothing (the background-color
    /// shows through).
    BackgroundImageUrl(Box<str>),
    /// A multi-layer `background-image` list: two or more comma-separated image
    /// layers (see [`BackgroundLayer`]), in CSS author order (first = top). CSS
    /// paints layers back-to-front, so the scene coordinator emits the list in
    /// reverse (last-listed = bottom, painted first). The **one-layer** case is
    /// never this variant — it keeps the flat single-value variant
    /// ([`LinearGradient`](Self::LinearGradient) / [`RadialGradient`](Self::RadialGradient)
    /// / [`ConicGradient`](Self::ConicGradient) / [`BackgroundImageUrl`](Self::BackgroundImageUrl))
    /// so a single-layer background stays byte-identical (spec §5.2).
    BackgroundLayers(Box<[BackgroundLayer]>),
    /// A `background-size` value (see [`BackgroundSize`]): how one copy of the
    /// `background-image` is scaled (`cover` / `contain` / `auto` / explicit
    /// per-axis lengths). The scene coordinator resolves it against the box and
    /// the image's intrinsic size to a concrete tile width/height at paint time.
    BackgroundSize(BackgroundSize),
    /// A `background-position` value (see [`BackgroundPosition`]): where the sized
    /// image tile's origin sits within the box, one length/percentage per axis.
    BackgroundPosition(BackgroundPosition),
    /// A `background-repeat` value (see [`BackgroundRepeat`]): whether the sized
    /// image tile is tiled along each axis to fill the box.
    BackgroundRepeat(BackgroundRepeat),
    /// An `object-fit` value (see [`ObjectFit`]): how a replaced element's content
    /// (e.g. an `<img>`'s pixels) is scaled within its content box. The scene
    /// coordinator resolves it against the box and the image's intrinsic size to a
    /// concrete tile rect at paint time.
    ObjectFit(ObjectFit),
    /// An ordered `box-shadow` list, in author order: the value of `box-shadow`.
    /// Each [`BoxShadow`] carries its offsets, blur, spread, optional color, and
    /// `inset` flag; `box-shadow: none` lowers to an **empty** list. CSS paints
    /// the first-listed shadow on top, so the scene coordinator emits the list in
    /// reverse order (spec §5.2).
    BoxShadow(Box<[BoxShadow]>),
    /// An ordered `filter` op list, in author order: the value of `filter` reduced
    /// to the functions the pipeline can honor. Each entry is a [`FilterOp`] —
    /// either a `drop-shadow(...)` (painted BEHIND the element like a `box-shadow`
    /// drop shadow) or a `blur(<length>)` (a real gaussian over the element's
    /// composited subtree on the GPU backend). Order is preserved. Other filter
    /// functions (`brightness()`, `contrast()`, color-matrix, …) are out of scope
    /// and dropped at lower time; `filter: none` / empty lowers to an **empty**
    /// list (painting nothing).
    Filter(Box<[FilterOp]>),
    /// An ordered `font-family` list, in author order. Each entry is either a
    /// concrete family name (for example `"Helvetica Neue"`) or a generic
    /// keyword (`"serif"`, `"sans-serif"`, `"monospace"`, `"cursive"`,
    /// `"fantasy"`, `"system-ui"`). Family names are stored **as strings**, not
    /// interned symbols: the runtime matches them against its font registry by
    /// string, so this value crosses the CSS/runtime interner boundary without a
    /// shared interner.
    FontFamilies(Box<[Box<str>]>),
    /// A grid track list, in author order: the value of `grid-template-columns`
    /// or `grid-template-rows`. The [`GridTrackList`] carries the [`GridTrack`]
    /// sizing functions (`100px`, `1fr`, `auto`, `minmax(…)`; `repeat(…)` already
    /// expanded to a flat list) plus the `[name]` names on each grid line. The
    /// runtime converts each [`GridTrack`] to `obscur_layout`'s `TrackSize`, and
    /// resolves a child's named-line placement against the line names, when it
    /// bridges the computed style into a grid layout input (spec §4.1, §4.13).
    GridTracks(GridTrackList),
    /// A single grid line placement edge: the value of one `grid-column-start` /
    /// `grid-column-end` / `grid-row-start` / `grid-row-end` longhand. The
    /// [`GridLineValue`] is a style-local line edge (`auto`, a 1-based line index,
    /// or `span N`); the runtime converts it to `obscur_layout`'s `GridLine` and
    /// assembles the four edges into a `GridPlacement` when it bridges the computed
    /// style into a grid layout input (spec §4.13).
    GridLine(GridLineValue),
    /// A parsed `grid-template-areas` grid: the value of a grid container's
    /// `grid-template-areas`. The [`GridAreaTemplate`] carries every named area's
    /// line-number rectangle (already validated as a filled rectangle at parse
    /// time); the runtime looks a child's `grid-area: <name>` up against its
    /// container's template to resolve a concrete placement (spec §4.2).
    GridAreas(GridAreaTemplate),
    /// A `grid-area: <custom-ident>` named-area reference on a grid item: the area
    /// name to resolve against the item's container `grid-template-areas`. The name
    /// is kept **as a string** (not an interned symbol) because it is matched
    /// against the container template's area names in the runtime, which does not
    /// share this crate's interner (spec §4.2). The numeric `grid-area:
    /// a / b / c / d` line form does not produce this value; it expands to the four
    /// [`GridLine`](PropertyValue::GridLine) longhands instead.
    GridAreaName(Box<str>),
    /// A resolved CSS 2D `transform`: the authored `<transform-function>+` list
    /// composed left-to-right into a single 2x3 affine (see [`Transform`]).
    /// `transform: none` lowers to an absent value, not this. The scene
    /// coordinator wraps the affine about the element's `transform-origin` and
    /// brackets the subtree's paint with it (spec §5.2).
    Transform(Transform),
    /// A resolved CSS `transform-origin` (see [`TransformOrigin`]): the point a
    /// `transform` pivots about, defaulting to the box center (`50% 50%`).
    TransformOrigin(TransformOrigin),
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
    /// Returns `true` when resolving this value needs the `0` glyph measured —
    /// that is, when it contains a `ch` term anywhere.
    ///
    /// # Why a caller asks first
    ///
    /// Every other unit folds from numbers the caller already has. `ch` alone
    /// needs text *shaped* to answer, which costs a font-fallback chain and a
    /// shaping call. Since almost no stylesheet writes `ch`, a caller checks this
    /// first and measures only on a hit; the overwhelmingly common answer is
    /// `false`, and this scan allocates nothing to produce it.
    ///
    /// # The arms that answer `false` while carrying lengths
    ///
    /// [`BoxShadow`](Self::BoxShadow), [`BackgroundSize`](Self::BackgroundSize)
    /// and friends do hold [`Length`]s — including, in principle, `ch` ones — and
    /// this still answers `false` for them. That is a **known gap**, not a
    /// derivation from anything: `20ch` inside a `box-shadow` offset resolves to
    /// `0`, silently.
    ///
    /// The gap is one-directional and worth stating exactly, because the honest
    /// answer is not the tidy one. `em` / `rem` / `vw` in those same properties
    /// *are* honored: the computed-value step absolutizes every length a
    /// [`PropertyValue`] carries, however nested. `ch` is the sole unit it cannot
    /// fold there, because it needs the `0` glyph shaped and the computed-value
    /// step has no shaper in reach. Only the callers that *can* measure — layout,
    /// via [`ComputedStyle::uses_ch`](crate::computed::ComputedStyle::uses_ch) —
    /// resolve `ch` at all, and they consume the value kinds below in already-
    /// absolutized form without re-inspecting them for `ch`.
    ///
    /// So widening this predicate alone would buy nothing: a `true` here would
    /// cost a shaping call whose measurement no consumer of those variants reads.
    /// Closing the gap means threading a measurement into the computed-value step,
    /// at which point these arms change with it.
    ///
    /// The match is deliberately exhaustive rather than wildcarded, so a new
    /// length-bearing variant cannot quietly default to "no `ch` here".
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        match self {
            Self::Length(length) => length.is_ch(),
            Self::Calc(expr) => expr.uses_ch(),
            Self::GridTracks(tracks) => tracks.uses_ch(),
            // Carrying no length at all, or carrying lengths no `ch`-measuring
            // caller reads back. See the note above.
            Self::Keyword(_)
            | Self::Number(_)
            | Self::Color(_)
            | Self::LinearGradient(_)
            | Self::RadialGradient(_)
            | Self::ConicGradient(_)
            | Self::BackgroundImageUrl(_)
            | Self::BackgroundLayers(_)
            | Self::BackgroundSize(_)
            | Self::BackgroundPosition(_)
            | Self::BackgroundRepeat(_)
            | Self::ObjectFit(_)
            | Self::BoxShadow(_)
            | Self::Filter(_)
            | Self::FontFamilies(_)
            | Self::GridLine(_)
            | Self::GridAreas(_)
            | Self::GridAreaName(_)
            | Self::Transform(_)
            | Self::TransformOrigin(_)
            | Self::CustomRef(_)
            | Self::Inherit
            | Self::Initial
            | Self::Unset => false,
        }
    }

    /// Returns the color if this value is a [`PropertyValue::Color`].
    #[must_use]
    pub const fn as_color(&self) -> Option<StyleColor> {
        match self {
            Self::Color(color) => Some(*color),
            _ => None,
        }
    }

    /// Returns the linear gradient if this value is a
    /// [`PropertyValue::LinearGradient`].
    #[must_use]
    pub const fn as_linear_gradient(&self) -> Option<&LinearGradient> {
        match self {
            Self::LinearGradient(gradient) => Some(gradient),
            _ => None,
        }
    }

    /// Returns the radial gradient if this value is a
    /// [`PropertyValue::RadialGradient`].
    #[must_use]
    pub const fn as_radial_gradient(&self) -> Option<&RadialGradient> {
        match self {
            Self::RadialGradient(gradient) => Some(gradient),
            _ => None,
        }
    }

    /// Returns the conic gradient if this value is a
    /// [`PropertyValue::ConicGradient`].
    #[must_use]
    pub const fn as_conic_gradient(&self) -> Option<&ConicGradient> {
        match self {
            Self::ConicGradient(gradient) => Some(gradient),
            _ => None,
        }
    }

    /// Returns the raw `url(...)` string if this value is a
    /// [`PropertyValue::BackgroundImageUrl`] (the authored URL/path, unresolved).
    #[must_use]
    pub fn as_background_image_url(&self) -> Option<&str> {
        match self {
            Self::BackgroundImageUrl(url) => Some(url),
            _ => None,
        }
    }

    /// Returns the ordered multi-layer `background-image` list if this value is a
    /// [`PropertyValue::BackgroundLayers`] (two or more layers, in author order).
    ///
    /// A single-layer background is *not* this variant (it stays a flat
    /// gradient/url value), so callers that want to treat both cases uniformly
    /// must also check the single accessors.
    #[must_use]
    pub fn as_background_layers(&self) -> Option<&[BackgroundLayer]> {
        match self {
            Self::BackgroundLayers(layers) => Some(layers),
            _ => None,
        }
    }

    /// Returns the `background-size` if this value is a
    /// [`PropertyValue::BackgroundSize`].
    #[must_use]
    pub const fn as_background_size(&self) -> Option<BackgroundSize> {
        match self {
            Self::BackgroundSize(size) => Some(*size),
            _ => None,
        }
    }

    /// Returns the `background-position` if this value is a
    /// [`PropertyValue::BackgroundPosition`].
    #[must_use]
    pub const fn as_background_position(&self) -> Option<BackgroundPosition> {
        match self {
            Self::BackgroundPosition(position) => Some(*position),
            _ => None,
        }
    }

    /// Returns the `background-repeat` if this value is a
    /// [`PropertyValue::BackgroundRepeat`].
    #[must_use]
    pub const fn as_background_repeat(&self) -> Option<BackgroundRepeat> {
        match self {
            Self::BackgroundRepeat(repeat) => Some(*repeat),
            _ => None,
        }
    }

    /// Returns the `object-fit` if this value is a [`PropertyValue::ObjectFit`].
    #[must_use]
    pub const fn as_object_fit(&self) -> Option<ObjectFit> {
        match self {
            Self::ObjectFit(fit) => Some(*fit),
            _ => None,
        }
    }

    /// Returns the ordered `box-shadow` list if this value is a
    /// [`PropertyValue::BoxShadow`] (an empty slice for `box-shadow: none`).
    #[must_use]
    pub fn as_box_shadows(&self) -> Option<&[BoxShadow]> {
        match self {
            Self::BoxShadow(shadows) => Some(shadows),
            _ => None,
        }
    }

    /// Returns the ordered `filter` op list if this value is a
    /// [`PropertyValue::Filter`] (an empty slice for `filter: none` or a filter
    /// with no honored functions). Each entry is a [`FilterOp`]
    /// (a `drop-shadow(...)` or a `blur(<length>)`), in author order.
    #[must_use]
    pub fn as_filter_ops(&self) -> Option<&[FilterOp]> {
        match self {
            Self::Filter(ops) => Some(ops),
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
    /// [`GridTrack`]s (with no line names), preserving author order.
    #[must_use]
    pub fn grid_tracks<I>(tracks: I) -> Self
    where
        I: IntoIterator<Item = GridTrack>,
    {
        Self::GridTracks(GridTrackList::from_tracks(tracks))
    }

    /// Builds a [`PropertyValue::GridTracks`] from a [`GridTrackList`] (tracks +
    /// `[name]` line names).
    #[must_use]
    pub const fn grid_track_list(list: GridTrackList) -> Self {
        Self::GridTracks(list)
    }

    /// Returns the ordered grid track slice if this value is a
    /// [`PropertyValue::GridTracks`].
    #[must_use]
    pub fn as_grid_tracks(&self) -> Option<&[GridTrack]> {
        match self {
            Self::GridTracks(list) => Some(list.tracks()),
            _ => None,
        }
    }

    /// Returns the full grid track list (tracks + line names) if this value is a
    /// [`PropertyValue::GridTracks`].
    #[must_use]
    pub const fn as_grid_track_list(&self) -> Option<&GridTrackList> {
        match self {
            Self::GridTracks(list) => Some(list),
            _ => None,
        }
    }

    /// Returns the grid line edge if this value is a [`PropertyValue::GridLine`].
    ///
    /// Returns a borrow so a [`GridLineValue::Named`] name is read without an
    /// allocation; numeric / span / auto edges are the common case.
    #[must_use]
    pub fn as_grid_line(&self) -> Option<&GridLineValue> {
        match self {
            Self::GridLine(line) => Some(line),
            _ => None,
        }
    }

    /// Returns the parsed grid-area template if this value is a
    /// [`PropertyValue::GridAreas`].
    #[must_use]
    pub const fn as_grid_areas(&self) -> Option<&GridAreaTemplate> {
        match self {
            Self::GridAreas(template) => Some(template),
            _ => None,
        }
    }

    /// Returns the named-area reference if this value is a
    /// [`PropertyValue::GridAreaName`].
    #[must_use]
    pub fn as_grid_area_name(&self) -> Option<&str> {
        match self {
            Self::GridAreaName(name) => Some(name),
            _ => None,
        }
    }

    /// Returns the resolved transform if this value is a
    /// [`PropertyValue::Transform`].
    #[must_use]
    pub const fn as_transform(&self) -> Option<Transform> {
        match self {
            Self::Transform(transform) => Some(*transform),
            _ => None,
        }
    }

    /// Returns the transform origin if this value is a
    /// [`PropertyValue::TransformOrigin`].
    #[must_use]
    pub const fn as_transform_origin(&self) -> Option<TransformOrigin> {
        match self {
            Self::TransformOrigin(origin) => Some(*origin),
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
    fn uses_ch_fires_only_on_values_that_resolve_against_a_length_context() {
        use crate::property::{GridTrack, GridTrackList, Length, LengthExpr};

        assert!(PropertyValue::Length(Length::Ch(20.0)).uses_ch());
        assert!(PropertyValue::Calc(LengthExpr::leaf(Length::Ch(1.0))).uses_ch());
        assert!(
            PropertyValue::GridTracks(GridTrackList::new(
                vec![GridTrack::Fixed(Length::Ch(20.0))].into(),
                Vec::new().into(),
            ))
            .uses_ch()
        );

        // Other units must not fire: a false positive costs a shaping call on
        // every ordinary stylesheet.
        assert!(!PropertyValue::Length(Length::Em(2.0)).uses_ch());
        assert!(!PropertyValue::Length(Length::px(10.0)).uses_ch());
        assert!(!PropertyValue::Number(1.0).uses_ch());
        assert!(!PropertyValue::Initial.uses_ch());
    }

    #[test]
    fn uses_ch_is_false_for_a_box_shadow_which_is_a_known_gap() {
        use crate::property::{BoxShadow, Length};

        // Pinned as a GAP, not as intended behaviour: a `ch` inside a box-shadow
        // resolves to 0. `em`/`rem` here are absolutized at computed-value time
        // and do work; `ch` alone cannot be, because that step has no shaper.
        // Answering `true` would cost a shaping call no consumer of this variant
        // reads back, so the fix is upstream — see `uses_ch`'s docs. When a
        // measurement reaches the computed-value step, this test should flip.
        let shadow = PropertyValue::BoxShadow(
            vec![BoxShadow {
                offset_x: Length::Ch(1.0),
                offset_y: Length::Ch(1.0),
                blur: Length::px(0.0),
                spread: Length::px(0.0),
                color: None,
                inset: false,
            }]
            .into_boxed_slice(),
        );
        assert!(!shadow.uses_ch());
    }

    #[test]
    fn as_color_extracts_only_colors() {
        let value = PropertyValue::Color(StyleColor::BLACK);
        assert_eq!(value.as_color(), Some(StyleColor::BLACK));
        assert_eq!(PropertyValue::Number(1.0).as_color(), None);
    }

    #[test]
    fn as_linear_gradient_extracts_only_gradients() {
        use crate::property::{GradientStop, LinearGradient};
        let gradient = LinearGradient::new(
            90.0,
            vec![
                GradientStop::new(StyleColor::BLACK, 0.0),
                GradientStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 1.0),
            ],
        );
        let value = PropertyValue::LinearGradient(gradient.clone());
        assert_eq!(value.as_linear_gradient(), Some(&gradient));
        assert_eq!(PropertyValue::Number(1.0).as_linear_gradient(), None);
    }

    #[test]
    fn as_radial_gradient_extracts_only_radial_gradients() {
        use crate::property::{GradientStop, RadialGradient, RadialShape, RadialSize};
        let gradient = RadialGradient::new(
            RadialShape::Ellipse,
            RadialSize::FarthestCorner,
            Length::Percent(50.0),
            Length::Percent(50.0),
            vec![
                GradientStop::new(StyleColor::BLACK, 0.0),
                GradientStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 1.0),
            ],
        );
        let value = PropertyValue::RadialGradient(gradient.clone());
        assert_eq!(value.as_radial_gradient(), Some(&gradient));
        assert_eq!(PropertyValue::Number(1.0).as_radial_gradient(), None);
        assert_eq!(value.as_linear_gradient(), None);
    }

    #[test]
    fn as_conic_gradient_extracts_only_conic_gradients() {
        use crate::property::{ConicGradient, GradientStop};
        let gradient = ConicGradient::new(
            90.0,
            Length::Percent(50.0),
            Length::Percent(50.0),
            vec![
                GradientStop::new(StyleColor::BLACK, 0.0),
                GradientStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 1.0),
            ],
        );
        let value = PropertyValue::ConicGradient(gradient.clone());
        assert_eq!(value.as_conic_gradient(), Some(&gradient));
        assert_eq!(PropertyValue::Number(1.0).as_conic_gradient(), None);
        assert_eq!(value.as_radial_gradient(), None);
    }

    #[test]
    fn as_background_image_url_extracts_only_urls() {
        let value = PropertyValue::BackgroundImageUrl("foo.png".into());
        assert_eq!(value.as_background_image_url(), Some("foo.png"));
        assert_eq!(PropertyValue::Number(1.0).as_background_image_url(), None);
        // A url value is not a gradient, and vice versa.
        assert_eq!(value.as_linear_gradient(), None);
    }

    #[test]
    fn as_background_layers_extracts_only_layer_lists() {
        use crate::property::{BackgroundLayer, GradientStop, LinearGradient};
        let layers: Box<[BackgroundLayer]> = Box::new([
            BackgroundLayer::Linear(LinearGradient::new(
                90.0,
                vec![
                    GradientStop::new(StyleColor::BLACK, 0.0),
                    GradientStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 1.0),
                ],
            )),
            BackgroundLayer::Url("x.png".into()),
        ]);
        let value = PropertyValue::BackgroundLayers(layers.clone());
        assert_eq!(value.as_background_layers(), Some(&*layers));
        // A single flat value is not a layer list, and vice versa.
        assert_eq!(
            PropertyValue::BackgroundImageUrl("x.png".into()).as_background_layers(),
            None
        );
        assert_eq!(value.as_background_image_url(), None);
    }

    #[test]
    fn background_size_position_repeat_accessors() {
        use crate::property::{BackgroundPosition, BackgroundRepeat, BackgroundSize};
        let size = PropertyValue::BackgroundSize(BackgroundSize::Cover);
        assert_eq!(size.as_background_size(), Some(BackgroundSize::Cover));
        assert_eq!(size.as_background_position(), None);

        let pos = PropertyValue::BackgroundPosition(BackgroundPosition::new(
            Length::Percent(100.0),
            Length::Percent(100.0),
        ));
        assert_eq!(
            pos.as_background_position(),
            Some(BackgroundPosition::new(
                Length::Percent(100.0),
                Length::Percent(100.0)
            ))
        );
        assert_eq!(pos.as_background_size(), None);

        let repeat = PropertyValue::BackgroundRepeat(BackgroundRepeat::NoRepeat);
        assert_eq!(
            repeat.as_background_repeat(),
            Some(BackgroundRepeat::NoRepeat)
        );
        assert_eq!(PropertyValue::Number(1.0).as_background_repeat(), None);
    }

    #[test]
    fn as_object_fit_extracts_only_object_fit() {
        use crate::property::ObjectFit;
        let fit = PropertyValue::ObjectFit(ObjectFit::Cover);
        assert_eq!(fit.as_object_fit(), Some(ObjectFit::Cover));
        assert_eq!(PropertyValue::Number(1.0).as_object_fit(), None);
    }

    #[test]
    fn as_box_shadows_extracts_only_shadow_lists() {
        use crate::property::{BoxShadow, Length};
        let shadows: Box<[BoxShadow]> = Box::new([BoxShadow::new(
            Length::px(2.0),
            Length::px(4.0),
            Length::px(6.0),
            Length::px(0.0),
            Some(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
            false,
        )]);
        let value = PropertyValue::BoxShadow(shadows.clone());
        assert_eq!(value.as_box_shadows(), Some(&*shadows));
        // `none` lowers to an empty list, still a shadow value.
        let none = PropertyValue::BoxShadow(Box::new([]));
        assert_eq!(none.as_box_shadows(), Some(&[][..]));
        assert_eq!(PropertyValue::Number(1.0).as_box_shadows(), None);
    }

    #[test]
    fn as_filter_ops_extracts_only_filter_lists() {
        use crate::property::{BoxShadow, FilterOp, Length};
        let ops: Box<[FilterOp]> = Box::new([
            FilterOp::DropShadow(BoxShadow::new(
                Length::px(4.0),
                Length::px(4.0),
                Length::px(8.0),
                Length::px(0.0),
                Some(StyleColor::rgba(0.0, 0.0, 0.0, 0.5)),
                false,
            )),
            FilterOp::Blur(6.0),
        ]);
        let value = PropertyValue::Filter(ops.clone());
        assert_eq!(value.as_filter_ops(), Some(&*ops));
        // `none` / no honored functions lowers to an empty list.
        let none = PropertyValue::Filter(Box::new([]));
        assert_eq!(none.as_filter_ops(), Some(&[][..]));
        // A filter list is not a box-shadow list, and vice versa.
        assert_eq!(value.as_box_shadows(), None);
        let shadows: Box<[BoxShadow]> = Box::new([BoxShadow::new(
            Length::px(4.0),
            Length::px(4.0),
            Length::px(8.0),
            Length::px(0.0),
            None,
            false,
        )]);
        assert_eq!(PropertyValue::BoxShadow(shadows).as_filter_ops(), None);
        assert_eq!(PropertyValue::Number(1.0).as_filter_ops(), None);
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
    fn as_grid_line_extracts_only_grid_lines() {
        use crate::property::GridLineValue;
        let value = PropertyValue::GridLine(GridLineValue::Line(3));
        assert_eq!(value.as_grid_line(), Some(&GridLineValue::Line(3)));
        assert_eq!(PropertyValue::Number(1.0).as_grid_line(), None);
        let named = PropertyValue::GridLine(GridLineValue::Named("main".into()));
        assert_eq!(
            named.as_grid_line(),
            Some(&GridLineValue::Named("main".into()))
        );
    }

    #[test]
    fn as_grid_areas_extracts_only_templates() {
        use crate::property::GridAreaTemplate;
        let template =
            GridAreaTemplate::from_rows(&[vec![Some("h".into()), Some("h".into())]]).expect("t");
        let value = PropertyValue::GridAreas(template.clone());
        assert_eq!(value.as_grid_areas(), Some(&template));
        assert_eq!(PropertyValue::Number(1.0).as_grid_areas(), None);
    }

    #[test]
    fn as_grid_area_name_extracts_only_names() {
        let value = PropertyValue::GridAreaName("main".into());
        assert_eq!(value.as_grid_area_name(), Some("main"));
        assert_eq!(PropertyValue::Number(1.0).as_grid_area_name(), None);
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
