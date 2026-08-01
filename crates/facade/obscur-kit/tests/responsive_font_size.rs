#![doc = include_str!("responsive_font_size.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::computed::ComputedValue;
use obscur_kit::style::property::{Length, PropertyId, PropertyValue};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("responsive_font_size.doc/font_size.md")]
fn font_size(css: &str, w: f32, h: f32) -> Option<f32> {
    let source = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&source).parse();
    let sheet = StylesheetCompiler::new().compile(&parsed).matchable;

    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(91))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into()))),
    ]));
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h));
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

    let root = runtime.tree().root()?;
    let node = runtime.tree().get(root)?.child_nodes()[0];
    match runtime
        .computed_style(node)?
        .get(PropertyId::FontSize)
        .map(ComputedValue::value)
    {
        Some(PropertyValue::Length(Length::Px(px))) => Some(px.0),
        _ => None,
    }
}

#[test]
fn calc_is_folded_rather_than_dropped() {
    assert_eq!(
        font_size(".box { font-size: calc(10px + 10px) }", 400.0, 400.0),
        Some(20.0),
        "a dropped declaration would leave the initial 16px"
    );
}

#[test]
fn min_and_max_pick_their_side() {
    assert_eq!(
        font_size(".box { font-size: min(20px, 30px) }", 400.0, 400.0),
        Some(20.0)
    );
    assert_eq!(
        font_size(".box { font-size: max(20px, 30px) }", 400.0, 400.0),
        Some(30.0)
    );
}

#[doc = include_str!("responsive_font_size.doc/clamp_tracks_the_window_between_its_bounds.md")]
#[test]
fn clamp_tracks_the_window_between_its_bounds() {
    let css = ".box { font-size: clamp(16px, 5vmin, 30px) }";

    // 5vmin of 360 is 18 — inside the bounds, so it tracks.
    assert_eq!(font_size(css, 360.0, 620.0), Some(18.0));
    // 5vmin of 1200 is 60 — above the ceiling, so it stops at 30.
    assert_eq!(font_size(css, 1200.0, 1500.0), Some(30.0));
    // 5vmin of 200 is 10 — below the floor, so it stops at 16.
    assert_eq!(font_size(css, 200.0, 200.0), Some(16.0));
}

#[doc = include_str!("responsive_font_size.doc/an_em_inside_the_expression_means_the_parent.md")]
#[test]
fn an_em_inside_the_expression_means_the_parent() {
    // The root's parent size is the initial 16px, so `calc(2em)` is 32.
    assert_eq!(
        font_size(".box { font-size: calc(2em + 0px) }", 400.0, 400.0),
        Some(32.0)
    );
}

#[doc = include_str!("responsive_font_size.doc/a_plain_length_is_unaffected.md")]
#[test]
fn a_plain_length_is_unaffected() {
    assert_eq!(
        font_size(".box { font-size: 26px }", 360.0, 620.0),
        Some(26.0)
    );
    assert_eq!(
        font_size(".box { font-size: 26px }", 1200.0, 1500.0),
        Some(26.0)
    );
}

#[doc = include_str!("responsive_font_size.doc/a_ceiling_only_size_keeps_shrinking_where_a_floor_would_stop.md")]
#[test]
fn a_ceiling_only_size_keeps_shrinking_where_a_floor_would_stop() {
    let ceiling = ".box { font-size: min(6vmin, 34px) }";
    let floored = ".box { font-size: clamp(18px, 6vmin, 34px) }";

    // Both agree while the window is roomy enough that neither bound bites.
    assert_eq!(
        font_size(ceiling, 400.0, 600.0),
        font_size(floored, 400.0, 600.0)
    );

    // Squeezed, the floor stops and the ceiling keeps going.
    assert_eq!(font_size(floored, 100.0, 160.0), Some(18.0));
    assert_eq!(font_size(ceiling, 100.0, 160.0), Some(6.0));

    // And the ceiling still holds at the other end.
    assert_eq!(font_size(ceiling, 1200.0, 1500.0), Some(34.0));
}
