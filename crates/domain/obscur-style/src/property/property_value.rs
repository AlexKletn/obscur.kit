#![doc = include_str!("../../docs/api/property/property_value.doc/_module.md")]

use crate::property::{
    BackgroundAttachment, BackgroundBox, BackgroundLayer, BackgroundPosition, BackgroundRepeat,
    BackgroundSize, BoxShadow, ConicGradient, FilterFunction, FilterOp, GridAreaTemplate,
    GridLineValue, GridTrack, GridTrackList, Length, LengthExpr, LinearGradient, ObjectFit,
    RadialGradient, StyleColor, Transform, TransformFunction, TransformOrigin,
};
use obscur_identity::InternedSymbol;
use std::sync::Arc;

use super::{ClipRadius, ClipRectangleKind, ContainerComponent};

#[doc = include_str!("../../docs/api/property/property_value.doc/PropertyValue.md")]
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
    #[doc = include_str!("../../docs/api/property/property_value.doc/ConicGradient.md")]
    ConicGradient(ConicGradient),
    #[doc = include_str!("../../docs/api/property/property_value.doc/BackgroundImageUrl.md")]
    BackgroundImageUrl(Box<str>),
    #[doc = include_str!("../../docs/api/property/property_value.doc/BackgroundLayers.md")]
    BackgroundLayers(Box<[BackgroundLayer]>),
    /// A `background-size` value (see [`BackgroundSize`]): how one copy of the
    /// `background-image` is scaled (`cover` / `contain` / `auto` / explicit
    /// per-axis lengths). The scene coordinator resolves it against the box and
    /// the image's intrinsic size to a concrete tile width/height at paint time.
    BackgroundSize(BackgroundSize),
    /// Background size values in authored layer order.
    BackgroundSizes(Box<[BackgroundSize]>),
    /// A `background-position` value (see [`BackgroundPosition`]): where the sized
    /// image tile's origin sits within the box, one length/percentage per axis.
    BackgroundPosition(BackgroundPosition),
    /// Background position values in authored layer order.
    BackgroundPositions(Box<[BackgroundPosition]>),
    /// A `background-repeat` value (see [`BackgroundRepeat`]): whether the sized
    /// image tile is tiled along each axis to fill the box.
    BackgroundRepeat(BackgroundRepeat),
    /// Repeat values in authored background-layer order.
    BackgroundRepeats(Box<[BackgroundRepeat]>),
    /// Blend keywords in authored background-layer order.
    BackgroundBlendModes(Box<[InternedSymbol]>),
    /// Painting boundaries in authored layer order.
    BackgroundClips(Box<[BackgroundBox]>),
    /// A clip using the rounded edges of a layout box.
    ClipBox(BackgroundBox),
    /// A rectangle with optional rounded corners.
    ClipRectangle {
        /// Reference geometry.
        reference_box: BackgroundBox,
        /// Interpretation of the four coordinates.
        kind: ClipRectangleKind,
        /// Coordinates in CSS function order.
        coordinates: Box<[LengthExpr; 4]>,
        /// Horizontal and vertical radius for each corner, clockwise from top left.
        radii: Option<Box<[[LengthExpr; 2]; 4]>>,
    },
    /// A CSS circle or ellipse.
    ClipEllipse {
        /// Reference geometry.
        reference_box: BackgroundBox,
        /// Center within the reference box.
        center: [LengthExpr; 2],
        /// Horizontal and vertical radii.
        radii: [ClipRadius; 2],
        /// Whether both axes use the same resolved circle radius.
        circle: bool,
    },
    /// An SVG path resolved from the reference box's top-left corner.
    ClipSvgPath {
        /// Reference geometry.
        reference_box: BackgroundBox,
        /// Parsed drawing commands.
        path: Arc<obscur_scene::path::Path>,
        /// Flattened geometry for hit testing.
        outline: Arc<obscur_scene::path::PathTrim>,
    },
    /// An authored polygon whose vertices resolve against the border box.
    ClipPolygon {
        /// Uses even-odd winding when true.
        even_odd: bool,
        /// Box used to resolve the vertex coordinates.
        reference_box: BackgroundBox,
        /// Pairs of horizontal and vertical offsets.
        points: Box<[[LengthExpr; 2]]>,
    },
    /// Positioning boundaries in authored layer order.
    BackgroundOrigins(Box<[BackgroundBox]>),
    /// Background scrolling modes in layer order.
    BackgroundAttachments(Box<[BackgroundAttachment]>),
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
    #[doc = include_str!("../../docs/api/property/property_value.doc/Filter.md")]
    Filter(Box<[FilterOp]>),
    #[doc = include_str!("../../docs/api/property/property_value.doc/FilterFunctions.md")]
    FilterFunctions(Box<[FilterFunction]>),
    #[doc = include_str!("../../docs/api/property/property_value.doc/FontFamilies.md")]
    FontFamilies(Box<[Box<str>]>),
    #[doc = include_str!("../../docs/api/property/property_value.doc/GridTracks.md")]
    GridTracks(GridTrackList),
    #[doc = include_str!("../../docs/api/property/property_value.doc/GridLine.md")]
    GridLine(GridLineValue),
    /// A parsed `grid-template-areas` grid: the value of a grid container's
    /// `grid-template-areas`. The [`GridAreaTemplate`] carries every named area's
    /// line-number rectangle (already validated as a filled rectangle at parse
    /// time); the runtime looks a child's `grid-area: <name>` up against its
    /// container's template to resolve a concrete placement (spec §4.2).
    GridAreas(GridAreaTemplate),
    #[doc = include_str!("../../docs/api/property/property_value.doc/GridAreaName.md")]
    GridAreaName(Box<str>),
    #[doc = include_str!("../../docs/api/property/property_value.doc/Name.md")]
    Name(Box<str>),
    /// A list of case-sensitive CSS query container names.
    ContainerNames(Arc<[Box<str>]>),
    #[doc = include_str!("../../docs/api/property/property_value.doc/String.md")]
    String(Box<str>),
    /// A resolved CSS 2D `transform`: the authored `<transform-function>+` list
    /// composed left-to-right into a single 2x3 affine (see [`Transform`]).
    /// `transform: none` lowers to an absent value, not this. The scene
    /// coordinator wraps the affine about the element's `transform-origin` and
    /// brackets the subtree's paint with it (spec §5.2).
    Transform(Transform),
    #[doc = include_str!("../../docs/api/property/property_value.doc/TransformFunctions.md")]
    TransformFunctions(Box<[TransformFunction]>),
    /// A resolved CSS `transform-origin` (see [`TransformOrigin`]): the point a
    /// `transform` pivots about, defaulting to the box center (`50% 50%`).
    TransformOrigin(TransformOrigin),
    /// An unresolved `var(--name)` reference to a custom property, named by its
    /// interned symbol. This is a pre-resolution placeholder: it is substituted
    /// with the referenced custom property's value before computed-value
    /// resolution (§4.14).
    CustomRef(InternedSymbol),
    /// A deferred reference replacing the complete container shorthand.
    ContainerRef {
        /// The custom property supplying the shorthand.
        name: InternedSymbol,
        /// The shorthand token fallback when the reference cannot be resolved.
        fallback: Option<Arc<str>>,
    },
    /// Container shorthand fragments to substitute before lowering either longhand.
    ContainerTemplate(Arc<[ContainerComponent]>),
    /// Deferred flex-flow tokens to substitute and parse as one shorthand.
    FlexFlowTemplate(Arc<[ContainerComponent]>),
    /// A gap shorthand deferred until variable substitution is complete.
    GapTemplate(Arc<[ContainerComponent]>),
    /// An ordinary declaration deferred until its variable and fallback tokens can be resolved.
    VariableTemplate(Arc<[ContainerComponent]>),
    #[doc = include_str!("../../docs/api/property/property_value.doc/RawTokens.md")]
    RawTokens(Box<str>),
    /// The CSS-wide `inherit` keyword.
    Inherit,
    /// The CSS-wide `initial` keyword.
    Initial,
    /// The CSS-wide `unset` keyword (behaves as `inherit` for inherited
    /// properties and `initial` otherwise).
    Unset,
}

impl PropertyValue {
    #[doc = include_str!("../../docs/api/property/property_value.doc/uses_ch.md")]
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        match self {
            Self::Length(length) => length.is_ch(),
            Self::Calc(expr) => expr.uses_ch(),
            Self::GridTracks(tracks) => tracks.uses_ch(),
            Self::BoxShadow(shadows) => shadows.iter().any(BoxShadow::uses_ch),
            Self::Filter(ops) => ops.iter().any(FilterOp::uses_ch),
            Self::FilterFunctions(list) => list.iter().any(FilterFunction::uses_ch),
            Self::TransformFunctions(list) => list.iter().any(TransformFunction::uses_ch),
            Self::ClipEllipse { center, radii, .. } => {
                center.iter().any(LengthExpr::uses_ch)
                    || radii.iter().any(|radius| matches!(radius, ClipRadius::Length(length) if length.uses_ch()))
            }
            Self::ClipRectangle { coordinates, radii, .. } => coordinates.iter().any(|length| length.uses_ch())
                || radii.as_ref().is_some_and(|radii| radii.iter().flatten().any(|length| length.uses_ch())),
            Self::ClipPolygon { points, .. } => points.iter().flatten().any(|length| length.uses_ch()),
            Self::BackgroundSize(size) => size.uses_ch(),
            Self::BackgroundSizes(sizes) => sizes.iter().any(|size| size.uses_ch()),
            Self::BackgroundPosition(position) => position.x().is_ch() || position.y().is_ch(),
            Self::BackgroundPositions(positions) => positions.iter().any(|position| position.x().is_ch() || position.y().is_ch()),
            Self::TransformOrigin(origin) => origin.x().is_ch() || origin.y().is_ch(),
            Self::LinearGradient(gradient) => gradient.uses_ch(),
            Self::RadialGradient(gradient) => gradient.uses_ch(),
            Self::ConicGradient(gradient) => gradient.uses_ch(),
            Self::BackgroundLayers(layers) => layers.iter().any(BackgroundLayer::uses_ch),
            // Carrying no length at all. `linear-gradient` carries an angle and
            // stop offsets; a `Transform` / `Filter` is the RESOLVED form of the
            // two variants above it, whose lengths are already numbers; a `var()`
            // reference is substituted before this is asked.
            Self::Keyword(_)
            | Self::Number(_)
            | Self::Color(_)
            | Self::BackgroundImageUrl(_)
            | Self::BackgroundRepeat(_)
            | Self::BackgroundRepeats(_)
            | Self::BackgroundBlendModes(_)
            | Self::BackgroundClips(_)
            | Self::ClipBox(_)
            | Self::ClipSvgPath { .. }
            | Self::BackgroundOrigins(_)
            | Self::BackgroundAttachments(_)
            | Self::ObjectFit(_)
            | Self::FontFamilies(_)
            | Self::GridLine(_)
            | Self::GridAreas(_)
            | Self::GridAreaName(_)
            | Self::Transform(_)
            // A `content` string and a registry name are both plain text; they
            // carry no length.
            | Self::String(_)
            | Self::ContainerNames(_)
            | Self::Name(_)
            | Self::ContainerRef { .. }
            | Self::ContainerTemplate(_)
            | Self::FlexFlowTemplate(_)
            | Self::GapTemplate(_)
            | Self::VariableTemplate(_)
            | Self::CustomRef(_)
            // A custom property's declared text carries whatever units it was
            // written with, `ch` included — but it is parsed into a real value
            // before it reaches any property, and it is that value which is
            // asked.
            | Self::RawTokens(_)
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

    #[doc = include_str!("../../docs/api/property/property_value.doc/as_background_layers.md")]
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

    #[doc = include_str!("../../docs/api/property/property_value.doc/as_grid_line.md")]
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

    /// This test used to assert the opposite, under the name
    /// `uses_ch_is_false_for_a_box_shadow_which_is_a_known_gap`, and its comment
    /// said: *"When a measurement reaches the computed-value step, this test should
    /// flip."* [`ChMetrics`](crate::computed::ChMetrics) is that measurement, so it
    /// has flipped.
    #[test]
    fn uses_ch_finds_a_ch_nested_in_a_box_shadow() {
        use crate::property::{BoxShadow, Length};

        // A `ch` reachable only through a nested length: if this answered `false`
        // the resolver would fold the shadow with no `ch` base and the offset would
        // survive as a symbolic term that paint reads as 0 — the exact defect that
        // made this a documented gap.
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
        assert!(shadow.uses_ch());

        // The same shadow with no `ch` must NOT fire: a false positive costs a
        // shaping call on every ordinary stylesheet, which is what the deferral in
        // `StyleResolver::resolve` exists to avoid.
        let absolute = PropertyValue::BoxShadow(
            vec![BoxShadow {
                offset_x: Length::Em(1.0),
                offset_y: Length::px(2.0),
                blur: Length::px(0.0),
                spread: Length::px(0.0),
                color: None,
                inset: false,
            }]
            .into_boxed_slice(),
        );
        assert!(!absolute.uses_ch());
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
