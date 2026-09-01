#![doc = include_str!("property_value.doc/_module.md")]

use crate::property::{
    BackgroundLayer, BackgroundPosition, BackgroundRepeat, BackgroundSize, BoxShadow,
    ConicGradient, FilterFunction, FilterOp, GridAreaTemplate, GridLineValue, GridTrack,
    GridTrackList, Length, LengthExpr, LinearGradient, ObjectFit, RadialGradient, StyleColor,
    Transform, TransformFunction, TransformOrigin,
};
use obscur_identity::InternedSymbol;

#[doc = include_str!("property_value.doc/PropertyValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    #[doc = include_str!("property_value.doc/Keyword.md")]
    Keyword(InternedSymbol),
    #[doc = include_str!("property_value.doc/Length.md")]
    Length(Length),
    #[doc = include_str!("property_value.doc/Calc.md")]
    Calc(LengthExpr),
    #[doc = include_str!("property_value.doc/Number.md")]
    Number(f32),
    #[doc = include_str!("property_value.doc/Color.md")]
    Color(StyleColor),
    #[doc = include_str!("property_value.doc/LinearGradient.md")]
    LinearGradient(LinearGradient),
    #[doc = include_str!("property_value.doc/RadialGradient.md")]
    RadialGradient(RadialGradient),
    #[doc = include_str!("property_value.doc/ConicGradient.md")]
    ConicGradient(ConicGradient),
    #[doc = include_str!("property_value.doc/BackgroundImageUrl.md")]
    BackgroundImageUrl(Box<str>),
    #[doc = include_str!("property_value.doc/BackgroundLayers.md")]
    BackgroundLayers(Box<[BackgroundLayer]>),
    #[doc = include_str!("property_value.doc/BackgroundSize.md")]
    BackgroundSize(BackgroundSize),
    #[doc = include_str!("property_value.doc/BackgroundPosition.md")]
    BackgroundPosition(BackgroundPosition),
    #[doc = include_str!("property_value.doc/BackgroundRepeat.md")]
    BackgroundRepeat(BackgroundRepeat),
    #[doc = include_str!("property_value.doc/ObjectFit.md")]
    ObjectFit(ObjectFit),
    #[doc = include_str!("property_value.doc/BoxShadow.md")]
    BoxShadow(Box<[BoxShadow]>),
    #[doc = include_str!("property_value.doc/Filter.md")]
    Filter(Box<[FilterOp]>),
    #[doc = include_str!("property_value.doc/FilterFunctions.md")]
    FilterFunctions(Box<[FilterFunction]>),
    #[doc = include_str!("property_value.doc/FontFamilies.md")]
    FontFamilies(Box<[Box<str>]>),
    #[doc = include_str!("property_value.doc/GridTracks.md")]
    GridTracks(GridTrackList),
    #[doc = include_str!("property_value.doc/GridLine.md")]
    GridLine(GridLineValue),
    #[doc = include_str!("property_value.doc/GridAreas.md")]
    GridAreas(GridAreaTemplate),
    #[doc = include_str!("property_value.doc/GridAreaName.md")]
    GridAreaName(Box<str>),
    #[doc = include_str!("property_value.doc/Name.md")]
    Name(Box<str>),
    #[doc = include_str!("property_value.doc/String.md")]
    String(Box<str>),
    #[doc = include_str!("property_value.doc/Transform.md")]
    Transform(Transform),
    #[doc = include_str!("property_value.doc/TransformFunctions.md")]
    TransformFunctions(Box<[TransformFunction]>),
    #[doc = include_str!("property_value.doc/TransformOrigin.md")]
    TransformOrigin(TransformOrigin),
    #[doc = include_str!("property_value.doc/CustomRef.md")]
    CustomRef(InternedSymbol),
    #[doc = include_str!("property_value.doc/RawTokens.md")]
    RawTokens(Box<str>),
    #[doc = include_str!("property_value.doc/Inherit.md")]
    Inherit,
    #[doc = include_str!("property_value.doc/Initial.md")]
    Initial,
    #[doc = include_str!("property_value.doc/Unset.md")]
    Unset,
}

impl PropertyValue {
    #[doc = include_str!("property_value.doc/uses_ch.md")]
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
            Self::BackgroundSize(size) => size.uses_ch(),
            Self::BackgroundPosition(position) => position.x().is_ch() || position.y().is_ch(),
            Self::TransformOrigin(origin) => origin.x().is_ch() || origin.y().is_ch(),
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
            | Self::LinearGradient(_)
            | Self::BackgroundImageUrl(_)
            | Self::BackgroundRepeat(_)
            | Self::ObjectFit(_)
            | Self::FontFamilies(_)
            | Self::GridLine(_)
            | Self::GridAreas(_)
            | Self::GridAreaName(_)
            | Self::Transform(_)
            // A `content` string and a registry name are both plain text; they
            // carry no length.
            | Self::String(_)
            | Self::Name(_)
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

    #[doc = include_str!("property_value.doc/as_color.md")]
    #[must_use]
    pub const fn as_color(&self) -> Option<StyleColor> {
        match self {
            Self::Color(color) => Some(*color),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_linear_gradient.md")]
    #[must_use]
    pub const fn as_linear_gradient(&self) -> Option<&LinearGradient> {
        match self {
            Self::LinearGradient(gradient) => Some(gradient),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_radial_gradient.md")]
    #[must_use]
    pub const fn as_radial_gradient(&self) -> Option<&RadialGradient> {
        match self {
            Self::RadialGradient(gradient) => Some(gradient),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_conic_gradient.md")]
    #[must_use]
    pub const fn as_conic_gradient(&self) -> Option<&ConicGradient> {
        match self {
            Self::ConicGradient(gradient) => Some(gradient),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_background_image_url.md")]
    #[must_use]
    pub fn as_background_image_url(&self) -> Option<&str> {
        match self {
            Self::BackgroundImageUrl(url) => Some(url),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_background_layers.md")]
    #[must_use]
    pub fn as_background_layers(&self) -> Option<&[BackgroundLayer]> {
        match self {
            Self::BackgroundLayers(layers) => Some(layers),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_background_size.md")]
    #[must_use]
    pub const fn as_background_size(&self) -> Option<BackgroundSize> {
        match self {
            Self::BackgroundSize(size) => Some(*size),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_background_position.md")]
    #[must_use]
    pub const fn as_background_position(&self) -> Option<BackgroundPosition> {
        match self {
            Self::BackgroundPosition(position) => Some(*position),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_background_repeat.md")]
    #[must_use]
    pub const fn as_background_repeat(&self) -> Option<BackgroundRepeat> {
        match self {
            Self::BackgroundRepeat(repeat) => Some(*repeat),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_object_fit.md")]
    #[must_use]
    pub const fn as_object_fit(&self) -> Option<ObjectFit> {
        match self {
            Self::ObjectFit(fit) => Some(*fit),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_box_shadows.md")]
    #[must_use]
    pub fn as_box_shadows(&self) -> Option<&[BoxShadow]> {
        match self {
            Self::BoxShadow(shadows) => Some(shadows),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_filter_ops.md")]
    #[must_use]
    pub fn as_filter_ops(&self) -> Option<&[FilterOp]> {
        match self {
            Self::Filter(ops) => Some(ops),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/font_families.md")]
    #[must_use]
    pub fn font_families<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        Self::FontFamilies(names.into_iter().map(Into::into).collect())
    }

    #[doc = include_str!("property_value.doc/as_font_families.md")]
    #[must_use]
    pub fn as_font_families(&self) -> Option<&[Box<str>]> {
        match self {
            Self::FontFamilies(names) => Some(names),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/grid_tracks.md")]
    #[must_use]
    pub fn grid_tracks<I>(tracks: I) -> Self
    where
        I: IntoIterator<Item = GridTrack>,
    {
        Self::GridTracks(GridTrackList::from_tracks(tracks))
    }

    #[doc = include_str!("property_value.doc/grid_track_list.md")]
    #[must_use]
    pub const fn grid_track_list(list: GridTrackList) -> Self {
        Self::GridTracks(list)
    }

    #[doc = include_str!("property_value.doc/as_grid_tracks.md")]
    #[must_use]
    pub fn as_grid_tracks(&self) -> Option<&[GridTrack]> {
        match self {
            Self::GridTracks(list) => Some(list.tracks()),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_grid_track_list.md")]
    #[must_use]
    pub const fn as_grid_track_list(&self) -> Option<&GridTrackList> {
        match self {
            Self::GridTracks(list) => Some(list),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_grid_line.md")]
    #[must_use]
    pub fn as_grid_line(&self) -> Option<&GridLineValue> {
        match self {
            Self::GridLine(line) => Some(line),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_grid_areas.md")]
    #[must_use]
    pub const fn as_grid_areas(&self) -> Option<&GridAreaTemplate> {
        match self {
            Self::GridAreas(template) => Some(template),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_grid_area_name.md")]
    #[must_use]
    pub fn as_grid_area_name(&self) -> Option<&str> {
        match self {
            Self::GridAreaName(name) => Some(name),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_transform.md")]
    #[must_use]
    pub const fn as_transform(&self) -> Option<Transform> {
        match self {
            Self::Transform(transform) => Some(*transform),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_transform_origin.md")]
    #[must_use]
    pub const fn as_transform_origin(&self) -> Option<TransformOrigin> {
        match self {
            Self::TransformOrigin(origin) => Some(*origin),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_length.md")]
    #[must_use]
    pub const fn as_length(&self) -> Option<Length> {
        match self {
            Self::Length(length) => Some(*length),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/as_calc.md")]
    #[must_use]
    pub const fn as_calc(&self) -> Option<&LengthExpr> {
        match self {
            Self::Calc(expr) => Some(expr),
            _ => None,
        }
    }

    #[doc = include_str!("property_value.doc/is_wide_keyword.md")]
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
