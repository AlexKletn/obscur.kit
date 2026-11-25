#![doc = include_str!("spec_ch4b_observers.doc/_module.md")]

mod spec_ch4b_support;

use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::layout::model::LayoutBox;
use obscur_kit::layout::observer::{IntersectionObserver, VisibilityObserver};
use spec_ch4b_support::harness::{approx, at, drive, el, geom, txt, view};

#[test]
fn paint_snapping_preserves_shared_edges_at_fractional_scale() {
    use obscur_kit::geometry::PixelSnapMode;

    let v = view(vec![el(1, "item", vec![])]);
    let mut rt = drive(300.0, 400.0, ".item { width: 10.47px; height: 5.34px }", &v);
    let target = at(&rt, &[0]);
    let original = rt.layout_boxes()[&target].bounds;
    approx(original.size.width, 10.47, "fractional layout width");
    rt.set_device_scale(1.25);
    let left = rt.snap_window_paint_rect(original, PixelSnapMode::Edges);
    let neighbor = Rect::new(Point::new(original.right(), 0.0), original.size);
    let right = rt.snap_window_paint_rect(neighbor, PixelSnapMode::Edges);
    assert_eq!(left.right(), right.origin.x);
    approx(left.right() * 1.25, 13.0, "shared physical pixel edge");
    assert_eq!(
        rt.snap_window_paint_rect(original, PixelSnapMode::None),
        original
    );
    assert_eq!(rt.layout_boxes()[&target].bounds, original);
}

#[test]
fn css_pixel_snap_aligns_backgrounds_in_surface_space() {
    use obscur_kit::scene::command::PaintCommand;
    use spec_ch4b_support::harness::{compile, runtime};

    let v = view(vec![el(1, "item", vec![])]);
    for (mode, physical_right) in [("edges", 13.0), ("none", 12.8375), ("auto", 12.8375)] {
        let sheet = compile(&format!(
            ".item {{ width: 10.47px; height: 5.34px; background-color: red; pixel-snap: {mode} }}"
        ));
        let mut rt = runtime(300.0, 400.0);
        rt.set_device_scale(1.25);
        rt.set_content_offset(0.2, 0.0);
        let _ = rt.render_now(&v, &[sheet]);
        let target = at(&rt, &[0]);
        approx(
            rt.layout_boxes()[&target].size().width,
            10.47,
            "layout width",
        );
        let snapshot = rt.snapshot();
        let bounds = snapshot
            .display_list()
            .commands()
            .iter()
            .find_map(|command| match command {
                PaintCommand::DrawPath { path, .. } => path.bounds(),
                _ => None,
            })
            .expect("background fill");
        approx((bounds.right() - 0.2) * 1.25, physical_right, mode);
    }
}

#[test]
fn local_points_map_through_scrolling_transform_and_fractional_scale() {
    use obscur_kit::geometry::space::{LocalSpace, PointIn, ScreenSpace};

    let v = view(vec![el(
        1,
        "scroller",
        vec![el(2, "row", vec![]), el(3, "row", vec![])],
    )]);
    let mut rt = drive(
        300.0,
        400.0,
        ".scroller { width: 200px; height: 100px; overflow: scroll;
                     transform: scale(2); transform-origin: 0 0 }
         .row { width: 200px; height: 100px }",
        &v,
    );
    rt.set_scroll_top(at(&rt, &[0]), 50.0);
    rt.set_content_offset(3.0, 4.0);
    rt.set_device_scale(1.25);
    let local = PointIn::<LocalSpace>::new(Point::new(5.0, 10.0));
    let parent = rt.local_to_parent(at(&rt, &[0, 1])).unwrap();
    assert_eq!(parent.map(local).point(), Point::new(5.0, 60.0));
    let mapper = rt.local_to_window(at(&rt, &[0, 1])).unwrap();
    let window = mapper.map(local);
    assert_eq!(window.point(), Point::new(7.0, 116.0));
    let physical = mapper.then(&rt.window_to_physical());
    assert_eq!(physical.map(local).point(), Point::new(8.75, 145.0));
    let screen = rt.window_to_screen(PointIn::<ScreenSpace>::new(Point::new(-1920.0, 32.0)));
    assert_eq!(
        mapper.then(&screen).map(local).point(),
        Point::new(-1911.25, 177.0)
    );
    assert_eq!(mapper.inverse().unwrap().map(window), local);

    let normalized = rt.local_to_normalized(at(&rt, &[0, 1])).unwrap();
    let point = PointIn::new(Point::new(100.0, 25.0));
    let fraction = normalized.map(point);
    assert_eq!(fraction.point(), Point::new(0.5, 0.25));
    assert_eq!(normalized.inverse().unwrap().map(fraction), point);
}

#[test]
fn an_empty_axis_has_no_normalized_mapping() {
    let v = view(vec![el(1, "empty", vec![])]);
    let rt = drive(300.0, 400.0, ".empty { width: 50px; height: 0 }", &v);
    assert!(rt.local_to_normalized(at(&rt, &[0])).is_none());
}

#[test]
fn runtime_transform_observer_reports_composed_visual_ancestors() {
    use std::{cell::RefCell, rc::Rc};

    let v = view(vec![el(1, "parent", vec![el(2, "child", vec![])])]);
    let original = ".parent { transform: translateX(10px) }
                    .child { width: 40px; height: 30px; transform: translateX(5px) }";
    let mut rt = drive(300.0, 400.0, original, &v);
    let target = at(&rt, &[0, 0]);
    let changes = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&changes);
    assert!(rt.observe_transform(target, move |change| {
        observed
            .borrow_mut()
            .push(change.current.apply(Point::ORIGIN));
        false
    }));
    let original = spec_ch4b_support::harness::compile(original);
    assert!(rt.run_due_frame(&v, &[original]).is_some());
    assert_eq!(*changes.borrow(), [Point::new(15.0, 0.0)]);

    let moved = spec_ch4b_support::harness::compile(
        ".parent { transform: translateX(30px) }
         .child { width: 40px; height: 30px; transform: translateX(5px) }",
    );
    let _ = rt.render_now(&v, std::slice::from_ref(&moved));
    let _ = rt.render_now(&v, &[moved]);
    assert_eq!(
        *changes.borrow(),
        [Point::new(15.0, 0.0), Point::new(35.0, 0.0)]
    );
    let _ = rt.render_now(&obscur_kit::view::view::View::empty(), &[]);
    assert!(!rt.unobserve_transform(target));
}

#[test]
fn runtime_visibility_and_intersection_observers_follow_scroll_clips() {
    use std::{cell::RefCell, rc::Rc};

    let css = ".outer { width: 300px; height: 180px; overflow: hidden }
               .scroller { width: 300px; height: 200px; overflow: scroll }
               .row { width: 300px; height: 100px }";
    let rows = (0..20).map(|i| el(20 + i, "row", vec![])).collect();
    let v = view(vec![el(2, "outer", vec![el(1, "scroller", rows)])]);
    let mut rt = drive(300.0, 400.0, css, &v);
    let item = at(&rt, &[0, 0, 15]);
    let scroller = at(&rt, &[0, 0]);
    let visible = Rc::new(RefCell::new(Vec::new()));
    let changes = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&visible);
    assert!(rt.observe_visibility(item, move |change| {
        observed.borrow_mut().push(change.visible);
        false
    }));
    let observed = Rc::clone(&changes);
    assert!(rt.observe_intersection(item, 0.75, move |change| {
        observed.borrow_mut().push(change);
        false
    }));
    let sheet = spec_ch4b_support::harness::compile(css);
    let sheets = std::slice::from_ref(&sheet);
    assert!(rt.run_due_frame(&v, sheets).is_some());
    assert_eq!(*visible.borrow(), [false]);
    assert!(!changes.borrow()[0].intersecting);

    rt.set_scroll_top(scroller, 1_350.0);
    let _ = rt.render_now(&v, sheets);
    assert_eq!(*visible.borrow(), [false, true]);
    assert_eq!(changes.borrow().len(), 1);
    rt.set_scroll_top(scroller, 1_400.0);
    let _ = rt.render_now(&v, sheets);
    assert!(changes.borrow()[1].intersecting);
    approx(
        changes.borrow()[1].ratio,
        0.8,
        "outer clip trims the intersection",
    );

    let _ = rt.render_now(&obscur_kit::view::view::View::empty(), &[]);
    assert!(!rt.unobserve_visibility(item));
    assert!(!rt.unobserve_intersection(item));
}

#[test]
fn runtime_layout_observer_reports_movement_without_resize() {
    use std::{cell::RefCell, rc::Rc};

    let v = view(vec![el(1, "item", vec![])]);
    let css = ".item { width: 100px; height: 30px }";
    let mut rt = drive(300.0, 400.0, css, &v);
    let target = at(&rt, &[0]);
    let layout = Rc::new(RefCell::new(Vec::new()));
    let resize = Rc::new(RefCell::new(Vec::new()));
    let reported = Rc::clone(&layout);
    assert!(rt.observe_layout(target, move |change| {
        reported.borrow_mut().push(change);
        false
    }));
    let reported = Rc::clone(&resize);
    assert!(rt.observe_resize(target, move |change| {
        reported.borrow_mut().push(change);
        false
    }));
    let original = spec_ch4b_support::harness::compile(css);
    assert!(rt.run_due_frame(&v, &[original]).is_some());
    let moved = spec_ch4b_support::harness::compile(
        ".item { width: 100px; height: 30px; margin-left: 50px }",
    );
    let _ = rt.render_now(&v, std::slice::from_ref(&moved));
    let _ = rt.render_now(&v, &[moved]);
    assert_eq!(layout.borrow().len(), 2);
    assert_eq!(resize.borrow().len(), 1);
    let change = layout.borrow()[1];
    assert_eq!(change.previous.unwrap().origin().x, 0.0);
    assert_eq!(change.current.origin().x, 50.0);
    assert_eq!(change.current.size(), Size::new(100.0, 30.0));

    let _ = rt.render_now(&obscur_kit::view::view::View::empty(), &[]);
    assert!(!rt.unobserve_layout(target));
    assert!(!rt.unobserve_resize(target));
}

#[test]
fn runtime_resize_observations_are_coalesced_and_end_at_unmount() {
    use std::{cell::RefCell, rc::Rc};

    let original = ".item { width: 100px; height: 30px }";
    let v = view(vec![el(1, "item", vec![])]);
    let mut rt = drive(300.0, 400.0, original, &v);
    let target = at(&rt, &[0]);
    let entries = Rc::new(RefCell::new(Vec::new()));
    let reported = Rc::clone(&entries);
    assert!(rt.observe_resize(target, move |change| {
        reported.borrow_mut().push(change);
        change.previous.is_none()
    }));
    let original = spec_ch4b_support::harness::compile(original);
    assert!(
        rt.run_due_frame(&v, std::slice::from_ref(&original))
            .is_some()
    );
    assert_eq!(entries.borrow().len(), 1);
    assert!(rt.has_pending_frame());
    assert!(rt.run_due_frame(&v, &[original]).is_some());
    assert_eq!(entries.borrow().len(), 1);

    let changed = spec_ch4b_support::harness::compile(".item { width: 180px; height: 30px }");
    let _ = rt.render_now(&v, &[changed]);
    let second = entries.borrow()[1];
    assert_eq!(second.previous, Some(Size::new(100.0, 30.0)));
    assert_eq!(second.current, Size::new(180.0, 30.0));
    let _ = rt.render_now(&obscur_kit::view::view::View::empty(), &[]);
    assert!(!rt.unobserve_resize(target));
    assert_eq!(entries.borrow().len(), 2);
}

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

#[test]
fn a_surface_camera_maps_layout_to_paint_and_pointer_coordinates() {
    use std::{cell::RefCell, rc::Rc};

    use obscur_kit::geometry::space::{LocalSpace, PointIn};
    use obscur_kit::interaction::gesture::ZoomPan;
    use obscur_kit::scene::command::PaintCommand;
    use spec_ch4b_support::harness::{compile, runtime};

    let v = view(vec![el(1, "item", vec![])]);
    let sheet = compile(".item { width: 40px; height: 20px; background-color: red }");
    let sheets = [sheet];
    let mut rt = runtime(300.0, 400.0);
    rt.set_device_scale(2.0);
    let _ = rt.render_now(&v, &sheets);
    let item = at(&rt, &[0]);
    let visibility = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&visibility);
    assert!(rt.observe_visibility(item, move |change| {
        observed.borrow_mut().push(change.visible);
        false
    }));
    let mut camera = ZoomPan::new(0.5, 4.0).unwrap();
    assert!(camera.zoom_at(2.0, Point::ORIGIN));
    assert!(camera.pan_by(Point::new(100.0, 40.0)));
    assert!(rt.set_view_camera(Some(camera)));
    assert!(rt.run_due_frame(&v, &sheets).is_some());

    assert_eq!(
        rt.visible_content_rect(),
        Rect::new(Point::new(-50.0, -20.0), Size::new(150.0, 200.0))
    );
    let position = rt
        .local_to_window(item)
        .unwrap()
        .then(&rt.window_to_physical())
        .map(PointIn::<LocalSpace>::new(Point::new(5.0, 6.0)))
        .point();
    assert_eq!(position, Point::new(220.0, 104.0));
    let chain = rt.hit_test(position);
    assert_eq!(chain.target().unwrap().node(), item);
    assert_eq!(chain.target().unwrap().local_point(), Point::new(5.0, 6.0));
    assert_eq!(rt.layout_boxes()[&item].size(), Size::new(40.0, 20.0));
    assert!(matches!(
        rt.snapshot().display_list().commands().first(),
        Some(PaintCommand::PushTransform(transform)) if transform.apply(Point::new(5.0, 6.0)) == position
    ));

    assert!(camera.pan_by(Point::new(400.0, 0.0)));
    assert!(rt.set_view_camera(Some(camera)));
    assert!(rt.run_due_frame(&v, &sheets).is_some());
    assert_eq!(*visibility.borrow(), [true, false]);

    assert!(rt.set_view_camera(None));
    assert!(rt.run_due_frame(&v, &sheets).is_some());
    assert!(
        rt.hit_test(position)
            .target()
            .is_none_or(|target| target.node() != item)
    );
}

#[test]
fn percentage_translate_uses_the_current_border_box_on_each_axis() {
    use obscur_kit::geometry::space::{LocalSpace, PointIn};
    use obscur_kit::scene::command::PaintCommand;
    use spec_ch4b_support::harness::{compile, runtime};

    let v = view(vec![el(1, "item", vec![])]);
    let mut rt = runtime(400.0, 300.0);
    for (width, expected_x) in [(100, 120.0), (140, 160.0)] {
        let sheet = compile(&format!(
            ".item {{ width: {width}px; height: 40px; background-color: red;
             transform-origin: 0 0; transform: scale(2) translate(calc(50% + 10px), 25%); }}"
        ));
        let _ = rt.render_now(&v, &[sheet]);
        let item = at(&rt, &[0]);
        let origin = rt
            .local_to_window(item)
            .unwrap()
            .map(PointIn::<LocalSpace>::new(Point::ORIGIN))
            .point();
        assert_eq!(origin, Point::new(expected_x, 20.0));
        assert_eq!(rt.layout_boxes()[&item].origin(), Point::ORIGIN);
        let target = rt.hit_test(Point::new(expected_x + 2.0, 22.0));
        assert_eq!(target.target().unwrap().node(), item);
        assert_eq!(target.target().unwrap().local_point(), Point::new(1.0, 1.0));
        assert!(rt.snapshot().display_list().commands().iter().any(
            |command| matches!(command, PaintCommand::PushTransform(matrix) if matrix.apply(Point::ORIGIN) == origin)
        ));
    }
}

#[test]
fn visible_content_observer_coalesces_camera_changes_and_reports_resize() {
    use std::{cell::RefCell, rc::Rc};

    use obscur_kit::interaction::gesture::ZoomPan;
    use obscur_kit::runtime::frame::FrameReason;
    use spec_ch4b_support::harness::runtime;

    let v = view(vec![el(1, "item", vec![])]);
    let mut rt = runtime(300.0, 200.0);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&seen);
    rt.observe_visible_content(move |region| {
        observed.borrow_mut().push(region);
        false
    });
    assert!(rt.run_due_frame(&v, &[]).is_some());
    assert_eq!(
        *seen.borrow(),
        [Rect::new(Point::ORIGIN, Size::new(300.0, 200.0))]
    );

    let mut camera = ZoomPan::new(0.5, 4.0).unwrap();
    assert!(camera.zoom_at(2.0, Point::ORIGIN));
    assert!(rt.set_view_camera(Some(camera)));
    assert!(camera.pan_by(Point::new(40.0, 20.0)));
    assert!(rt.set_view_camera(Some(camera)));
    assert!(rt.run_due_frame(&v, &[]).is_some());
    assert_eq!(
        seen.borrow()[1],
        Rect::new(Point::new(-20.0, -10.0), Size::new(150.0, 100.0))
    );
    let _ = rt.render_now(&v, &[]);
    assert_eq!(seen.borrow().len(), 2);

    rt.set_viewport(Size::new(400.0, 200.0));
    rt.request_frame(FrameReason::WindowChange);
    assert!(rt.run_due_frame(&v, &[]).is_some());
    assert_eq!(seen.borrow()[2].size, Size::new(200.0, 100.0));

    assert!(rt.unobserve_visible_content());
    assert!(!rt.unobserve_visible_content());
    assert!(rt.set_view_camera(None));
    assert!(rt.run_due_frame(&v, &[]).is_some());
    assert_eq!(seen.borrow().len(), 3);
}

#[test]
fn typed_world_mapping_matches_camera_visibility_and_physical_coordinates() {
    use obscur_kit::geometry::space::{PointIn, RectIn, WindowSpace, WorldSpace};
    use obscur_kit::interaction::gesture::ZoomPan;
    use spec_ch4b_support::harness::runtime;

    let mut rt = runtime(300.0, 200.0);
    rt.set_device_scale(2.0);
    rt.set_content_offset(10.0, 5.0);
    let mut camera = ZoomPan::new(0.5, 4.0).unwrap();
    assert!(camera.zoom_at(2.0, Point::ORIGIN));
    assert!(camera.pan_by(Point::new(40.0, 20.0)));
    assert!(rt.set_view_camera(Some(camera)));

    let world = PointIn::<WorldSpace>::new(Point::new(15.0, 8.0));
    let mapping = rt.world_to_window();
    let window = mapping.map(world);
    assert_eq!(window.point(), Point::new(50.0, 26.0));
    assert_eq!(mapping.inverse().unwrap().map(window), world);
    assert_eq!(
        mapping.then(&rt.window_to_physical()).map(world).point(),
        Point::new(100.0, 52.0),
    );
    let viewport = RectIn::<WindowSpace>::new(Rect::new(Point::ORIGIN, Size::new(300.0, 200.0)));
    assert_eq!(
        mapping.inverse().unwrap().map_rect(viewport),
        rt.visible_world_rect()
    );
}

#[test]
fn camera_driven_materialization_retains_nodes_that_stay_in_view() {
    use std::{cell::RefCell, rc::Rc};

    use obscur_kit::geometry::space::RectIn;
    use obscur_kit::identity::NodeId;
    use obscur_kit::interaction::gesture::ZoomPan;
    use obscur_kit::interaction::spatial::{SpatialIndex, SpatialMaterialization};
    use spec_ch4b_support::harness::runtime;

    let mut index = SpatialIndex::new();
    for (id, x) in [(1, 0.0), (2, 20.0), (3, 40.0)] {
        index.insert(
            NodeId::from_raw(id),
            Rect::new(Point::new(x, 0.0), Size::new(20.0, 10.0)),
            0,
        );
    }
    let materialized = Rc::new(RefCell::new(SpatialMaterialization::default()));
    let mut rt = runtime(20.0, 10.0);
    let visible = Rc::clone(&materialized);
    rt.set_root_view(move || {
        view(
            visible
                .borrow()
                .active()
                .map(|node| el(10, "item", vec![]).with_key(node.raw()))
                .collect(),
        )
    });
    let mut camera = ZoomPan::new(0.5, 4.0).unwrap();
    assert!(camera.pan_by(Point::new(-10.0, 0.0)));
    assert!(rt.set_view_camera(Some(camera)));
    rt.observe_visible_content(move |region| {
        !materialized
            .borrow_mut()
            .update(&index, RectIn::new(region))
            .is_empty()
    });

    assert!(rt.run_due_frame_reactive().is_some());
    assert!(rt.run_due_frame_reactive().is_some());
    let removed = at(&rt, &[0]);
    let shared = at(&rt, &[1]);

    assert!(camera.pan_by(Point::new(-20.0, 0.0)));
    assert!(rt.set_view_camera(Some(camera)));
    assert!(rt.run_due_frame_reactive().is_some());
    assert!(rt.run_due_frame_reactive().is_some());
    assert!(!rt.tree().contains(removed));
    assert_eq!(at(&rt, &[0]), shared);
    assert_ne!(at(&rt, &[1]), shared);
}

#[test]
fn a_zoomed_world_marquee_can_select_and_snap_in_scene_coordinates() {
    use obscur_kit::geometry::space::{PointIn, RectIn, WindowSpace};
    use obscur_kit::identity::NodeId;
    use obscur_kit::interaction::gesture::ZoomPan;
    use obscur_kit::interaction::spatial::{SpatialIndex, WorldGrid, snap_world_point};
    use spec_ch4b_support::harness::runtime;

    let mut rt = runtime(200.0, 100.0);
    let mut camera = ZoomPan::new(0.5, 4.0).unwrap();
    assert!(camera.zoom_at(2.0, Point::ORIGIN));
    assert!(camera.pan_by(Point::new(20.0, 10.0)));
    assert!(rt.set_view_camera(Some(camera)));
    let to_world = rt.world_to_window().inverse().unwrap();

    let mut index = SpatialIndex::new();
    let [inside, crossing] = [1, 2].map(NodeId::from_raw);
    index.insert(
        inside,
        Rect::new(Point::new(3.0, 3.0), Size::new(3.0, 3.0)),
        0,
    );
    index.insert(
        crossing,
        Rect::new(Point::new(9.0, 5.0), Size::new(3.0, 3.0)),
        0,
    );
    let marquee =
        RectIn::<WindowSpace>::new(Rect::new(Point::new(20.0, 10.0), Size::new(20.0, 20.0)));
    let region = to_world.map_rect(marquee).rect();
    assert_eq!(index.entries_intersecting(region).len(), 2);
    assert_eq!(
        index
            .entries_contained_by(region)
            .iter()
            .map(|(node, _)| *node)
            .collect::<Vec<_>>(),
        [inside]
    );

    let grid = WorldGrid::new(PointIn::new(Point::ORIGIN), Size::new(5.0, 5.0)).unwrap();
    let pointer = to_world.map(PointIn::new(Point::new(29.0, 19.0)));
    let tolerance = to_world.map_extent(Size::new(2.0, 2.0));
    let snapped = snap_world_point(pointer, Some(grid), &[4.0], &[], tolerance);
    assert_eq!(
        rt.world_to_window().map(snapped).point(),
        Point::new(28.0, 20.0)
    );
}

#[doc = include_str!("spec_ch4b_observers.doc/css_visibility_changes_exposure_without_changing_intersection.md")]
#[test]
fn css_visibility_changes_exposure_without_changing_intersection() {
    use spec_ch4b_support::harness::compile;
    use std::{cell::RefCell, rc::Rc};

    let view = view(vec![el(1, "parent", vec![el(2, "item", vec![])])]);
    let css = ".parent { width: 40px; height: 40px } .item { width: 20px; height: 20px }";
    let mut rt = drive(100.0, 100.0, css, &view);
    let item = at(&rt, &[0, 0]);
    let visible = Rc::new(RefCell::new(Vec::new()));
    let intersections = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&visible);
    assert!(rt.observe_visibility(item, move |change| {
        observed.borrow_mut().push(change.visible);
        false
    }));
    let observed = Rc::clone(&intersections);
    assert!(rt.observe_intersection(item, 0.5, move |change| {
        observed.borrow_mut().push(change.intersecting);
        false
    }));

    for rule in [
        "",
        ".parent { visibility: hidden }",
        ".parent { visibility: hidden } .item { visibility: visible }",
    ] {
        let sheet = compile(&format!("{css} {rule}"));
        rt.render_now(&view, &[sheet]);
    }
    assert_eq!(*visible.borrow(), [true, false, true]);
    assert_eq!(*intersections.borrow(), [true]);
}
