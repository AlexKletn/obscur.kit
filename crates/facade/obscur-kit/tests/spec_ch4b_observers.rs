#![doc = include_str!("spec_ch4b_observers.doc/_module.md")]

mod spec_ch4b_support;

use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::layout::model::LayoutBox;
use obscur_kit::layout::observer::{IntersectionObserver, VisibilityObserver};
use spec_ch4b_support::harness::{approx, at, drive, el, geom, txt, view};

#[doc = include_str!("spec_ch4b_observers.doc/ch4_4_d_visibility_and_intersection_observers_track_the_scroll_chain.md")]
#[test]
fn ch4_4_d_visibility_and_intersection_observers_track_the_scroll_chain() {
    let css = "
        .scroller { width: 300px; height: 200px; overflow: scroll; }
        .item { width: 300px; height: 100px; }
    ";
    let items: Vec<_> = (0..20)
        .map(|i| el(100 + i, "item", vec![txt("x")]))
        .collect();
    let v = view(vec![el(1, "scroller", items)]);

    let mut rt = drive(300.0, 400.0, css, &v);
    let scroller = at(&rt, &[0]);
    let item15 = at(&rt, &[0, 15]);

    // The observers must exist and be constructible by those names.
    let mut intersection = IntersectionObserver::new(item15, 0.0);
    let mut visibility = VisibilityObserver::new(item15);

    let viewport_rect = |rt: &obscur_kit::runtime::UiRuntime| {
        let (x, y, w, h) = geom(rt, scroller);
        Rect::new(Point::new(x, y), Size::new(w, h))
    };
    let item_box = |rt: &obscur_kit::runtime::UiRuntime| {
        let (x, y, w, h) = geom(rt, item15);
        LayoutBox::new(Rect::new(Point::new(x, y), Size::new(w, h)))
    };

    // --- offset 0: item 15 sits at content y 1500..1600, far below the 200px
    //     container viewport.
    let root = viewport_rect(&rt);
    let target = item_box(&rt);
    let first_intersection = intersection
        .observe(target.bounds, root)
        .expect("the first observation reports the initial state");
    assert!(
        !first_intersection.intersecting,
        "at offset 0 item 15 must not intersect the 200px container viewport"
    );
    let first_visibility = visibility
        .observe(target, Some(root))
        .expect("the first observation reports the initial state");
    assert!(
        !first_visibility.visible,
        "at offset 0 item 15 must not be visible in the 200px container viewport"
    );

    // --- scroll to 1400: item 14 at the container top, item 15 at y 100..200.
    rt.set_scroll_top(scroller, 1400.0);
    let _ = rt.render_now(&v, &[]);

    let root = viewport_rect(&rt);
    let target = item_box(&rt);
    let second_intersection = intersection
        .observe(target.bounds, root)
        .expect("item 15 crossed into the viewport, so the observer fires");
    assert!(
        second_intersection.intersecting,
        "after scrolling to 1400 item 15 intersects the container viewport"
    );
    approx(
        second_intersection.ratio,
        1.0,
        "the intersection covers the full 100px of item 15",
    );
    let second_visibility = visibility
        .observe(target, Some(root))
        .expect("item 15 became visible, so the observer fires");
    assert!(
        second_visibility.visible,
        "after scrolling to 1400 item 15 is visible"
    );
}
