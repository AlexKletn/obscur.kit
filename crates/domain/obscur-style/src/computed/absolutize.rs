#![doc = include_str!("absolutize.doc/_module.md")]

use crate::property::{
    BackgroundLayer, BackgroundPosition, BackgroundSize, BoxShadow, ConicGradient, FilterOp,
    GridTrack, GridTrackList, Length, LengthContext, LengthExpr, PropertyValue, RadialGradient,
    RadialSize, TransformOrigin,
};

#[doc = include_str!("absolutize.doc/absolutize_value.md")]
pub(super) fn absolutize_value(value: PropertyValue, ctx: &LengthContext) -> PropertyValue {
    match value {
        PropertyValue::Length(length) => PropertyValue::Length(length.absolutize(ctx)),
        PropertyValue::Calc(expr) => collapse_calc(expr.absolutize(ctx), ctx),
        PropertyValue::BoxShadow(shadows) => {
            PropertyValue::BoxShadow(shadows.iter().map(|s| absolutize_shadow(s, ctx)).collect())
        }
        PropertyValue::Filter(ops) => {
            PropertyValue::Filter(ops.iter().map(|op| absolutize_filter(op, ctx)).collect())
        }
        PropertyValue::BackgroundSize(size) => {
            PropertyValue::BackgroundSize(absolutize_background_size(size, ctx))
        }
        PropertyValue::BackgroundPosition(position) => PropertyValue::BackgroundPosition(
            BackgroundPosition::new(position.x().absolutize(ctx), position.y().absolutize(ctx)),
        ),
        PropertyValue::TransformOrigin(origin) => PropertyValue::TransformOrigin(
            TransformOrigin::new(origin.x().absolutize(ctx), origin.y().absolutize(ctx)),
        ),
        PropertyValue::RadialGradient(gradient) => {
            PropertyValue::RadialGradient(absolutize_radial(&gradient, ctx))
        }
        PropertyValue::ConicGradient(gradient) => {
            PropertyValue::ConicGradient(absolutize_conic(&gradient, ctx))
        }
        PropertyValue::BackgroundLayers(layers) => PropertyValue::BackgroundLayers(
            layers.iter().map(|l| absolutize_layer(l, ctx)).collect(),
        ),
        PropertyValue::GridTracks(list) => PropertyValue::GridTracks(GridTrackList::new(
            list.tracks()
                .iter()
                .map(|t| absolutize_track(t, ctx))
                .collect(),
            list.line_names().to_vec().into_boxed_slice(),
        )),
        // `linear-gradient` carries an angle and stop offsets, never a length;
        // `transform` and `filter: blur()` were already flattened to numbers by
        // the CSS lowering (a relative unit is lost *there*, not here — see the
        // module docs of `obscur_css`'s property compiler); and the rest carry
        // no length to fold.
        PropertyValue::Keyword(_)
        | PropertyValue::Number(_)
        | PropertyValue::Color(_)
        | PropertyValue::LinearGradient(_)
        | PropertyValue::BackgroundImageUrl(_)
        | PropertyValue::BackgroundRepeat(_)
        | PropertyValue::ObjectFit(_)
        | PropertyValue::FontFamilies(_)
        | PropertyValue::GridLine(_)
        | PropertyValue::GridAreas(_)
        | PropertyValue::GridAreaName(_)
        | PropertyValue::Transform(_)
        // `content` is a plain string; it carries no length to fold.
        | PropertyValue::String(_)
        | PropertyValue::CustomRef(_)
        | PropertyValue::Inherit
        | PropertyValue::Initial
        | PropertyValue::Unset => value,
    }
}

#[doc = include_str!("absolutize.doc/collapse_calc.md")]
fn collapse_calc(expr: LengthExpr, ctx: &LengthContext) -> PropertyValue {
    match expr.resolve(ctx) {
        Some(px) => PropertyValue::Length(Length::px(px)),
        None => PropertyValue::Calc(expr),
    }
}

#[doc = include_str!("absolutize.doc/absolutize_shadow.md")]
fn absolutize_shadow(shadow: &BoxShadow, ctx: &LengthContext) -> BoxShadow {
    BoxShadow::new(
        shadow.offset_x().absolutize(ctx),
        shadow.offset_y().absolutize(ctx),
        shadow.blur().absolutize(ctx),
        shadow.spread().absolutize(ctx),
        shadow.color(),
        shadow.is_inset(),
    )
}

#[doc = include_str!("absolutize.doc/absolutize_filter.md")]
fn absolutize_filter(op: &FilterOp, ctx: &LengthContext) -> FilterOp {
    match op {
        FilterOp::DropShadow(shadow) => FilterOp::DropShadow(absolutize_shadow(shadow, ctx)),
        FilterOp::Blur(_)
        | FilterOp::Brightness(_)
        | FilterOp::Contrast(_)
        | FilterOp::Saturate(_)
        | FilterOp::Grayscale(_)
        | FilterOp::Sepia(_)
        | FilterOp::Invert(_)
        | FilterOp::HueRotate(_)
        | FilterOp::Opacity(_) => op.clone(),
    }
}

#[doc = include_str!("absolutize.doc/absolutize_background_size.md")]
fn absolutize_background_size(size: BackgroundSize, ctx: &LengthContext) -> BackgroundSize {
    match size {
        BackgroundSize::Explicit { width, height } => BackgroundSize::Explicit {
            width: width.absolutize(ctx),
            height: height.absolutize(ctx),
        },
        BackgroundSize::Auto | BackgroundSize::Cover | BackgroundSize::Contain => size,
    }
}

#[doc = include_str!("absolutize.doc/absolutize_radial.md")]
fn absolutize_radial(gradient: &RadialGradient, ctx: &LengthContext) -> RadialGradient {
    let size = match gradient.size() {
        RadialSize::Length(length) => RadialSize::Length(length.absolutize(ctx)),
        keyword => keyword,
    };
    RadialGradient::new(
        gradient.shape(),
        size,
        gradient.center_x().absolutize(ctx),
        gradient.center_y().absolutize(ctx),
        gradient.stops().to_vec(),
    )
}

#[doc = include_str!("absolutize.doc/absolutize_conic.md")]
fn absolutize_conic(gradient: &ConicGradient, ctx: &LengthContext) -> ConicGradient {
    ConicGradient::new(
        gradient.from_angle_deg(),
        gradient.center_x().absolutize(ctx),
        gradient.center_y().absolutize(ctx),
        gradient.stops().to_vec(),
    )
}

#[doc = include_str!("absolutize.doc/absolutize_layer.md")]
fn absolutize_layer(layer: &BackgroundLayer, ctx: &LengthContext) -> BackgroundLayer {
    match layer {
        BackgroundLayer::Radial(gradient) => {
            BackgroundLayer::Radial(absolutize_radial(gradient, ctx))
        }
        BackgroundLayer::Conic(gradient) => BackgroundLayer::Conic(absolutize_conic(gradient, ctx)),
        BackgroundLayer::Linear(_) | BackgroundLayer::Url(_) => layer.clone(),
    }
}

#[doc = include_str!("absolutize.doc/absolutize_track.md")]
fn absolutize_track(track: &GridTrack, ctx: &LengthContext) -> GridTrack {
    match track {
        GridTrack::Fixed(length) => GridTrack::Fixed(length.absolutize(ctx)),
        GridTrack::MinMax(min, max) => GridTrack::MinMax(
            Box::new(absolutize_track(min, ctx)),
            Box::new(absolutize_track(max, ctx)),
        ),
        GridTrack::AutoRepeat { fill, tracks } => GridTrack::AutoRepeat {
            fill: *fill,
            tracks: tracks.iter().map(|t| absolutize_track(t, ctx)).collect(),
        },
        // No length to absolutize: a fraction, and the content-sized keywords.
        GridTrack::Fr(_) | GridTrack::Auto | GridTrack::MinContent | GridTrack::MaxContent => {
            track.clone()
        }
    }
}

#[doc = include_str!("absolutize.doc/absolute_font_size.md")]
pub(super) fn absolute_font_size(
    specified: Option<Length>,
    parent_font_size: f32,
    root_font_size: f32,
    viewport_width: f32,
    viewport_height: f32,
    parent_ch_width: Option<f32>,
) -> f32 {
    let Some(specified) = specified else {
        return parent_font_size;
    };
    // Every base here is the PARENT's, per §6.1.2 above: its size is both the `em`
    // base and the `%` basis, and its `0` advance is the `ch` base. A stated basis
    // is honest here in a way it is not anywhere else.
    let mut ctx = LengthContext::new(
        parent_font_size,
        root_font_size,
        viewport_width,
        viewport_height,
    )
    .with_percent_basis(parent_font_size);
    if let Some(ch_width) = parent_ch_width {
        ctx = ctx.with_ch_width(ch_width);
    }
    match specified {
        // Unmeasured `ch` must not reach `resolve_px`, which would silently apply
        // `ch_base`'s `0.5em` substitute to a font-size and halve the text.
        Length::Ch(_) if parent_ch_width.is_none() => parent_font_size,
        length => length.resolve_px(&ctx).unwrap_or(parent_font_size),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::{GradientStop, LengthExpr, RadialShape, StyleColor, Transform};

    fn ctx() -> LengthContext {
        // font 16, root 20 (distinct so `em` vs `rem` cannot be confused),
        // viewport 800x600.
        LengthContext::new(16.0, 20.0, 800.0, 600.0)
    }

    #[test]
    fn a_bare_length_folds() {
        assert_eq!(
            absolutize_value(PropertyValue::Length(Length::Rem(2.0)), &ctx()),
            PropertyValue::Length(Length::px(40.0))
        );
    }

    #[test]
    fn a_calc_folds_its_relative_leaves_only() {
        let expr = LengthExpr::Sum(
            Box::new(LengthExpr::leaf(Length::Em(2.0))),
            Box::new(LengthExpr::leaf(Length::Percent(50.0))),
        );
        assert_eq!(
            absolutize_value(PropertyValue::Calc(expr), &ctx()),
            PropertyValue::Calc(LengthExpr::Sum(
                Box::new(LengthExpr::leaf(Length::px(32.0))),
                Box::new(LengthExpr::leaf(Length::Percent(50.0))),
            ))
        );
    }

    /// The other half of the rule above: once nothing in the expression is
    /// unknown, it *is* a length and must say so. A `Calc` that stays a `Calc`
    /// here is dropped by every consumer that reads a length — which is what made
    /// `border-radius: min(3vmin, 14px)` paint square corners.
    #[test]
    fn a_calc_with_nothing_left_unknown_becomes_a_plain_length() {
        // min(3vmin, 14px) against a 800×600 viewport: vmin = 6, so 18px vs 14px.
        let expr = LengthExpr::Min(vec![
            LengthExpr::leaf(Length::Vmin(3.0)),
            LengthExpr::leaf(Length::px(14.0)),
        ]);
        assert_eq!(
            absolutize_value(PropertyValue::Calc(expr), &ctx()),
            PropertyValue::Length(Length::px(14.0))
        );
    }

    /// `auto` has no number at computed-value time either, so it keeps the
    /// expression alive exactly as a percentage does.
    #[test]
    fn a_calc_over_auto_stays_an_expression() {
        let expr = LengthExpr::Max(vec![
            LengthExpr::leaf(Length::Auto),
            LengthExpr::leaf(Length::px(4.0)),
        ]);
        assert!(matches!(
            absolutize_value(PropertyValue::Calc(expr), &ctx()),
            PropertyValue::Calc(_)
        ));
    }

    #[test]
    fn every_box_shadow_length_folds() {
        // The audit's case: an `em` shadow used to paint at 0 on all four terms.
        let value = PropertyValue::BoxShadow(
            vec![BoxShadow::new(
                Length::Em(1.0),
                Length::Rem(1.0),
                Length::Em(0.5),
                Length::Vw(1.0),
                Some(StyleColor::BLACK),
                true,
            )]
            .into_boxed_slice(),
        );
        let PropertyValue::BoxShadow(out) = absolutize_value(value, &ctx()) else {
            panic!("still a shadow list");
        };
        assert_eq!(out[0].offset_x(), Length::px(16.0));
        assert_eq!(out[0].offset_y(), Length::px(20.0));
        assert_eq!(out[0].blur(), Length::px(8.0));
        assert_eq!(out[0].spread(), Length::px(8.0));
        // Colour and `inset` ride through untouched.
        assert_eq!(out[0].color(), Some(StyleColor::BLACK));
        assert!(out[0].is_inset());
    }

    #[test]
    fn a_drop_shadow_filter_folds_but_a_blur_radius_has_no_length_left() {
        let value = PropertyValue::Filter(
            vec![
                FilterOp::DropShadow(BoxShadow::new(
                    Length::Em(1.0),
                    Length::Em(1.0),
                    Length::px(2.0),
                    Length::px(0.0),
                    None,
                    false,
                )),
                FilterOp::Blur(4.0),
            ]
            .into_boxed_slice(),
        );
        let PropertyValue::Filter(out) = absolutize_value(value, &ctx()) else {
            panic!("still a filter list");
        };
        let FilterOp::DropShadow(shadow) = &out[0] else {
            panic!("first op is the drop shadow");
        };
        assert_eq!(shadow.offset_x(), Length::px(16.0));
        // `blur()`'s radius is already a number by lowering time; unchanged here.
        assert_eq!(out[1], FilterOp::Blur(4.0));
    }

    #[test]
    fn background_size_position_and_transform_origin_fold() {
        assert_eq!(
            absolutize_value(
                PropertyValue::BackgroundSize(BackgroundSize::Explicit {
                    width: Length::Em(2.0),
                    height: Length::Percent(50.0),
                }),
                &ctx()
            ),
            PropertyValue::BackgroundSize(BackgroundSize::Explicit {
                width: Length::px(32.0),
                height: Length::Percent(50.0),
            })
        );
        assert_eq!(
            absolutize_value(
                PropertyValue::BackgroundPosition(BackgroundPosition::new(
                    Length::Rem(1.0),
                    Length::Vh(50.0)
                )),
                &ctx()
            ),
            PropertyValue::BackgroundPosition(BackgroundPosition::new(
                Length::px(20.0),
                Length::px(300.0)
            ))
        );
        assert_eq!(
            absolutize_value(
                PropertyValue::TransformOrigin(TransformOrigin::new(
                    Length::Em(1.0),
                    Length::Percent(50.0)
                )),
                &ctx()
            ),
            PropertyValue::TransformOrigin(TransformOrigin::new(
                Length::px(16.0),
                Length::Percent(50.0)
            ))
        );
    }

    #[test]
    fn a_radial_gradient_folds_its_center_and_explicit_radius() {
        let stops = vec![
            GradientStop::new(StyleColor::BLACK, 0.0),
            GradientStop::new(StyleColor::TRANSPARENT, 1.0),
        ];
        let gradient = RadialGradient::new(
            RadialShape::Circle,
            RadialSize::Length(Length::Em(4.0)),
            Length::Rem(1.0),
            Length::Percent(50.0),
            stops.clone(),
        );
        let PropertyValue::RadialGradient(out) =
            absolutize_value(PropertyValue::RadialGradient(gradient), &ctx())
        else {
            panic!("still a radial gradient");
        };
        assert_eq!(out.size(), RadialSize::Length(Length::px(64.0)));
        assert_eq!(out.center_x(), Length::px(20.0));
        assert_eq!(out.center_y(), Length::Percent(50.0));
        assert_eq!(out.stops(), &stops[..]);
    }

    #[test]
    fn a_conic_gradient_layer_inside_a_layer_list_folds() {
        let stops = vec![GradientStop::new(StyleColor::BLACK, 0.0)];
        let value = PropertyValue::BackgroundLayers(
            vec![
                BackgroundLayer::Conic(ConicGradient::new(
                    90.0,
                    Length::Em(1.0),
                    Length::Em(2.0),
                    stops.clone(),
                )),
                BackgroundLayer::Url("x.png".into()),
            ]
            .into_boxed_slice(),
        );
        let PropertyValue::BackgroundLayers(out) = absolutize_value(value, &ctx()) else {
            panic!("still a layer list");
        };
        let BackgroundLayer::Conic(gradient) = &out[0] else {
            panic!("first layer is the conic gradient");
        };
        assert_eq!(gradient.center_x(), Length::px(16.0));
        assert_eq!(gradient.center_y(), Length::px(32.0));
        assert_eq!(out[1], BackgroundLayer::Url("x.png".into()));
    }

    #[test]
    fn grid_tracks_fold_through_minmax_and_auto_repeat_keeping_line_names() {
        let list = GridTrackList::new(
            vec![
                GridTrack::Fixed(Length::Em(2.0)),
                GridTrack::MinMax(
                    Box::new(GridTrack::Fixed(Length::Rem(1.0))),
                    Box::new(GridTrack::Fr(1.0)),
                ),
                GridTrack::AutoRepeat {
                    fill: true,
                    tracks: vec![GridTrack::Fixed(Length::Vw(10.0))].into_boxed_slice(),
                },
            ]
            .into_boxed_slice(),
            vec![
                vec!["start".into()].into_boxed_slice(),
                Vec::new().into_boxed_slice(),
                Vec::new().into_boxed_slice(),
                vec!["end".into()].into_boxed_slice(),
            ]
            .into_boxed_slice(),
        );
        let PropertyValue::GridTracks(out) =
            absolutize_value(PropertyValue::GridTracks(list), &ctx())
        else {
            panic!("still a track list");
        };
        assert_eq!(out.tracks()[0], GridTrack::Fixed(Length::px(32.0)));
        assert_eq!(
            out.tracks()[1],
            GridTrack::MinMax(
                Box::new(GridTrack::Fixed(Length::px(20.0))),
                Box::new(GridTrack::Fr(1.0)),
            )
        );
        assert_eq!(
            out.tracks()[2],
            GridTrack::AutoRepeat {
                fill: true,
                tracks: vec![GridTrack::Fixed(Length::px(80.0))].into_boxed_slice(),
            }
        );
        // The `[name]` buckets must survive the rebuild.
        assert_eq!(out.line_names().len(), 4);
        assert_eq!(&*out.line_names()[0][0], "start");
        assert_eq!(&*out.line_names()[3][0], "end");
    }

    #[test]
    fn values_without_lengths_ride_through_unchanged() {
        for value in [
            PropertyValue::Number(0.5),
            PropertyValue::Color(StyleColor::BLACK),
            PropertyValue::Transform(Transform::IDENTITY),
            PropertyValue::BackgroundImageUrl("x.png".into()),
        ] {
            assert_eq!(absolutize_value(value.clone(), &ctx()), value);
        }
    }

    #[test]
    fn font_size_resolves_em_and_percent_against_the_parent() {
        // `2em` and `200%` are the same declaration on `font-size`.
        assert_eq!(
            absolute_font_size(Some(Length::Em(2.0)), 10.0, 16.0, 800.0, 600.0, None),
            20.0
        );
        assert_eq!(
            absolute_font_size(Some(Length::Percent(200.0)), 10.0, 16.0, 800.0, 600.0, None),
            20.0
        );
    }

    #[test]
    fn font_size_resolves_rem_against_the_root_not_the_parent() {
        // The audit's headline case, at its source: `2rem` is 2 x the ROOT size.
        assert_eq!(
            absolute_font_size(Some(Length::Rem(2.0)), 10.0, 16.0, 800.0, 600.0, None),
            32.0
        );
    }

    #[test]
    fn font_size_resolves_viewport_units_and_absolute_px() {
        assert_eq!(
            absolute_font_size(Some(Length::Vw(5.0)), 16.0, 16.0, 800.0, 600.0, None),
            40.0
        );
        assert_eq!(
            absolute_font_size(Some(Length::px(24.0)), 16.0, 16.0, 800.0, 600.0, None),
            24.0
        );
    }

    #[test]
    fn an_absent_or_unresolvable_font_size_falls_back_to_the_parent() {
        // Absent -> inherit the parent's size (`font-size` inherits).
        assert_eq!(
            absolute_font_size(None, 24.0, 16.0, 800.0, 600.0, None),
            24.0
        );
        // `auto` is not a font-size; `ch` needs a shaper this step has no reach
        // to. Both take the parent's size rather than invent one.
        assert_eq!(
            absolute_font_size(Some(Length::Auto), 24.0, 16.0, 800.0, 600.0, None),
            24.0
        );
        assert_eq!(
            absolute_font_size(Some(Length::Ch(2.0)), 24.0, 16.0, 800.0, 600.0, None),
            24.0,
            "not 0.5em x 2 = the parent size by accident: a 3ch would give 24 too"
        );
        assert_eq!(
            absolute_font_size(Some(Length::Ch(9.0)), 24.0, 16.0, 800.0, 600.0, None),
            24.0
        );
    }
}
