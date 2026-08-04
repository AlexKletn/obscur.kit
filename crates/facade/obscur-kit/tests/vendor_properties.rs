#![doc = include_str!("vendor_properties.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::color::Color;
use obscur_kit::scene::paint::Brush;
use obscur_kit::view::view::{LOCAL_NAME, View, ViewNode, ViewProperty, ViewValue};

const PANEL: ComponentId = ComponentId::from_raw(0x00C0_0009);

#[doc = include_str!("vendor_properties.doc/staged.md")]
fn staged(css: &str) -> (UiRuntime, CssPipeline, Rc<Cell<u32>>) {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    let mut pipeline = CssPipeline::new();
    pipeline.install(&mut rt, &[CssSource::anonymous(css)]);

    let runs = Rc::new(Cell::new(0));
    let counter = Rc::clone(&runs);
    rt.register_named_canvas(
        "sparkline",
        Box::new(move |region, _paint| {
            counter.set(counter.get() + 1);
            let size = region.size();
            region.painter().fill_rect(
                Rect::new(Point::new(0.0, 0.0), size),
                Brush::solid(Color::rgba(1.0, 0.0, 0.0, 1.0)),
            );
            false
        }),
    );

    let view = View::single(ViewNode::element(PANEL).with_prop(ViewProperty::new(
        LOCAL_NAME,
        ViewValue::Text("chart".into()),
    )));
    let _ = rt.render_now(&view, pipeline.sheets());
    (rt, pipeline, runs)
}

#[doc = include_str!("vendor_properties.doc/a_stylesheet_binds_a_painter_to_a_plain_element.md")]
#[test]
fn a_stylesheet_binds_a_painter_to_a_plain_element() {
    // An ordinary element, no props, a stylesheet naming a painter — the whole
    // reason the `-obscur-` namespace exists. Nothing in any CSS specification
    // has a word for "draw this box with that routine".
    let (_rt, _pipeline, runs) = staged("chart{width:80px;height:40px;-obscur-painter:sparkline}");
    assert!(runs.get() >= 1, "the painter named in CSS ran");
}

#[doc = include_str!("vendor_properties.doc/another_vendors_property_raises_no_warning.md")]
#[test]
fn another_vendors_property_raises_no_warning() {
    // The contract of a prefix: a renderer that does not know it ignores it.
    // A stylesheet shared with a browser build carries `-webkit-` declarations
    // BY DESIGN, and a console full of warnings about them trains the reader to
    // ignore the warnings that matter.
    let (_rt, pipeline, _runs) = staged(
        "chart{-webkit-box-reflect:below;-moz-osx-font-smoothing:grayscale;width:80px;height:40px}",
    );
    assert!(pipeline.errors().is_empty(), "not a parse error either");
    for feature in pipeline.unsupported() {
        assert_eq!(
            feature.kind(),
            obscur_kit::css::error::UnsupportedFeatureKind::VendorProperty,
            "{feature} should have been passed over, not faulted"
        );
    }
}

#[doc = include_str!("vendor_properties.doc/another_vendors_prefix_is_not_an_alias.md")]
#[test]
fn another_vendors_prefix_is_not_an_alias() {
    // Spec §4.13 excludes vendor hacks, and this is the one that matters:
    // `-webkit-painter` does not become `-obscur-painter`. Accepting a foreign
    // prefix as an alias is how a renderer ends up implementing another
    // vendor's history.
    let (_rt, _pipeline, runs) = staged("chart{width:80px;height:40px;-webkit-painter:sparkline}");
    assert_eq!(runs.get(), 0, "no painter was bound");
}

#[doc = include_str!("vendor_properties.doc/a_typo_in_our_own_namespace_is_reported.md")]
#[test]
fn a_typo_in_our_own_namespace_is_reported() {
    // Our own namespace gets no indulgence. Silence there would let a
    // misspelled rule sit in a stylesheet doing nothing, which is exactly the
    // failure the quiet treatment of foreign prefixes would otherwise hide.
    let (_rt, pipeline, runs) = staged("chart{width:80px;height:40px;-obscur-paintr:sparkline}");
    assert_eq!(runs.get(), 0);
    assert!(
        pipeline
            .unsupported()
            .iter()
            .any(|f| f.name() == "-obscur-paintr"
                && f.kind() == obscur_kit::css::error::UnsupportedFeatureKind::Property),
        "a typo in our own namespace is a fault: {:?}",
        pipeline.unsupported()
    );
}

#[doc = include_str!("vendor_properties.doc/none_unbinds_a_painter.md")]
#[test]
fn none_unbinds_a_painter() {
    let (_rt, _pipeline, runs) = staged(
        "chart{width:80px;height:40px;-obscur-painter:sparkline}chart{-obscur-painter:none}",
    );
    assert_eq!(runs.get(), 0, "the later rule unbound it");
}
